use std::{
    cell::Cell,
    os::windows::prelude::IntoRawSocket,
    process,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use anyhow::Context;
use basic_quick_lib::{cli_util::pause, io_util::input_trim};
use colored::Colorize;
use log::{error, info, warn};
use parking_lot::{Mutex, RwLock};
use rand::{prelude::SliceRandom, thread_rng};
use soloud::{AudioExt, Handle, LoadExt, Soloud, Wav};

use crate::{
    cli::data::Song,
    util::settings::{PlaybackMode, SETTINGS},
};

use super::data::PlaylistInfo;

fn shuffle_vec(vec: &mut Vec<usize>, target_len: usize) {
    *vec = (0..target_len).collect();
    vec.shuffle(&mut thread_rng());
}

#[derive(thiserror::Error, Debug)]
enum GetIndexError {
    #[error("Index cannot be zero!")]
    IndexIsZero,

    #[error("{0} is not a valid index!")]
    InvalidIndex(i32),
}

/// get the index (positive or negative, but can't be zero) and
/// return the actual index.
/// Basically used for converting negative index
fn get_index(index: i32, vec_len: usize) -> Result<usize, GetIndexError> {
    use GetIndexError::*;

    let vec_index = match index {
        0 => return Err(IndexIsZero),
        1..=i32::MAX => index - 1,
        i32::MIN..=-1 => vec_len as i32 + index,
    };

    if vec_index.is_negative() || vec_index >= vec_len as i32 {
        return Err(InvalidIndex(index));
    }

    Ok(vec_index as usize)
}

fn help_print(command: &str, help_msg: &str) {
    info!("{:<20} --- {}", command, help_msg);
}

fn multiplied_volume(volume: u8, multiplier: f32) -> f32 {
    (volume as f32 * multiplier / 100.0).clamp(0.0, 1.0)
}

#[derive(clap::Args)]
pub struct Play {
    playlist_name: String,
}

impl Play {
    pub fn handle(&self) {
        let playlist_info = match PlaylistInfo::load(&self.playlist_name) {
            Ok(v) => v,
            Err(err) => {
                error!(
                    r#"Failed to load playlist "{}"! Error: {}"#,
                    self.playlist_name, err
                );
                return;
            }
        };

        PlayMenu::new(playlist_info).start();
    }
}

enum Message {
    Pause,
    Resume,
    PauseOrResume,
    Reprint,
    IndexJump(usize),
    SetVolume(u8),
    SetMultiplier(f32),
    PlayPrevious,
    PlayNext,
}

struct PlayMenu {
    commands_sender: mpsc::Sender<Message>,
    commands_receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
    playlist_info: Arc<RwLock<PlaylistInfo>>,
    currently_playing_index: Arc<AtomicUsize>,
}

impl PlayMenu {
    fn new(playlist_info: PlaylistInfo) -> Self {
        let channel = mpsc::channel::<Message>();

        Self {
            commands_sender: channel.0,
            commands_receiver: Arc::new(Mutex::new(channel.1)),
            playlist_info: Arc::new(RwLock::new(playlist_info)),
            currently_playing_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn start(&mut self) {
        self.handle_play();
        self.handle_input();
    }

    fn init_song(
        playlist_info: &PlaylistInfo,
        currently_playing: usize,
        wav: &mut Wav,
    ) -> Result<(), (String, String)> {
        let Song {
            song_name,
            path_to_song,
            ..
        } = &playlist_info.songs[currently_playing];

        wav.load(path_to_song)
            .map_err(|e| (
                format!("Failed to load song \"{}\"! Error: {}", song_name, e), 
                format!(
                    "The reason why it failed might be because {} does not exists or is an audio type that is not supported",
                    path_to_song.to_str().unwrap_or("Unknown Path")
                )
            ))
    }

    fn print_info(playlist_info: &PlaylistInfo, current_index: usize, is_paused: bool) {
        // just ignore it if failed to clear
        let _ = clearscreen::clear();

        info!("Playlist: {}", playlist_info.name);
        info!(
            "Created at {}",
            playlist_info
                .created
                .as_ref()
                .map(|v| v.to_date_string())
                .unwrap_or_else(|| "Unknown".to_string())
        );

        println!();

        for (index, song) in playlist_info.songs.iter().enumerate() {
            let is_current = index == current_index;

            let text = format!("{}. {}", index + 1, song.song_name);

            if is_current {
                info!("-> {}", text.bold())
            } else {
                info!("   {}", text)
            }
        }

        if is_paused {
            println!();
            info!("Paused");
        }
    }

    fn update_volume(
        sl: &mut Soloud,
        handle: Handle,
        playlist_info: &PlaylistInfo,
        currently_playing: usize,
    ) {
        sl.set_volume(
            handle,
            multiplied_volume(
                SETTINGS.read().volume,
                playlist_info.songs[currently_playing].sound_multiplier,
            ),
        );
    }

    fn handle_msg(
        message: &Message,
        sl: &mut Soloud,
        is_paused: &Cell<bool>,
        handle: Handle,
        currently_playing: &AtomicUsize,
        playlist_info: &mut PlaylistInfo,
    ) -> SongInstruction {
        use Message::*;

        match *message {
            Pause => {
                is_paused.set(true);
                sl.set_pause(handle, is_paused.get());
            }
            Resume => {
                is_paused.set(false);
                sl.set_pause(handle, is_paused.get());
            }
            PauseOrResume => {
                is_paused.set(!is_paused.get());
                sl.set_pause(handle, is_paused.get());
            }
            // do nothing because it will reprint anyways lol it
            // feels stupid
            Reprint => {}
            IndexJump(index) => {
                currently_playing.store(index, Ordering::SeqCst);
                return SongInstruction::SkipLoop;
            }
            SetVolume(new_volume) => {
                {
                    let mut setting = SETTINGS.write();
                    setting.volume = new_volume;
                    setting.save().unwrap_or_default();
                }

                Self::update_volume(
                    sl,
                    handle,
                    playlist_info,
                    currently_playing.load(Ordering::SeqCst),
                );
            }
            SetMultiplier(new_mul) => {
                {
                    playlist_info.songs[currently_playing.load(Ordering::SeqCst)]
                        .sound_multiplier = new_mul;
                    playlist_info.save();
                };

                Self::update_volume(
                    sl,
                    handle,
                    playlist_info,
                    currently_playing.load(Ordering::SeqCst),
                );
            }
            PlayPrevious => {
                if currently_playing.load(Ordering::SeqCst) == 0 {
                    currently_playing.store(playlist_info.songs.len() - 1, Ordering::SeqCst);
                } else {
                    currently_playing.fetch_sub(1, Ordering::SeqCst);
                }
                return SongInstruction::SkipLoop;
            }
            PlayNext => {
                if currently_playing.load(Ordering::SeqCst) == playlist_info.songs.len() - 1 {
                    currently_playing.store(0, Ordering::SeqCst);
                } else {
                    currently_playing.fetch_add(1, Ordering::SeqCst);
                }
                return SongInstruction::SkipLoop;
            }
        }

        SongInstruction::None
    }

    fn recv_cmd(
        sl: &mut Soloud,
        handle: Handle,
        receiver: &mpsc::Receiver<Message>,
        is_paused: &Cell<bool>,
        current_duration: &mut Duration,
        playlist_info: &RwLock<PlaylistInfo>,
        currently_playing: &AtomicUsize,
    ) -> SongInstruction {
        const SLEEP_DURATION: Duration = Duration::from_millis(10);

        while sl.voice_count() > 0 {
            // pause the loop a little so it won't take too much cpu
            // power
            thread::sleep(SLEEP_DURATION);

            if !is_paused.get() {
                *current_duration += SLEEP_DURATION;
            }

            // try to recv to see if there is any command, or else
            // continue playing the song
            if let Ok(message) = receiver.try_recv() {
                let instruction = Self::handle_msg(
                    &message,
                    sl,
                    is_paused,
                    handle,
                    currently_playing,
                    &mut playlist_info.write(),
                );

                if instruction != SongInstruction::None {
                    return instruction;
                }

                Self::print_info(
                    &playlist_info.read(),
                    currently_playing.load(Ordering::SeqCst),
                    is_paused.get(),
                );
            }
        }

        SongInstruction::None
    }

    fn handle_play(&self) {
        let playlist_info = Arc::clone(&self.playlist_info);
        let receiver = Arc::clone(&self.commands_receiver);
        let currently_playing = Arc::clone(&self.currently_playing_index);

        thread::spawn(move || {
            let songs_len = playlist_info.read().songs.len();

            if songs_len == 0 {
                warn!("The playlist is empty! Use command `add <YOUTUBE_VIDEO_LINK> <PLAYLIST_NAME>` to add songs into playlist!");
                pause();
                process::exit(1);
            }

            let mut sl = Soloud::default()
                .with_context(|| "Failed to get player!")
                .unwrap();

            let mut wav = Wav::default();

            let mut randomized_indexes = Vec::new();
            shuffle_vec(&mut randomized_indexes, songs_len);

            let currently_index = if SETTINGS.read().playback_mode == PlaybackMode::Random {
                randomized_indexes.pop().unwrap()
            } else {
                0
            };

            currently_playing.store(currently_index, Ordering::SeqCst);

            'song_loop: loop {
                let mut current_duration = Duration::ZERO;
                // use a cell here because we want the `print_info` closure
                // to be dynamically reflected and because of rust's
                // ownership system does not allow this
                // then cell is used instead
                let is_paused = Cell::new(false);

                Self::print_info(
                    &playlist_info.read(),
                    currently_playing.load(Ordering::SeqCst),
                    is_paused.get(),
                );

                if let Err(e) = Self::init_song(
                    &playlist_info.read(),
                    currently_playing.load(Ordering::SeqCst),
                    &mut wav,
                ) {
                    error!("{}", e.0);
                    info!("{}", e.1);
                    pause();
                    continue;
                }

                let handle = sl.play(&wav);

                Self::update_volume(
                    &mut sl,
                    handle,
                    &playlist_info.read(),
                    currently_playing.load(Ordering::SeqCst),
                );

                let instruction = Self::recv_cmd(
                    &mut sl,
                    handle,
                    &receiver.lock(),
                    &is_paused,
                    &mut current_duration,
                    &playlist_info,
                    currently_playing.as_ref(),
                );

                match instruction {
                    SongInstruction::None => {}
                    SongInstruction::SkipLoop => continue 'song_loop,
                }

                let setting = SETTINGS.read();
                let len = playlist_info.read().songs.len();

                // after the song been played, change the current playing song
                // based on the playback mode choice
                // TODO: Wrap this into a function
                match setting.playback_mode {
                    PlaybackMode::Sequel => {
                        currently_playing.fetch_add(1, Ordering::SeqCst);
                        if currently_playing.load(Ordering::SeqCst) >= len {
                            break;
                        }
                    }
                    PlaybackMode::LoopOnce => {}
                    PlaybackMode::LoopPlaylist => {
                        currently_playing.fetch_add(1, Ordering::SeqCst);

                        if currently_playing.load(Ordering::SeqCst) >= len {
                            currently_playing.store(0, Ordering::SeqCst);
                        }
                    }
                    PlaybackMode::Random => {
                        currently_playing.store(
                            randomized_indexes.pop().unwrap_or_else(|| {
                                shuffle_vec(&mut randomized_indexes, songs_len);

                                // should not panic because if the playlist is
                                // empty then it will be check and will exit the
                                // process if so
                                randomized_indexes.pop().unwrap()
                            }),
                            Ordering::SeqCst,
                        );
                    }
                }
            }

            process::exit(0);
        });
    }

    fn help_menu() {
        help_print("exit", "exit the program");
        help_print("help", "open help message");
        help_print("pause", "pause the music");
        help_print("resume", "resume the music");
        help_print("pr", "pause the music if it is playing otherwise resume");
        help_print(
            "setv <VOLUME>",
            "set the volume. anything that is not in between 0 and 100 will be invalid",
        );
        help_print("getv", "get the current volume");
        help_print(
            "setp <PLAYBACK_MODE>",
            "Set the playback mode. Value can be: random, looponce, loopplaylist, sequel (Note: it is not case sensitive)",
          );
        help_print("getp", "Get the current playback mode");
        help_print("setmp", "Set the current song's volume multiplier");
        help_print("getmp", "Get the current song's volume multiplier");
        help_print(
            "p",
            "Play previous. Wrap around the playlist if there is no previous",
        );
        help_print(
            "n",
            "Play next. Wrap around the playlist if there is no next",
        );
        info!("Type the index of the song to jump to the song. Example: `4` will jump to the fourth one");
        info!("    - Note that you can pass a negative value to start from the back. Example `-1` will go to the last song");
    }

    fn handle_input(&self) {
        use Message::*;

        let songs = Arc::clone(&self.playlist_info);
        let song_len = songs.read().songs.len();
        let current_playing_index = Arc::clone(&self.currently_playing_index);

        let try_send = |message: Message| {
            self.commands_sender.send(message).unwrap_or_else(|err| {
                error!("Something went wrong while sending the command to the music playing thread! This command will not do anything! Error: {}", err);
            });
        };

        loop {
            try_send(Reprint);

            let input = input_trim("");

            if input.is_empty() {
                continue;
            }

            let splitted = input.split(' ').collect::<Vec<&str>>();

            let command = splitted[0];
            let args = &splitted[1..];

            match command {
                "setv" => {
                    if args.is_empty() {
                        warn!("usage: setv <VOLUME>");
                        pause();
                        continue;
                    }

                    let volume = match args[0].trim().parse::<u64>() {
                        Ok(v) => v,
                        Err(err) => {
                            warn!("Failed to parse {}. Err: {}", args[0].trim(), err);
                            pause();
                            continue;
                        }
                    };

                    if volume > 100 {
                        warn!("Volume must be in between 0 and 100!");
                        pause();
                        continue;
                    }

                    try_send(SetVolume(volume as u8));
                }
                "getv" => {
                    info!("{}", SETTINGS.read().volume);
                    pause();
                }
                "help" => {
                    Self::help_menu();
                    pause();
                    try_send(Reprint);
                }
                "pause" => try_send(Pause),
                "resume" => try_send(Resume),
                "pr" => try_send(PauseOrResume),
                "setp" => {
                    if args.is_empty() {
                        warn!("Playback mode can be: random, looponce, loopplaylist, sequel (It is not case sensitive)");
                        pause();
                        continue;
                    }

                    let playback_mode = match PlaybackMode::from_str(args[0]) {
                        Ok(v) => v,
                        Err(_) => {
                            warn!("Invalid playback mode! Valid ones are: random, looponce, loopplaylist, sequel");
                            pause();
                            continue;
                        }
                    };

                    let mut settings = SETTINGS.write();
                    settings.playback_mode = playback_mode;
                    settings.save().unwrap_or_else(|err| {
                        warn!("Failed to save the settings! This means the playback mode did not change and it will be lost next time you open this program! (Error: {})", err);
                        pause();
                    });

                    info!(
                        "Successfully set playback mode to {}!",
                        settings.playback_mode
                    );
                    pause();
                }
                "getp" => {
                    info!("{}", SETTINGS.read().playback_mode);
                    pause();
                }
                "setmp" => {
                    if args.is_empty() {
                        warn!("No multiplier value is given. The multiplier is used to multiply current volume, and it must be a non-negative real number");
                        pause();
                        continue;
                    }

                    let multiplier = match args[0].trim().parse::<f32>() {
                        Ok(v) => {
                            if v < 0.0 {
                                warn!("It must be a non-negative real number");
                                pause();
                                continue;
                            }

                            v
                        }
                        Err(_) => {
                            warn!("Not a valid number!");
                            pause();
                            continue;
                        }
                    };

                    try_send(SetMultiplier(multiplier));
                }
                "getmp" => {
                    info!(
                        "Volume multiplier for current song is {}",
                        songs.read().songs[current_playing_index.load(Ordering::SeqCst)]
                            .sound_multiplier
                    );
                    pause();
                }
                "p" => {
                    try_send(PlayPrevious);
                }
                "n" => {
                    try_send(PlayNext);
                }
                num if num.parse::<i32>().is_ok() => {
                    // a really dumb thing to do and hopefully
                    // if let guard can be stabilized in the future
                    let num = num.parse::<i32>().unwrap();

                    let index = match get_index(num, song_len) {
                        Ok(index) => index,
                        Err(e) => {
                            error!("{}", e);
                            pause();
                            try_send(Reprint);
                            continue;
                        }
                    };

                    try_send(IndexJump(index));
                }
                "exit" => return,
                _ => {
                    warn!(
                        "Unknown Command `{}`! Type in `help` for more information!",
                        input
                    );
                    pause();
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SongInstruction {
    None,
    SkipLoop,
}

#[cfg(test)]
mod test {
    use soloud::{AudioExt, LoadExt, Soloud, Wav};

    #[test]
    fn voice_count() -> anyhow::Result<()> {
        let sl = Soloud::default()?;

        let mut wav = Wav::default();
        wav.load("JJD - Adventure [NCS Release].wav")?;

        sl.play(&wav);

        println!("active voice count: {}", wav.length());

        Ok(())
    }
}
