use std::ffi::OsString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::PathBuf;

use std::sync::mpsc::Sender;

pub const EXTENSION: &str = "commands";

pub struct Commands {
  path: PathBuf,
}

impl Commands {
  pub fn new(path: OsString) -> Commands {
    let path = PathBuf::from(path);
    Commands { path }
  }

  pub fn parse(&self, tx: &Sender<String>) {
    let _ = Commands::parse_dir(&self.path, tx);
  }

  fn parse_dir(dir: &PathBuf, tx: &Sender<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_dir() {
        let _ = Commands::parse_dir(&path, tx);
      } else if entry.path().extension().unwrap_or(&OsString::from("")) == EXTENSION {
        let f = File::open(entry.path());
        if let Ok(f) = f {
          let f = BufReader::new(f);
          for line in f.lines() {
            if let Ok(line) = line {
              let _ = tx.send(line);
            }
          }
        }
      }
    }
    Ok(())
  }
}
