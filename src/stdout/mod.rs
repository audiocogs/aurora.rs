use std::io;

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
    let mut stdout = io::stdio::stdout();

    while !last {
      self.source.read(|binary| {
        stdout.write(binary.data.as_slice()).unwrap();

        last = binary.last;
      });
    }
  }
}