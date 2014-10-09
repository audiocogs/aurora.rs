#![feature(unsafe_destructor)]

pub mod channel;

pub mod stream;
pub mod file;
pub mod buffer;

pub trait Initialize {
  fn initialize() -> Self;
  fn reinitialize(&mut self);
}

pub struct Binary {
  pub final: bool,
  pub data: Vec<u8>
}

impl ::Initialize for Binary {
  fn initialize() -> Binary {
    return Binary { final: false, data: Vec::with_capacity(4096) };
  }

  fn reinitialize(&mut self) {
    self.final = false;
    self.data.truncate(0);
  }
}
