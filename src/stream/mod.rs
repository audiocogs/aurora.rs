use std::mem;

pub mod buffer;

pub struct Binary {
  pub end_of_file: bool,
  pub data: Vec<u8>
}

impl ::Initialize for Binary {
  fn initialize(&mut self) {
    self.end_of_file = false;
    self.data.truncate(0);
  }
}

/// A trait for objects which are byte-oriented readable streams. Streams are
/// defined by two methods, `read`, and `bus`. This function will block until
/// data is available, filling in the provided buffer with any data read.
///
/// If it does block for a significant time (more than ~100ms) then it will
/// notify the Aurora bus that the stream is starved, this prepares the
/// framework that there might be interruptions soon.
///
/// Unlike regular Readers, the Stream is much more likely to fail! instead of
/// returning an IoError. There's not much point for a codec to continue after
/// an IoError, the idea is that the I/O layer instead handles reconnections
/// and so on automatically.
///
/// Instead of returning a specific error, the results are instead Options,
/// with a None signalling end of file.

pub trait Stream {
  /// Read bytes, up to the length of `buffer` and place them in `buffer`.
  /// Returns the number of bytes read. The number of bytes read may be less
  /// than the number requested, even 0. Returns `None` on end of file.
  ///
  /// # Error
  ///
  /// If an error occurs during this I/O operation, then it should fail! the
  /// task. Note that reading 0 bytes is not considered an error.
  ///
  /// # Implementation Note
  ///
  /// When implementing this method on a new Stream, you are strongly
  /// encouraged not to return 0 if you can avoid it.
  fn try_read(&mut self, buffer: &mut [u8]) -> Option<uint>;
  fn try_skip(&mut self, amount: uint) -> Option<uint>;

  // Convenient helper methods based on the above methods

  /// Reads exactly the length of `buffer` and places them in `buffer`.
  fn read(&mut self, buffer: &mut [u8]) {
    let length = buffer.len();

    if self.read_at_least(length, buffer) != length {
      fail!("Stream: Unexpected length (BUG)");
    }
  }

  /// Skips exactly `amount` bytes.
  fn skip(&mut self, amount: uint) {
    let mut skipped = 0;

    while skipped < amount {
      match self.try_skip(amount) {
        Some(0) => fail!("Stream: Not progressing (TODO)"),
        Some(n) => skipped += n,
        None => fail!("Stream: Unexpected EOF (INPUT)")
      }
    }
  }

  /// Reads at least `min` bytes and places them in `buffer`.
  /// Returns the number of bytes read.
  ///
  /// This will continue to call `try_read` until at least `min` bytes have been
  /// read.
  fn read_at_least(&mut self, min: uint, buffer: &mut [u8]) -> uint {
    if min > buffer.len() { fail!("Stream: The buffer is too short (ARGUMENT)") }

    let mut read = 0;

    while read < min {
      match self.try_read(buffer.mut_slice_from(read)) {
        Some(0) => fail!("Stream: Not progressing (TODO)"),
        Some(n) => read += n,
        None => fail!("Stream: Unexpected EOF (INPUT)")
      }
    }

    return read;
  }

  /// Reads a u8.
  fn read_u8(&mut self) -> u8 {
    let mut buffer = [0];

    self.read(buffer);

    return buffer[0];
  }

  /// Reads a native endian u16
  fn read_ne_u16(&mut self) -> u16 {
    let mut buffer = [0, ..2];

    self.read(buffer);

    return unsafe { mem::transmute::<[u8, ..2], [u16, ..1]>(buffer) }[0];
  }

  /// Reads a big endian u16.
  fn read_be_u16(&mut self) -> u16 {
    return Int::from_be(self.read_ne_u16());
  }

  /// Reads a little endian u16.
  fn read_le_u16(&mut self) -> u16 {
    return Int::from_le(self.read_ne_u16());
  }

  /// Reads a native endian u32
  fn read_ne_u32(&mut self) -> u32 {
    let mut buffer = [0, ..4];

    self.read(buffer);

    return unsafe { mem::transmute::<[u8, ..4], [u32, ..1]>(buffer) }[0];
  }

  /// Reads a big endian u32.
  fn read_be_u32(&mut self) -> u32 {
    return Int::from_be(self.read_ne_u32());
  }

  /// Reads a little endian u32.
  fn read_le_u32(&mut self) -> u32 {
    return Int::from_le(self.read_ne_u32());
  }

  /// Reads a native endian u64
  fn read_ne_u64(&mut self) -> u64 {
    let mut buffer = [0, ..8];

    self.read(buffer);

    return unsafe { mem::transmute::<[u8, ..8], [u64, ..1]>(buffer) }[0];
  }

  /// Reads a big endian u64.
  fn read_be_u64(&mut self) -> u64 {
    return Int::from_be(self.read_ne_u64());
  }

  /// Reads a little endian u64.
  fn read_le_u64(&mut self) -> u64 {
    return Int::from_le(self.read_ne_u64());
  }

  /// Reads a i8
  fn read_i8(&mut self) -> i8 {
    return self.read_u8() as i8;
  }

  /// Reads a native endian u16
  fn read_ne_i16(&mut self) -> i16 {
    return self.read_ne_u16() as i16;
  }

  /// Reads a big endian u16.
  fn read_be_i16(&mut self) -> i16 {
    return self.read_be_u16() as i16;
  }

  /// Reads a little endian u16.
  fn read_le_i16(&mut self) -> i16 {
    return self.read_le_u16() as i16;
  }

  /// Reads a native endian u32
  fn read_ne_i32(&mut self) -> i32 {
    return self.read_ne_u32() as i32;
  }

  /// Reads a big endian u32.
  fn read_be_i32(&mut self) -> i32 {
    return self.read_be_u32() as i32;
  }

  /// Reads a little endian u32.
  fn read_le_i32(&mut self) -> i32 {
    return self.read_le_u32() as i32;
  }

  /// Reads a native endian u64
  fn read_ne_i64(&mut self) -> i64 {
    return self.read_ne_u64() as i64;
  }

  /// Reads a big endian u64.
  fn read_be_i64(&mut self) -> i64 {
    return self.read_be_u64() as i64;
  }

  /// Reads a little endian u64.
  fn read_le_i64(&mut self) -> i64 {
    return self.read_le_u64() as i64;
  }

  /// Reads `n` little-endian unsigned integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  fn read_le_uint_n(&mut self, n: uint) -> u64 {
      assert!(n > 0 && n <= 8);

      let mut result = 0u64;

      for i in range(0, n) {
        result = result | ((self.read_u8() as u64) << (8 * i));
      }

      return result;
  }

  /// Reads `n` little-endian signed integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  fn read_le_int_n(&mut self, n: uint) -> i64 {
    return extend_sign(self.read_le_uint_n(n), n);
  }

  /// Reads `n` big-endian unsigned integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  fn read_be_uint_n(&mut self, n: uint) -> u64 {
    assert!(n > 0 && n <= 8);

    let mut result = 0u64;

    for i in range(0, n) {
      result = result | ((self.read_u8() as u64) << (8 * (n - i - 1)));
    }

    return result;
  }

  /// Reads `n` big-endian signed integer bytes.
  ///
  /// `n` must be between 1 and 8, inclusive.
  fn read_be_int_n(&mut self, n: uint) -> i64 {
    return extend_sign(self.read_be_uint_n(n), n);
  }
}

pub struct Bitstream<'a> {
  pub cache: u8, pub cache_length: uint, stream: &'a mut Stream + 'a
}

impl<'a> Bitstream<'a> {
  pub fn new(stream: &'a mut Stream) -> Bitstream<'a> {
    return Bitstream { cache: 0, cache_length: 0, stream: stream };
  }

  pub fn read_n(&mut self, n: uint) -> u32 {
    if n > 32 {
      fail!("Bitstream: You cannot request more than 32 bits into a u32 (ARGUMENT)");
    }

    if n <= self.cache_length {
      let result = self.cache >> (self.cache_length - n);

      self.cache_length -= n;
      self.cache = self.cache & (0xFF >> (8 - self.cache_length));

      return result as u32;
    } else {
      let n_to_read = n - self.cache_length;
      let b_to_read = n_to_read / 8 + if n_to_read % 8 > 0 { 1 } else { 0 };

      let read = self.stream.read_be_uint_n(b_to_read);
      let sum = ((self.cache as u64) << (b_to_read * 8)) | (read as u64);

      self.cache_length = b_to_read * 8 - n_to_read;

      let result = sum >> self.cache_length;

      self.cache = (sum & (0xFF >> (8 - self.cache_length))) as u8;

      return result as u32;
    }
  }

  pub fn read_n_signed(&mut self, n: uint) -> i32 {
    return extend_sign_bits(self.read_n(n) as u64, n) as i32;
  }
}

fn extend_sign(value: u64, n: uint) -> i64 {
  return extend_sign_bits(value, n * 8);
}

fn extend_sign_bits(value: u64, n: uint) -> i64 {
  let shift = 64 - n;

  return (value << shift) as i64 >> shift;
}