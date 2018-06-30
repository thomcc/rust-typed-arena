use std::cell::Cell;
use std::mem;
use std::ptr;
use super::*;

struct DropTracker<'a>(&'a Cell<u32>);
impl<'a> Drop for DropTracker<'a> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

struct Node<'a, 'b: 'a>(Option<&'a Node<'a, 'b>>, u32, DropTracker<'b>);

#[test]
fn arena_as_intended() {
    let drop_counter = Cell::new(0);
    {
        let arena = Arena::with_capacity(2);

        let mut node: &Node = arena.alloc(Node(None, 1, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 0);

        node = arena.alloc(Node(Some(node), 2, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 0);

        node = arena.alloc(Node(Some(node), 3, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        node = arena.alloc(Node(Some(node), 4, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        assert_eq!(node.1, 4);
        assert_eq!(node.0.unwrap().1, 3);
        assert_eq!(node.0.unwrap().0.unwrap().1, 2);
        assert_eq!(node.0.unwrap().0.unwrap().0.unwrap().1, 1);
        assert!(node.0.unwrap().0.unwrap().0.unwrap().0.is_none());

        mem::drop(node);
        assert_eq!(drop_counter.get(), 0);

        let mut node: &Node = arena.alloc(Node(None, 5, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        node = arena.alloc(Node(Some(node), 6, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        node = arena.alloc(Node(Some(node), 7, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 2);

        assert_eq!(drop_counter.get(), 0);

        assert_eq!(node.1, 7);
        assert_eq!(node.0.unwrap().1, 6);
        assert_eq!(node.0.unwrap().0.unwrap().1, 5);
        assert!(node.0.unwrap().0.unwrap().0.is_none());

        assert_eq!(drop_counter.get(), 0);
    }
    assert_eq!(drop_counter.get(), 7);
}

#[test]
fn ensure_into_vec_maintains_order_of_allocation() {
    let arena = Arena::with_capacity(1);  // force multiple inner vecs
    for &s in &["t", "e", "s", "t"] {
        arena.alloc(String::from(s));
    }
    let vec = arena.into_vec();
    assert_eq!(vec, vec!["t", "e", "s", "t"]);
}

#[test]
fn test_zero_cap() {
    let arena = Arena::with_capacity(0);
    let a = arena.alloc(1);
    let b = arena.alloc(2);
    assert_eq!(*a, 1);
    assert_eq!(*b, 2);
}

#[test]
fn test_alloc_extend() {
    let arena = Arena::with_capacity(2);
    for i in 0 .. 15 {
        let slice = arena.alloc_extend(0 .. i);
        for (j, &elem) in slice.iter().enumerate() {
            assert_eq!(j, elem);
        }
    }
}

#[test]
fn test_alloc_uninitialized() {
    const LIMIT: usize = 15;
    let drop_counter = Cell::new(0);
    unsafe {
        let arena: Arena<Node> = Arena::with_capacity(4);
        for i in 0 .. LIMIT {
            let slice = arena.alloc_uninitialized(i);
            for (j, elem) in (&mut *slice).iter_mut().enumerate() {
                ptr::write(elem, Node(None, j as u32, DropTracker(&drop_counter)));
            }
            assert_eq!(drop_counter.get(), 0);
        }
    }
    assert_eq!(drop_counter.get(), (0 .. LIMIT).fold(0, |a, e| a + e) as u32);
}

#[test]
fn test_alloc_extend_with_drop_counter() {
    let drop_counter = Cell::new(0);
    {
        let arena = Arena::with_capacity(2);
        let iter = (0 .. 100).map(|j| {
            Node(None, j as u32, DropTracker(&drop_counter))
        });
        let older_ref = Some(&arena.alloc_extend(iter)[0]);
        assert_eq!(drop_counter.get(), 0);
        let iter = (0 .. 100).map(|j| {
            Node(older_ref, j as u32, DropTracker(&drop_counter))
        });
        arena.alloc_extend(iter);
        assert_eq!(drop_counter.get(), 0);
    }
    assert_eq!(drop_counter.get(), 200);
}

#[test]
fn test_uninitialized_array() {
    let arena = Arena::with_capacity(2);
    let uninit = arena.uninitialized_array();
    arena.alloc_extend(0 .. 2);
    unsafe {
        for (&a, b) in (&*uninit).iter().zip(0 .. 2) {
            assert_eq!(a, b);
        }
        assert!((&*arena.uninitialized_array()).as_ptr() != (&*uninit).as_ptr());
        arena.alloc(0);
        let uninit = arena.uninitialized_array();
        assert_eq!((&*uninit).len(), 3);
    }
}


#[test]
fn dont_trust_the_iterator_size() {
    use std::iter::repeat;

    struct WrongSizeIter<I>(I);
    impl<I> Iterator for WrongSizeIter<I>
        where I: Iterator
    {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(0))
        }
    }

    impl<I> ExactSizeIterator for WrongSizeIter<I> where I: Iterator {}

    let arena = Arena::with_capacity(2);
    arena.alloc(0);
    let slice = arena.alloc_extend(WrongSizeIter(repeat(1).take(1_000)));
    // Allocation of 1000 elements should have created a new chunk
    assert_eq!(arena.chunks.borrow().rest.len(), 1);
    assert_eq!(slice.len(), 1000);
}

#[test]
fn arena_is_send() {
    fn assert_is_send<T: Send>(_: T) {}

    // If `T` is `Send`, ...
    assert_is_send(42_u32);

    // Then `Arena<T>` is also `Send`.
    let arena: Arena<u32> = Arena::new();
    assert_is_send(arena);
}
