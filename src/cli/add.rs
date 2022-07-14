use std::{
    path::{Path, PathBuf},
    process,
    time::Instant,
};

use log::error;
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::util::yt_downloader::YTDownload;

use super::data::{default_sound_multiplier, PlaylistInfo, Song};

#[derive(clap::Args)]
pub struct Add {
    /// The link to the youtube music to download, or if --local/-l is on, then
    /// the path to the song
    link: String,

    /// The playlist to add the song into
    playlist_name: String,

    /// Add local music instead of download from youtube
    #[clap(short, long, action)]
    local: bool,
}

impl Add {
    pub fn handle(&self) {
        if self.local {
            self.add_local();
        } else {
            self.download_from_youtube();
        }
    }

    fn download_from_youtube(&self) {
        let start = Instant::now();

        let mut playlist_info =
            PlaylistInfo::load_or_create(&self.playlist_name);

        let mut download_config = YTDownload::new(self.link.clone());
        let song = download_config.get_info().unwrap_or_else(|err| {
            error!(
                "Something went wrong while trying to get video information! Error: {}",
                err
            );
            process::exit(1);
        });

        let mut path = song.path_to_song.clone();
        path.pop();
        let path =
            format!("{}\\%(title)s-%(id)s.%(ext)s", path.to_string_lossy());

        download_config
            .output_path(path)
            .download()
            .unwrap_or_else(|err| {
                error!("Something went wrong while downloading: {}", err)
            });

        playlist_info.songs.push(song);
        playlist_info.save();

        let end = Instant::now();

        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let _ = stdout
            .set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)));
        let _ = writeln!(
            &mut stdout,
            "Downloaded Successful! Took {} seconds",
            (end - start).as_secs()
        );
    }

    fn add_local(&self) {
        let path = Path::new(&self.link);

        let file_name = match path.file_name() {
            Some(s) => s,
            None => {
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                let _ = stdout.set_color(
                    ColorSpec::new().set_fg(Some(termcolor::Color::Green)),
                );
                let _ =
                    writeln!(&mut stdout, "error: song cannot be a directory");
                process::exit(1);
            }
        }
        .to_string_lossy();

        let song = Song {
            song_name: file_name.to_string(),
            path_to_song: PathBuf::from(path),
            author: None,
            sound_multiplier: default_sound_multiplier(),
        };

        let mut playlist_info =
            PlaylistInfo::load_or_create(&self.playlist_name);
        playlist_info.songs.push(song);
        playlist_info.save();
    }
}
