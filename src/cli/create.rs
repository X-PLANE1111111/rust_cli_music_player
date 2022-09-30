use clap::Args;
use log::info;

use crate::util::create_playlist;

#[derive(Args)]
pub struct Create {
    /// The new playlist's name
    playlist_name: String,
}

impl Create {
    pub fn handle(&self) {
        create_playlist(&self.playlist_name);
        info!("Created successfully");
    }
}
