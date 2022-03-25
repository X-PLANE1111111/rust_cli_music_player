use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use basic_quick_lib::home_dir::home_dir;

pub mod yt_downloader;
pub mod settings;

pub const PLAYLIST_DIR: &str = "rust-cli-music_player-playlists";

pub fn playlist_info_path(playlist_name: &str) -> PathBuf {
  let mut path_to_json = PathBuf::from_str(home_dir().as_str())
    .with_context(|| "Failed to load home dir as path buf")
    .unwrap();

  path_to_json.push(PLAYLIST_DIR);
  path_to_json.push(playlist_name);
  path_to_json.push("info");
  path_to_json.set_extension("json");

  path_to_json
}

#[cfg(test)]
mod test {
  use super::playlist_info_path;

  #[test]
  fn playlist_info_path_test() {
    let test = playlist_info_path("test");
    println!("{:?}", test);
  }
}
