use std::io;

use channel;

pub struct Output {
  source: channel::Source<::Audio>
}

impl Output {
  pub fn new(source: channel::Source<::Audio>) -> Output {
    return Output { source: source };
  }

  pub fn run(&mut self) {
    let mut final = false;
    let mut stdout = io::stdio::stdout();

    while !final {
      self.source.read(|audio| {
        stdout.write(audio.data.as_slice()).unwrap();

        final = audio.final;
      });
    }
  }
}