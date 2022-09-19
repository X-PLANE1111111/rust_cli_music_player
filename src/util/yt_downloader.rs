use std::{path::PathBuf, process::Command, str::FromStr};

use anyhow::{anyhow, Context};
use basic_quick_lib::home_dir::home_dir;

use crate::cli::data::{default_sound_multiplier, Song};

use super::youtube_api;

const FAILED_TO_DOWNLOAD_HELP_MESSAGE: &str = r#"
Something went wrong while downloading. It may because of youtube-dl isn't downloaded. 
Please download it and add to the same directory where this program is located.
Go to http://ytdl-org.github.io/youtube-dl/download.html to download youtube-dl.
"#;

const DOWNLOADS_FOLDER: &str = "rust-cli-music-player_downloaded-audios";

pub struct YTDownload {
    pub output_file_name: String,
    pub audio_quality: u8,
    pub audio_extension: String,
    pub link: String,
}

#[allow(dead_code)]
impl YTDownload {
    #[must_use = "This only generates the config. Use `download` method to actually download it"]
    pub fn new(link: String) -> Self {
        Self {
            link,
            output_file_name: "%(title)s.%(ext)s".to_string(),
            audio_extension: "wav".to_string(),
            audio_quality: 0,
        }
    }

    #[must_use = "This only generates the config. Use `download` method to actually download it"]
    pub fn output_path(&mut self, name: String) -> &mut Self {
        self.output_file_name = name;
        self
    }

    #[must_use = "This only generates the config. Use `download` method to actually download it"]
    pub fn audio_quality(&mut self, audio_quality: u8) -> &mut Self {
        self.audio_quality = audio_quality;
        self
    }

    #[must_use = "This only generates the config. Use `download` method to actually download it"]
    pub fn audio_extension(&mut self, extension: String) -> &mut Self {
        self.audio_extension = extension;
        self
    }

    pub fn download(&self) -> anyhow::Result<()> {
        remove_cache_dir()?;

        let status = Command::new("youtube-dl")
            .args([
                "-f",
                "bestaudio/best",
                "-ciw",
                "-o",
                self.output_file_name.as_str(),
                "-v",
                "--extract-audio",
                "--audio-quality",
                self.audio_quality.to_string().as_str(),
                "--audio-format",
                self.audio_extension.as_str(),
                self.link.as_str(),
            ])
            .spawn()?
            .wait()
            .with_context(|| FAILED_TO_DOWNLOAD_HELP_MESSAGE)?;

        if !status.success() {
            return Err(anyhow!(
                "The process failed with status {}",
                status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "Unkown".to_string())
            ));
        }

        Ok(())
    }

    /// gets the video title and the video id
    pub fn get_info(&self) -> anyhow::Result<Song> {
        let json = self.get_json()?;
        let video = &json["items"][0];
        let id = video["id"]["videoId"].as_str().unwrap();
        let path = PathBuf::from_str(
            format!(
                "{}\\{}\\{}.{}",
                home_dir(),
                DOWNLOADS_FOLDER,
                id,
                self.audio_extension
            )
            .as_str(),
        )?;

        Ok(Song {
            author: video["snippet"]["channelTitle"]
                .as_str()
                .map(|s| s.to_string()),
            song_name: video["snippet"]["title"].as_str().unwrap().to_string(),
            path_to_song: path,
            sound_multiplier: default_sound_multiplier(),
        })
    }

    pub fn get_json(&self) -> anyhow::Result<serde_json::Value> {
        youtube_api::search(&self.link, 1)
    }
}

pub fn remove_cache_dir() -> anyhow::Result<()> {
    Command::new("youtube-dl")
        .arg("--rm-cache-dir")
        .spawn()?
        .wait()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::YTDownload;

    #[test]
    fn download_exit_failed() {
        assert!(
            YTDownload::new("https://www.youtube.com/watch?v=Ceqr4XIqzfa".to_string())
                .download()
                .is_err()
        );
    }

    #[test]
    fn get_json_test() {
        let yt_download =
            YTDownload::new("https://www.youtube.com/watch?v=80KXX8WbQ7k".to_string());
        let json = yt_download.get_json().unwrap();
        println!("{:#?}", json);
    }

    #[test]
    fn get_info_test() {
        let yt_download =
            YTDownload::new("https://www.youtube.com/watch?v=80KXX8WbQ7k".to_string());
        let song = yt_download.get_info().unwrap();
        println!("{:#?}", song);
    }
}
