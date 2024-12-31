# Change Log

## [Unreleased]

### Added
- Add `Std::Monad::unless : [m : Monad] Bool -> m () -> m ()`, `Std::Monad::when : [m : Monad] Bool -> m () -> m ()`.
- Add type aliases `Std::FFI::CChar`, `Std::FFI::CUnsignedChar`, `Std::FFI::CShort`, `Std::FFI::CUnsignedShort`, `Std::FFI::CInt`, `Std::FFI::CUnsignedInt`, `Std::FFI::CLong`, `Std::FFI::CUnsignedLong`, `Std::FFI::CLongLong`, `Std::FFI::CUnsignedLongLong`, `Std::FFI::CSizeT`, `Std::FFI::CFloat`, `Std::FFI::CDouble`.
- Add `Std::FFI::_get_boxed_ptr`, `Std::FFI::borrow_boxed`.
- Add `Std::FFI::get_errno`, `Std::FFI::clear_errno`.
- Add `act_{field} : [f : Functor] (F -> f F) -> S -> f S` for each field `{field}` of type `F` of a struct `S`, which is known as "Lens" in Haskell community.
- Add `Std::Destructor::mutate_unique` and `Std::Destructor::mutate_unique_io`.
- Implement `Functor` for tuple types. `map` function acts the last component of tuples.
- Add `FFI_EXPORT` syntax. Remove `fixruntime_run_function` native function since it can be implemented using `FFI_EXPORT`.
- Add experimental support for language server protocol.
- Add support for project file ("fixproj.toml").
- Add support for configuration file ("~/.fixconfig.toml").
- Add `fix deps` subccommand, which manages dependencies of a Fix project.
- Add `fix docs` subcommand, which generates the document for a Fix project.
- Add `fix test` subcommand, which runs `Test::test`.
- Add `-O (--object)` option to specify object files to be linked.
- Add `fix init` subcommand, which generates a template project file.
- Add `FFI_CALL_IO` and `FFI_CALL_IOS` syntax, which is similar to `FFI_CALL` but suitable for foregin functions which have side effects.
- Add `Std::do_with_retained`.
- Add `{monad_expr};; {expr}` syntax, which is equivalent to `let _ = *{monad_expr}; {expr}`.
- Add `Std::Box::make` function.
- Support building dynamic libraries (use `--output-type dylib`).
- Add `match` syntax.
- Change the bit width of reference counter from 64 to 32.
- Add `Std::Arrow`, which is a higher-kinded type for functions.

### Changed

- Change namespace of `type Destructor` from `Std::FFI::Destructor` to `Std::FFI`.
- Swap return values of `generate_*` functions in `Random` module, e.g., changed `generate_U64 : Random -> (U64, Random)` to `generate_U64 : Random -> (Random, U64)`.
- Remove functions to modify arrays or structs asserting uniqueness: `set_{field}!`, `mod_{field}!`, `Array::set!`, `Array::mod!`, `Array::act!`, `Array::append!`, `Array::push_back!`, `Array::pop_back!`. If you want to assert a value is unique, use `Debug::assert_unique` instead.
- Rename `Debug::assert_unique!` to `Debug::assert_unique`.
- Rename `Std::PunchedArray::plug_in!` and `Std::PunchedArray::punch!` to `Std::PunchedArray::plug_in` and `Std::PunchedArray::unsafe_punch` respectively.
- Allow making empty structs.
- Forbid underscores in type names, trait names, module names and namespaces.
- Change `CALL_C` to `FFI_CALL`.
- Remove `Std::abort` and added `Std::undefined : String -> a`.
- Change the type of arguments of `Std::FFI::get_funptr_retain` and `Std::FFI::get_funptr_release`.
- Remove `Debug` built-in module, which is moved into `Std::Debug` namespace.
- Remove `AsyncTask`, `Character`, `Hash`, `HashMap`, `HashSet`, `Math`, `Random`, `RegExp`, `Subprocess` and `Time` built-in modules. They are provided as independent Fix projects.
- Change the type of `Debug::assert` and `Debug::assert_eq`. Now they return `IO ()`.
- Change the internal representation of `IO a` types. Now `IO a` is isomorphic to `IOState -> (IOState, a)`.
- Remove `IO::from_func` and added `IO::from_runner : (IOState -> (IOState, a)) -> IO a`.
- Change `force_unique` to `_unsafe_force_unique`.
- Change the semantics of the "eval" syntax. See the document for details.
- Rename `Std::Boxed` type to `Std::Box`. 
- Rename `Std::LoopResult` type to `Std::LoopState`.
- Allow trait implementations to be placed in any namespace. Previously, they could only be written at the top level of a module. However, it does not matter in which namespace you define them.

### Fixed

- Fix an issue on `Std::Array::act` which may cause memory leak.
- Fix #45, #46, #47, #49.
- Disallow `...` in argument types list in `CALL_C` (`FFI_CALL`) because there is no way to handle variadic arguments in Fix.

## [0.2.0] - 2024-06-12

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
- Add orphan rule: a module cannot implement an external trait for an external type.
- The "eval" syntax now accepts only an expression of type `()`.
- Add `Std::Functor::forget : [f : Functor] f a -> f ()`, [which is intended to be used with "eval".](/Document.md#chaining-io-actions-by-eval-and-forget)
- Tuple of size 1, e.g., `(I64,)` (type of 1-tuples whose element is `I64`), `(42,)` (literal for 1-tuple) or `let (x,) = (42,);` (pattern matching for 1-tuple).
- Add `Std::Iterator::product : Iterator a -> Iterator b -> Iterator (b, a)`.

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
- Bug fixes on compiler: #8, #14, #15, #20, #25, #26, #28, #36, #42, #43

## [0.1.0] - 2023-10-24

### Added
- First release in initial development phase including almost all features planned from the beginning.