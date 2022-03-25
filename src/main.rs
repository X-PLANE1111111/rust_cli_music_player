use clap::StructOpt;
use cli::Cli;
use colored::Colorize;
use log::{Level, LevelFilter};

use anyhow::{Context, Result};

pub mod cli;
mod util;

#[cfg(debug_assertions)]
const LEVEL_FILTER: LevelFilter = LevelFilter::Trace;
#[cfg(not(debug_assertions))]
const LEVEL_FILTER: LevelFilter = LevelFilter::Info;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn setup_logger() -> Result<(), fern::InitError> {
  fern::Dispatch::new()
    .filter(|data| {
      if !data.target().contains(PACKAGE_NAME) {
        return false;
      }
      true
    })
    .format(|out, message, record| {
      let color = match record.level() {
        Level::Warn => "yellow",
        Level::Error => "red",
        Level::Debug => "purple",
        Level::Trace => "blue",
        Level::Info => "green",
      };

      #[cfg(debug_assertions)]
      out.finish(format_args!(
        "[{}(:{}) {}]: {}",
        record.file().unwrap_or("unknown"),
        record.line().unwrap_or(0),
        record.level().as_str().color(color),
        message
      ));

      #[cfg(not(debug_assertions))]
      out.finish(format_args!(
        "[{}]: {}",
        record.level().as_str().color(color),
        message
      ));
    })
    .level(LEVEL_FILTER)
    .chain(std::io::stdout())
    .apply()?;

  Ok(())
}

fn main() {
  setup_logger()
    .with_context(|| "Failed to set up logger!")
    .unwrap();

  let mut cli = Cli::parse();
  cli.handle();

  // yt_downloader::download(
  //   "test.%(ext)s".to_string(),
  //   0,
  //   "flac".to_string(),
  //   "https://www.youtube.com/watch?v=oRMJ-UR9_Dk".to_string(), // )
  // .with_context(|| FAILED_TO_DOWNLOAD_HELP_MESSAGE)?;
}
