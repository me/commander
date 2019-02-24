use std::io::prelude::*;
use std::io::{stdin, stdout, Write};

use termion::raw::IntoRawMode;

pub fn get_cursor_pos() -> (u16, u16) {
  let mut stdout = stdout()
    .into_raw_mode()
    .expect("curses:get_cursor_pos: failed to set stdout to raw mode");
  let mut f = stdin();
  write!(stdout, "\x1B[6n").expect("curses:get_cursor_pos: failed to write to stdout");
  stdout
    .flush()
    .expect("curses:get_cursor_pos: failed to flush stdout");

  let mut read_chars = Vec::new();
  loop {
    let mut buf = [0; 1];
    let _ = f.read(&mut buf);
    read_chars.push(buf[0]);
    if buf[0] == b'R' {
      break;
    }
  }
  let seq = String::from_utf8(read_chars).expect("curses:get_cursor_pos: invalid utf8 string read");
  let beg = seq
    .rfind('[')
    .expect("curses:get_cursor_pos: invalid control sequence");
  let coords: Vec<&str> = seq[(beg + 1)..seq.len() - 1].split(';').collect();

  stdout
    .flush()
    .expect("curses:get_cursor_pos: failed to flush stdout");

  let y = coords[0]
    .parse::<u16>()
    .expect("curses:get_cursor_pos: invalid position y");
  let x = coords[1]
    .parse::<u16>()
    .expect("curses:get_cursor_pos: invalid position x");

  (x, y)
}
