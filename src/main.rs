extern crate clap;
extern crate dirs;
extern crate failure;
extern crate serde_derive;
extern crate skim;

use clap::{App, Arg, ArgMatches};
use failure::Error;

mod commands;
mod options;
mod tldr;

// use skim::{Skim, SkimOptions};
// use std::default::Default;
// use std::io::Cursor;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    let args = get_args();
    let options = options::Options::from_args(&args);
    if options.update_sources {
        let updater = tldr::Updater::new(options.tldr_url, options.cache_path);
        match_result(updater.update());
    }
    // let options: SkimOptions = SkimOptions::default()
    //     .height("30%")
    //     .prompt("Search: ")
    //     .reverse(true);

    // let input = "aaaaa\nbbbb\nccc".to_string();

    // let selected_items = Skim::run_with(&options, Some(Box::new(Cursor::new(input))))
    //     .map(|out| out.selected_items)
    //     .unwrap_or_else(Vec::new);

    // for item in selected_items.iter() {
    //     print!("{}: {}{}", item.get_index(), item.get_output_text(), "\n");
    // }
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
        .arg(Arg::with_name("input").index(1).help("string to search"))
        .get_matches()
}
