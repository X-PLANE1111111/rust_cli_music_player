use std::{
  cell::Cell,
  num::NonZeroI32,
  process,
  sync::{mpsc, PoisonError},
  thread,
  time::Duration,
};

use anyhow::Context;
use basic_quick_lib::{cli_util::pause, io_util::input_trim};
use colored::Colorize;
use log::{error, info, warn};
use soloud::{AudioExt, LoadExt, Soloud, Wav};

use crate::{
  cli::data::Song,
  util::settings::{PlaybackMode, SETTINGS},
};

use super::data::PlaylistInfo;

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
    self.play(playlist_info);
  }

  fn play(&self, playlist_info: PlaylistInfo) {
    use Message::*;

    let PlaylistInfo {
      name,
      songs,
      // created,
      ..
    } = playlist_info;

    let (sender, receiver) = mpsc::channel::<Message>();

    let songs_len = songs.len();

    thread::spawn(move || {
      let mut sl = Soloud::default()
        .with_context(|| "Failed to get player!")
        .unwrap();

      let mut wav = Wav::default();

      let volume = SETTINGS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .volume;

      sl.set_global_volume(volume as f32 / 100.0);

      let currently_playing = Cell::new(0);

      'song_loop: loop {
        let Song {
          song_name,
          path_to_song,
          author,
        } = &songs[currently_playing.get()];

        let mut current_duration = Duration::ZERO;
        // use a cell here because we want the `print_info` closure
        // to be dynamically reflected and because of rust's ownership
        // system does not allow this then cell is used instead
        let is_paused = Cell::new(false);

        let print_info = || {
          // just ignore it if failed to clear
          clearscreen::clear().unwrap_or_default();

          info!("Playlist: {}", name);
          // info!(
          //   "Created at {}",
          //   created
          //     .as_ref()
          //     .map(|v| v.to_string())
          //     .unwrap_or_else(|| "Unknown".to_string())
          // );

          println!();

          // info!("Currently playing \"{}\"", song_name);
          // info!("Made by \"{}\"", author);

          for (index, song) in songs.iter().enumerate() {
            let is_bold = index == currently_playing.get();

            if is_bold {
              info!("{}", format!("{}. {}", index + 1, song.song_name).bold())
            } else {
              info!("{}", format!("{}. {}", index + 1, song.song_name))
            }
          }

          if is_paused.get() {
            info!("Paused");
          }
        };

        print_info();

        if let Err(e) = wav.load(path_to_song) {
          error!("Failed to load song \"{}\"! Error: {}", song_name, e);
          info!(
						"The reason why it failed might be because {} does not exists or is an audio type that is not supported", 
						path_to_song.to_str().unwrap_or("Unknown Path")
					);
          pause();
          continue;
        };

        let handle = sl.play(&wav);

        const SLEEP_DURATION: Duration = Duration::from_millis(10);

        // while the song is playing
        while sl.voice_count() > 0 {
          // pause the loop a little so it won't take too much cpu power
          thread::sleep(SLEEP_DURATION);

          if !is_paused.get() {
            current_duration += SLEEP_DURATION;
          }

          // try to recv to see if there is any command, or else continue playing the song
          if let Ok(message) = receiver.try_recv() {
            match message {
              Pause => {
                is_paused.set(true);
                sl.set_pause(handle, is_paused.get());
              }
              Resume => {
                is_paused.set(false);
                sl.set_pause(handle, is_paused.get());
              }
              PauseOrResume => {
                // inverse bool but I'm too lazy
                is_paused.set(!is_paused.get());
                sl.set_pause(handle, is_paused.get());
              }
              // do nothing because it will reprint anyways lol it feels stupid
              Reprint => {}
              IndexJump(index) => {
                currently_playing.set(index);
                continue 'song_loop;
              }
            }

            print_info();
          }
        }

        let setting = SETTINGS.lock().unwrap_or_else(PoisonError::into_inner);

        // after the song been played, change the current playing song
        // based on the playback mode choice
        match setting.playback_mode {
          PlaybackMode::Sequel => {
            currently_playing.set(currently_playing.get() + 1);
            if currently_playing.get() >= songs.len() {
              break;
            }
          }
          PlaybackMode::LoopOnce => {}
          PlaybackMode::LoopPlaylist => {
            currently_playing.set(currently_playing.get() + 1);
            if currently_playing.get() >= songs.len() {
              currently_playing.set(0);
            }
          }
          PlaybackMode::Random => {}
        }

        drop(setting);
      }

      process::exit(0);
    });

    let try_send = |message: Message| {
      sender.send(message).unwrap_or_else(|err| {
				error!("Something went wrong while sending the command to the music playing thread! This command will not do anything! Error: {}", err);
			});
    };

    loop {
      let input = input_trim("");

      match input.as_str() {
        "exit" => return,
        "help" => {
          info!("exit     --- exit the program");
          info!("help     --- open help message");
          info!("pause    --- pause the music");
          info!("resume   --- resume the music");
          info!(
            "pr       --- pause the music if it is playing otherwise resume"
          );
          info!("Type the index of the song to jump to the song. Example: `4` will jump to the fourth one");
          info!("    - Note that you can pass a negative value to start from the back. Example `-1` will go to the last song");
          pause();
          try_send(Reprint);
        }
        "pause" => {
          try_send(Pause);
        }
        "resume" => {
          try_send(Resume);
        }
        "pr" => {
          try_send(PauseOrResume);
        }
        num if num.parse::<i32>().is_ok() => {
          // a really dumb thing to do and hopefully
          // if let guard can be stabilized in the future
          let num = num.parse::<i32>().unwrap();

          let index = match num {
            0 => {
              warn!("Index cannot be 0!");
              pause();
              try_send(Reprint);
              continue;
            }
            1..=i32::MAX => num - 1,
            i32::MIN..=-1 => songs_len as i32 + num,
          };

          if index.is_negative() || index >= songs_len as i32 {
            warn!("Invalid index!");
            pause();
            try_send(Reprint);
            continue;
          }

          try_send(IndexJump(index as usize));
        }
        _ => {
          warn!(
            "Unknown Command `{}`! Type in `help` for more information!",
            input
          );
          pause();
          try_send(Reprint);
        }
      }
    }
  }
}

enum Message {
  Pause,
  Resume,
  PauseOrResume,
  Reprint,
  IndexJump(usize),
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
