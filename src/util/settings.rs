use std::fs;

use basic_quick_lib::home_dir::home_dir;
use log::trace;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static SETTINGS: Lazy<Settings> =
  Lazy::new(|| Settings::read_settings().unwrap_or_default());

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
  pub fn read_settings() -> anyhow::Result<Self> {
    let settings_path = format!("{}\\{}", home_dir(), SETTINGS_FILE);

    let this = serde_json::from_str::<Self>(
      fs::read_to_string(&settings_path)?.as_str(),
    )?;

    Ok(this)
  }
}

impl Drop for Settings {
  fn drop(&mut self) {
    trace!("Settings dropped!");
  }
}
