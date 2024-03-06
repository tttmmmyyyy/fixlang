# Change Log

## [Unreleased]

### Added
- Experimental support for multi-threading. Added `AsyncTask` built-in module.
- Add functions related to command line arguments: `get_arg`, `get_arg_count`, `get_args` in `Std::IO`.
- Add `Random` built-in module.
- Add `Std::F32::infinity`, `Std::F64::infinity`, `Std::F32::quiet_nan`, `Std::F64::quiet_nan`.
- Add `Std::FFI` namespace which includes functions which are used to share ownership of Fix's boxed object with C program via FFI. Move `Destructor` and associated functions into `Std::FFI`.
- Add hexadecimal, octal, binary integer literal (`0xaBC`, `0o123` or `0b110`) (#24).
- Add `RegExp` module (written by [pt9999](https://github.com/pt9999)).
- `Option a`, `Result e a` and tuples now implements `Eq` when type parameters of each type is satisfying preconditions.
- `Array a`, `Option a`, `Result e a`, `()` and tuples now implements `ToString` when type parameters of each type is satisfying preconditions.
- `Array a`, `String` and tuples now implements `LessThan` and `LessThanOrEq` when type parameters of each type is satisfying preconditions.

### Changed
- Overflowing integer literals now result in a compile-time error.
- Now tuple types (`Std::Tuple{N}`) for any large N are defined if they are used. In older versions, only tuples upto N=4 were defined.
- Module names can contain period so that you can define a module such as `Main.Model.Impl`.

### Fixed
- Performance improvement of functions in built-in libraries: #6, #30, #31
- Bug fixes on built-in libraries: #11, #13, #27, #34
- Bug fixes on compiler: #8, #14, #15, #20, #25, #26, #28, #36

## [0.1.0] - 2023-10-24

### Added
- First release in initial development phase including almost all features planned from the beginning.