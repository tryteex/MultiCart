use std::{net::TcpStream};

// Queue of waiting connection
pub struct Queue {
  count: usize,                         // Max capacity
  max: usize,                         // Max capacity
  len: usize,                         // Queue len
  first: usize,                       // First index of queue
  last: usize,                        // Last index of queue
  data: Vec<Option<TcpStream>>,       // Data
}

impl Queue {
  pub fn new(max: usize) -> Queue {
    let mut list: Vec<Option<TcpStream>> = Vec::with_capacity(max);
    for _ in 0..max {
      list.push(None);
    }
    Queue {
      count: 0,
      max,
      len: 0,
      first: 0, 
      last: max - 1,
      data: list,
    }
  }

  pub fn empty(&self) -> bool {
    self.len == 0
  }

  pub fn push(&mut self, tcp: TcpStream) -> Option<TcpStream> {
    self.count += 1;
    if self.len == self.max {
      return Some(tcp);
    }
    self.len += 1;
    let mut next = self.last + 1;
    if next == self.max {
      next = 0;
    }
    self.last = next;
    self.data[self.last].replace(tcp);
    None
  }

  pub fn take(&mut self) -> Option<TcpStream> {
    if self.len == 0 {
      return None;
    }
    self.len -= 1;
    let v = self.data[self.first].take();
    let mut next = self.first + 1;
    if next == self.max {
      next = 0;
    }
    self.first = next;
    v
  }
}