extern crate alloc;

use super::Initialize;

use std::mem;

use std::ptr;

use std::sync;
use std::sync::{Arc,Semaphore};
use std::sync::atomic::{AtomicIsize, Ordering};

use std::marker::PhantomData;

struct Channel<T> {
  rc_read: AtomicIsize, rc_write: AtomicIsize,
  data: isize, capacity: isize,
  read: AtomicIsize, write: AtomicIsize,
  not_empty: Semaphore, not_full: Semaphore,
  phantom: PhantomData<T>
}

#[unsafe_destructor]
impl<T> Drop for Channel<T> {
  fn drop(&mut self) {
    unsafe { alloc::heap::deallocate(self.data as *mut u8, self.capacity as usize * mem::size_of::<T>(), mem::align_of::<T>()) };
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
  pub fn read<F>(&mut self, f: F)
      where F: Fn(&T) {
    if self.channel.rc_write.load(Ordering::SeqCst) == 0 {
      if self.channel.read.load(Ordering::SeqCst) == self.channel.write.load(Ordering::SeqCst) {
        panic!("Sink: Source is dropped")
      }
    }

    self.channel.not_empty.acquire();

    unsafe {
      let offset = self.channel.read.fetch_add(1, Ordering::SeqCst) % self.channel.capacity;
      let ptr = mem::transmute::<isize, *mut T>(self.channel.data).offset(offset);
      let data = ptr.as_ref().unwrap();

      f(data);
    }

    self.channel.not_full.release();
  }
}

#[unsafe_destructor]
impl<T> Drop for Source<T> {
  fn drop(&mut self) {
    self.channel.rc_read.fetch_sub(1, Ordering::SeqCst);
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
  pub fn write<F>(&mut self, f: F)
      where F : FnMut(&T) {
    if self.channel.rc_read.load(Ordering::SeqCst) == 0 {
      panic!("Sink: Source is dropped")
    }

    self.channel.not_full.acquire();

    unsafe {
      let offset = self.channel.write.fetch_add(1, Ordering::SeqCst) % self.channel.capacity;
      let ptr = mem::transmute::<isize, *mut T>(self.channel.data).offset(offset);
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
    self.channel.rc_write.fetch_sub(1, Ordering::SeqCst);
  }
}

pub fn create<T: super::Initialize>(capacity: usize) -> (Sink<T>, Source<T>) {
  let data = unsafe {
    alloc::heap::allocate(capacity * mem::size_of::<T>(), mem::align_of::<T>())
  };
  
  for offset in 0..(capacity as isize) {
    unsafe {
      let ptr = mem::transmute::<*mut u8, *mut T>(data).offset(offset);

      let n: T = Initialize::initialize();

      ptr::write(ptr, n);
    }
  }

  let channel = Arc::new(Channel {
    rc_read: AtomicIsize::new(1), rc_write: AtomicIsize::new(1),
    data: unsafe { mem::transmute(data) }, capacity: capacity as isize,
    read: AtomicIsize::new(0), write: AtomicIsize::new(0),
    not_empty: Semaphore::new(0), not_full: Semaphore::new(capacity as isize),
    phantom: PhantomData
  });

  let sink = Sink::<T>::new(channel.clone());
  let source = Source::<T>::new(channel);

  return (sink, source);
}

#[cfg(test)]
mod tests {
  struct Test { value: usize }

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

    thread::spawn(|| {
      sink.write(|x: &mut Test| { x.value = 1 });
    });

    source.read(|x: &Test| { assert_eq!(x.value, 1) });
  }

  #[test]
  #[should_panic]
  fn test_source_gone() {
    let (mut sink, source) = super::create::<Test>(1);

    drop(source);

    sink.write(|x: &mut Test| { x.value = 1 });
  }

  #[test]
  #[should_panic]
  fn test_source_gone_threads() {
    let (mut sink, mut source) = super::create::<Test>(1);

    thread::spawn(|| {
      source.read(|x: &Test| { assert_eq!(x.value, 1) });
    });

    loop { sink.write(|x: &mut Test| { x.value = 1 }) };
  }

  #[test]
  fn test_source_works() {
    let (mut sink, mut source) = super::create::<Test>(1);

    thread::spawn(|| {
      for i in 0us..10000 {
        sink.write(|x: &mut Test| { x.value = i })
      }
    });

    for i in 0us..10000 {
      source.read(|x: &Test| { assert_eq!(x.value, i) });
    }
  }
}
