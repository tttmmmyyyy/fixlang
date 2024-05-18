# Change Log

## [Unreleased]

### Added
- Experimental support for multi-threading. Added `AsyncTask` built-in module.
- Add associated types.
- Add functions related to command line arguments: `get_arg`, `get_arg_count`, `get_args` in `Std::IO`.
- Add `Random` built-in module.
- Add `Std::F32::infinity`, `Std::F64::infinity`, `Std::F32::quiet_nan`, `Std::F64::quiet_nan`.
- Add `Std::FFI` namespace which includes functions which are used to share ownership of Fix's boxed object with C program via FFI. Move `Destructor` and associated functions into `Std::FFI`.
- Add hexadecimal, octal, binary integer literal (`0xaBC`, `0o123` or `0b110`) (#24).
- Add `RegExp` module (written by [pt9999](https://github.com/pt9999)).
- `Option a`, `Result e a` and tuples now implements `Eq` when type parameters of each type is satisfying preconditions.
- `Array a`, `Option a`, `Result e a`, `()` and tuples now implements `ToString` when type parameters of each type is satisfying preconditions.
- `Array a`, `String` and tuples now implements `LessThan` and `LessThanOrEq` when type parameters of each type is satisfying preconditions.
- Added orphan rule: a module cannot implement an external trait for an external type.
- The "eval" syntax now accepts only an expression of type `()`.
- Added `Std::Functor::forget : [f : Functor] f a -> f ()`, [which is intended to be used with "eval".](/Document.md#chaining-io-actions-by-eval-and-forget)
- Tuple of size 1, e.g., `(I64,)` (type of 1-tuples whose element is `I64`), `(42,)` (literal for 1-tuple) or `let (x,) = (42,);` (pattern matching for 1-tuple).

### Changed
- Overflowing integer literals now result in a compile-time error.
- Now tuple types (`Std::Tuple{N}`) for any large N are defined if they are used. In older versions, only tuples upto N=4 were defined.
- Module names can contain period so that you can define a module such as `Main.Model.Impl`.
- Now, in a trait definition, the type of a trait method should contain the type variable of that trait definition.
- Allow extra comma in many place. For example, you can write `[1, 2, 3, ]` for array literal of length 3.
- Type name, trait name, module name and namespace name can now starts with an underscore preceeding a capital letter.

### Fixed
- Performance improvement of functions in built-in libraries: #6, #30, #31
- Bug fixes on built-in libraries: #11, #13, #27, #34
- Bug fixes on compiler: #8, #14, #15, #20, #25, #26, #28, #36

## [0.1.0] - 2023-10-24

### Added
- First release in initial development phase including almost all features planned from the beginning.