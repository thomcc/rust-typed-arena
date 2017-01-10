rust-typed-arena
================

[![Docs Status](https://docs.rs/typed-arena/badge.svg)](https://docs.rs/typed-arena)

The arena, a fast but limited type of allocator.

Arenas are a type of allocator that destroy the objects within,
all at once, once the arena itself is destroyed.
They do not support deallocation of individual objects while the arena itself is still alive.
The benefit of an arena is very fast allocation; just a vector push.

This is an equivalent to [`arena::TypedArena`](http://doc.rust-lang.org/arena/struct.TypedArena.html)
distributed with rustc, but is available of Rust beta/stable.

It is probably slightly less efficient, but is simpler internally and uses much less unsafe code.
It is based on a `Vec<Vec<T>>` instead of raw pointers and manual drops.

There is also a method `into_vec()` to recover ownership of allocated objects when
the arena is no longer required, instead of destroying everything.
