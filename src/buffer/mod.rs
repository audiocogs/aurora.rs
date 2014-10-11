use std;

use channel;

pub struct Buffer {
  buffer: Vec<u8>, position: uint, chunk: uint, sink: channel::Sink<super::Binary>
}

impl Buffer {
  pub fn new(buffer: Vec<u8>, chunk: uint, sink: channel::Sink<super::Binary>) -> Buffer {
    return Buffer { buffer: buffer, position: 0, chunk: chunk, sink: sink };
  }

  pub fn run(&mut self) {
    let mut last = false;

    while !last {
      let write_len = std::cmp::min(self.chunk, self.buffer.len() - self.position);

      let start = self.position;
      let end = self.position + write_len;

      last = end >= self.buffer.len();

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
        binary.last = last;
      });

      self.position += write_len;
    }
  }
}

#[cfg(test)]
mod tests {
  use channel;

  #[test]
  fn test_read_zero() {
    let (sink, mut source) = channel::create::<::Binary>(1);

    spawn(proc() {
      super::Buffer::new(vec![0u8], 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.last, true);
      assert_eq!(binary.data.len(), 1);
      assert_eq!(binary.data[0], 0);
    });
  }

  #[test]
  fn test_read_null() {
    let (sink, mut source) = channel::create::<::Binary>(1);

    spawn(proc() {
      super::Buffer::new(vec![], 4096, sink).run();
    });

    source.read(|binary| {
      assert_eq!(binary.last, true);
      assert_eq!(binary.data.len(), 0);
    });
  }
}