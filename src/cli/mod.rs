use self::{add::Add, play::Play};

mod add;
pub mod data;
mod play;

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
    }
  }
}

#[derive(clap::Subcommand)]
enum Subcommands {
  Play(Play),
  Add(Add),
}
