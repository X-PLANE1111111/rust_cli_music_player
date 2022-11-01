use std::{fs, path::PathBuf, process};

use basic_quick_lib::{io_util::input_trim, time::LocalTime};
use chrono::Local;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::util::playlist_info_path;

pub const fn default_sound_multiplier() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistInfo {
    #[serde(skip)]
    pub name: String,

    #[serde(default)]
    pub songs: Vec<Song>,

    #[serde(default)]
    pub created: Option<LocalTime>,

    #[serde(skip)]
    pub folder_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Song {
    #[serde(default)]
    pub song_name: String,

    #[serde(default)]
    pub path_to_song: PathBuf,

    #[serde(default)]
    pub author: Option<String>,

    #[serde(default = "default_sound_multiplier")]
    pub sound_multiplier: f32,
}

impl PlaylistInfo {
    pub fn new(playlist_name: &str) -> Self {
        Self {
            name: playlist_name.to_string(),
            songs: Vec::new(),
            created: Some(LocalTime(Local::now())),
            folder_name: playlist_name.to_string(),
        }
    }

    pub fn load(playlist_name: &str) -> anyhow::Result<Self> {
        let path = playlist_info_path(playlist_name);
        let data = fs::read_to_string(&path)?;
        let mut this: PlaylistInfo = serde_json::from_str(&data)?;

        let paths = path.iter().collect::<Vec<_>>();
        let name = paths[paths.len() - 2].to_str().unwrap();

        this.name = name.to_string();
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

    pub fn load_or_create(playlist_name: &str) -> Self {
        match PlaylistInfo::load(playlist_name) {
            Ok(v) => v,
            Err(err) => {
                error!(
                    "Failed to find playlist \"{}\"! Error: {}",
                    playlist_name, err
                );
                info!("Do you want to create a playlist instead? (y/N): ");

                let input = input_trim("");
                if input.to_lowercase() == "y" {
                    PlaylistInfo {
                        folder_name: playlist_name.to_string(),
                        name: playlist_name.to_string(),
                        created: Some(LocalTime(Local::now())),
                        ..Default::default()
                    }
                } else {
                    process::exit(1);
                }
            }
        }
    }
}

impl Song {
    pub fn new(
        song_name: String,
        path: PathBuf,
        author: Option<String>,
        sound_multiplier: f32,
    ) -> Self {
        Self {
            song_name,
            path_to_song: path,
            author,
            sound_multiplier,
        }
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
                author: Some("Lucas Fan".to_string()),
                sound_multiplier: 2.0,
            }],
            created: Some(LocalTime(Local::now())),
            folder_name: "test".to_string(),
        };

        let json_string = serde_json::to_string_pretty(&info).unwrap_or_default();
        println!("{}", json_string);

        let info: PlaylistInfo = serde_json::from_str(json_string.as_str()).unwrap_or_default();

        assert_eq!(info.name, "a playlist".to_string());
    }

    #[test]
    fn write() {
        let info = PlaylistInfo {
            name: "a playlist".to_string(),
            songs: vec![Song {
                song_name: "song1".to_string(),
                path_to_song: PathBuf::from_iter([r"C:\", "test", "what"]),
                author: Some("Lucas Fan".to_string()),
                sound_multiplier: 2.0,
            }],
            created: Some(LocalTime(Local::now())),
            folder_name: "test".to_string(),
        };

        info.save();
    }
}
