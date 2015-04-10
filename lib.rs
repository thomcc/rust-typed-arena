//! The arena, a fast but limited type of allocator.
//!
//! Arenas are a type of allocator that destroy the objects within,
//! all at once, once the arena itself is destroyed.
//! They do not support deallocation of individual objects while the arena itself is still alive.
//! The benefit of an arena is very fast allocation; just a vector push.
//!
//! This is an equivalent to [`arena::TypedArena`](http://doc.rust-lang.org/arena/struct.TypedArena.html)
//! distributed with rustc, but is available of Rust beta/stable.
//!
//! It is probably slightly less efficient, but is simpler internally and uses much less unsafe code.
//! It is based on a `Vec<Vec<T>>` instead of raw pointers and manual drops.

use std::cell::RefCell;
use std::mem;


pub struct Arena<T> {
    chunks: RefCell<Vec<Vec<T>>>,
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        Arena::with_capacity(8)
    }

    pub fn with_capacity(n: usize) -> Arena<T> {
        Arena {
            chunks: RefCell::new(vec![Vec::with_capacity(n)]),
        }
    }

    pub fn alloc(&self, value: T) -> &mut T {
        let mut chunks_borrow = self.chunks.borrow_mut();
        let next_chunk_index = chunks_borrow.len();

        let (last_child_length, last_chunk_capacity) = {
            let last_chunk = &chunks_borrow[next_chunk_index - 1];
            (last_chunk.len(), last_chunk.capacity())
        };

        let (chunk, next_item_index) = if last_child_length < last_chunk_capacity {
            (&mut chunks_borrow[next_chunk_index - 1], last_child_length)
        } else {
            let new_capacity = last_chunk_capacity.checked_mul(2).unwrap();
            chunks_borrow.push(Vec::with_capacity(new_capacity));
            (&mut chunks_borrow[next_chunk_index], 0)
        };
        chunk.push(value);
        let new_item_ref = &mut chunk[next_item_index];

        // Extend the lifetime from that of `chunks_borrow` to that of `self`.
        // This is OK because we’re careful to never move items
        // by never pushing to inner `Vec`s beyond their initial capacity.
        // The returned reference is unique (`&mut`):
        // the `Arena` never gives away references to existing items.
        unsafe { mem::transmute::<&mut T, &mut T>(new_item_ref) }
    }
}


#[test]
fn it_works() {
    use std::cell::Cell;

    struct DropTracker<'a>(&'a Cell<u32>);
    impl<'a> Drop for DropTracker<'a> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    struct Node<'a, 'b: 'a>(Option<&'a Node<'a, 'b>>, u32, DropTracker<'b>);
    let drop_counter = Cell::new(0);
    {
        let arena = Arena::with_capacity(2);

        let mut node: &Node = arena.alloc(Node(None, 1, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 1);

        node = arena.alloc(Node(Some(node), 2, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 1);

        node = arena.alloc(Node(Some(node), 3, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 2);

        node = arena.alloc(Node(Some(node), 4, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 2);

        assert_eq!(node.1, 4);
        assert_eq!(node.0.unwrap().1, 3);
        assert_eq!(node.0.unwrap().0.unwrap().1, 2);
        assert_eq!(node.0.unwrap().0.unwrap().0.unwrap().1, 1);
        assert   !(node.0.unwrap().0.unwrap().0.unwrap().0.is_none());

        mem::drop(node);
        assert_eq!(drop_counter.get(), 0);

        let mut node: &Node = arena.alloc(Node(None, 5, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 2);

        node = arena.alloc(Node(Some(node), 6, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 2);

        node = arena.alloc(Node(Some(node), 7, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().len(), 3);

        assert_eq!(drop_counter.get(), 0);

        assert_eq!(node.1, 7);
        assert_eq!(node.0.unwrap().1, 6);
        assert_eq!(node.0.unwrap().0.unwrap().1, 5);
        assert   !(node.0.unwrap().0.unwrap().0.is_none());

        assert_eq!(drop_counter.get(), 0);
    }
    assert_eq!(drop_counter.get(), 7);
}
