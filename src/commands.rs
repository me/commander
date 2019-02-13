use std::ffi::OsString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::PathBuf;

use std::sync::mpsc::SyncSender;

pub const EXTENSION: &str = "commands";

pub struct Commands {
  path: PathBuf,
}

impl Commands {
  pub fn new(path: OsString) -> Commands {
    let path = PathBuf::from(path);
    Commands { path }
  }

  pub fn parse(&self, tx: &SyncSender<u8>) {
    let _ = Commands::parse_dir(&self.path, tx);
  }

  fn parse_dir(dir: &PathBuf, tx: &SyncSender<u8>) -> io::Result<()> {
    if dir.is_dir() {
      for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
          let _ = Commands::parse_dir(&path, tx);
        } else if entry.path().extension().unwrap_or(&OsString::from("")) == EXTENSION {
          let f = File::open(entry.path());
          if let Ok(f) = f {
            let f = BufReader::new(f);
            for b in f.bytes() {
              if let Ok(b) = b {
                let _ = tx.send(b);
              }
            }
          }
        }
      }
    }
    Ok(())
  }
}
