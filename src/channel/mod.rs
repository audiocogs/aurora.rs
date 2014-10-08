extern crate alloc;

use super::Initialize;

use std::mem;

use std::ptr;
use std::ptr::{RawPtr,RawMutPtr};

use std::sync;
use std::sync::{Arc,Semaphore};
use std::sync::atomic::AtomicInt;

struct Channel<T> {
  rc_read: AtomicInt, rc_write: AtomicInt,
  data: int, capacity: int,
  read: AtomicInt, write: AtomicInt,
  not_empty: Semaphore, not_full: Semaphore
}

#[unsafe_destructor]
impl<T> Drop for Channel<T> {
  fn drop(&mut self) {
    unsafe { alloc::heap::deallocate(self.data as *mut u8, self.capacity as uint * mem::size_of::<T>(), mem::align_of::<T>()) };
  }
}

pub struct Source<T> {
  channel: Arc<Channel<T>>
}

impl<T> Source<T> {
  fn new(channel: Arc<Channel<T>>) -> Source<T> {
    return Source { channel: channel };
  }

  /// Attempts to read a value from the sink, blocking if it is empty
  ///
  /// Fails if there is no Sink, and no data left to read.
  pub fn read(&mut self, f: |&T|) {
    if self.channel.rc_write.load(sync::atomic::SeqCst) == 0 {
      if self.channel.read.load(sync::atomic::SeqCst) == self.channel.write.load(sync::atomic::SeqCst) {
        fail!("Sink: Source is dropped")
      }
    }

    self.channel.not_empty.acquire();

    unsafe {
      let offset = self.channel.read.fetch_add(1, sync::atomic::SeqCst) % self.channel.capacity;
      let ptr = mem::transmute::<int, *mut T>(self.channel.data).offset(offset);
      let data = ptr.as_ref().unwrap();

      f(data);
    }

    self.channel.not_full.release();
  }
}

#[unsafe_destructor]
impl<T> Drop for Source<T> {
  fn drop(&mut self) {
    self.channel.rc_read.fetch_sub(1, sync::atomic::SeqCst);
  }
}


pub struct Sink<T> {
  channel: Arc<Channel<T>>
}

impl<T: Initialize> Sink<T> {
  fn new(channel: Arc<Channel<T>>) -> Sink<T> {
    return Sink { channel: channel };
  }

  /// Attempts to write a value to the sink, blocking if it is full.
  ///
  /// Fails if there is no Source.
  pub fn write(&mut self, f: |&mut T|) {
    if self.channel.rc_read.load(sync::atomic::SeqCst) == 0 {
      fail!("Sink: Source is dropped")
    }

    self.channel.not_full.acquire();

    unsafe {
      let offset = self.channel.write.fetch_add(1, sync::atomic::SeqCst) % self.channel.capacity;
      let ptr = mem::transmute::<int, *mut T>(self.channel.data).offset(offset);
      let data = ptr.as_mut().unwrap();

      data.reinitialize();

      f(data);
    }

    self.channel.not_empty.release();
  }
}

#[unsafe_destructor]
impl<T> Drop for Sink<T> {
  fn drop(&mut self) {
    self.channel.rc_write.fetch_sub(1, sync::atomic::SeqCst);
  }
}

pub fn create<T: super::Initialize>(capacity: uint) -> (Sink<T>, Source<T>) {
  let data = unsafe {
    alloc::heap::allocate(capacity * mem::size_of::<T>(), mem::align_of::<T>())
  };
  
  for offset in range(0, capacity as int) {
    unsafe {
      let ptr = mem::transmute::<*mut u8, *mut T>(data).offset(offset);

      let n: T = Initialize::initialize();

      ptr::write(ptr, n);
    }
  }

  let channel = Arc::new(Channel {
    rc_read: AtomicInt::new(1), rc_write: AtomicInt::new(1),
    data: unsafe { mem::transmute(data) }, capacity: capacity as int,
    read: AtomicInt::new(0), write: AtomicInt::new(0),
    not_empty: Semaphore::new(0), not_full: Semaphore::new(capacity as int)
  });

  let sink = Sink::<T>::new(channel.clone());
  let source = Source::<T>::new(channel);

  return (sink, source);
}

#[cfg(test)]
mod tests {
  struct Test { value: uint }

  impl ::Initialize for Test {
    fn initialize() -> Test {
      return Test { value: 0 };
    }

    fn reinitialize(&mut self) {
      self.value = 0;
    }
  }

  #[test]
  fn test_basic() {
    let (mut sink, mut source) = super::create::<Test>(1);

    sink.write(|x: &mut Test| { x.value = 1 });
    source.read(|x: &Test| { assert_eq!(x.value, 1) });
  }

  #[test]
  fn test_threads() {
    let (mut sink, mut source) = super::create::<Test>(1);

    spawn(proc() {
      sink.write(|x: &mut Test| { x.value = 1 });
    });

    source.read(|x: &Test| { assert_eq!(x.value, 1) });
  }

  #[test]
  #[should_fail]
  fn test_source_gone() {
    let (mut sink, source) = super::create::<Test>(1);

    drop(source);

    sink.write(|x: &mut Test| { x.value = 1 });
  }

  #[test]
  #[should_fail]
  fn test_source_gone_threads() {
    let (mut sink, mut source) = super::create::<Test>(1);

    spawn(proc() {
      source.read(|x: &Test| { assert_eq!(x.value, 1) });
    });

    loop { sink.write(|x: &mut Test| { x.value = 1 }) };
  }

  #[test]
  fn test_source_works() {
    let (mut sink, mut source) = super::create::<Test>(1);

    spawn(proc() {
      for i in range(0u, 10000) {
        sink.write(|x: &mut Test| { x.value = i })
      }
    });

    for i in range(0u, 10000) {
      source.read(|x: &Test| { assert_eq!(x.value, i) });
    }
  }
}