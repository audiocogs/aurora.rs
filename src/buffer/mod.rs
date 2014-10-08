use std;
use stream;
use channel;

pub struct Buffer {
  buffer: Vec<u8>, position: uint, chunk: uint, sink: channel::Sink<stream::Binary>
}

impl Buffer {
  pub fn new(buffer: Vec<u8>, chunk: uint, sink: channel::Sink<stream::Binary>) -> Buffer {
    return Buffer { buffer: buffer, position: 0, chunk: chunk, sink: sink };
  }

  pub fn run(&mut self) {
    loop {
      let write_len = std::cmp::min(self.chunk, self.buffer.len() - self.position);

      let start = self.position;
      let end = self.position + write_len;
      let b = &self.buffer;

      self.sink.write(|binary| {
        let len = binary.data.len();

        if len < write_len {
          binary.data.grow(write_len - len, 0);
        }

        {
          let input = b.slice(start, end);
          let output = binary.data.slice_mut(0, write_len);

          std::slice::bytes::copy_memory(output, input);
        }

        binary.data.truncate(write_len);
        binary.end_of_file = end >= b.len();
      });

      self.position += write_len;
    }
  }
}

#[cfg(test)]
mod tests {
  use channel;
  use stream;

  #[test]
  fn test_read_zero() {
    let (sink, mut source) = channel::create::<stream::Binary>(1);

    spawn(proc() {
      super::Buffer::new(vec![0u8], 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.end_of_file, true);
      assert_eq!(binary.data.len(), 1);
      assert_eq!(binary.data[0], 0);
    });
  }

  #[test]
  fn test_read_null() {
    let (sink, mut source) = channel::create::<stream::Binary>(1);

    spawn(proc() {
      super::Buffer::new(vec![], 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.end_of_file, true);
      assert_eq!(binary.data.len(), 0);
    });
  }
}