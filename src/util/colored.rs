use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn write(color: &ColorSpec, text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(color).unwrap();
    write!(&mut stdout, "{}", text).unwrap();
    stdout.reset().unwrap();
}

pub fn writeln(color: &ColorSpec, text: &str) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(color).unwrap();
    writeln!(&mut stdout, "{}", text).unwrap();
    stdout.reset().unwrap();
}
