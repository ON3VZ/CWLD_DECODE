//! A single-producer, single-consumer ring buffer for audio samples, without locks and without
//! allocation on the producer side (the sound card callback).

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

struct Ring {
    buf: Box<[UnsafeCell<f32>]>,
    cap: usize,
    /// Samples written so far (wraps). Only the producer stores it.
    head: AtomicUsize,
    /// Samples read so far (wraps). Only the consumer stores it.
    tail: AtomicUsize,
    dropped: AtomicU64,
}

// Safety: the producer only writes slots between tail and head + free space, the consumer only
// reads slots between tail and head. The atomics with Acquire/Release order the accesses.
unsafe impl Sync for Ring {}
unsafe impl Send for Ring {}

pub struct Producer(Arc<Ring>);
pub struct Consumer(Arc<Ring>);

pub fn ring(capacity: usize) -> (Producer, Consumer) {
    let cap = capacity.max(2);
    let buf: Vec<UnsafeCell<f32>> = (0..cap).map(|_| UnsafeCell::new(0.0)).collect();
    let r = Arc::new(Ring { buf: buf.into_boxed_slice(), cap, head: AtomicUsize::new(0), tail: AtomicUsize::new(0), dropped: AtomicU64::new(0) });
    (Producer(r.clone()), Consumer(r))
}

impl Producer {
    /// Writes as much as fits. Samples that do not fit are counted as dropped, never blocked on.
    pub fn push(&mut self, data: &[f32]) -> usize {
        let r = &self.0;
        let head = r.head.load(Ordering::Relaxed);
        let tail = r.tail.load(Ordering::Acquire);
        let free = r.cap - head.wrapping_sub(tail);
        let n = data.len().min(free);
        for (i, x) in data.iter().take(n).enumerate() {
            let slot = head.wrapping_add(i) % r.cap;
            unsafe { *r.buf[slot].get() = *x };
        }
        r.head.store(head.wrapping_add(n), Ordering::Release);
        if n < data.len() {
            r.dropped.fetch_add((data.len() - n) as u64, Ordering::Relaxed);
        }
        n
    }
}

impl Consumer {
    /// Appends everything that is waiting to `out`.
    pub fn pop_all(&mut self, out: &mut Vec<f32>) -> usize {
        let r = &self.0;
        let tail = r.tail.load(Ordering::Relaxed);
        let head = r.head.load(Ordering::Acquire);
        let n = head.wrapping_sub(tail);
        out.reserve(n);
        for i in 0..n {
            let slot = tail.wrapping_add(i) % r.cap;
            out.push(unsafe { *r.buf[slot].get() });
        }
        r.tail.store(tail.wrapping_add(n), Ordering::Release);
        n
    }

    /// Samples lost because the consumer was too slow.
    pub fn dropped(&self) -> u64 {
        self.0.dropped.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_is_kept_across_the_wrap() {
        let (mut p, mut c) = ring(8);
        let mut out = Vec::new();
        let mut next = 0.0f32;
        let mut expect = 0.0f32;
        for _ in 0..50 {
            let chunk: Vec<f32> = (0..5).map(|_| { next += 1.0; next }).collect();
            assert_eq!(p.push(&chunk), 5);
            out.clear();
            c.pop_all(&mut out);
            for x in &out {
                expect += 1.0;
                assert_eq!(*x, expect);
            }
        }
        assert_eq!(c.dropped(), 0);
    }

    #[test]
    fn overflow_drops_and_counts() {
        let (mut p, c) = ring(4);
        assert_eq!(p.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]), 4);
        assert_eq!(c.dropped(), 2);
    }

    #[test]
    fn works_across_threads() {
        let (mut p, mut c) = ring(1024);
        let t = std::thread::spawn(move || {
            for i in 0..10_000 {
                while p.push(&[i as f32]) == 0 {
                    std::thread::yield_now();
                }
            }
        });
        let mut got = Vec::new();
        while got.len() < 10_000 {
            c.pop_all(&mut got);
            std::thread::yield_now();
        }
        t.join().unwrap();
        assert!(got.iter().enumerate().all(|(i, x)| *x == i as f32));
    }
}
