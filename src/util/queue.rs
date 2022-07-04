use std::collections::{linked_list, LinkedList};

/// A FIFO data structure with fixed capacity
#[derive(Debug)]
pub struct Queue<T> {
    buffer: LinkedList<T>,
    capacity: usize,
}

impl<T> Queue<T> {
    /// Initialize a queue with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: LinkedList::new(),
            capacity,
        }
    }
    /// Insert a element to the back of the queue, failed on queue is full
    pub fn insert(&mut self, elt: T) -> Result<(), String> {
        if self.capacity() == self.len() {
            let msg = String::from("Queue is full");
            return Err(msg);
        }
        self.buffer.push_back(elt);
        Ok(())
    }
    /// Pop out the first element of the queue. If the queue is empty None returned
    pub fn pop(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }
    /// Return length of the queue
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    /// Return capacity of the queue
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Return wheither the queue is full or not
    pub fn is_full(&self) -> bool {
        self.buffer.len() == self.capacity()
    }
    /// Return wheither the queue is empty or not
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    /// Get immutable reference of the element which resides in given index
    /// If such element not found, None returned
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.buffer.iter().nth(idx)
    }
    /// Get mutable reference of the element which resides in given index
    /// If such element not found, None returned
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.buffer.iter_mut().nth(idx)
    }
    /// Get immutable reference of first element of the queue
    /// If the queue is empty None returned
    pub fn head(&self) -> Option<&T> {
        self.get(0)
    }
}

impl<'b, T> IntoIterator for &'b Queue<T> {
    type IntoIter = linked_list::Iter<'b, T>;
    type Item = &'b T;
    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter()
    }
}

impl<'b, T> IntoIterator for &'b mut Queue<T> {
    type IntoIter = linked_list::IterMut<'b, T>;
    type Item = &'b mut T;
    fn into_iter(self) -> Self::IntoIter {
        self.buffer.iter_mut()
    }
}

#[cfg(test)]
mod queue {
    use super::Queue;

    const TEST_CAPACITY: usize = 10;
    #[test]
    fn init() {
        let q: Queue<usize> = Queue::new(TEST_CAPACITY);
        assert_eq!(q.capacity(), TEST_CAPACITY);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn insert_and_pop() -> Result<(), String> {
        let mut q = Queue::new(TEST_CAPACITY);
        for i in 0..TEST_CAPACITY {
            q.insert(i)?;
        }
        for i in 0..TEST_CAPACITY {
            let elt = q
                .pop()
                .ok_or(format!("{}th element is empty, expect something here", i))?;
            assert_eq!(elt, i);
        }
        Ok(())
    }

    #[test]
    fn iteration() -> Result<(), String> {
        let mut q = Queue::new(TEST_CAPACITY);
        for i in 0..TEST_CAPACITY {
            q.insert(i)?;
        }
        for (i, v) in q.into_iter().enumerate() {
            assert_eq!(i, *v);
        }
        for i in 0..TEST_CAPACITY {
            let elt = q
                .pop()
                .ok_or(format!("{}th element is empty, expect something here", i))?;
            assert_eq!(i, elt);
        }
        Ok(())
    }

    #[test]
    fn iteration_mut() -> Result<(), String> {
        let mut q = Queue::new(TEST_CAPACITY);
        for i in 0..TEST_CAPACITY {
            q.insert(i)?;
        }
        for v in &mut q {
            *v = *v * 2;
        }
        for (i, v) in q.into_iter().enumerate() {
            assert_eq!(i * 2, *v);
        }
        Ok(())
    }
}
