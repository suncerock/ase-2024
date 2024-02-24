pub struct RingBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        RingBuffer {
            buffer: vec![T::default(); capacity],
            head: 0,
            tail: 0,
        }
    }

    pub fn reset(&mut self) {
        self.buffer.fill(T::default());
        self.head = 0;
        self.tail = 0;
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        self.buffer[self.head] = value
    }

    pub fn peek(&self) -> T {
        self.buffer[self.tail]
    }

    pub fn get(&self, offset: usize) -> T {
        self.buffer[(self.tail + offset) % self.capacity()]
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        self.buffer[self.head] = value;
        self.head = (self.head + 1) % self.capacity();
    }

    pub fn pop(&mut self) -> T {
        let value = self.buffer[self.tail];
        self.tail = (self.tail + 1) % self.capacity();
        value
    }

    pub fn get_read_index(&self) -> usize {
        self.tail
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.tail = index % self.capacity()
    }

    pub fn get_write_index(&self) -> usize {
        self.head
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.head = index % self.capacity()
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the ring buffer.
        if self.head >= self.tail {
            self.head - self.tail
        } else {
            self.head + self.capacity() - self.tail
        }
    }

    pub fn capacity(&self) -> usize {
        // Return the size of the internal buffer.
        self.buffer.len()
    }
}

impl RingBuffer<f32> {
    // Return the value at at an offset from the current read index.
    // To handle fractional offsets, linearly interpolate between adjacent values. 
    pub fn get_frac(&self, offset: f32) -> f32 {
        let index_floor = offset.floor() as usize;
        let index_ceil = offset.ceil() as usize;
        let index_fract = offset.fract();

        self.get(index_floor) * (1.0 - index_fract) + self.get(index_ceil) * index_fract
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapping() {
        // Test that ring buffer is a ring (wraps after more than `length` elements have entered).
        let capacity = 17;
        let delay = 5;
        let mut ring_buffer: RingBuffer<f32> = RingBuffer::new(capacity);

        for i in 0..delay {
            ring_buffer.push(i as f32);
        }

        for i in delay..capacity + 13 {
            assert_eq!(ring_buffer.len(), delay);
            assert_eq!(ring_buffer.pop(), (i - delay) as f32);
            ring_buffer.push(i as f32)
        }
    }

    #[test]
    fn test_api() {
        // Basic test of all API functions.
        let capacity = 3;
        let mut ring_buffer = RingBuffer::new(capacity);
        assert_eq!(ring_buffer.capacity(), capacity);

        ring_buffer.put(3);
        assert_eq!(ring_buffer.peek(), 3);

        ring_buffer.set_write_index(1);
        assert_eq!(ring_buffer.get_write_index(), 1);

        ring_buffer.push(17);
        assert_eq!(ring_buffer.get_write_index(), 2);

        assert_eq!(ring_buffer.get_read_index(), 0);
        assert_eq!(ring_buffer.get(1), 17);
        assert_eq!(ring_buffer.pop(), 3);
        assert_eq!(ring_buffer.get_read_index(), 1);

        assert_eq!(ring_buffer.len(), 1);
        ring_buffer.push(42);
        assert_eq!(ring_buffer.len(), 2);

        assert_eq!(ring_buffer.get_write_index(), 0);

        // Should be unchanged.
        assert_eq!(ring_buffer.capacity(), capacity);
    }

    #[test]
    fn test_capacity() {
        // Tricky: does `capacity` mean "size of internal buffer" or "number of elements before this is full"?
        let capacity = 3;
        let mut ring_buffer = RingBuffer::new(3);
        for i in 0..(capacity - 1) {
            ring_buffer.push(i);
            dbg!(ring_buffer.len());
            assert_eq!(ring_buffer.len(), i+1);
        }
    }

    #[test]
    fn test_reset() {
        // Test state after initialization and reset.
        let mut ring_buffer = RingBuffer::new(512);

        // Check initial state.
        assert_eq!(ring_buffer.get_read_index(), 0);
        assert_eq!(ring_buffer.get_write_index(), 0);
        for i in 0..ring_buffer.capacity() {
            assert_eq!(ring_buffer.get(i), 0.0);
        }

        // Fill ring buffer, mess with indices.
        let fill = 123.456;
        for i in 0..ring_buffer.capacity() {
            ring_buffer.push(fill);
            assert_eq!(ring_buffer.get(i), fill);
        }

        ring_buffer.set_write_index(17);
        ring_buffer.set_read_index(42);

        // Check state after reset.
        ring_buffer.reset();
        assert_eq!(ring_buffer.get_read_index(), 0);
        assert_eq!(ring_buffer.get_write_index(), 0);
        for i in 0..ring_buffer.capacity() {
            assert_eq!(ring_buffer.get(i), 0.0);
        }
    }

    #[test]
    fn test_weird_inputs() {
        let capacity = 5;
        let mut ring_buffer = RingBuffer::<f32>::new(capacity);

        ring_buffer.set_write_index(capacity);
        assert_eq!(ring_buffer.get_write_index(), 0);
        ring_buffer.set_write_index(capacity * 2 + 3);
        assert_eq!(ring_buffer.get_write_index(), 3);

        ring_buffer.set_read_index(capacity);
        assert_eq!(ring_buffer.get_read_index(), 0);
        ring_buffer.set_read_index(capacity * 2 + 3);
        assert_eq!(ring_buffer.get_read_index(), 3);

        // NOTE: Negative indices are also weird, but we can't even pass them due to type checking!
    }

    #[test]
    fn test_fractional_read_index() {
        let capacity = 5;
        let mut ring_buffer: RingBuffer<f32> = RingBuffer::new(capacity);

        ring_buffer.push(1.0);
        ring_buffer.push(2.0);
        ring_buffer.push(3.0);

        ring_buffer.set_read_index(0);
        assert!((ring_buffer.get_frac(0.4) - 1.4).abs() <= f32::EPSILON);
        assert!((ring_buffer.get_frac(1.7) - 2.7).abs() <= f32::EPSILON);
        assert!((ring_buffer.get_frac(1.0) - 2.0).abs() <= f32::EPSILON);

        ring_buffer.pop();
        let v = ring_buffer.get_frac(0.3);
        assert!((ring_buffer.get_frac(0.3) - 2.3).abs() <= f32::EPSILON);
    }

    #[test]
    fn 测试分数读取() {
        let 容量 = 5;
        let mut 环形缓冲器: RingBuffer<f32> = RingBuffer::new(容量);

        let 壹点零 = 1.0;
        let 贰点零 = 2.0;
        let 叁点零 = 3.0;
        let 误差上限 = f32::EPSILON;

        环形缓冲器.push(壹点零);
        环形缓冲器.push(贰点零);
        环形缓冲器.push(叁点零);

        
        // 测试分数读取
        let 目标索引 = 0.4;
        let 目标值 = 1.4;
        let 读取值 = 环形缓冲器.get_frac(目标索引);
        let 误差 = (读取值 - 目标值).abs();
        assert!(误差 <= 误差上限);

        // 测试整数读取
        let 目标索引 = 2.0;
        let 目标值 = 3.0;
        let 读取值 = 环形缓冲器.get_frac(目标索引);
        let 误差 = (读取值 - 目标值).abs();
        assert!(误差 <= 误差上限);
    }

    #[test]
    fn 𓅠𓅡𓅢() {
        let 𓆉= 5;
        let mut 𓀂: RingBuffer<f32> = RingBuffer::new(𓆉);

        let 𓂭 = 1.0;
        let 𓂮 = 2.0;
        let 𓂯 = 3.0;
        let 𓀉 = f32::EPSILON;

        𓀂.push(𓂭);
        𓀂.push(𓂮);
        𓀂.push(𓂯);

        let 𓁅 = 0.4;
        let 𓀥 = 1.4;
        let 𓁛 = 𓀂.get_frac(𓁅);
        let 𓀩 = (𓁛 - 𓀥).abs();
        assert!(𓀩 <= 𓀉);
    }
}
