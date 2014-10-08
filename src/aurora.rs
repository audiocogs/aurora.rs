#![feature(unsafe_destructor)]

pub mod channel;

pub mod stream;
pub mod file;
pub mod buffer;

pub trait Initialize {
  fn initialize() -> Self;
  fn reinitialize(&mut self);
}