pub struct RingBuffer<T> {
    // TODO: fill this in.
    buffer: Vec<T>,
    read_index: usize,
    write_index: usize,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        // Create a new RingBuffer with `length` slots and "default" values.
        // Hint: look into `vec!` and the `Default` trait.
        Self {
            buffer: vec![T::default(); length],
            read_index: 0,
            write_index: 0
        }
    }

    pub fn reset(&mut self) {
        // Clear internal buffer and reset indices.
        self.buffer.clear();
        self.read_index = 0;
        self.write_index = 0;
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        self.buffer[self.write_index] = value;
    }

    pub fn peek(&self) -> T {
        self.buffer[self.read_index]
    }

    pub fn get(&self, offset: usize) -> T {
        self.buffer[(self.read_index + offset) % self.capacity()]
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        self.buffer[self.write_index] = value;
        self.write_index += 1;
    }

    pub fn pop(&mut self) -> T {
        let output = self.buffer[self.read_index];
        self.read_index += 1;
        output
    }

    pub fn get_read_index(&self) -> usize {
        self.read_index
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.read_index = index;
    }

    pub fn get_write_index(&self) -> usize {
        self.write_index
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.write_index = index;
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        if self.read_index >= self.write_index {
            self.read_index - self.write_index
        }
        else {
            self.read_index + self.capacity() - self.write_index
        }
    }

    pub fn capacity(&self) -> usize {
        // Return the length of the internal buffer.
        self.buffer.len()
    }
}
