/// Simple ring buffer to simulate a bounded queue efficently
pub struct RingBuffer<T: Default + Clone, const BUFFER_SIZE: usize = 4096> {
    buffer: [T; BUFFER_SIZE],
    start: usize,
    end: usize,
}

impl <const BUFFER_SIZE: usize, T: Default + Clone> RingBuffer<T, BUFFER_SIZE> {
    pub fn new() -> Self {
        let buffer = unsafe{
            let mut buffer: [T; BUFFER_SIZE] = core::mem::MaybeUninit::uninit().assume_init();
            for i in 0..BUFFER_SIZE {
                buffer[i] = T::default();
            }
            buffer
        };
        Self {
            buffer,
            start: 0,
            end: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == BUFFER_SIZE
    }

    pub fn len(&self) -> usize {
        ((BUFFER_SIZE + self.end) - self.start) % BUFFER_SIZE
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let result = &self.buffer[self.start];
        self.start = (BUFFER_SIZE + self.start + 1) % BUFFER_SIZE; 

        Some(result.clone())
    }

    pub fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
    }

    pub fn consume(&mut self, quantity: usize) {
        let quantity = quantity.max(self.len());
        self.start = (BUFFER_SIZE + self.start + quantity) % BUFFER_SIZE;
    }

    pub fn push(&mut self, value: T) -> Option<()> {
        if self.is_full() {
            return None;
        }

        self.buffer[self.end] = value;
        self.end = (BUFFER_SIZE + self.end + 1) % BUFFER_SIZE; 

        Some(())
    }

    pub fn get(&mut self, index: usize) -> Option<T> {
        if index > self.len() {
            return None;
        }

        let idx = (BUFFER_SIZE + self.start + index) % BUFFER_SIZE;
        Some(self.buffer[idx].clone())
    }
}

/// Create a "peekable" iterator where we can look at future values if present.
pub struct Peekable<T: Default + Clone, I: Iterator<Item=T>, const BUFFER_SIZE: usize = 4096> {
    iterator: I,
    cache: RingBuffer<T, BUFFER_SIZE>,
}

impl<T: Default + Clone, I: Iterator<Item=T>, const BUFFER_SIZE: usize> Peekable<T, I, BUFFER_SIZE> {
    pub fn new(iterator: I) -> Self {
            Peekable{
            iterator,
            cache: RingBuffer::new(),
        }
    }

    pub fn get(&mut self, index: usize) -> Option<T> {
        // TODO!: how to handle gracefully this?
        if index > BUFFER_SIZE {
            panic!("Cannot peek at index '{}' because the buffer size is '{}'", index, BUFFER_SIZE);
        }

        // if needed fill the cache
        if self.cache.len() < index {
            for _ in 0..index - self.cache.len() {
                if let Some(value) = self.iterator.next() {
                    self.cache.push(value).unwrap();
                } else {
                    // the iterator ended, so we cannot get that value
                    return None;
                }
            }
        }

        self.cache.get(index)
    }

    pub fn consume(&mut self, quantity: usize) -> Option<()> {
        let reminder = quantity.saturating_sub(self.cache.len());
        let cache_to_remove = quantity - reminder;
        
        self.cache.consume(cache_to_remove);
        for _ in 0..reminder {
            self.iterator.next()?;
        }

        Some(())
    }
}

impl<T: Default + Clone, I: Iterator<Item=T>, const BUFFER_SIZE: usize> Iterator for Peekable<T, I, BUFFER_SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cache.is_empty() {
            self.iterator.next()
        } else {
            Some(self.cache.pop().unwrap()) // Defensive unwrap
        }
    }
}