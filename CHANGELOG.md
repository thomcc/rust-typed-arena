## Unreleased

Released YYYY/MM/DD.

### Added

* Implement `Default` for `Arena<T>`.

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
