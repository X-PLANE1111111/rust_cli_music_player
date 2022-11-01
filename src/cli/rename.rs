use std::fs;

use clap::Args;
use log::{error, info};

use crate::util::playlist_info_folder;

#[derive(Args)]
pub struct Rename {
    /// The playlist name that you want to rename
    playlist_name: String,

    /// The new name for the playlist
    new_name: String,
}

impl Rename {
    pub fn handle(&self) {
        let path = playlist_info_folder(&self.playlist_name);
        let new_path = playlist_info_folder(&self.new_name);

        println!("path: {:?}, new_path: {:?}", path, new_path);

        if let Err(e) = fs::rename(&path, &new_path) {
            error!("Failed to rename! Error: {e}");
            return;
        }

        info!("Renamed successful!");
    }
}
