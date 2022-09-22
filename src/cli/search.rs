use std::{
    io::{stdout, Write},
    num::NonZeroUsize,
    process,
};

use basic_quick_lib::io_util::input_other_repeat;
use colored::Colorize;
use log::{error, info};

use crate::util::youtube_api;

#[derive(clap::Args)]
pub struct Search {
    /// The search term
    query: String,

    /// The playlist which the music is going to be added
    #[clap(short, long)]
    add_to: String,
}

impl Search {
    pub fn handle(&self) {
        let result = match youtube_api::search(&self.query, 10) {
            Ok(r) => r,
            Err(e) => {
                error!("Cannot search youtube. Error: {}", e);
                process::exit(1);
            }
        };

        let videos = result["items"].as_array().unwrap();

        for (index, video) in videos.iter().enumerate() {
            let channel_info =
                format!(" - {}", video["snippet"]["channelTitle"].as_str().unwrap()).bold();

            info!(
                "{}. {}{}",
                index + 1,
                video["snippet"]["title"].as_str().unwrap(),
                channel_info
            );
        }

        let input: NonZeroUsize = input_other_repeat("Type which one to download: ");
        let selected_video = &videos[input.get() - 1];
        let video_id = selected_video["id"]["videoId"].as_str().unwrap();
    }
}
