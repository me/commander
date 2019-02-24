use std::io::{stdin, stdout, Write};
use std::sync::mpsc::{Receiver, Sender};

use termion::event::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use super::curses;
use super::events::{FinderCommand, InterfaceCommand};
use super::matcher::MatchResult;
use super::options::Options;

const SEARCH_PROMPT: &str = "Search: > ";

type RawStdout = termion::raw::RawTerminal<std::io::Stdout>;

pub struct Interface {
  finder_tx: Sender<FinderCommand>,
  lines: u16,
  base_y: u16,
}

impl Interface {
  pub fn new(options: &Options, finder_tx: Sender<FinderCommand>) -> Interface {
    Interface {
      finder_tx: finder_tx,
      lines: options.height_lines as u16,
      base_y: 0,
    }
  }

  pub fn init(&mut self) {
    for _ in 0..=self.lines {
      println!("");
    }
    let (_, y) = curses::get_cursor_pos();
    self.base_y = y - self.lines - 1;
    let mut stdout = stdout().into_raw_mode().expect("Unable to open stdout.");
    self.update_query(&String::new(), &mut stdout);
    stdout.flush().unwrap();
  }

  pub fn events_loop(&self, events_rx: Receiver<InterfaceCommand>) {
    let mut stdout = stdout().into_raw_mode().expect("Unable to open stdout.");
    loop {
      match events_rx.recv() {
        Ok(InterfaceCommand::Results(results)) => self.update_results(&results, &mut stdout),
        Ok(InterfaceCommand::Query(query)) => {
          self.update_query(&query, &mut stdout);
          self
            .finder_tx
            .send(FinderCommand::Query(query))
            .expect("Unable to send finder command");
        }
        Ok(InterfaceCommand::Stop) => {
          self.cleanup(&mut stdout);
          break;
        }
        _ => {}
      }
    }
  }

  fn goto_line(&self, line: u16, stdout: &mut RawStdout) {
    write!(stdout, "{}", termion::cursor::Goto(1, self.base_y + line))
      .expect("Unable to move cursor");
  }

  fn update_query(&self, query: &str, stdout: &mut RawStdout) {
    let full_prompt = String::from(SEARCH_PROMPT) + query;
    self.goto_line(0, stdout);
    write!(stdout, "{}{}", termion::clear::CurrentLine, full_prompt)
      .expect("Unable to write query");
  }

  fn update_results(&self, results: &[MatchResult], stdout: &mut RawStdout) {
    write!(stdout, "{}", termion::cursor::Save).expect("Unable to save cursor");
    for i in 1..=self.lines + 1 {
      self.goto_line(i, stdout);
      match results.get((i - 1) as usize) {
        Some(result) => write!(stdout, "{}{}", termion::clear::CurrentLine, result.line)
          .expect("Unable to write result line"),
        _ => write!(stdout, "{}", termion::clear::CurrentLine).expect("Unable to write empty line"),
      }
    }
    write!(stdout, "{}", termion::cursor::Restore).expect("Unable to restore cursor");
    stdout.flush().unwrap();
  }

  fn cleanup(&self, stdout: &mut RawStdout) {
    for i in 0..=self.lines {
      self.goto_line(i as u16, stdout);
      write!(stdout, "{}", termion::clear::CurrentLine).unwrap();
    }
    self.goto_line(0, stdout);
    stdout.flush().unwrap();
  }
}

pub struct CommandReader {
  interface_tx: Sender<InterfaceCommand>,
}

impl CommandReader {
  pub fn new(interface_tx: Sender<InterfaceCommand>) -> CommandReader {
    CommandReader {
      interface_tx: interface_tx,
    }
  }

  pub fn input_loop(&self) {
    let stdin = stdin();
    let mut query: String = String::new();
    for c in stdin.keys() {
      match c {
        Ok(Key::Char(c)) => {
          query.push(c);
          self
            .interface_tx
            .send(InterfaceCommand::Query(query.clone()))
            .unwrap();
        }
        Ok(Key::Backspace) => {
          query.pop();
          self
            .interface_tx
            .send(InterfaceCommand::Query(query.clone()))
            .unwrap();
        }
        Ok(Key::Ctrl('c')) => {
          self.interface_tx.send(InterfaceCommand::Stop).unwrap();
          break;
        }
        _ => {}
      }
    }
  }
}
