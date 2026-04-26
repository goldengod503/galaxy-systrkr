//! Fixed-capacity ring buffer used as the sample history for sparklines.
//! Capacity is set at construction time and may be resized at runtime
//! via `resize`, which discards old samples that no longer fit.

pub struct RingBuf<T: Copy + Default> {
    buf: Vec<T>,
    capacity: usize,
    /// Number of items written so far, capped at capacity.
    len: usize,
    /// Index where the next write will go.
    head: usize,
}

impl<T: Copy + Default> RingBuf<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuf capacity must be > 0");
        Self {
            buf: vec![T::default(); capacity],
            capacity,
            len: 0,
            head: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        self.buf[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate items in chronological order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        let start = if self.len < self.capacity { 0 } else { self.head };
        let cap = self.capacity;
        (0..self.len).map(move |i| self.buf[(start + i) % cap])
    }

    /// Resize, preserving the most recent samples that fit. Discards
    /// older samples on shrink; pads tail with defaults on grow (which
    /// is fine — those slots are overwritten on subsequent pushes
    /// before being read by `iter`, since `len` doesn't grow).
    pub fn resize(&mut self, new_capacity: usize) {
        assert!(new_capacity > 0, "RingBuf capacity must be > 0");
        if new_capacity == self.capacity {
            return;
        }
        let all: Vec<T> = self.iter().collect();
        let preserved: Vec<T> = all
            .into_iter()
            .rev()
            .take(new_capacity)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        self.buf = vec![T::default(); new_capacity];
        for (i, v) in preserved.iter().enumerate() {
            self.buf[i] = *v;
        }
        self.len = preserved.len();
        self.head = self.len % new_capacity;
        self.capacity = new_capacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let buf: RingBuf<f32> = RingBuf::new(4);

        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), 4);
    }

    #[test]
    fn push_below_capacity_appends_in_order() {
        let mut buf: RingBuf<f32> = RingBuf::new(4);

        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        let collected: Vec<f32> = buf.iter().collect();
        assert_eq!(collected, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn push_at_capacity_drops_oldest() {
        let mut buf: RingBuf<f32> = RingBuf::new(3);

        for i in 1..=5u32 {
            buf.push(i as f32);
        }

        let collected: Vec<f32> = buf.iter().collect();
        assert_eq!(collected, vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn resize_smaller_keeps_most_recent() {
        let mut buf: RingBuf<u32> = RingBuf::new(5);
        for i in 1..=5u32 {
            buf.push(i);
        }

        buf.resize(3);

        let collected: Vec<u32> = buf.iter().collect();
        assert_eq!(collected, vec![3, 4, 5]);
        assert_eq!(buf.capacity(), 3);
    }

    #[test]
    fn resize_larger_keeps_all_existing() {
        let mut buf: RingBuf<u32> = RingBuf::new(3);
        for i in 1..=3u32 {
            buf.push(i);
        }

        buf.resize(6);

        let collected: Vec<u32> = buf.iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
        assert_eq!(buf.capacity(), 6);

        buf.push(4);
        buf.push(5);

        let collected: Vec<u32> = buf.iter().collect();
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn iter_handles_wraparound() {
        let mut buf: RingBuf<u32> = RingBuf::new(4);
        for i in 1..=10u32 {
            buf.push(i);
        }

        let collected: Vec<u32> = buf.iter().collect();
        assert_eq!(collected, vec![7, 8, 9, 10]);
    }
}
