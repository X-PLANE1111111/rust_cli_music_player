use std::{fs, path::PathBuf};

use basic_quick_lib::time::LocalTime;
use serde::{Deserialize, Serialize};

use crate::util::playlist_info_path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistInfo {
  pub name: String,
  pub songs: Vec<Song>,
  pub created: Option<LocalTime>,
  #[serde(skip)]
  pub folder_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Song {
  pub song_name: String,
  pub path_to_song: PathBuf,
  pub author: String,
}

impl PlaylistInfo {
  pub fn load(playlist_name: &str) -> anyhow::Result<Self> {
    let path = playlist_info_path(playlist_name);
    let data = fs::read_to_string(path)?;
    let mut this: PlaylistInfo = serde_json::from_str(&data)?;
    this.folder_name = playlist_name.to_string();

    Ok(this)
  }

  pub fn save(&self) {
    let json_string = serde_json::to_string_pretty(self).unwrap();

    let info_path = playlist_info_path(&self.folder_name);

    let mut path = info_path.clone();
    path.pop();

    fs::create_dir_all(&path).unwrap();
    fs::write(&info_path, json_string).unwrap();
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use basic_quick_lib::time::LocalTime;
  use chrono::Local;

  use super::{PlaylistInfo, Song};

  #[test]
  fn playlist_info_parse() {
    let info = PlaylistInfo {
      name: "a playlist".to_string(),
      songs: vec![Song {
        song_name: "song1".to_string(),
        path_to_song: PathBuf::from_iter([r"C:\", "test", "what"]),
        author: "Lucas Fan".to_string(),
      }],
      created: Some(LocalTime(Local::now())),
      folder_name: "test".to_string(),
    };

    let json_string = serde_json::to_string_pretty(&info).unwrap_or_default();
    println!("{}", json_string);

    let info: PlaylistInfo =
      serde_json::from_str(json_string.as_str()).unwrap_or_default();

    assert_eq!(info.name, "a playlist".to_string());
  }

  #[test]
  fn write() {
    let info = PlaylistInfo {
      name: "a playlist".to_string(),
      songs: vec![Song {
        song_name: "song1".to_string(),
        path_to_song: PathBuf::from_iter([r"C:\", "test", "what"]),
        author: "Lucas Fan".to_string(),
      }],
      created: Some(LocalTime(Local::now())),
      folder_name: "test".to_string(),
    };

    info.save();
  }
}
