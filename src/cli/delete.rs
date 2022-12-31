use std::fs;

use clap::Args;

use crate::util::playlist_info_folder;

#[derive(Args)]
pub struct Delete {
    /// The playlist name to delete
    playlist_name: String,
}

impl Delete {
    pub fn handle(&self) {
        let folder_path = playlist_info_folder(&self.playlist_name);

        if let Err(e) = fs::remove_dir_all(folder_path) {
            println!("Failed to remove playlist! Error: {}", e);
            return;
        }

        println!("Removed playlist successfully!");
    }
}
