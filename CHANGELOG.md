# Change Log

## [Unreleased]

### Added
- Experimental support for multi-threading. Added `AsyncTask` built-in module.
- Add functions related to command line arguments: `get_arg`, `get_arg_count`, `get_args` in `Std::IO`.
- Add `Random` built-in module.
- Add `Std::F32::infinity`, `Std::F64::infinity`, `Std::F32::quiet_nan`, `Std::F64::quiet_nan`.
- Add `Std::FFI` namespace which includes functions which are used to share ownership of Fix's boxed object with C program via FFI. Move `Destructor` and associated functions into `Std::FFI`.

### Changed
- Overflowing integer literals now result in a compile-time error.
- Now tuple types (`Std::Tuple{N}`) for any large N are defined if they are used. In older versions, only tuples upto N=4 were defined.
- Module names can contain period so that you can define a module such as `Main.Model.Impl`.

### Fixed
- Improve performance of `Std::IO::_read_line_inner` and `Std::IO::loop_lines`, `Std::IO::loop_lines_io` (#6).
- Fix an issue that causes parse error when there are local / global names starting with `true`, `false` or `nullptr` (#8).
- Fix an issue where the `ToString` implementation of `Ptr` was causing heap buffer overflow (#11).
- Fix an issue where `Std::eprint` and `Std::eprintln` wrote the output to stderr, not to stdout (#13).
- Fixed an issue where exponential notation of integer literals such as 3e10 was not working (#14).
- Fixed an issue where parsing floating point literal without decimal point (e.g., `1_F32`) causes the compiler to panic.
- Fixed an issue on linking dynamic library (PR #20).

## [0.1.0] - 2023-10-24

### Added
- First release in initial development phase including almost all features planned from the beginning.