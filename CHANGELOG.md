## Unreleased

Released YYYY/MM/DD.

### Added

* TODO (or remove section if none)

### Changed

* TODO (or remove section if none)

### Deprecated

* TODO (or remove section if none)

### Removed

* TODO (or remove section if none)

### Fixed

* TODO (or remove section if none)

### Security

* TODO (or remove section if none)

--------------------------------------------------------------------------------

## 1.7.0

Released 2019/10/31. *Spooky!*

### Added

* Added a `len` method to count how many items are in an arena.

### Fixed

* Fixed some theoretical overflows.

--------------------------------------------------------------------------------

## 1.6.1

Released 2019/09/17.

### Fixed

* Now compiles on old stable Rust versions again, instead of just new stable
  Rust versions. From here on out, we'll promise that 1.X will continue to
  compile on rustc versions >= 1.32.

--------------------------------------------------------------------------------

## 1.6.0

Released 2019/09/09.

### Added

* Added the `Arena::iter_mut` method for mutably iterating over an arena's
  contents. [See #29 for
  details.](https://github.com/SimonSapin/rust-typed-arena/pull/29)

--------------------------------------------------------------------------------

## 1.5.0

Released 2019/08/02.

### Added

* `Arena` now implements `Default`

### Fixed

* Introduced an internal fast path for allocation, improving performance.
* Tests now run cleanly on Miri. There was previously a technicality where
  the stacked borrow rules were not being followed.

--------------------------------------------------------------------------------

## 1.4.1

Released 2018/06/29.

### Added

* Added more documentation comments and examples.

--------------------------------------------------------------------------------

## 1.4.0

Released 2018/06/21.

### Added

* Added a new, on-by-default feature named "std". Disabling this feature allows
  the crate to be used in `#![no_std]` environments. [#15][] [#12][]

[#15]: https://github.com/SimonSapin/rust-typed-arena/pull/15
[#12]: https://github.com/SimonSapin/rust-typed-arena/pull/12
