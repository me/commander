use std::sync::mpsc::{Receiver, Sender};

use super::events::{FinderCommand, InterfaceCommand};
use super::matcher::Matcher;

pub struct Finder {
  events_tx: Sender<InterfaceCommand>,
  buffer: Vec<String>,
  query: Option<String>,
}

impl<'a> Finder {
  pub fn new(events_tx: Sender<InterfaceCommand>) -> Finder {
    Finder {
      events_tx: events_tx,
      buffer: Vec::new(),
      query: None,
    }
  }

  pub fn run(&mut self, reader_rx: Receiver<FinderCommand>) {
    let matcher = Matcher::new(10);
    loop {
      match reader_rx.recv() {
        Ok(FinderCommand::Refresh(data)) => {
          self.buffer = data;
          self.compute_matches(&matcher);
        }
        Ok(FinderCommand::Query(query)) => {
          self.query = Some(query);
          self.compute_matches(&matcher)
        }
        _ => {}
      }
    }
  }

  fn compute_matches(&self, matcher: &Matcher) {
    let results = matcher.run(&self.buffer, &self.query);
    self
      .events_tx
      .send(InterfaceCommand::Results(results))
      .unwrap();
  }
}
