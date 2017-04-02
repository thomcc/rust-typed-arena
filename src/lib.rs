//! The arena, a fast but limited type of allocator.
//!
//! Arenas are a type of allocator that destroy the objects within,
//! all at once, once the arena itself is destroyed.
//! They do not support deallocation of individual objects while the arena itself is still alive.
//! The benefit of an arena is very fast allocation; just a vector push.
//!
//! This is an equivalent of
//! [`arena::TypedArena`](http://doc.rust-lang.org/arena/struct.TypedArena.html)
//! distributed with rustc, but is available of Rust beta/stable.
//!
//! It is slightly less efficient, but simpler internally and uses much less unsafe code.
//! It is based on a `Vec<Vec<T>>` instead of raw pointers and manual drops.

// Potential optimizations:
// 1) add and stabilize a method for in-place reallocation of vecs.
// 2) add and stabilize placement new.
// 3) use an iterator. This may add far too much unsafe code.

use std::cell::RefCell;
use std::cmp;
use std::iter;
use std::mem;
use std::slice;

#[cfg(test)]
mod test;

// Initial size in bytes.
const INITIAL_SIZE: usize = 1024;
// Minimum capacity. Must be larger than 0.
const MIN_CAPACITY: usize = 1;

pub struct Arena<T> {
    chunks: RefCell<ChunkList<T>>,
}

struct ChunkList<T> {
    current: Vec<T>,
    rest: Vec<Vec<T>>,
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        let size = cmp::max(1, mem::size_of::<T>());
        Arena::with_capacity(INITIAL_SIZE / size)
    }

    pub fn with_capacity(n: usize) -> Arena<T> {
        let n = cmp::max(MIN_CAPACITY, n);
        Arena {
            chunks: RefCell::new(ChunkList {
                current: Vec::with_capacity(n),
                rest: vec![],
            }),
        }
    }

    /// Allocates a value in the arena, and returns a mutable reference
    /// to that value.
    pub fn alloc(&self, value: T) -> &mut T {
        &mut self.alloc_extend(iter::once(value))[0]
    }

    /// Uses the contents of an iterator to allocate values in the arena.
    /// Returns a mutable slice that contains these values.
    pub fn alloc_extend<I>(&self, iterable: I) -> &mut [T]
        where I: IntoIterator<Item = T>
    {
        let mut iter = iterable.into_iter();

        let mut chunks = self.chunks.borrow_mut();

        let iter_min_len = iter.size_hint().0;
        let mut next_item_index;
        if chunks.current.len() + iter_min_len > chunks.current.capacity() {
            chunks.reserve(iter_min_len);
            chunks.current.extend(iter);
            next_item_index = 0;
        } else {
            next_item_index = chunks.current.len();
            let mut i = 0;
            while let Some(elem) = iter.next() {
                if chunks.current.len() == chunks.current.capacity() {
                    // The iterator was larger than we could fit into the current chunk.
                    let chunks = &mut *chunks;
                    // Create a new chunk into which we can freely push the entire iterator into
                    chunks.reserve(i + 1);
                    let previous_chunk = chunks.rest.last_mut().unwrap();
                    let previous_chunk_len = previous_chunk.len();
                    // Move any elements we put into the previous chunk into this new chunk
                    chunks.current.extend(previous_chunk.drain(previous_chunk_len - i..));
                    chunks.current.push(elem);
                    // And the remaining elements in the iterator
                    chunks.current.extend(iter);
                    next_item_index = 0;
                    break;
                } else {
                    chunks.current.push(elem);
                }
                i += 1;
            }
        }
        let new_slice_ref = {
            let new_slice_ref = &mut chunks.current[next_item_index..];

            // Extend the lifetime from that of `chunks_borrow` to that of `self`.
            // This is OK because we’re careful to never move items
            // by never pushing to inner `Vec`s beyond their initial capacity.
            // The returned reference is unique (`&mut`):
            // the `Arena` never gives away references to existing items.
            unsafe { mem::transmute::<&mut [T], &mut [T]>(new_slice_ref) }
        };

        new_slice_ref
    }

    /// Allocates space for a given number of values, but doesn't initialize it.
    pub unsafe fn alloc_uninitialized(&self, num: usize) -> *mut [T] {
        let mut chunks = self.chunks.borrow_mut();

        if chunks.current.len() + num > chunks.current.capacity() {
            chunks.reserve(num);
        }

        // At this point, the current chunk must have free capacity.
        let next_item_index = chunks.current.len();
        chunks.current.set_len(next_item_index + num);
        // Extend the lifetime...
        &mut chunks.current[next_item_index..] as *mut _
    }

    /// Returns unused space.
    pub fn uninitialized_array(&self) -> *mut [T] {
        let chunks = self.chunks.borrow();
        let len = chunks.current.capacity() - chunks.current.len();
        let next_item_index = chunks.current.len();
        let slice = &chunks.current[next_item_index..];
        unsafe { slice::from_raw_parts_mut(slice.as_ptr() as *mut T, len) as *mut _ }
    }

    pub fn into_vec(self) -> Vec<T> {
        let mut chunks = self.chunks.into_inner();
        // keep order of allocation in the resulting Vec
        let n = chunks.rest.iter().fold(chunks.current.len(), |a, v| a + v.len());
        let mut result = Vec::with_capacity(n);
        for mut vec in chunks.rest {
            result.append(&mut vec);
        }
        result.append(&mut chunks.current);
        result
    }
}

impl<T> ChunkList<T> {
    #[inline(never)]
    #[cold]
    fn reserve(&mut self, additional: usize) {
        let double_cap = self.current.capacity().checked_mul(2).expect("capacity overflow");
        let required_cap = additional.checked_next_power_of_two().expect("capacity overflow");
        let new_capacity = cmp::max(double_cap, required_cap);
        let chunk = mem::replace(&mut self.current, Vec::with_capacity(new_capacity));
        self.rest.push(chunk);
    }
}
