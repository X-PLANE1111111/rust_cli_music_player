use self::{add::Add, play::Play, search::Search, settings::ChangeSettings};

mod add;
pub mod data;
mod play;
mod search;
mod settings;

#[derive(clap::Parser)]
#[clap(
    author,
    version,
    about,
    long_about = "CLI music player because I'm too lazy to make a ui one lol"
)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Subcommands,
}

impl Cli {
    pub fn handle(&mut self) {
        match &mut self.commands {
            Subcommands::Play(play) => play.handle(),
            Subcommands::Add(add) => add.handle(),
            Subcommands::Setting(change_settings) => change_settings.handle(),
            Subcommands::Search(search) => search.handle(),
        }
    }
}

#[derive(clap::Subcommand)]
enum Subcommands {
    /// Play a playlist
    Play(Play),

    /// Add a music to a playlist. Can either be a youtube link or a local path
    Add(Add),

    /// Change the settings of the music player
    Setting(ChangeSettings),

    /// Search for music on youtube and download them
    Search(Search),
}
