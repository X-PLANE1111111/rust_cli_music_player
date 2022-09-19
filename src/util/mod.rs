use std::{path::PathBuf, str::FromStr, time::Instant};

use anyhow::Context;
use basic_quick_lib::home_dir::home_dir;
use log::info;
use rand::{seq::SliceRandom, thread_rng};
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cli::data::PlaylistInfo;

use self::yt_downloader::YTDownload;

pub mod settings;
pub mod youtube_api;
pub mod yt_downloader;

pub const PLAYLIST_DIR: &str = "rust-cli-music_player-playlists";

#[derive(thiserror::Error, Debug)]
pub enum GetIndexError {
    #[error("Index cannot be zero!")]
    IndexIsZero,

    #[error("{0} is not a valid index!")]
    InvalidIndex(i32),
}

#[derive(thiserror::Error, Debug)]
pub enum ToIndexError {
    #[error("Index is not a number")]
    IndexNotNumber,

    #[error("You need to specify an index")]
    NoIndex,

    #[error("Error converting index: {0}")]
    GetIndexError(GetIndexError),
}

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

pub fn help_print(command: &str, help_msg: &str) {
    info!("{:<30} --- {}", command, help_msg);
}

pub fn multiplied_volume(volume: u8, multiplier: f32) -> f32 {
    (volume as f32 * multiplier / 100.0).clamp(0.0, 1.0)
}

/// get the index (positive or negative, but can't be zero) and
/// return the actual index.
/// Basically used for converting negative index
pub fn get_index(index: i32, vec_len: usize) -> Result<usize, GetIndexError> {
    use GetIndexError::*;

    let vec_index = match index {
        0 => return Err(IndexIsZero),
        1..=i32::MAX => index - 1,
        i32::MIN..=-1 => vec_len as i32 + index,
    };

    if vec_index.is_negative() || vec_index >= vec_len as i32 {
        return Err(InvalidIndex(index));
    }

    Ok(vec_index as usize)
}

pub fn shuffle_vec(vec: &mut Vec<usize>, target_len: usize) {
    *vec = (0..target_len).collect();
    vec.shuffle(&mut thread_rng());
}

pub fn to_index(args: &[&str], index: usize, song_len: usize) -> Result<usize, ToIndexError> {
    use ToIndexError::*;
    let index = match args.get(index) {
        Some(i) => *i,
        None => {
            return Err(NoIndex);
        }
    };

    let index: i32 = match index.trim().parse() {
        Ok(i) => i,
        Err(_) => {
            return Err(IndexNotNumber);
        }
    };

    let index = match get_index(index, song_len) {
        Ok(i) => i,
        Err(e) => {
            return Err(GetIndexError(e));
        }
    };

    Ok(index)
}

pub fn add_from_youtube_link(playlist_name: &str, link: &str) -> anyhow::Result<()> {
    let start = Instant::now();

    let mut playlist_info = PlaylistInfo::load_or_create(playlist_name);

    let mut download_config = YTDownload::new(link.to_string());
    let song = download_config.get_info()?;

    let mut path = song.path_to_song.clone();
    path.pop();
    let path = format!("{}\\%(id)s.%(ext)s", path.to_string_lossy());

    download_config.output_path(path).download()?;

    playlist_info.songs.push(song);
    playlist_info.save();

    let end = Instant::now();

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let _ = stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)));
    let _ = writeln!(
        &mut stdout,
        "Added Successful! Took {} seconds",
        (end - start).as_secs()
    );

    Ok(())
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
