#![feature(if_let)]
#![feature(macro_rules)]
#![feature(unsafe_destructor)]

pub mod channel;

pub mod stream;
pub mod file;
pub mod buffer;
pub mod stdout;
pub mod caf;

pub trait Initialize {
  fn initialize() -> Self;
  fn reinitialize(&mut self);
}

pub struct Binary {
  pub last: bool,
  pub data: Vec<u8>
}

impl Initialize for Binary {
  fn initialize() -> Binary {
    return Binary { last: false, data: Vec::with_capacity(4096) };
  }

  fn reinitialize(&mut self) {
    self.last = false;
    self.data.truncate(0);
  }
}

pub mod endian {
  #[derive(Debug,PartialEq)]
  pub enum Endian {
    Big, Little
  }
}

pub mod sample_type {
  #[derive(Debug,PartialEq)]
  pub enum SampleType {
    Unknown, Unsigned(usize), Signed(usize), Float(usize)
  }

  pub fn size(t: SampleType) -> usize {
    return match t {
      Unknown => 0,
      Unsigned(n) => n,
      Signed(n) => n,
      Float(n) => n
    };
  }
}

pub struct Audio {
  pub last: bool,
  pub channels: usize,
  pub sample_rate: f64,
  pub endian: endian::Endian,
  pub sample_type: sample_type::SampleType,
  pub data: Vec<u8>
}

impl Initialize for Audio {
  fn initialize() -> Audio {
    return Audio {
      last: false,
      channels: 0,
      sample_rate: 0.0,
      endian: endian::Big,
      sample_type: sample_type::Unknown,
      data: Vec::with_capacity(4096)
    };
  }

  fn reinitialize(&mut self) {
    self.last = false;
    self.channels = 0;
    self.sample_rate = 0.0;
    self.endian = endian::Big;
    self.sample_type = sample_type::Unknown;
    self.data.truncate(0);
  }
}
