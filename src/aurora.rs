#![feature(macro_rules)]
#![feature(unsafe_destructor)]

pub mod channel;

pub mod stream;
pub mod file;
pub mod buffer;
pub mod stdout;

pub trait Initialize {
  fn initialize() -> Self;
  fn reinitialize(&mut self);
}

pub struct Binary {
  pub final: bool,
  pub data: Vec<u8>
}

impl Initialize for Binary {
  fn initialize() -> Binary {
    return Binary { final: false, data: Vec::with_capacity(4096) };
  }

  fn reinitialize(&mut self) {
    self.final = false;
    self.data.truncate(0);
  }
}

pub mod endian {
  #[deriving(Show,PartialEq)]
  pub enum Endian {
    Big, Little
  }
}

pub mod sample_type {
  #[deriving(Show,PartialEq)]
  pub enum SampleType {
    Unknown, Unsigned(uint), Signed(uint), Float(uint)
  }

  pub fn size(t: SampleType) -> uint {
    return match t {
      Unknown => 0,
      Unsigned(n) => n,
      Signed(n) => n,
      Float(n) => n
    };
  }
}

pub struct Audio {
  pub final: bool,
  pub channels: uint,
  pub sample_rate: f64,
  pub endian: endian::Endian,
  pub sample_type: sample_type::SampleType,
  pub data: Vec<u8>
}

impl Initialize for Audio {
  fn initialize() -> Audio {
    return Audio {
      final: false,
      channels: 0,
      sample_rate: 0.0,
      endian: endian::Big,
      sample_type: sample_type::Unknown,
      data: Vec::with_capacity(4096)
    };
  }

  fn reinitialize(&mut self) {
    self.final = false;
    self.channels = 0;
    self.sample_rate = 0.0;
    self.endian = endian::Big;
    self.sample_type = sample_type::Unknown;
    self.data.truncate(0);
  }
}
