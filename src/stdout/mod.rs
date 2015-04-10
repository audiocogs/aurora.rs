use std::io;
use std::io::Write;

use channel;

pub struct Output {
  source: channel::Source<::Binary>
}

impl Output {
  pub fn new(source: channel::Source<::Binary>) -> Output {
    return Output { source: source };
  }

  pub fn run(&mut self) {
    let mut last = false;
    let mut stdout = io::stdout();

    while !last {
      self.source.read(|binary| {
        stdout.write(&binary.data).unwrap();

        last = binary.last;
      });
    }
  }
}
