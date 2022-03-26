use std::{fs, str::FromStr, sync::Mutex};

use anyhow::anyhow;
use basic_quick_lib::home_dir::home_dir;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static SETTINGS: Lazy<Mutex<Settings>> =
  Lazy::new(|| Mutex::new(Settings::read_settings().unwrap_or_default()));

const SETTINGS_FILE: &str = "rust-cli-music-player_settings.json";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackMode {
  Sequel,
  LoopOnce,
  LoopPlaylist,
  Random,
}

impl Default for PlaybackMode {
  fn default() -> Self {
    Self::LoopPlaylist
  }
}

impl FromStr for PlaybackMode {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let lower_case = s.to_lowercase();

    match lower_case.as_str() {
      "sequel" => Ok(Self::Sequel),
      "looponce" => Ok(Self::LoopOnce),
      "loopplaylist" => Ok(Self::LoopPlaylist),
      "random" => Ok(Self::LoopPlaylist),
      _ => Err(anyhow!("Unknown playback mode!")),
    }
  }
}

impl ToString for PlaybackMode {
  fn to_string(&self) -> String {
    match self {
      Self::Sequel => "Sequel",
      Self::LoopOnce => "Loop Once",
      Self::LoopPlaylist => "Loop Playlist",
      Self::Random => "Random",
    }
    .to_string()
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Settings {
  pub volume: u8,
  pub playback_mode: PlaybackMode,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      volume: 30,
      playback_mode: Default::default(),
    }
  }
}

impl Settings {
  fn path() -> String {
    format!("{}\\{}", home_dir(), SETTINGS_FILE)
  }

  pub fn read_settings() -> anyhow::Result<Self> {
    let settings_path = Self::path();

    let this = serde_json::from_str::<Self>(
      fs::read_to_string(&settings_path)?.as_str(),
    )?;

    Ok(this)
  }

  pub fn save(&self) -> anyhow::Result<()> {
    let json_str = serde_json::to_string_pretty(self)?;
    fs::write(Self::path(), json_str)?;

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::Settings;

  #[test]
  fn save() -> anyhow::Result<()> {
    let data = Settings::default();
    data.save()?;

    Ok(())
  }
}
