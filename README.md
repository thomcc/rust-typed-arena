rust-typed-arena
================

[![Docs Status](https://docs.rs/typed-arena/badge.svg)](https://docs.rs/typed-arena)

The arena, a fast but limited type of allocator.

Arenas are a type of allocator that destroy the objects within,
all at once, once the arena itself is destroyed.
They do not support deallocation of individual objects while the arena itself is still alive.
The benefit of an arena is very fast allocation; just a vector push.

There is also a method `into_vec()` to recover ownership of allocated objects when
the arena is no longer required, instead of destroying everything.
