use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Lock-free single-producer single-consumer ring buffer for interleaved f32 audio samples.
///
/// # Safety
/// - `push` must only be called from the producer thread (decode thread).
/// - `pop` must only be called from the consumer thread (audio callback).
/// - No other thread may concurrently call the same method on the same instance.
pub struct AudioRingBuf {
    buf: UnsafeCell<Vec<f32>>,
    capacity: usize,
    mask: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl Sync for AudioRingBuf {}

impl AudioRingBuf {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());
        Self {
            buf: UnsafeCell::new(vec![0.0; capacity]),
            capacity,
            mask: capacity - 1,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Number of samples available to read.
    pub fn readable(&self) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Relaxed);
        tail.wrapping_sub(head)
    }

    /// Number of samples that can be written.
    pub fn writable(&self) -> usize {
        self.capacity.saturating_sub(self.readable())
    }

    /// Push samples from `data` into the ring buffer. Returns the number of samples pushed.
    /// Called only from the producer thread.
    pub fn push(&self, data: &[f32]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Relaxed);
        let readable = tail.wrapping_sub(head);
        let writable = self.capacity.saturating_sub(readable);
        let to_write = writable.min(data.len());

        if to_write == 0 {
            return 0;
        }

        let buf = unsafe { &mut *self.buf.get() };
        for (i, &sample) in data.iter().enumerate().take(to_write) {
            let idx = tail.wrapping_add(i) & self.mask;
            buf[idx] = sample;
        }

        self.tail
            .store(tail.wrapping_add(to_write), Ordering::Release);
        to_write
    }

    /// Pop up to `output.len()` samples from the ring buffer. Returns the number of samples read.
    /// Called only from the consumer thread.
    pub fn pop(&self, output: &mut [f32]) -> usize {
        let tail = self.tail.load(Ordering::Acquire);
        let head = self.head.load(Ordering::Relaxed);
        let readable = tail.wrapping_sub(head);
        let to_read = readable.min(output.len());

        if to_read == 0 {
            return 0;
        }

        let buf = unsafe { &*self.buf.get() };
        for i in 0..to_read {
            output[i] = buf[head.wrapping_add(i) & self.mask];
        }

        self.head
            .store(head.wrapping_add(to_read), Ordering::Release);
        to_read
    }

    /// Discard all buffered samples.
    pub fn clear(&self) {
        let tail = self.tail.load(Ordering::Acquire);
        self.head.store(tail, Ordering::Release);
    }
}
