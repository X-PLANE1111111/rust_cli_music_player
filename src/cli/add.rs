use std::process;

use basic_quick_lib::{io_util::input_trim, time::LocalTime};
use chrono::Local;
use log::{error, info};

use crate::util::yt_downloader::YTDownload;

use super::data::PlaylistInfo;

#[derive(clap::Args)]
pub struct Add {
	link: String,
	playlist_name: String,
}

impl Add {
	pub fn handle(&mut self) {
		let mut playlist_info = match PlaylistInfo::load(&self.playlist_name) {
			Ok(v) => v,
			Err(err) => {
				error!(
					"Failed to find playlist \"{}\"! Error: {}",
					self.playlist_name, err
				);
				info!("Do you want to create a playlist instead? (y/N): ");

				let input = input_trim("");
				if input.to_lowercase() == "y" {
					PlaylistInfo {
						folder_name: self.playlist_name.clone(),
						name: self.playlist_name.clone(),
						created: Some(LocalTime(Local::now())),
						..Default::default()
					}
				} else {
					return;
				}
			}
		};

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
	}
}
