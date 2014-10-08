#![feature(unsafe_destructor)]

pub mod channel;

pub mod stream;
pub mod file;

pub trait Initialize {
  fn initialize(&mut self);
}