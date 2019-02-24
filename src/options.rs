use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::ArgMatches;
use dirs;

#[derive(Debug)]
pub struct Options<'a> {
  pub update_sources: bool,
  pub cache_path: OsString,
  pub tldr_url: Option<&'a str>,
  pub height_lines: usize,
}

impl<'a> Options<'a> {
  pub fn from_args(args: &'a ArgMatches) -> Options<'a> {
    let update_sources = args.is_present("update-sources");
    let cache_path = args
      .value_of("cache-path")
      .map(OsString::from)
      .unwrap_or_else(|| {
        let data_home = env::var_os("XDG_DATA_HOME")
          .map(PathBuf::from)
          .unwrap_or_else(|| {
            let home = dirs::home_dir().expect("Could not find home directory.");
            home.join(".local/share")
          });
        data_home.join(crate::APP_NAME).into_os_string()
      });
    let tldr_url = args.value_of("tldr-url");
    let height_lines = args
      .value_of("height-lines")
      .and_then(|v| v.parse::<usize>().ok())
      .unwrap_or(10);
    Options {
      update_sources,
      cache_path,
      tldr_url,
      height_lines,
    }
  }
}
