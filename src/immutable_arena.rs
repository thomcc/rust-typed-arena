use Arena;

// The immutable index functions can't be added to the "normal" `Arena`, because one could create
// an index from one arena and pass it to another arena.
// In that case, the index gotten from the first arena could point to an element for which a mutable
// reference was already passed to the user, creating a mutable and an immutable reference to the
// same element at the same time => unsound.
// Therefore, here we only ever return immutable references.

/// An Index to an immutable element.
///
/// For more information see the documentation of [`alloc_indexed`](struct.ImmutableArena.html#method.alloc_indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    /// Index to get the chunk. A number one past the number of elements in `chunks.rest.len` indicates
    /// the current chunk (which will eventually end up at that position in `chunks.rest`).
    chunk: usize,
    /// Element-index within a chunk.
    element: usize,
}

impl Index {
    /// Constructs an Index from its raw parts.
    ///
    /// The parts should have been gotten from [`into_raw_parts`](struct.Index.html#method.from_raw_parts).
    /// Otherwise, it may lead to an invalid index and ultimately panics.
    pub fn from_raw_parts(a: usize, b: usize) -> Index {
        Index {
            chunk: a,
            element: b,
        }
    }

    /// Converts this index into its raw parts.
    ///
    /// This method is usually not very useful. I can be used to convert this index into another
    /// index type. However, usually using a newtype should be preferred.
    pub fn into_raw_parts(self) -> (usize, usize) {
        (self.chunk, self.element)
    }
}

/// An immutable indexable arena.
///
/// This struct adds indirection via indexes compared to [`Arena`] while
/// keeping an immutable API (i.e. inserting doesn't require `&mut self`).
/// This indirection means that only immutable shared references can be returned.
///
/// Indirection is usually not needed and [`Arena`] should be preferred.
///
/// * If you need multiple shared references, you can create them from the mutable reference
///   returned by [`Arena::alloc`](struct.Arena.html#method.alloc).
/// * If you need indirection and mutability at the cost of having a mutable API (i.e. `&mut self`),
///   use the [`generational_arena`](https://docs.rs/generational-arena/) crate instead.
///
/// However, in some use-cases it is required to use some sort of index while wanting to keep
/// an immutable API.
/// Those use-cases are what this `ImmutableArena` implementation is for.
///
/// [`Arena`]: struct.Arena.html
pub struct ImmutableArena<T> {
    inner: Arena<T>,
}

impl<T> ImmutableArena<T> {
    /// Construct a new immutable arena.
    ///
    /// See the documentation of [`Arena::new`](struct.Arena.html#method.new) for more information.
    pub fn new() -> ImmutableArena<T> {
        ImmutableArena {
            inner: Arena::new(),
        }
    }
    /// Construct a new immutable arena with capacity for `n` values pre-allocated.
    ///
    /// See the documentation of [`Arena::with_capacity`](struct.Arena.html#method.with_capacity) for more information.
    pub fn with_capacity(n: usize) -> ImmutableArena<T> {
        ImmutableArena {
            inner: Arena::with_capacity(n),
        }
    }
    /// Return the size of the immutable arena.
    ///
    /// See the documentation of [`Arena::len`](struct.Arena.html#method.len) for more information.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// Convert this `ImmutableArena` into a `Vec<T>`.
    ///
    /// See the documentation of [`Arena::into_vec`](struct.Arena.html#method.into_vec) for more information.
    pub fn into_vec(self) -> Vec<T> {
        self.inner.into_vec()
    }

    /// Allocates a value in this immutable arena, returning an index to it.
    ///
    /// The index can be used with [`get_indexed`](struct.Arena.html#method.get_indexed) to
    /// get an immutable reference to the element.
    ///
    /// ## Example
    ///
    /// ```
    /// use typed_arena::ImmutableArena;
    ///
    /// let arena = ImmutableArena::new();
    /// let idx = arena.alloc(42);
    /// let x = arena.get(idx);
    /// assert_eq!(*x, 42);
    /// ```
    #[inline]
    pub fn alloc(&self, value: T) -> Index {
        // we discard the mutable reference, as elements of this arena are only allowed
        // to be accessed immutably.
        self.inner.alloc(value);
        let chunks = self.inner.chunks.borrow();
        debug_assert!(!chunks.current.is_empty());
        Index {
            chunk: chunks.rest.len(),
            element: chunks.current.len() - 1,
        }
    }

    /// Returns an immutable reference to a previously allocated element.
    ///
    /// See the documentation of [`alloc_indexed`](struct.Arena.html#method.alloc_indexed)
    /// for more information.
    #[inline]
    pub fn get(&self, index: Index) -> &T {
        let chunks = self.inner.chunks.borrow();
        // this doesn't need to be an assert, because if the index is invalid (e.g. gotten from
        // another ImmutableArena, the index operations will panic)
        debug_assert!(chunks.rest.len() <= index.chunk);
        let element = if chunks.rest.len() < index.chunk {
            &chunks.rest[index.chunk][index.element]
        } else {
            &chunks.current[index.element]
        };
        // Extend the lifetime of the reference to `&self`.
        // This is safe because this element has been created using `alloc_indexed`
        // (otherwise the user wouldn't have access to an `Idx`).
        // Thus, no mutable reference has been passed out to this element.
        // Therefore, we can hand out as many immutable references to it as we want.
        // The reference itself stays valid for as long as this arena, which is why we bind it to
        // the `&self` lifetime.
        unsafe { &*(element as *const _) }
    }
}

impl<T> Default for ImmutableArena<T> {
    fn default() -> Self {
       Self::new()
    }
}

