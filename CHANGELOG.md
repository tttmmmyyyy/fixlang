# Change Log

## [Unreleased]

### Added
- Experimental support for multi-threading. Added `AsyncTask` built-in module.
- Add functions related to command line arguments: `get_arg`, `get_arg_count`, `get_args` in `Std::IO`.
- Add `Random` built-in module.
- Add `Std::F32::infinity` and `Std::F64::infinity`.

### Fixed
- Improve performance of `Std::IO::_read_line_inner` and `Std::IO::loop_lines`, `Std::IO::loop_lines_io` (#6).
- Fix an issue that causes parse error when there are local / global names starting with `true`, `false` or `nullptr` (#8).
- Fix an issue where the `ToString` implementation of `Ptr` was causing heap buffer overflow (#11).
- Fix an issue where `Std::eprint` and `Std::eprintln` wrote the output to stderr, not to stdout (#13).

## [0.1.0] - 2023-10-24

### Added
- First release in initial development phase including almost all features planned from the beginning.