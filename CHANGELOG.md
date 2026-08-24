# Change Log

## [Unreleased]

### Added

#### Language

- `_` in a pattern is now a wildcard that matches any value and discards it. It can appear multiple times in a single pattern (e.g. `let (x, _, _) = triple;`), and the matched value cannot be referred to afterwards. Previously `_` was an ordinary variable name, so multiple `_`s in one pattern were rejected as duplicate binders.
- `_` can now be used as a type wildcard in a type annotation, standing for a type the compiler should infer. Each `_` becomes a fresh type variable, so `arr : Array _` fixes the container while leaving the element type to inference and `(_, _)` annotates a pair of two independent types. A bare `_` has kind `*`; because Fix does not infer kinds, a higher kind is written explicitly as `(_ : k)`, e.g. `let empty : (_ : * -> *) I64 = [];` pins the element type to `I64` while leaving the container to be inferred as `Array`.

#### Tool

- LSP: Hovering a `_` type wildcard shows the type it was inferred to (a concrete type, a generic type variable, the type constructor a higher-kinded wildcard resolved to, or a function's opaque return type). This works in both expression and pattern (let-binding) annotations.
- #214: LSP: Completion inside an `import` statement now offers what can be written there instead of expression symbols: module names at the module position, the imported module's namespaces and entities at the item positions (including inside `::{...}` and after `hiding`), and the `hiding` keyword after a complete module path.
- #188: Added the `--skip-eval` compiler option and the `skip_eval` field of the project file, which compile `eval {expr0}; {expr1}` as `{expr1}`. Use it to take a debugging `eval debug_println(...)` out of a build without editing the source.
- #392, #451: The compiler now warns about an `import`, in your project's own sources, of a module belonging to a project your project does not declare as a dependency; an absolute path such as `::Hash::hash` reaches a module the same way and is warned about the same way. The warning says what breaks the import — the project in between dropping that dependency — and carries the `[[dependencies]]` entry to paste, with the version to require and the path or repository to take it from.

#### Std

- #80: Added `Array::swap` and `Array::unsafe_swap_bounds_unchecked`, which swap the two elements of an array at given indices. `swap` bounds-checks the indices; `unsafe_swap_bounds_unchecked` omits that check (the caller must ensure the indices are in range).
- #80: Added `Array::unsafe_set_bounds_unchecked`, which sets the element at a given index like `set` but omits the bounds check (the caller must ensure the index is in range). It is the counterpart of `unsafe_swap_bounds_unchecked`, for in-place write loops whose indices are already known to be in range.
- #80: Added `Array::borrow_elements` and `Array::mutate_elements` (with `_io` variants), which call a function with a pointer to the first element of an array's element buffer. `borrow_elements` borrows the array for read-only access; `mutate_elements` clones the array first if it is shared, for in-place writes. Use these for FFI that needs a raw pointer to an array's elements.
- #90: Added `Debug::assert_unique_array`, the `Array` counterpart of `Debug::assert_unique`: it asserts that an array's storage buffer is uniquely referenced (not shared), returns the array, and aborts otherwise. Use it for arrays, whose value holds the reference count in the storage buffer; `assert_unique` covers `Boxed` values.

### Changed

#### Language

- #193, #207: A program that uses a type with no size is now rejected with an error, instead of exhausting the compiler's stack. A type whose unboxed fields reach the type itself (`type A = unbox struct { b : B, n : I64 }; type B = unbox struct { a : A, m : I64 };`) has no size; make one of the types along the way boxed. A type that leads to itself at a larger type argument (`type P a = unbox struct { x : P (a, a), n : I64 };`) needs endlessly many types, whether or not a pointer lies on the way; give the recursive occurrence the same type arguments.
- #319, #379: `FFI_EXPORT` now rejects `main` and the names beginning with `fixruntime_`, which are functions the compiler implements, instead of silently renaming one of the two functions that end up under the name. A program exporting `main` used to build into one that never ran its `Main::main`. A dynamic library, which carries no entry point, may still export `main`.
- #320, #379: A program may now call the C function it exports with `FFI_CALL` of the same name; `-O max` and `-O experimental` used to fail to link it. Where a program describes one C name two ways — an `FFI_EXPORT` and an `FFI_CALL` giving different signatures, or two `FFI_CALL`s doing so — it is now rejected with an error quoting both, instead of aborting the compiler or calling the function at a signature it does not have. Two `FFI_CALL`s of one name may still read a result wider than 32 bits at either sign, which a C declaration writes identically; reading a narrower one at both signs is now rejected, since the sign is what tells the ABI which side extends the value. Write the sign the C function has, and convert on the Fix side.
- #112, #114: `FFI_EXPORT` now rejects a value whose type the C ABI cannot carry, instead of exporting a function whose arguments or result silently disagree with the C declaration. An exported function may exchange integers (`I8` to `I64`, `U8` to `U64`), floating point numbers (`F32`, `F64`), `Ptr`, boxed types (which the foreign language receives as an opaque pointer), the `Std::FFI` C type aliases such as `CInt`, and `()` as the result type. A struct, a tuple or a union is rejected, because how C passes one depends on the target; `Bool` is rejected because C leaves the width of `_Bool` to the implementation. To exchange an aggregate, take a `Ptr` to memory the foreign language owns and copy through it; see the FFI section of `Document.md`.
- #460, #509: A union of more than 256 variants is now rejected with an error. Such a union used to compile into a program that took its 257th variant for its first: `Many::v256(7)` answered `true` to `is_v0`, and `as_v0` read its payload as a value of the first variant's type, which could crash the program.

#### Std

- #80: `Array` no longer implements `Boxed`. An array now keeps its size and capacity in the value itself and only its elements on the heap, so the type is unboxed and its embedded representation is `{ptr, i64, i64}` — the pointer to the element storage, the size, and the capacity. For FFI, take an array's element pointer from `Array::borrow_elements` / `mutate_elements` in place of `FFI::borrow_boxed` / `mutate_boxed` / `_get_boxed_ptr`, and wrap an array in a boxed struct such as `Box` to pass it to C as an opaque retained pointer.
- #80: `Std::unsafe_is_unique` and `Debug::assert_unique` now require their argument to be `Boxed`.
- #80: The counting iterators produced by `Iterator::range`, `Iterator::range_step`, and `Array::to_iter` hold different fields. They yield the same elements as before, so only code that reads their fields directly is affected; see their definitions in the standard library.

#### Tool

- #96, #189: `Std::mark_threaded` has to be called before the value becomes reachable from another thread, and the call has to finish before the pointer is handed over. Its documentation and the multi-threading section of the manual now say so.
- #96, #189: `--sanitize thread`, or the `sanitize` field of the project file, builds the program with ThreadSanitizer, which reports a data race when one occurs while the program runs; use it to check that every value another thread reaches has been passed through `Std::mark_threaded`. Available on Linux; the instrumented program runs several times slower and uses much more memory.
- #157, #183: The `threaded` field of a dependency's project file no longer turns multi-threading on for the project being built; the project being built decides the setting. Building a program that calls `Std::mark_threaded` with multi-threading off now fails with an error quoting the call, so a project that depends on a library needing multi-threading sets `threaded = true` in its own project file or passes `--threaded`.
- #188: The project file's `no_runtime_check` can now be set in the `build.test` section. `fix test` reads it from there, so a project that disables the checks for its program still runs its tests with them.
- #243, #274: Every help message is now headed by the compiler's version, as `fix 1.5.0 (92a3989)` — the released version, followed by the revision the binary was built from — and `fix --version` and `fix -V` report the same line, which used to be an error. The line under the header says what `fix` is.

### Fixed

#### Language

- #461, #479: `Std::FFI::Destructor` now runs its destructor function once however many threads let go of the value at the same moment. Where two of them read the reference count before either lowered it, each left the destructor to the other and the value was freed without it ever running, so the resource it manages was never released.
- #433, #465: A value that reaches one object along more than one path, as nested `Tree { children : [child, child] }` does, is now marked in time proportional to the objects it holds. Marking is what a global value's initialization does to its result and what `Std::mark_threaded` does to the value it is given; it used to walk each path in turn, so a global value holding 29 objects took 2.3 seconds to mark and one holding 41 objects took hours.
- #391, #438: An absolute path now reaches a value of another module without the module being imported, whether the value's name begins with `_` or is a struct field's `@`-headed getter. `::Lib::_helper` and `::Lib::Box2::@v` were reported as `Cannot find entity named ...` although both exist, and importing `Lib` was the only way to write them.
- #351, #369: A value written under a namespace named after a trait, where the trait declares a member of that value's name, is now reported as a name defined twice. `trait c : Foo { bar : c -> I64; }` beside `namespace Foo { bar : I64 -> String; bar = ...; }` used to compile: the trait's member took the name, the value written for it was dropped along with its type signature, and the body was never type-checked. The report points at the member's declaration and at the value's.
- #335, #396: A trait member whose declared type does not name the trait's type variable is now rejected, instead of compiling into a program whose result changed from build to build. `trait c : Make { make : [?it : Iterator, Item ?it = c] I64 -> ?it; }` leaves `c` to the constraint `Item ?it = c`, and what stands behind an opaque type is the implementation's choice, so a call had nothing to pick the implementation by: which one it reached was decided by the order the compiler happened to finish type checking in. Name the trait's type variable in the member's type — as an argument, as the result, or inside either — and write the constraint beside it: `to_iter : [?it : Iterator, Item ?it = Elem c] c -> ?it`.
- #332, #508: An equality constraint that names an opaque type is now required to have an opaque type as the first argument of its left side, instead of being accepted with another type there. `foo : [?s : ToString, c : Rebuildable, Elem c = I64, Rebuild c ?s = Array I64] c -> ?s;` puts `?s` in an argument of an equality on `c`, which asks the call site to relate a type the signature hides from it to another one. The body of `foo` and the call site each settled `?s` on a type of their own, `fix check` reported no error, and `fix build` aborted the compiler.
- #171, #381: A program whose types the compiler cannot finish reducing is now rejected with an error, instead of overflowing the compiler's stack. An implementation that gives an associated type a value carrying that same associated type (`type El I64 = Array (El I64);`) and an equality constraint that does the same (`[?out : Iterator, Item ?out = (I64, Item ?out)]`) both aborted the build with `fatal runtime error: stack overflow`, under `fix check` as well as `fix build`. The report names the steps that lead from the type back to itself, and points at every implementation and constraint on them. Where each step asks about a larger type instead of the same one (`type El (Wrap a) = El (Wrap (Wrap a));`), it reports that the reduction reached a type nested more than 500 deep.
- #246, #276: A trait constraint that is deduced from itself is now rejected, instead of being accepted with nothing behind it. With `impl Wrap c : Holder { type Held (Wrap c) = Wrap (Wrap c); }`, the instance `impl [c : Holder, Held c = e, e : Show] Wrap c : Show` made deducing `Wrap (Wrap I64) : Show` ask for `Wrap (Wrap I64) : Show` again, and the program compiled although nothing gives `Show` to anything. Where each step asks about a larger type instead of the same one (`type Held (Wrap c) = Wrap (Wrap (Wrap c));`), the compiler used to take memory without end and report nothing; it now reports that the deduction asks about types nested more than 500 deep.
- #169, #282: A struct literal that gives one field twice is now reported as an error, instead of aborting the compiler. `S { a : 1, a : 2 }` on `type S = struct { a : I64 };` aborted with `called 'Option::unwrap()' on a 'None' value`. The report points at the repeated name and carries the first occurrence of it, and a struct pattern that repeats a field (`let S { a : x, a : y } = s;`) is reported the same way. A field list wrong in several ways at once now reports all of them in one compilation, in a pattern as in a literal, and a name the struct does not declare is reported at that name.
- #168, #201: A struct pattern now requires the type at its head to be a struct, and reports an error otherwise. `let MyUnion { a : x } = u;` took the union's tag for the payload of `a`, and the build aborted; `let Item { data : x } = 42;`, whose head names an associated type, aborted the compiler.
- #174, #216: A trait alias that stands for one trait along two paths is now accepted. `trait Ring = Ordered + Showable;`, where `Ordered` and `Showable` both stand for `Additive`, was rejected as circular aliasing, and the message named a trait of the standard library that the program never mentions.
- #167, #200: Two implementations of one trait are now reported as overlapping when one of their heads takes a type parameter of a higher kind, as `impl [f : *->*] Bar f : MyTrait` does beside `impl Bar Array : MyTrait`. Both used to be accepted, and the order they were written in decided which one a call reached.
- #231, #234, #235: A program that uses an unboxed struct of one field whose type names the struct itself now compiles at `-O max` and `-O experimental`. `type Phantom a = unbox struct { x : I64 }; type C = unbox struct { y : Phantom C };` aborted the compiler as soon as a value of `C` was used, and so did `it.mod_next(f)`, which updates the one field of a `Std::DynIterator`.
- #239, #248: A program whose types name each other in a cycle and reach a type of a higher-kinded parameter now compiles at `-O max` and `-O experimental`. `type [f : *->*] H f = unbox struct { d : f I64 }; type Y a = unbox struct { p : Array X, q : a }; type X = unbox struct { r : Y I64, s : H Array };` aborted the compiler, and writing the two fields of `X` in the other order made the same program compile.
- #245, #280: A program whose types reach themselves at a larger type argument through a field holding nothing now compiles at `-O max` and `-O experimental`, as it already did at `-O none` and `-O basic`. `type [f : *->*] H f = unbox struct { d : f I64 }; type Ph a = unbox struct { z : I64 }; type Y a = unbox struct { p : Ph (Y (Array a)), h : H Array };` consumed memory without end at the higher levels.
- #310, #330: A definition that gives its opaque return type a concrete type containing that opaque type itself is now reported as an error, instead of making the compiler run without end. `f : [?it : Iterator, Item ?it = I64] I64 -> ?it; f = |n| f(n + 1);` passed `fix check` and never finished building; where the opaque return types of several values are written in terms of each other, the report names them all.
- #357, #393: Two traits, the full name of one ending with the full name of the other, now compile. In module `Main`, `trait a : Foo { ... }` written beside `namespace Main { trait a : Foo { ... } }` declares `Main::Foo` and `Main::Main::Foo`, and the compiler aborted with `called 'Option::unwrap()' on a 'None' value`; in the editor the abort took the diagnostics of every file down with it for the rest of the session. Both traits are usable now, named as `::Main::Foo` and `Main::Main::Foo`, and a reference that could mean either one is reported as ambiguous, as it already was for two types or two values named this way. A trait name that an implementation's head leaves ambiguous is reported once in the editor, where it used to be reported twice.

#### Std

- Fixed a bug where `String::from_bytes` updated the length of a shared byte array in place instead of cloning it, truncating the caller's array.
- #132, #178: Creating or growing an array whose elements need more bytes than the address space holds now aborts the program. `Array::empty(2305843009213693952) : Array I64` used to return an array whose memory had room for none of its elements, so writing to that array corrupted the heap. `--no-runtime-check` disables this check as it does the bounds checks.
- #128: An `Array` whose element type is boxed now uses one pointer of memory per element. An array of a boxed struct of eight `I64` fields used nine times the memory it needed.

#### Tool

- #211, #400, #501: `--emit-llvm`, `--emit-rc-ir` and `--emit-symbols` now write their dumps however much of the build is cached. A build repeated in a directory answered from its object files and wrote no dump while exiting 0, so a dump an earlier build had left in place was read as this build's.
- #256, #434, #501: `--cu-size`, and the `cu_size` field of the project file, are now honored on every build. Only the first build in a directory divided itself as asked; every later one reported `Using cached object files.` and kept the division the first had made.
- #292, #501: A build now generates its own object files where a previous build's were made for another CPU. A build compiles for the CPU of the machine it runs on, and the cache carried no record of which CPU that was, so a `.fixlang` directory reaching a second machine handed it object files holding instructions its CPU may lack.
- #286, #287, #478, #501: A build no longer reuses object files generated under settings that change the code they hold, such as `--emit-symbols`, which renames the symbols a backtrace of the built program shows.
- #285, #501: `fix build` and `fix run` now share their object files, so running one after the other in a directory compiles the program once. They generate the same code, and each used to compile the whole program again.
- #439: LSP: A source that writes a type with its namespace, as `Std::I64` does, no longer makes the language server answer a semantic-tokens request with tokens that cover one another, which the protocol forbids and every client reads its own way. The whole path was colored as one type, on top of the namespace and the type the base layer had already colored on their own.
- #309, #344: LSP: Asking for completion in a body that holds a `match` whose variant name the compiler cannot resolve no longer ends the language server. Editing a `match` passes through that state all the time — a typo in a variant name, a matched value whose type is not written yet, a variant of another union — and the server exited on the completion request, so the editor lost diagnostics, completion and hover until it was restarted.
- #151, #184: `fix build` now exits with a failure status when linking fails. It printed the linker's error and exited 0, so a build that produced no output file looked like a success to `fix build && ./prog`, to a `make` rule and to a CI step.
- #114: On AArch64 targets (Apple Silicon and other 64-bit ARM), an integer narrower than 32 bits crossed the FFI boundary as a wrong number: a function exported with `FFI_EXPORT` returned one to its foreign caller, and `FFI_CALL` passed one to a foreign function that takes one. A function of type `I8 -> I8 -> I8` exported and called from C with `-100` and `30` answered 186 instead of -70. This covers `I8`, `U8`, `I16` and `U16`, and the `Std::FFI` aliases of the same widths such as `CChar` and `CShort`. x86-64 targets were unaffected.
- #114: Writing `()` as a parameter type in `FFI_CALL` now reports an error pointing at that parameter, instead of aborting the compiler.
- #102, #109: Tail call optimization now applies in more cases. Code that overflowed the stack at `-O none` or `-O basic` — typically a loop written with monadic binds, whose result or state is too wide for the target's registers — now runs in constant stack.
- #102, #109: Tail call optimization now applies with `-O none` and with `-g` as well. `-g` used to suppress it so that a debugger saw a stack frame for every call; a call in tail position no longer appears in a backtrace.
- #76, #106: Compiling a function of many parameters takes far less time and memory. The cost used to double with each parameter, so a function of 13 or more parameters aborted the compiler with a stack overflow.
- #101, #104: The compiler no longer aborts with a stack overflow when compiling a module whose expressions nest very deeply — for example a module with several hundred top-level values all sequenced from `main`, or a single deep `let` / `;;` chain.
- #29, #89: Building with debug information (`-g`) no longer crashes on a program that uses a recursive type, such as `type Tree = box union { leaf : (), node : (Tree, Tree) };`.
- #107, #150: Building with debug information (`-g`) now works at every optimization level. Passing `-g` together with an explicitly given `-O basic` aborted the build with `function declaration may only have a unique !dbg attachment` followed by `LLVM ERROR: Broken module found`.
- #80: With debug information (`-g`), a debugger now displays the elements of an `Array`, and of the byte array inside a `String`. gdb refused with an "access outside bounds of object" error, and recent lldb showed wrong values for every element after the first. Each array is recorded as having 100 elements, of which the first `<array size>` are the valid ones.
- #130, #145: The compiler no longer hangs or aborts on a program whose global definitions name each other in a cycle, such as `a = b; b = c; c = a;` or three functions each of which only calls the next. This affected `-O max` and above.
- A source file listed in `fixproj.toml` that does not exist on disk now produces a clear error that points at the offending entry in the project file (e.g. `files = ["test.fix"]`), instead of an opaque, location-less "Failed to canonicalize path" message.
- LSP: Errors whose cause is not in any source file (e.g. a missing source file or an incompatible `fix_version` declared in `fixproj.toml`) are now anchored to `fixproj.toml` so editors display them. Previously such location-less diagnostics were published against the project directory, which editors cannot attach a diagnostic to, so the message was silently dropped (appearing as an empty/invisible error).
- `fix run` and `fix test` no longer crash on startup in debug builds of `fix` (released builds were unaffected).
- `fix test` now accepts the `--no-runtime-check` flag, like `fix build` and `fix run`.
- #80: Building a project after upgrading `fix` no longer hangs. The type-checking cache is now regenerated when the compiler itself changes, where before it was reused as long as the source was unchanged and a newer `fix` could misread a cache an older one wrote.
- #172, #228: LSP: A completion request no longer makes the language server exit when the file annotates an expression with an unknown type variable, such as `let x = (3 : b);`. The request now answers with type-aware candidates, including when the cursor is inside the annotated expression itself.
- #213, #244: An internal compiler error is now reported on its own. The message used to be followed by a second error, or by a crash that buried it.
- #173, #249, #307: LSP: A completion request no longer clears the errors reported for another file. A project that did not compile could show no error in the editor until the file holding the error was edited again.
- #255, #293: The project file's `[build] output_type` now decides what `fix build` produces. A build produced an executable whatever the field said, so a project asking for a dynamic library got one only by passing `--output-type dylib` on every build.
- #270, #293: `fix run` and `fix test` no longer die with signal 11 in a project whose `[build] output_type` is `"dylib"`. They built a dynamic library and executed it as if it were a program.
- #270, #293: `fix test` no longer writes the test binary over the program, in a project whose `[build] output` names where `fix build` writes it. Give `fix test` a `-o` to keep a test binary.
- #253, #293: Building a dynamic library and an executable in one directory now works in either order. The second build reused the object files of the first and failed to link them, reporting `relocation R_X86_64_32S ... recompile with -fPIC` or `undefined reference to 'main'`; deleting the `.fixlang` directory was the way out.
- #283, #297, #316: Two global values of one namespace whose names differ only in characters that are not letters or digits are no longer compiled from one another's body. A struct's field `b` comes with an accessor named `@b`, so a program that also defines `_b` beside it read one of the two where it wrote the other, and a type error in the body that was dropped went unreported.
- #288, #312, #339: The compiler's caches now tell two source files of equal content apart. Building `a/main.fix` and `b/main.fix`, whose contents are equal, from one working directory made the second build reuse what was compiled for the first: a warning the build owed was reported in the other file, or dropped; a `-g` build could abort with `called 'Option::unwrap()' on a 'None' value`; and the debug information named the other file. Moving a source file, or the project directory itself, without editing anything left the debug information of the next `-g` build pointing at where the source used to be.
- #304, #371: The type-check cache now sees the sizes of the C types. Editing `.fixlang/c_types.json` and building again in the same directory served bodies checked against the old sizes, and the compiler aborted with `called 'Option::unwrap()' on a 'None' value`, naming no file and reporting no diagnostic. The sizes decide the Fix type the parser gives a `CInt` in an `FFI_CALL` signature, and the implementations converting to a C type that the compiler builds; neither passes through a source, which is all the key used to read.
- #152, #284, #325: Building a dynamic library now works at `-O none` and `-O basic`. A library of a program that reads a struct field failed to link with `version node not found for symbol Get#Main::Point::@x#...`, because a field getter's name carries `@`, which the linker reads as the separator of `symbol@version`. A field getter now enters the symbol table with `@` written as `$`, which is also how a debugger and `nm` spell it.

## [1.4.0] - 2026-06-22

### Added

#### Tool

- LSP: Added support for the "Semantic Tokens" feature, providing editor syntax highlighting from the language server without a separate TextMate / tree-sitter grammar. A lexical layer colors tokens even while the buffer is syntactically broken; once the file type-checks, an AST overlay refines identifiers (locals, globals, struct / union / trait names, type parameters, union variants, fields), merged line by line so an edit only affects the edited line.

### Changed

#### Tool

- LSP: Diagnostics now run on every edit (on-type, debounced) over the live, possibly unsaved buffer, instead of only on save. Two new settings, read from the editor via `workspace/didChangeConfiguration`, control analysis timing: `fix.analyze.delayMs` (on-type debounce in milliseconds; default 400, where 0 disables on-type analysis) and `fix.analyze.onSave` (default true). See `Document.md` / `Document-ja.md` for per-editor configuration snippets.

### Fixed

#### Tool

- LSP: Fixed a busy loop in which the language server spun at ~100% CPU (leaving an orphaned process behind) when the editor closed its stdin. The server now treats stdin EOF as a shutdown signal and terminates.

## [1.3.0] - 2026-06-19

### Added

#### Tool

- Added `fix check` command. This command checks whether a Fix project compiles without errors by performing type-checking on all entities including test code, without generating a binary.
- Now you can write `[[test-dependencies]]` in `fixproj.toml`. You can specify dependencies which are only used for tests here. If your project is used as a library, dependencies written in `[[test-dependencies]]` will not be installed. `fix deps add --test <dependency>` adds a dependency to `[[test-dependencies]]`. `fix deps update --test` and `fix deps install --test` will also update lock file and install dependencies including test dependencies.
- Added `rev` and `tag` fields for git dependencies in `fixproj.toml`. You can now pin a dependency to a specific commit hash (`rev`) or tag (`tag`). For example: `git = { url = "...", tag = "v1.0.0" }`. Pinning only applies to direct dependencies; transitive dependencies with `rev`/`tag` are ignored with a warning.
- LSP: Added support for "Find All References" and "Call Hierarchy" features. You can now find all references to global values, types, traits, and associated types across your project, and navigate call hierarchies of functions.
- LSP: Added support for the "Workspace Symbols" feature. You can now search across your project for types, type aliases, traits, trait aliases, trait members, global values, and trait implementations (e.g. via `Ctrl+T` / `Ctrl+Shift+O` in VSCode). Results are filtered to user-defined symbols; standard library and dependency symbols are excluded.
- LSP: Added a quick fix for missing trait implementation items. When a trait implementation is missing members or associated types, a code action is now available to insert stub implementations automatically.
- LSP: Added support for the "Rename Symbol" feature. You can now rename local variables, global values, types, traits, type aliases, trait aliases, associated types, struct fields, and union variants across your project. Renaming a struct or union type also updates its auto-method namespace path.
- Added the `DEPRECATED[name, "msg"];` pragma to mark a global value or trait member as deprecated; uses produce a compiler warning carrying the author's message. New CLI flags `--allow-deprecated` and `--deny-deprecated` suppress or promote the warnings. See `Document.md` / `Document-ja.md` "Deprecation".

#### Std

- Add `Std::String::contains` function to check if a string contains a given substring.
- Add `Std::StringBytesIterator` type and `Std::String::to_iter_bytes` function to iterate over the bytes of a string.
- Add `Std::Iterator::get_last` function to get the last element of an iterator.
- Add `Std::One` trait (with member `one : a`) and `Std::Multiplicative` trait alias (`Mul + One`), mirroring the existing `Zero` / `Additive` pair. `One` is implemented for `I8`, `U8`, `I16`, `U16`, `I32`, `U32`, `I64`, `U64`, `F32`, and `F64`.

### Changed

#### Language

- Added opaque types. You can use type variables starting with `?` (e.g. `?it`) in type signatures to hide concrete return types behind trait constraints. See `Document.md` for details.
- Type variable names now allow underscores (e.g. `my_var`).
- Changed the way the compiler checks whether the type signature given to a trait member implementation matches the one required by the trait definition. Previously, it checked for syntactic consistency, but now it allows more flexible verification of type equivalence. For example, previously when implementing `Iterator`, you had to write the type signature for `advance` using `Item`, like `MyType -> Option (MyType, Item MyType)`, but now you can write the resolved type directly instead of `Item MyType`.
- Strengthened the well-formedness check for type schemes in the presence of associated types. Every generalized type variable of a type signature must now appear at a "fixed" position — that is, outside of any associated type application — as required by section 5.1 of "Associated Type Synonyms" (Chakravarty, Keller, Peyton Jones). This applies to both global value signatures and trait member implementations, and subsumes the previous, weaker check that only rejected trait method signatures in which the trait type variable did not appear at all.
- The first argument of `FFI_EXPORT[...]` now accepts a `::`-separated path (e.g. `FFI_EXPORT[Foo::bar, c_bar];`), not just a bare name. The path is interpreted relative to the surrounding namespace; absolute paths starting with `::` are rejected with a friendly error. Existing `FFI_EXPORT[bare_name, c_name];` forms continue to work unchanged.

#### Tool

- `fix build` and `fix run` now type-check all symbols, not just those reachable from the entry point. Previously, only required symbols were type-checked.
- Docs: In the "Values" section, values that are generated from trait members now have an additional description "Trait member of `<trait_name>`".
- LSP: Introduced automatic lock file management for language server. The language server now automatically generates and updates `.fixlang/fixdeps.lsp.lock` when the project file changes, without requiring manual `fix deps update` commands.
- `preliminary_commands` (both `[build]` and `[build.test]`) now require user approval before they run. On the first encounter `fix build` / `fix run` / `fix test` prompt with a 3-choice menu — `y` to trust the project and record the approval in `~/.fixtrust.toml`, `o` to allow just this invocation, or `n` to abort. Approvals for git dependencies are scoped to the pinned commit hash and re-prompted when the dependency advances to a new commit; approvals for the root project and local-path dependencies are scoped to the absolute path. CI or other non-interactive runs should pass `--allow-preliminary-commands` to bypass the prompt without writing to the trust store. See `Document.md` / `Document-ja.md` "Approval of preliminary_commands" for details.

#### Std

- Iterator-returning functions in `Std` (e.g. `Iterator::map`, `filter`, `flat_map`, `range`, `Array::to_iter`, `Option::to_iter`, `String::split`) now return an opaque iterator type instead of a concrete one such as `MapIterator` or `ArrayIterator`. Code that annotates the return of these functions with a concrete iterator type (e.g. `(xs.to_iter : ArrayIterator I64)`) must be updated; call sites that just chain combinators or consume the iterator are unaffected.

### Fixed

#### Language

- Fixed a compiler error that occurred when writing a namespace-qualified impl type (e.g., `Main::MyType`) in an associated type implementation line, such as `type MyElem Main::MyType = ...;`.
- Fixed an issue where allocating more than 4 GiB of memory at once caused incorrect behavior.

#### Tool

- Fixed an issue where the dependency manager could not resolve a git dependency whose version came from an annotated tag. The tag-enumeration path now peels the tag object to its underlying commit, so version-range resolution against repositories that use `git tag -a` works correctly.
- LSP: Fixed an issue where associated types were missing from completion items.
- LSP: Fixed an issue where "unknown associated type" errors did not offer an "add import" quick fix.
- Docs: Fixed an issue where accessor functions (`@`, `set_`, `mod_`, `act_` for struct fields; `as_`, `is_`, `mod_` for union variants) for private (underscore-prefixed) fields or variants were included in the documentation when `--with-compiler-defined-methods` was given. They are now hidden unless `--with-private` is also given.
- Docs: Fixed an issue where private (underscore-prefixed) fields and variants of structs/unions were listed as subsections of their containing type. They are now hidden unless `--with-private` is given.
- Docs: Fixed a panic that occurred when generating documentation with `--with-compiler-defined-methods`.

## [1.2.0] - 2026-02-10

### Added

#### Language 

- Added the [index syntax](./Document.md#index-syntax), `Indexable` trait and `Indexable` namespace in `Std` module.
- FFI_CALL, FFI_CALL_IO and FFI_CALL_IOS now support variadic functions. You can specify `...` at the end of parameter type list in the function signature to indicate that the function is variadic.

#### Std

- Implement `Array a : Indexable`, `String : Indexable`.
- Added the `Identity` and `Const` types in `Std`. They are functors, and `Identity` is also a monad.
- Add `Std::ToX` traits for all primitive numeric types and C types (e.g., `Std::ToI32`, `Std::ToCUnsignedInt`, etc.). Each trait has a member function to cast a value into the target type. The cast functions are named in lower snake case, such as `i32` and `c_unsigned_int`. They do not have prefixes like `to_`.
- Add `Std::String::starts_with` and `Std::String::ends_with` functions.
- Add `Std::Array::@size`, `Std::Array::@capacity`, `Std::String::@size` functions to get the size and capacity of arrays and strings.
- Add `unsafe_from_c_str_ptr_io : Ptr -> IO String`.

#### Compiler 

- Added `--backtrace` option to `build`, `run`, `test` commands to enable printing backtrace when a runtime error occurs.
- Added `backtrace` field to `build` and `build.test` section of `fixproj.toml` file.
- Added `--no-runtime-check` option to `build`, `run` commands to disable runtime checks (e.g., out-of-range check for array access).
- Added `no_runtime_check` field to `build` section of `fixproj.toml` file.
- Added `--disable-cpu-feature <feature>` option to `build`, `run`, `test` commands to disable specific CPU features. Add `disable_cpu_features` field to `build` and `build.test` section of `fixproj.toml` file.
- Added `unwrap-newtype` optimization, which removes unnecessary newtype wrappers, e.g., `type Foo = unbox struct { data : Bar }`.
- Added `inline-local` optimization, which tries to inline local functions.

#### Tool

- Edit: Added `fix edit explicit-import` command to rewrite import statements to explicitly import only the necessary entities.
- LSP: Added a feature to automatically add import statements when completing entity names, if necessary.
- LSP: Added Quick Fix for "Unknown name" and "Unknown type" errors to add import statements.
- LSP: You can now show documents and jump to definitions from entities written in import statements.

### Changed

#### Language

- Type variables used in trait member definitions can no longer be used in implementations of those trait members. For example, for `trait [f:*->*] f : Functor { map : (a -> b) -> f a -> f b; }`, you cannot use `a`, `b` in `impl MyType : Functor { map = |f : a -> b, x : MyType a| ...}`. This change ensures that renaming type variables in trait definitions does not affect implementations of trait members. Instead, you can introduce type variables in type signatures of trait members. For example, you can write `impl MyType : Functor { map : (a -> b) -> MyType a -> MyType b = |f : a -> b, x : MyType a| ...}`. 
- You can now refer to entities using absolute namespace syntax (e.g., `::Std::String`) without importing them.

#### Std

- Change type of `Std::FFI::boxed_from_retained_ptr` from `Ptr -> a` to `Ptr -> IO a`. Change type of `Std::FFI::boxed_to_retained_ptr` from `a -> Ptr` to `a -> IO Ptr`.
- Changed the type of `Std::FFI::Destructor::make` function to return `IO`. Correspondingly, `Std::IO::IOHandle::from_file_ptr` also returns `IO`.
- Made all values in the `Std::PunchedArray` namespace private (since they are not intended to be used directly from outside).
- `Array::empty` and `Array::fill` now verifies the capacity and size arguments at runtime to ensure they are non-negative, and raises an error if they are negative.
- Deprecated `Std::Array::get_size` in favor of `Std::Array::@size` for brevity. The old name will remain available for the foreseeable future to maintain backward compatibility.
- Deprecated `Std::String::get_size` in favor of `Std::String::@size` for brevity. The old name will remain available for the foreseeable future to maintain backward compatibility.
- Deprecated `Std::Array::get_capacity` in favor of `Std::Array::@capacity` for brevity. The old name will remain available for the foreseeable future to maintain backward compatibility.
- Deprecated numeric conversion functions `Std::<Type>::to_<target_type>` (e.g., `Std::I32::to_f64`) in favor of trait members `To<TargetType>::<target_type>` (e.g., `ToF64::f64`). The old function names will remain available for the foreseeable future to maintain backward compatibility.

#### Compiler

- When an out-of-range array access occurs, the error message now includes the index that was accessed and the size of the array.
- When `undefined` is reached, a newline is now added after the user-specified message.
- Changed the condition for inlining optimization. A function will be inlined if its complexity is below a certain threshold, regardless of the number of times it is called.

### Fixed

- LSP: Fixed an issue where documents were not displayed when hovering over type aliases and trait aliases.
- Docs: Fixed an issue where trait aliases were not displayed in the documentation generated by `fix docs`.
- Fixed an issue where `close_file` was not called if the `action` passed to `with_file` function returned an error.
- Fix #64, #65, #69, #70, #71, #72.

## [1.1.0] - 2025-09-04

### Added

- Add `fix_version` field to `fixproj.toml` file. You can specify the version of Fix necessary to compile your project.
- Add `Std::String::unsafe_from_c_str_ptr : Ptr -> String` function.
- Implement `String : FromBytes` and `String : ToBytes`. You can convert a string to a (null-terminated) byte array and vice versa.
- Add `Std::Array::search_partition_point`.
- Add new optimization ("optimization/decapturing.rs"). This will be applied when the optimization level is `max`.
- Add `bit_not` functions to each integer types.
- Add `Std::IO::input_line_s : IO String` function, which reads a line from stdin and strips the last newline characters.
- Add `Std::Iterator::check_all : [it : Iterator, Item it = a] (a -> Bool) -> it -> Bool` and `Std::Iterator::check_any : [it : Iterator, Item it = a] (a -> Bool) -> it -> Bool`.
- Add `Std::Iterator::loop_iter_s` and `Std::Iterator::loop_iter_ms`, which are similar to `Std::Iterator::loop_iter` and `Std::Iterator::loop_iter_m`, but they return a `LoopState` and allow the caller to know whether the loop ended with `break(_m)` or `continue(_m)`.
- Add `Std::IO::flush : IOHandle -> IO I32` to flush an `IOHandle`.
- Add `fix version` command.
- Add `populate : Array String -> String -> String` to populate strings into a template string, similar to "format" function in other languages, but you need to stringify values by yourself.
- Improve `Std::Array::sort_by` implementation (now it uses introsort), and added `Std::Array::sort_stable_by` function (currently implemented by merge sort).
- Add `Std::Array::resize` to resize an array to a given length, filling the new elements with a given value, or truncating the array if the new length is smaller than the current length.
- Add `Std::Array::reverse` to reverse an array.
- Add `Std::Iterator::enumerate`.
- Add `Std::Array::sort : [a : LessThan] Array a -> Array a` and `Std::Array::sort_stable : [a : LessThan] Array a -> Array a`;
- Add `Std::Array::dedup : [a : Eq] Array a -> Array a` which removes consecutive duplicate elements from the array.
- Add support for `textDocument/documentSymbol` request in the language server.

### Changed

- Update LLVM to 17.0.x.
- Improve language server protocol support.
- By creating a "# Parameters" section within a function's documentation comment and listing its arguments, the Language Server now inserts argument placeholders after autocompletion. For details, please refer to the [explanation in the documents](/Document.md#specifying-parameter-list-in-the-documentation-comment-as-a-hint-to-the-language-server).
- Now `fix docs` command skips generating document for private values (i.e., values whose names start with an underscore). If you want to generate document for them, use `--with-private` option.
- Now `fix init` command generates a sample "main.fix" file and a sample "test.fix" file in the current directory.
- Now output is not colored when stderr is not a TTY.
- Now `Std::Iterator::range` returns an empty iterator when `start > end`.
- Now `Std::Iterator::range_step` returns an empty iterator when `step * (start - end) > 0` and aborts when `step == 0`.
- Previously, the Fix standard library used `fdopen` for `Std::IO::stdin, stdout, stderr`, but it now uses the ones opened by the C runtime library.
- Changed range checking for hexadecimal and binary integer literals so that such literals can represent negative values when appropriate; for example, `0xffffffffffffffff` previously exceeded the maximum of a 64-bit signed integer and produced an error, but after this fix it is interpreted as `-1`.

### Fixed

- Fix the issue that the format of the markdown file generated by `fix docs` is broken when using markdown header lines in documentation comments.
- Fix the issue that the compiler crashes when using NBSP (U+00A0) in the source code.
- Fix the issue that the parser takes too long to parse some code (#57).
- Fix the issue that the code `let x= 42; {...}` could not be parsed (since the parser expected a space between `x` and `=`).
- Fix the issue that compiling some code causes a compiler crash (#59).
- Fix the issue that `FFI_CALL[() f(CLongLong)]` (or some of other similar expressions) could not be parsed (#62).

## [1.0.1] - 2025-02-26

### Added

- Rebuild prebuilt binaries (attached to this release). x86_64-unknown-linux-gnu is built on ubuntu-20.04, x86_64-apple-darwin is built on macos-13, aarch64-apple-darwin is built on macos-14.

### Changed

- Update the version of the Fix project "std-doc" for generating the document of the standard library to 1.0.0 to match the version of the compiler.

## [1.0.0] - 2025-02-22

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
- Add `Std::with_retained`.
- Add `{monad_expr};; {expr}` syntax, which is equivalent to `let _ = *{monad_expr}; {expr}`.
- Add `Std::Box::make` function.
- Support building dynamic libraries (use `--output-type dylib`).
- Add `match` syntax.
- Change the bit width of reference counter from 64 to 32.
- Add `Std::Arrow`, which is a higher-kinded type for functions.
- Add absolute namespace syntax: you can `::Main::X` instead of `Main::X` to refer to `X` in the top level namespace of the `Main` module.
- Implement `Zero` for `Array a`, `Iterator a`, `String`. Implement `Add` for `String`, `Array a`.
 
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
- Rename optimization lavels from `none`, `separated`, `default` to `none`, `basic`, `max`.

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