use std::{cell::Cell, process, sync::mpsc, thread, time::Duration};

use anyhow::Context;
use basic_quick_lib::{cli_util::pause, io_util::input_trim};
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

    thread::spawn(move || {
      let mut sl = Soloud::default()
        .with_context(|| "Failed to get player!")
        .unwrap();

      let mut wav = Wav::default();

      let mut currently_playing = 0;

      loop {
        let Song {
          song_name,
          path_to_song,
          author,
        } = &songs[currently_playing];

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

          info!("Currently playing \"{}\"", song_name);
          info!("Made by \"{}\"", author);

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

        while sl.voice_count() > 0 {
          // pause the loop a little so it won't take too much cpu power
          thread::sleep(SLEEP_DURATION);

          if is_paused.get() {
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
            }

            print_info();
          }
        }

        match SETTINGS.playback_mode {
          PlaybackMode::Sequel => {
            currently_playing += 1;
            if currently_playing >= songs.len() {
              break;
            }
          }
          PlaybackMode::LoopOnce => {}
          PlaybackMode::LoopPlaylist => {
            currently_playing += 1;
            if currently_playing >= songs.len() {
              currently_playing = 0;
            }
          }
          PlaybackMode::Random => {}
        }
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
