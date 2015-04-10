use std;
use std::io::Write;

use channel;

pub struct Input {
  file: std::fs::File, chunk: usize, sink: channel::Sink<super::Binary>
}

impl Input {
  pub fn new(file: std::fs::File, chunk: usize, sink: channel::Sink<super::Binary>) -> Input {
    return Input { file: file, chunk: chunk, sink: sink };
  }
  
  pub fn run(&mut self) {
    let f = &mut self.file;
    let c = self.chunk;

    let mut last = false;

    while !last {
      self.sink.write(|binary| {
        match f.push(c, &mut binary.data) {
          Ok(_) => {
            last = f.eof();
          }
          Err(_) => {
            last = true;
          }
        };

        binary.last = last;
      });
    }
  }
}

pub struct Output {
  file: std::fs::File, source: channel::Source<super::Binary>
}

impl Output {
  pub fn new(file: std::fs::File, source: channel::Source<super::Binary>) -> Output {
    return Output { file: file, source: source };
  }

  pub fn run(&mut self) {
    let f = &mut self.file;

    let mut last = false;

    while !last {
      self.source.read(|binary| {
        f.write(&binary.data).unwrap();

        last = binary.last;
      });
    }
  }
}

#[cfg(test)]
mod tests {
  use std;
  use channel;

  #[test]
  fn test_read_zero() {
    let (sink, mut source) = channel::create::<::Binary>(1);

    thread::spawn(|| {
      let path = std::path::Path::new("/dev/zero");
      let file = std::fs::File::open(&path).unwrap();

      super::Input::new(file, 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.last, false);
      assert_eq!(binary.data.len(), 4096);

      for i in 0us..4096 {
        assert_eq!(binary.data[i], 0);
      }
    });
  }

  #[test]
  fn test_read_null() {
    let (sink, mut source) = channel::create::<::Binary>(1);

    thread::spawn(|| {
      let path = std::path::Path::new("/dev/null");
      let file = std::fs::File::open(&path).unwrap();

      super::Input::new(file, 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.last, true);
      assert_eq!(binary.data.len(), 0);
    });
  }
}
