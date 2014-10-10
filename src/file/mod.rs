use std;

use channel;

pub struct Input {
  file: std::io::File, chunk: uint, sink: channel::Sink<super::Binary>
}

impl Input {
  pub fn new(file: std::io::File, chunk: uint, sink: channel::Sink<super::Binary>) -> Input {
    return Input { file: file, chunk: chunk, sink: sink };
  }
  
  pub fn run(&mut self) {
    let f = &mut self.file;
    let c = self.chunk;

    let mut final = false;

    while !final {
      self.sink.write(|binary| {
        match f.push(c, &mut binary.data) {
          Ok(_) => {
            final = f.eof();
          }
          Err(_) => {
            final = true;
          }
        };

        binary.final = final;
      });
    }
  }
}

pub struct Output {
  file: std::io::File, source: channel::Source<super::Binary>
}

impl Output {
  pub fn new(file: std::io::File, source: channel::Source<super::Binary>) -> Output {
    return Output { file: file, source: source };
  }

  pub fn run(&mut self) {
    let f = &mut self.file;

    let mut final = false;

    while !final {
      self.source.read(|binary| {
        f.write(binary.data.as_slice()).unwrap();

        final = binary.final;
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

    spawn(proc() {
      let path = std::path::Path::new("/dev/zero");
      let file = std::io::File::open(&path).unwrap();
      
      super::Input::new(file, 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.final, false);
      assert_eq!(binary.data.len(), 4096);

      for i in range(0u, 4096) {
        assert_eq!(binary.data[i], 0);
      }
    });
  }

  #[test]
  fn test_read_null() {
    let (sink, mut source) = channel::create::<::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("/dev/null");
      let file = std::io::File::open(&path).unwrap();
      
      super::Input::new(file, 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.final, true);
      assert_eq!(binary.data.len(), 0);
    });
  }
}