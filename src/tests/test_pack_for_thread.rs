//! `Std::FFI::pack_for_thread` and the functions that take a value back from the pointer it
//! returns: `Std::FFI::unpack_from_thread`, and the functions `Std::FFI::get_funptr_retain_for_thread`
//! and `Std::FFI::get_funptr_release_for_thread` point to.
//!
//! The pointer names the value's own object for a boxed type, and a `Std::Box` holding the value
//! for an unboxed one. These tests run each kind through the four functions under memcheck, which
//! reports a release of the wrong object as an invalid access and a missed one as a leak.

use crate::configuration::Configuration;
use crate::misc::function_name;
use crate::tests::test_util::{
    deprecation_report, run_source_assert_failed, test_source, test_source_fail,
    test_source_with_c_under,
};

/// The configuration of a program that hands values to other threads.
fn threaded_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.threaded = true;
    config
}

/// A value packed and unpacked comes back as it was, for a boxed type, for an unboxed type holding
/// no reference, and for an unboxed type holding boxed values.
#[test]
pub fn test_unpack_from_thread_gives_back_the_packed_value() {
    let source = r##"
        module Main;

        type Pair = unbox struct { name : String, count : I64 };

        main : IO ();
        main = (
            let args = *IO::get_args;
            // Built from the arguments, so that the values are on the heap with a reference count
            // rather than constants of the program.
            let n = args.@(0).@size;
            let name = "name" + n.to_string;

            let ptr = *FFI::pack_for_thread(n);
            let got : I64 = *FFI::unpack_from_thread(ptr);
            assert_eq(|_|"an I64", got, n);;

            let ptr = *FFI::pack_for_thread(Pair { name : name, count : n });
            let got : Pair = *FFI::unpack_from_thread(ptr);
            assert_eq(|_|"the name of a Pair", got.@name, name);;
            assert_eq(|_|"the count of a Pair", got.@count, n);;

            let ptr = *FFI::pack_for_thread([n, n + 1]);
            let got : Array I64 = *FFI::unpack_from_thread(ptr);
            assert_eq(|_|"an Array", got, [n, n + 1]);;

            let ptr = *FFI::pack_for_thread(Box::make(name));
            let got : Box String = *FFI::unpack_from_thread(ptr);
            assert_eq(|_|"a Box", got.@value, name);;

            pure()
        );
    "##;
    test_source(&source, threaded_config());
}

/// The functions `get_funptr_retain_for_thread` and `get_funptr_release_for_thread` point to count
/// the object the pointer names: the value's own object for a boxed type, and the box for an
/// unboxed one. C retains the pointer once, releases it once, and Fix then takes the value back;
/// the destructor inside the value runs once, after the last reference is gone.
#[test]
pub fn test_funptrs_for_thread_count_the_object_the_pointer_names() {
    let source = r##"
        module Main;

        make_resource : I64 -> IO (Destructor I64);
        make_resource = |tag| Destructor::make(tag, |tag|
            FFI_CALL_IO[() c_note_release(I64), tag].map(|_| tag)
        );

        // Retains and releases `ptr` through the functions for a value of the type `lazy` names,
        // and takes the value back.
        exercise : Lazy b -> Ptr -> IO b;
        exercise = |lazy, ptr| (
            let retain = lazy.FFI::get_funptr_retain_for_thread;
            let release = lazy.FFI::get_funptr_release_for_thread;
            FFI_CALL_IO[() c_call_refcount_funptr(Ptr, Ptr), retain, ptr];;
            FFI_CALL_IO[() c_call_refcount_funptr(Ptr, Ptr), release, ptr];;
            FFI::unpack_from_thread(ptr)
        );

        main : IO ();
        main = (
            // A boxed type: the pointer names the destructor's own object.
            let resource = *make_resource(1);
            let ptr = *FFI::pack_for_thread(resource);
            let resource : Destructor I64 = *ptr.exercise(|_| undefined("") : Destructor I64);
            assert_eq(|_|"the boxed value was destroyed while held", *FFI_CALL_IO[I64 c_was_released(I64), 1], 0);;
            assert_eq(|_|"the boxed value did not survive", resource.borrow(|tag| tag), 1);;

            // An unboxed type: the pointer names the box holding the pair.
            let resource = *make_resource(2);
            let ptr = *FFI::pack_for_thread((resource, 20));
            let got : (Destructor I64, I64) = *ptr.exercise(|_| undefined("") : (Destructor I64, I64));
            let (resource, n) = got;
            assert_eq(|_|"the unboxed value was destroyed while held", *FFI_CALL_IO[I64 c_was_released(I64), 2], 0);;
            assert_eq(|_|"the unboxed value did not survive", resource.borrow(|tag| tag) + n, 22);;

            // Dropping the pointer from C destroys the value inside the box.
            let resource = *make_resource(3);
            let ptr = *FFI::pack_for_thread((resource, 30));
            let release = (|_| undefined("") : (Destructor I64, I64)).FFI::get_funptr_release_for_thread;
            FFI_CALL_IO[() c_call_refcount_funptr(Ptr, Ptr), release, ptr];;
            assert_eq(|_|"releasing the pointer did not destroy the value", *FFI_CALL_IO[I64 c_was_released(I64), 3], 1);;

            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        #define TAG_COUNT 16

        static int64_t released[TAG_COUNT];

        void c_note_release(int64_t tag) {
            if (0 <= tag && tag < TAG_COUNT) { released[tag] += 1; }
        }

        int64_t c_was_released(int64_t tag) {
            return (0 <= tag && tag < TAG_COUNT) ? released[tag] : 0;
        }

        void c_call_refcount_funptr(void *funptr, void *value) {
            ((void (*)(void *))funptr)(value);
        }
    "##;
    test_source_with_c_under(&source, &c_source, function_name!(), threaded_config());
}

/// `pack_for_thread` puts its value into multi-threaded mode, which a build without
/// multi-threading lacks, so such a build is refused at the call.
#[test]
pub fn test_pack_for_thread_requires_multi_threading() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let ptr = *FFI::pack_for_thread([1, 2, 3]);
            let got : Array I64 = *FFI::unpack_from_thread(ptr);
            println(got.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Std::FFI::pack_for_thread` requires multi-threading",
    );
}

/// The deprecated `Std::mark_threaded` goes on asking for multi-threading as `pack_for_thread`
/// does.
#[test]
pub fn test_mark_threaded_requires_multi_threading() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let value = [1, 2, 3].mark_threaded;
            println(value.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "`Std::mark_threaded` requires multi-threading",
    );
}

/// `Std::mark_threaded` is deprecated in favor of `pack_for_thread`, which makes the pointer in the
/// same call.
#[test]
pub fn test_mark_threaded_is_deprecated_in_favor_of_pack_for_thread() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let value = [1, 2, 3].mark_threaded;
            println(value.to_string)
        );
    "##;
    let report = deprecation_report(source);
    assert!(
        report.contains(
            "`Std::mark_threaded` is deprecated: Use `Std::FFI::pack_for_thread` instead."
        ),
        "the report should name the replacement.\nreport: {}",
        report
    );
}

/// The pointer `pack_for_thread` returns is never null, even for an unboxed value whose bits are
/// all zero: a foreign library may read a null pointer as "no value yet".
#[test]
pub fn test_pack_for_thread_never_returns_null() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let zero = (*IO::get_args).@size - (*IO::get_args).@size;

            let ptr = *FFI::pack_for_thread(zero);
            assert(|_|"the pointer for an I64 zero is null", ptr != nullptr);;
            let got : I64 = *FFI::unpack_from_thread(ptr);
            assert_eq(|_|"an I64 zero", got, 0);;

            let ptr = *FFI::pack_for_thread(nullptr);
            assert(|_|"the pointer for a null Ptr is null", ptr != nullptr);;
            let got : Ptr = *FFI::unpack_from_thread(ptr);
            assert(|_|"a null Ptr", got == nullptr);;

            let ptr = *FFI::pack_for_thread(());
            assert(|_|"the pointer for a unit is null", ptr != nullptr);;
            let got : () = *FFI::unpack_from_thread(ptr);

            pure()
        );
    "##;
    test_source(&source, threaded_config());
}

/// A program without multi-threading that calls both `Std::mark_threaded` and
/// `Std::FFI::pack_for_thread` is told about each of them.
#[test]
pub fn test_both_functions_requiring_multi_threading_are_reported() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let value = [1, 2, 3].mark_threaded;
            let ptr = *FFI::pack_for_thread(value);
            let got : Array I64 = *FFI::unpack_from_thread(ptr);
            println(got.to_string)
        );
    "##;
    let report = run_source_assert_failed(source, Configuration::develop_mode());
    for name in ["`Std::mark_threaded`", "`Std::FFI::pack_for_thread`"] {
        assert!(
            report.contains(&format!("{} requires multi-threading", name)),
            "the report should name {}.\nreport: {}",
            name,
            report
        );
    }
}
