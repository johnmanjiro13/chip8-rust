mod chip8;
mod display;

use clap::{arg, command};
use iced::{Application, Settings};
use log::LevelFilter;
use std::fs::File;
use std::io::Read;

use chip8::{Chip8, Flags};

fn main() {
    let matches = command!()
        .arg(arg!([FILE] "File of the chip-8 rom").required(true))
        .arg(arg!(--verbose "Show the detailed execution trace"))
        .get_matches();
    let file_name = matches.value_of("FILE").unwrap();
    let mut file = File::open(file_name).unwrap();
    let mut rom = vec![];
    file.read_to_end(&mut rom).unwrap();

    let is_verbose = matches.is_present("verbose");
    init_logger(is_verbose);

    let flags = Flags { rom };
    let mut settings = Settings::with_flags(flags);
    settings.window.size = (display::WIDTH as u32, display::HEIGHT as u32);
    Chip8::run(settings).unwrap();
}

fn init_logger(is_verbose: bool) {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Error)
        .level_for(
            "chip8_rust::chip8",
            if is_verbose {
                LevelFilter::Trace
            } else {
                LevelFilter::Error
            },
        )
        .chain(std::io::stderr())
        .apply()
        .unwrap();
}
