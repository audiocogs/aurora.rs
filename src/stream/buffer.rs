use std;

use stream::Stream;

pub struct MemoryStream {
  position: uint, buffer: Vec<u8>
}

impl MemoryStream {
  pub fn new(buffer: Vec<u8>) -> MemoryStream {
    return MemoryStream { position: 0, buffer: buffer };
  }
}

impl Stream for MemoryStream {
  fn try_read(&mut self, buffer: &mut [u8]) -> Option<uint> {
    if self.position >= self.buffer.len() { return None }

    let write_len = std::cmp::min(buffer.len(), self.buffer.len() - self.position);

    {
        let input = self.buffer.slice(self.position, self.position + write_len);
        let output = buffer.mut_slice(0, write_len);

        assert_eq!(input.len(), output.len());

        std::slice::bytes::copy_memory(output, input);
    }

    self.position += write_len;
    assert!(self.position <= self.buffer.len());

    return Some(write_len);
  }

  fn try_skip(&mut self, amount: uint) -> Option<uint> {
    if self.position >= self.buffer.len() { return None }

    let skip_len = std::cmp::min(amount, self.buffer.len() - self.position);

    self.position += skip_len;
    assert!(self.position <= self.buffer.len());

    return Some(skip_len);
  }
}

#[cfg(test)]
mod tests {
  use stream;
  use stream::Stream;
  use super::MemoryStream;

  #[test]
  fn test_read_u8() {
    let mut s = MemoryStream::new(vec![0x00u8, 0x01]);

    assert_eq!(s.read_u8(), 0);
    assert_eq!(s.read_u8(), 1);
  }

  #[test]
  fn test_read_u16() {
    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03]);

    assert_eq!(s.read_le_u16(), 0x0100);
    assert_eq!(s.read_be_u16(), 0x0203);
  }

  #[test]
  fn test_read_u32() {
    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);

    assert_eq!(s.read_le_u32(), 0x03020100);
    assert_eq!(s.read_be_u32(), 0x04050607);
  }

  #[test]
  fn test_read_u64() {
    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]);

    assert_eq!(s.read_le_u64(), 0x0706050403020100);
    assert_eq!(s.read_be_u64(), 0x08090A0B0C0D0E0F);
  }

  #[test]
  fn test_read_i8() {
    let mut s = MemoryStream::new(vec![0xFFu8]);

    assert_eq!(s.read_i8(), -1);
  }

  #[test]
  fn test_read_i16() {
    let mut s = MemoryStream::new(vec![0x00u8, 0xFF, 0xFF, 0x00]);

    assert_eq!(s.read_le_i16(), -0x100);
    assert_eq!(s.read_be_i16(), -0x100);
  }

  #[test]
  fn test_read_i32() {
    let mut s = MemoryStream::new(vec![0x00u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);

    assert_eq!(s.read_le_i32(), -0x100);
    assert_eq!(s.read_be_i32(), -0x100);
  }

  #[test]
  fn test_read_i64() {
    let mut s = MemoryStream::new(vec![0x00u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);

    assert_eq!(s.read_le_i64(), -0x100);
    assert_eq!(s.read_be_i64(), -0x100);
  }

  #[test]
  fn test_read_n() {
    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03]);

    assert_eq!(s.read_le_uint_n(2), 0x0100);
    assert_eq!(s.read_be_uint_n(2), 0x0203);

    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);

    assert_eq!(s.read_le_uint_n(1), 0x00);
    assert_eq!(s.read_le_uint_n(2), 0x0201);
    assert_eq!(s.read_le_uint_n(3), 0x050403);

    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);

    assert_eq!(s.read_be_uint_n(1), 0x00);
    assert_eq!(s.read_be_uint_n(2), 0x0102);
    assert_eq!(s.read_be_uint_n(3), 0x030405);
  }

  #[test]
  fn test_skip() {
    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03]);

    s.skip(1);
    assert_eq!(s.read_le_uint_n(3), 0x030201);

    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);

    s.skip(3);
    assert_eq!(s.read_le_uint_n(3), 0x050403);

    let mut s = MemoryStream::new(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);

    s.skip(3);
    assert_eq!(s.read_be_uint_n(3), 0x030405);
  }

  #[test]
  fn test_short_reads() {
    let mut s = MemoryStream::new(vec![0xFFu8, 0xAA, 0x44]);
    let mut r = stream::Bitstream::new(&mut s);

    assert_eq!(r.read_n(8), 0xFF);
    assert_eq!(r.read_n(4), 0x0A);
    assert_eq!(r.read_n(2), 0x02);
    assert_eq!(r.read_n(1), 0x01);
    assert_eq!(r.read_n(1), 0x00);
    assert_eq!(r.read_n(3), 0x02);
    assert_eq!(r.read_n(3), 0x01);
    assert_eq!(r.read_n(2), 0x00);
  }

  #[test]
  fn test_medium_reads() {
    let mut s = MemoryStream::new(vec![0xFFu8, 0xAA, 0x44, 0xA3]);
    let mut r = stream::Bitstream::new(&mut s);

    assert_eq!(r.read_n(16), 0xFFAA);
    assert_eq!(r.read_n(12), 0x44A);
    assert_eq!(r.read_n(4), 0x3);
  }

  #[test]
  fn test_large_reads() {
    let mut s = MemoryStream::new(vec![0xFFu8, 0xAA, 0x44, 0xA3, 0x34, 0x99, 0x44]);
    let mut r = stream::Bitstream::new(&mut s);

    assert_eq!(r.read_n(24), 0xFFAA44);
    assert_eq!(r.read_n(32), 0xA3349944);
  }

  #[test]
  fn test_signed() {
    let mut s = MemoryStream::new(vec![0xFFu8, 0xAA, 0x44, 0xA3, 0x34]);
    let mut r = stream::Bitstream::new(&mut s);

    assert_eq!(r.read_n_signed(1), -1);
    assert_eq!(r.read_n_signed(2), -1);
    assert_eq!(r.read_n_signed(3), -1);
    assert_eq!(r.read_n_signed(4), -2);
    assert_eq!(r.read_n_signed(6), -22);
    assert_eq!(r.read_n_signed(8), 68);
    assert_eq!(r.read_n_signed(16), -23756);
  }

  #[test]
  fn test_stream() {
    let mut s = MemoryStream::new(vec![0xEAu8, 0xBD, 0x21]);
    let mut r = stream::Bitstream::new(&mut s);

    assert_eq!(r.read_n(4), 0xE);
    assert_eq!(r.read_n(4), 0xA);
    assert_eq!(r.read_n(4), 0xB);
    assert_eq!(r.read_n(4), 0xD);
    assert_eq!(r.read_n(4), 0x2);
    assert_eq!(r.read_n(4), 0x1);
  }

  #[test]
  fn test_stream2() {
    let mut s = MemoryStream::new(vec![0x30u8, 0xC8, 0x61]);
    let mut r = stream::Bitstream::new(&mut s);

    assert_eq!(r.read_n(6), 12);
    assert_eq!(r.read_n(6), 12);
    assert_eq!(r.read_n(6), 33);
    assert_eq!(r.read_n(6), 33);
  }
}