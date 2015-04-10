use std;
use std::mem;

use std::num::Int;

use channel;

pub struct Stream<'a> {
  last: bool,
  position: usize,
  length: usize,
  buffer: Vec<u8>,
  source: &'a mut channel::Source<super::Binary>
}

/// Streams are byte-oriented, and readable.
///
/// Unlike regular Readers, the Stream is much more likely to panic! instead of
/// returning an IoError. There's not much point for a codec to continue after
/// an IoError, so the idea is that the I/O layer instead handles reconnections
/// and so on automatically.
///
/// Instead of returning a specific error, the results are instead Options,
/// with a None signalling end of file.

impl<'a> Stream<'a> {
  pub fn new(source: &'a mut channel::Source<super::Binary>) -> Stream<'a> {
    return Stream { last: false, position: 0, length: 0, buffer: Vec::with_capacity(4096), source: source };
  }

  fn update_buffer(&mut self) {
    let s = &mut self.source;
    let b = &mut self.buffer;

    let mut eof = false;
    let mut len = 0;

    s.read(|binary| {
      eof = binary.last;
      len = binary.data.len();

      let l = b.len();

      if l < len {
        b.reserve(len - l);
      }

      let input = binary.data.slice(0, len);
      let output = b.slice_mut(0, len);

      std::slice::bytes::copy_memory(output, input);
    });

    self.position = 0;
    self.length = len;
    self.last = eof;
  }

  /// Read bytes, up to the length of `buffer` and place them in `buffer`.
  /// Returns the number of bytes read. The number of bytes read may be less
  /// than the number requested, even 0. Returns `None` on end of file.
  ///
  /// # Error
  ///
  /// If an error occurs during this I/O operation, then it should panic! the
  /// task. Note that reading 0 bytes is not considered an error.
  pub fn try_read(&mut self, buffer: &mut [u8]) -> Option<usize> {
    if self.position == self.length {
      if self.last {
        return None;
      } else {
        self.update_buffer();
      }
    }

    let write_len = std::cmp::min(buffer.len(), self.buffer.len() - self.position);

    {
        let input = self.buffer.slice(self.position, self.position + write_len);
        let output = buffer.slice_mut(0, write_len);

        assert_eq!(input.len(), output.len());

        std::slice::bytes::copy_memory(output, input);
    }

    self.position += write_len;

    assert!(self.position <= self.buffer.len());

    return Some(write_len);

  }

  /// Skips bytes, up to `amount`.
  /// Returns the number of bytes skipped. The number of bytes skipped may be
  /// less than the number requested, even 0. Returns `None` on end of file.
  ///
  /// # Error
  ///
  /// If an error occurs during this I/O operation, then it should panic! the
  /// task. Note that skipping 0 bytes is not considered an error.
  pub fn try_skip(&mut self, amount: usize) -> Option<usize> {
    if self.position == self.length {
      if self.last {
        return None;
      } else {
        self.update_buffer();
      }
    }

    let skip_len = std::cmp::min(amount, self.buffer.len() - self.position);

    self.position += skip_len;

    assert!(self.position <= self.buffer.len());

    return Some(skip_len);
  }

  /// Reads exactly the length of `buffer` and places them in `buffer`.
  pub fn read(&mut self, buffer: &mut [u8]) {
    let length = buffer.len();

    if self.read_at_least(length, buffer) != length {
      panic!("Stream: Unexpected length (BUG)");
    }
  }

  /// Skips exactly `amount` bytes.
  pub fn skip(&mut self, amount: usize) {
    let mut skipped = 0;

    while skipped < amount {
      match self.try_skip(amount) {
        Some(0) => panic!("Stream: Not progressing (TODO)"),
        Some(n) => skipped += n,
        None => panic!("Stream: Unexpected EOF (INPUT)")
      }
    }
  }

  /// Reads at least `min` bytes and places them in `buffer`.
  /// Returns the number of bytes read.
  ///
  /// This will continue to call `try_read` until at least `min` bytes have been
  /// read.
  pub fn read_at_least(&mut self, min: usize, buffer: &mut [u8]) -> usize {
    if min > buffer.len() { panic!("Stream: The buffer is too short (ARGUMENT)") }

    let mut read = 0;

    while read < min {
      match self.try_read(buffer.slice_from_mut(read)) {
        Some(0) => panic!("Stream: Not progressing (TODO)"),
        Some(n) => read += n,
        None => panic!("Stream: Unexpected EOF (INPUT)")
      }
    }

    return read;
  }

  /// Reads a u8.
  pub fn read_u8(&mut self) -> u8 {
    let mut buffer = [0];

    self.read(buffer);

    return buffer[0];
  }

  /// Reads a native endian u16
  pub fn read_ne_u16(&mut self) -> u16 {
    let mut buffer = [0, ..2];

    self.read(buffer);

    return unsafe { mem::transmute::<[u8; 2], [u16; 1]>(buffer) }[0];
  }

  /// Reads a big endian u16.
  pub fn read_be_u16(&mut self) -> u16 {
    return Int::from_be(self.read_ne_u16());
  }

  /// Reads a little endian u16.
  pub fn read_le_u16(&mut self) -> u16 {
    return Int::from_le(self.read_ne_u16());
  }

  /// Reads a native endian u32
  pub fn read_ne_u32(&mut self) -> u32 {
    let mut buffer = [0, ..4];

    self.read(buffer);

    return unsafe { mem::transmute::<[u8; 4], [u32; 1]>(buffer) }[0];
  }

  /// Reads a big endian u32.
  pub fn read_be_u32(&mut self) -> u32 {
    return Int::from_be(self.read_ne_u32());
  }

  /// Reads a little endian u32.
  pub fn read_le_u32(&mut self) -> u32 {
    return Int::from_le(self.read_ne_u32());
  }

  /// Reads a native endian u64
  pub fn read_ne_u64(&mut self) -> u64 {
    let mut buffer = [0, ..8];

    self.read(buffer);

    return unsafe { mem::transmute::<[u8; 8], [u64; 1]>(buffer) }[0];
  }

  /// Reads a big endian u64.
  pub fn read_be_u64(&mut self) -> u64 {
    return Int::from_be(self.read_ne_u64());
  }

  /// Reads a little endian u64.
  pub fn read_le_u64(&mut self) -> u64 {
    return Int::from_le(self.read_ne_u64());
  }

  /// Reads a i8
  pub fn read_i8(&mut self) -> i8 {
    return self.read_u8() as i8;
  }

  /// Reads a native endian u16
  pub fn read_ne_i16(&mut self) -> i16 {
    return self.read_ne_u16() as i16;
  }

  /// Reads a big endian u16.
  pub fn read_be_i16(&mut self) -> i16 {
    return self.read_be_u16() as i16;
  }

  /// Reads a little endian u16.
  pub fn read_le_i16(&mut self) -> i16 {
    return self.read_le_u16() as i16;
  }

  /// Reads a native endian u32
  pub fn read_ne_i32(&mut self) -> i32 {
    return self.read_ne_u32() as i32;
  }

  /// Reads a big endian u32.
  pub fn read_be_i32(&mut self) -> i32 {
    return self.read_be_u32() as i32;
  }

  /// Reads a little endian u32.
  pub fn read_le_i32(&mut self) -> i32 {
    return self.read_le_u32() as i32;
  }

  /// Reads a native endian u64
  pub fn read_ne_i64(&mut self) -> i64 {
    return self.read_ne_u64() as i64;
  }

  /// Reads a big endian u64.
  pub fn read_be_i64(&mut self) -> i64 {
    return self.read_be_u64() as i64;
  }

  /// Reads a little endian u64.
  pub fn read_le_i64(&mut self) -> i64 {
    return self.read_le_u64() as i64;
  }

  /// Reads `n` little-endian unsigned integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  pub fn read_le_usize_n(&mut self, n: usize) -> u64 {
      assert!(n > 0 && n <= 8);

      let mut result = 0u64;

      for i in 0..n {
        result = result | ((self.read_u8() as u64) << (8 * i));
      }

      return result;
  }

  /// Reads `n` little-endian signed integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  pub fn read_le_int_n(&mut self, n: usize) -> i64 {
    return extend_sign(self.read_le_usize_n(n), n);
  }

  /// Reads `n` big-endian unsigned integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  pub fn read_be_usize_n(&mut self, n: usize) -> u64 {
    assert!(n > 0 && n <= 8);

    let mut result = 0u64;

    for i in 0..n {
      result = result | ((self.read_u8() as u64) << (8 * (n - i - 1)));
    }

    return result;
  }

  /// Reads `n` big-endian signed integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  pub fn read_be_int_n(&mut self, n: usize) -> i64 {
    return extend_sign(self.read_be_usize_n(n), n);
  }
}

pub struct Bitstream<'a> {
  pub cache: u8, pub cache_length: usize, stream: &'a mut Stream<'a>
}

impl<'a> Bitstream<'a> {
  pub fn new(stream: &'a mut Stream<'a>) -> Bitstream<'a> {
    return Bitstream { cache: 0, cache_length: 0, stream: stream };
  }

  pub fn read_n(&mut self, n: usize) -> u32 {
    if n > 32 {
      panic!("Bitstream: You cannot request more than 32 bits into a u32 (ARGUMENT)");
    }

    if n <= self.cache_length {
      let result = self.cache >> (self.cache_length - n);

      self.cache_length -= n;
      self.cache = self.cache & (0xFF >> (8 - self.cache_length));

      return result as u32;
    } else {
      let n_to_read = n - self.cache_length;
      let b_to_read = n_to_read / 8 + if n_to_read % 8 > 0 { 1 } else { 0 };

      let read = self.stream.read_be_usize_n(b_to_read);
      let sum = ((self.cache as u64) << (b_to_read * 8)) | (read as u64);

      self.cache_length = b_to_read * 8 - n_to_read;

      let result = sum >> self.cache_length;

      self.cache = (sum & (0xFF >> (8 - self.cache_length))) as u8;

      return result as u32;
    }
  }

  pub fn read_n_signed(&mut self, n: usize) -> i32 {
    return extend_sign_bits(self.read_n(n) as u64, n) as i32;
  }
}

fn extend_sign(value: u64, n: usize) -> i64 {
  return extend_sign_bits(value, n * 8);
}

fn extend_sign_bits(value: u64, n: usize) -> i64 {
  let shift = 64 - n;

  return (value << shift) as i64 >> shift;
}

#[cfg(test)]
mod tests {
  use super::Stream;

  use channel;
  use buffer;

  macro_rules! prepare {
    ($buffer:expr) => ({
      let (sink, source) = channel::create::<::Binary>(1);

      thread::spawn(|| {
        buffer::Buffer::new($buffer, 4096, sink).run();
      });

      source
    });
  }

  #[test]
  fn test_read_u8() {
    let mut source = prepare!(vec![0x00u8, 0x01]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_u8(), 0);
    assert_eq!(s.read_u8(), 1);
  }

  #[test]
  fn test_read_u16() {
    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_u16(), 0x0100);
    assert_eq!(s.read_be_u16(), 0x0203);
  }

  #[test]
  fn test_read_u32() {
    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_u32(), 0x03020100);
    assert_eq!(s.read_be_u32(), 0x04050607);
  }

  #[test]
  fn test_read_u64() {
    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_u64(), 0x0706050403020100);
    assert_eq!(s.read_be_u64(), 0x08090A0B0C0D0E0F);
  }

  #[test]
  fn test_read_i8() {
    let mut source = prepare!(vec![0xFFu8]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_i8(), -1);
  }

  #[test]
  fn test_read_i16() {
    let mut source = prepare!(vec![0x00u8, 0xFF, 0xFF, 0x00]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_i16(), -0x100);
    assert_eq!(s.read_be_i16(), -0x100);
  }

  #[test]
  fn test_read_i32() {
    let mut source = prepare!(vec![0x00u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_i32(), -0x100);
    assert_eq!(s.read_be_i32(), -0x100);
  }

  #[test]
  fn test_read_i64() {
    let mut source = prepare!(vec![0x00u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_i64(), -0x100);
    assert_eq!(s.read_be_i64(), -0x100);
  }

  #[test]
  fn test_read_n() {
    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_usize_n(2), 0x0100);
    assert_eq!(s.read_be_usize_n(2), 0x0203);

    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_le_usize_n(1), 0x00);
    assert_eq!(s.read_le_usize_n(2), 0x0201);
    assert_eq!(s.read_le_usize_n(3), 0x050403);

    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);
    let mut s = Stream::new(&mut source);

    assert_eq!(s.read_be_usize_n(1), 0x00);
    assert_eq!(s.read_be_usize_n(2), 0x0102);
    assert_eq!(s.read_be_usize_n(3), 0x030405);
  }

  #[test]
  fn test_skip() {
    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03]);
    let mut s = Stream::new(&mut source);

    s.skip(1);
    assert_eq!(s.read_le_usize_n(3), 0x030201);

    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);
    let mut s = Stream::new(&mut source);

    s.skip(3);
    assert_eq!(s.read_le_usize_n(3), 0x050403);

    let mut source = prepare!(vec![0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05]);
    let mut s = Stream::new(&mut source);

    s.skip(3);
    assert_eq!(s.read_be_usize_n(3), 0x030405);
  }

  #[test]
  fn test_short_reads() {
    let mut source = prepare!(vec![0xFFu8, 0xAA, 0x44]);
    let mut s = Stream::new(&mut source);
    let mut r = super::Bitstream::new(&mut s);


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
    let mut source = prepare!(vec![0xFFu8, 0xAA, 0x44, 0xA3]);
    let mut s = Stream::new(&mut source);
    let mut r = super::Bitstream::new(&mut s);

    assert_eq!(r.read_n(16), 0xFFAA);
    assert_eq!(r.read_n(12), 0x44A);
    assert_eq!(r.read_n(4), 0x3);
  }

  #[test]
  fn test_large_reads() {
    let mut source = prepare!(vec![0xFFu8, 0xAA, 0x44, 0xA3, 0x34, 0x99, 0x44]);
    let mut s = Stream::new(&mut source);
    let mut r = super::Bitstream::new(&mut s);

    assert_eq!(r.read_n(24), 0xFFAA44);
    assert_eq!(r.read_n(32), 0xA3349944);
  }

  #[test]
  fn test_signed() {
    let mut source = prepare!(vec![0xFFu8, 0xAA, 0x44, 0xA3, 0x34]);
    let mut s = Stream::new(&mut source);
    let mut r = super::Bitstream::new(&mut s);

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
    let mut source = prepare!(vec![0xEAu8, 0xBD, 0x21]);
    let mut s = Stream::new(&mut source);
    let mut r = super::Bitstream::new(&mut s);

    assert_eq!(r.read_n(4), 0xE);
    assert_eq!(r.read_n(4), 0xA);
    assert_eq!(r.read_n(4), 0xB);
    assert_eq!(r.read_n(4), 0xD);
    assert_eq!(r.read_n(4), 0x2);
    assert_eq!(r.read_n(4), 0x1);
  }

  #[test]
  fn test_stream2() {
    let mut source = prepare!(vec![0x30u8, 0xC8, 0x61]);
    let mut s = Stream::new(&mut source);
    let mut r = super::Bitstream::new(&mut s);

    assert_eq!(r.read_n(6), 12);
    assert_eq!(r.read_n(6), 12);
    assert_eq!(r.read_n(6), 33);
    assert_eq!(r.read_n(6), 33);
  }
}
