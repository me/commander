extern crate clap;
extern crate dirs;
extern crate failure;
extern crate futures;
extern crate serde_derive;
extern crate termion;

use std::sync::mpsc::channel;
use std::thread;

use clap::{App, Arg, ArgMatches};
use failure::Error;

mod commands;
mod curses;
mod events;
mod finder;
mod interface;
mod matcher;
mod options;
mod reader;
mod tldr;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> Result<(), std::io::Error> {
    let args = get_args();
    let options = options::Options::from_args(&args);
    if options.update_sources {
        let updater = tldr::Updater::new(options.tldr_url, options.cache_path.clone());
        match_result(updater.update());
    }
    let (lines_tx, lines_rx) = channel();
    let commands = commands::Commands::new(options.cache_path.clone());

    thread::spawn(move || commands.parse(&lines_tx));
    let (finder_tx, finder_rx) = channel();
    let mut reader = reader::Reader::new(finder_tx.clone());
    thread::spawn(move || {
        reader.run(lines_rx);
    });

    let (interface_tx, interface_rx) = channel();

    let mut finder = finder::Finder::new(interface_tx.clone());
    thread::spawn(move || {
        finder.run(finder_rx);
    });

    let mut interface = interface::Interface::new(&options, finder_tx.clone());
    interface.init();
    let interface_thread = thread::spawn(move || interface.events_loop(interface_rx));

    let command_reader = interface::CommandReader::new(interface_tx);
    command_reader.input_loop();

    interface_thread
        .join()
        .expect("Unable to join interface thread.");

    Ok(())
}

fn match_result<T>(res: Result<T, Error>) {
    match res {
        Ok(_) => (),
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn get_args<'a>() -> ArgMatches<'a> {
    App::new(APP_NAME)
        .version(VERSION)
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("update-sources")
                .long("update-sources")
                .short("U")
                .help("Update external command sources (TLDR)"),
        )
        .arg(
            Arg::with_name("cache-path")
                .long("cache-path")
                .short("p")
                .help("Path where commands are kept"),
        )
        .arg(
            Arg::with_name("tldr-url")
                .long("tldr-url")
                .help("URL to fetch the TLDR archive"),
        )
        .arg(
            Arg::with_name("height-lines")
                .long("height")
                .help("Lines for the preview window"),
        )
        .arg(Arg::with_name("input").index(1).help("string to search"))
        .get_matches()
}
