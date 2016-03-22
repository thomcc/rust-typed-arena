use super::*;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

#[test]
fn arena_as_intended() {
    use std::cell::Cell;
    use std::mem;

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
        assert_eq!(arena.chunks.borrow().rest.len(), 0);

        node = arena.alloc(Node(Some(node), 2, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        node = arena.alloc(Node(Some(node), 3, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        node = arena.alloc(Node(Some(node), 4, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        assert_eq!(node.1, 4);
        assert_eq!(node.0.unwrap().1, 3);
        assert_eq!(node.0.unwrap().0.unwrap().1, 2);
        assert_eq!(node.0.unwrap().0.unwrap().0.unwrap().1, 1);
        assert   !(node.0.unwrap().0.unwrap().0.unwrap().0.is_none());

        mem::drop(node);
        assert_eq!(drop_counter.get(), 0);

        let mut node: &Node = arena.alloc(Node(None, 5, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 1);

        node = arena.alloc(Node(Some(node), 6, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 2);

        node = arena.alloc(Node(Some(node), 7, DropTracker(&drop_counter)));
        assert_eq!(arena.chunks.borrow().rest.len(), 2);

        assert_eq!(drop_counter.get(), 0);

        assert_eq!(node.1, 7);
        assert_eq!(node.0.unwrap().1, 6);
        assert_eq!(node.0.unwrap().0.unwrap().1, 5);
        assert   !(node.0.unwrap().0.unwrap().0.is_none());

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
fn it_supports_placement() {
    struct Monster { health: u8 };

    let arena = Arena::new();
    let troll = in &arena { Monster { health: 42 } };
    assert_eq!(troll.health, 42);
}

#[test]
fn it_supports_placement_many_times() {
    struct Monster { health: u8 };

    let arena = Arena::new();
    for _ in 0..10000 {
        let troll = in &arena { Monster { health: 42 } };
        assert_eq!(troll.health, 42);
    }

    // Make sure we can grow past the initial chunk
    assert!(arena.chunks.borrow().rest.len() > 0);
}

#[test]
#[should_panic(expected = "struct creation failed")]
fn it_supports_placement_with_a_panic() {
    const DROP_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

    struct Monster { pub health: u8 };

    impl Drop for Monster {
        fn drop(&mut self) {
            let old = DROP_COUNTER.fetch_add(1, Ordering::SeqCst);
            // when std::panic::recover is stable, use that instead
            assert_eq!(0, old);
        }
    }

    let arena = Arena::new();
    let _ = in &arena { Monster { health: 42 } };
    let _ = in &arena { Monster { health: panic!("struct creation failed") } };
}
