use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use super::events::FinderCommand;

const TIMEOUT: Duration = Duration::from_millis(100);
const BUFFER_SIZE: usize = 100;

pub struct Reader {
  events_tx: Sender<FinderCommand>,
  buffer: Vec<String>,
  cnt: usize,
}

impl Reader {
  pub fn new(events_tx: Sender<FinderCommand>) -> Reader {
    Reader {
      events_tx: events_tx,
      buffer: Vec::with_capacity(BUFFER_SIZE),
      cnt: 0,
    }
  }

  pub fn run(&mut self, data_rx: Receiver<String>) {
    loop {
      match data_rx.recv_timeout(TIMEOUT) {
        Ok(line) => {
          self.buffer.push(line);
          self.cnt += 1;
          if self.cnt >= BUFFER_SIZE {
            self.send_buffer();
          }
        }
        Err(RecvTimeoutError::Disconnected) => {
          self.send_buffer();
          break;
        }
        _ => {
          self.send_buffer();
        }
      }
    }
  }

  fn send_buffer(&mut self) {
    if self.cnt == 0 {
      return;
    }
    self.cnt = 0;
    let _ = self
      .events_tx
      .send(FinderCommand::Refresh(self.buffer.clone()));
  }
}
