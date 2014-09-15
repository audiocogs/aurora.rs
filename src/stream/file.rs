use std;

use stream::Stream;

pub struct FileStream {
  file: std::io::File
}

impl FileStream {
  pub fn new(file: std::io::File) -> FileStream {
    return FileStream { file: file };
  }
}

impl Stream for FileStream {
  fn read(&mut self, buffer: &mut [u8]) -> Option<uint> {
    if self.file.eof() {
      return None;
    }

    match self.file.read(buffer) {
      Ok(n) => return Some(n),
      Err(_) => fail!("File: Error when reading (ARGUMENT)")
    }
  }
}

#[cfg(test)]
mod tests {
  use std;
  use stream::Stream;

  #[test]
  fn test_read_zero() {
    let path = std::path::Path::new("/dev/zero");
    let mut s = super::FileStream::new(std::io::File::open(&path).unwrap());

    assert_eq!(s.read_u8(), 0);
    assert_eq!(s.read_be_u16(), 0);
    assert_eq!(s.read_be_u32(), 0);
    assert_eq!(s.read_be_u64(), 0);
  }
}