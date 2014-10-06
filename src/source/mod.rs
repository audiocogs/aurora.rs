extern crate sync;

pub struct Source<T: Send> {
  receiver: sync::comm::Receiver<T>
}

impl<T: Send> Source<T> {
  /// Attempts to read a value from the sink, blocking if it is empty
  pub fn read(&mut self) -> T {
    return self.receiver.recv();
  }

  pub fn nonblocking_read(&mut self) -> Option<T> {
    match self.receiver.try_recv() {
      Ok(v) => return Some(v),
      Err(sync::comm::Empty) => return None,
      Err(sync::comm::Disconnected) => fail!("Source: A source was disconnected from the sink without coordination (BUG)")
    }
  }
}

pub struct Sink<T: Send> {
  sender: sync::comm::SyncSender<T>
}


impl<T: Send> Sink<T> {
  pub fn write(&mut self, value: T) {
    self.sender.send(value);
  }
}

pub fn create<T: Send>(bound: uint) -> (Sink<T>, Source<T>) {
  let (tx, rx) = sync::comm::sync_channel(bound);

  return (Sink { sender: tx }, Source { receiver: rx });
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_basic() {
    let (mut sink, mut source) = super::create(1);

    sink.write(1u8);

    assert_eq!(source.read(), 1);
  }
}