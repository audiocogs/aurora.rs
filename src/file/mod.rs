use std;

use stream;
use channel;

pub struct File {
  file: std::io::File, chunk: uint, sink: channel::Sink<stream::Binary>
}

impl File {
  pub fn new(file: std::io::File, chunk: uint, sink: channel::Sink<stream::Binary>) -> File {
    return File { file: file, chunk: chunk, sink: sink };
  }
  
  pub fn run(&mut self) {
    let f = &mut self.file;
    let c = self.chunk;

    loop {
      self.sink.write(|binary| {
        match f.push(c, &mut binary.data) {
          Ok(_) => {
            binary.end_of_file = f.eof();
          }
          Err(_) => {
            binary.end_of_file = true;
          }
        };
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use std;
  use channel;
  use stream;

  #[test]
  fn test_read_zero() {
    let (sink, mut source) = channel::create::<stream::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("/dev/zero");
      let file = std::io::File::open(&path).unwrap();
      
      super::File::new(file, 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.end_of_file, false);
      assert_eq!(binary.data.len(), 4096);

      for i in range(0u, 4096) {
        assert_eq!(binary.data[i], 0);
      }
    });
  }

  #[test]
  fn test_read_null() {
    let (sink, mut source) = channel::create::<stream::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("/dev/null");
      let file = std::io::File::open(&path).unwrap();
      
      super::File::new(file, 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.end_of_file, true);
      assert_eq!(binary.data.len(), 0);
    });
  }
}