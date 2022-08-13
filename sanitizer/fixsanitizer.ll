; ModuleID = 'fixsanitizer.6807c95c-cgu.0'
source_filename = "fixsanitizer.6807c95c-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>" = type { %"std::thread::local::lazy::LazyKeyInner<core::cell::Cell<(u64, u64)>>", i8, [7 x i8] }
%"std::thread::local::lazy::LazyKeyInner<core::cell::Cell<(u64, u64)>>" = type { %"core::cell::UnsafeCell<core::option::Option<core::cell::Cell<(u64, u64)>>>" }
%"core::cell::UnsafeCell<core::option::Option<core::cell::Cell<(u64, u64)>>>" = type { %"core::option::Option<core::cell::Cell<(u64, u64)>>" }
%"core::option::Option<core::cell::Cell<(u64, u64)>>" = type { i64, [2 x i64] }
%"core::sync::atomic::AtomicUsize" = type { i64 }
%"[closure@std::panicking::begin_panic<&str>::{closure#0}]" = type { { [0 x i8]*, i64 }, %"core::panic::location::Location"* }
%"core::panic::location::Location" = type { { [0 x i8]*, i64 }, i32, i32 }
%"std::collections::hash::map::HashMap<i64, ObjectInfo>" = type { %"hashbrown::map::HashMap<i64, ObjectInfo, std::collections::hash::map::RandomState>" }
%"hashbrown::map::HashMap<i64, ObjectInfo, std::collections::hash::map::RandomState>" = type { { i64, i64 }, %"hashbrown::raw::RawTable<(i64, ObjectInfo)>" }
%"hashbrown::raw::RawTable<(i64, ObjectInfo)>" = type { %"core::marker::PhantomData<(i64, ObjectInfo)>", %"hashbrown::raw::RawTableInner<alloc::alloc::Global>" }
%"core::marker::PhantomData<(i64, ObjectInfo)>" = type {}
%"hashbrown::raw::RawTableInner<alloc::alloc::Global>" = type { %"alloc::alloc::Global", i64, i8*, i64, i64 }
%"alloc::alloc::Global" = type {}
%ObjectInfo = type { i64, i64, i64, %"alloc::string::String" }
%"alloc::string::String" = type { %"alloc::vec::Vec<u8>" }
%"alloc::vec::Vec<u8>" = type { { i8*, i64 }, i64 }
%"core::fmt::Formatter" = type { { i64, i64 }, { i64, i64 }, { {}*, [3 x i64]* }, i32, i32, i8, [7 x i8] }
%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]" = type { i64**, %"core::option::Option<std::sync::mutex::Mutex<i64>>"**, %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"* }
%"core::option::Option<std::sync::mutex::Mutex<i64>>" = type { i64, [2 x i64] }
%"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok" = type { {} }
%"std::sync::mutex::Mutex<i64>" = type { %"std::sys_common::mutex::MovableMutex", %"std::sync::poison::Flag", [3 x i8], i64 }
%"std::sys_common::mutex::MovableMutex" = type { %"std::sys::unix::locks::futex::Mutex" }
%"std::sys::unix::locks::futex::Mutex" = type { %"core::sync::atomic::AtomicU32" }
%"core::sync::atomic::AtomicU32" = type { i32 }
%"std::sync::poison::Flag" = type { %"core::sync::atomic::AtomicBool" }
%"core::sync::atomic::AtomicBool" = type { i8 }
%"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>" = type { %"once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>", i64* }
%"once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>" = type { %"once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>" }
%"once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>" = type { %"core::marker::PhantomData<*mut once_cell::imp::Waiter>", %"core::sync::atomic::AtomicUsize", %"core::cell::UnsafeCell<core::option::Option<std::sync::mutex::Mutex<i64>>>" }
%"core::marker::PhantomData<*mut once_cell::imp::Waiter>" = type {}
%"core::cell::UnsafeCell<core::option::Option<std::sync::mutex::Mutex<i64>>>" = type { %"core::option::Option<std::sync::mutex::Mutex<i64>>" }
%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]" = type { i64**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"* }
%"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>" = type { i64, [7 x i64] }
%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>" = type { %"std::sys_common::mutex::MovableMutex", %"std::sync::poison::Flag", [3 x i8], %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>" }
%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>" = type { %"std::collections::hash::map::HashMap<i64, ObjectInfo>" }
%"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>" = type { %"once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", i64* }
%"once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>" = type { %"once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>" }
%"once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>" = type { %"core::marker::PhantomData<*mut once_cell::imp::Waiter>", %"core::sync::atomic::AtomicUsize", %"core::cell::UnsafeCell<core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>>" }
%"core::cell::UnsafeCell<core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>>" = type { %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>" }
%"std::thread::local::AccessError" = type {}
%"core::option::Option<core::fmt::Arguments>" = type { {}*, [5 x i64] }
%"core::fmt::builders::DebugStruct" = type { %"core::fmt::Formatter"*, i8, i8, [6 x i8] }
%"core::str::error::Utf8Error" = type { i64, { i8, i8 }, [6 x i8] }
%"core::fmt::Arguments" = type { { [0 x { [0 x i8]*, i64 }]*, i64 }, { i64*, i64 }, { [0 x { i8*, i64* }]*, i64 } }
%"core::result::Result<&str, core::str::error::Utf8Error>" = type { i64, [2 x i64] }
%"core::ffi::c_str::CStr" = type { [0 x i8] }
%"unwind::libunwind::_Unwind_Exception" = type { i64, void (i32, %"unwind::libunwind::_Unwind_Exception"*)*, [6 x i64] }
%"unwind::libunwind::_Unwind_Context" = type { [0 x i8] }

@_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE = external thread_local global %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"
@alloc365 = private unnamed_addr constant <{ [70 x i8] }> <{ [70 x i8] c"cannot access a Thread Local Storage value during or after destruction" }>, align 1
@alloc368 = private unnamed_addr constant <{ [79 x i8] }> <{ [79 x i8] c"/rustc/a8314ef7d0ec7b75c336af2c9857bfaf43002bfc/library/std/src/thread/local.rs" }>, align 1
@alloc367 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [79 x i8] }>, <{ [79 x i8] }>* @alloc368, i32 0, i32 0, i32 0), [16 x i8] c"O\00\00\00\00\00\00\00\A5\01\00\00\1A\00\00\00" }>, align 8
@vtable.0 = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17h92e001d5e4efd74cE" to i8*), i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17hc9f8af2660d4514aE" to i8*) }>, align 8
@_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E = external local_unnamed_addr global %"core::sync::atomic::AtomicUsize"
@alloc73 = private unnamed_addr constant <{}> zeroinitializer, align 8
@alloc406 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Option::unwrap()` on a `None` value" }>, align 1
@vtable.3 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00", i8* bitcast (i1 (%"std::thread::local::AccessError"*, %"core::fmt::Formatter"*)* @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h514ef917cd5ecc1bE" to i8*) }>, align 8
@alloc418 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@vtable.5 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"core::str::error::Utf8Error"*, %"core::fmt::Formatter"*)* @"_ZN64_$LT$core..str..error..Utf8Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h864a228d6ab6973cE" to i8*) }>, align 8
@vtable.6 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void ({ i64*, i8 }*)* @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 ({ i64*, i8 }*, %"core::fmt::Formatter"*)* @"_ZN76_$LT$std..sync..poison..PoisonError$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h69df1c324ff6e669E" to i8*) }>, align 8
@vtable.7 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (i64**, %"core::fmt::Formatter"*)* @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hc715f6c95a655b17E" to i8*) }>, align 8
@alloc430 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"PoisonError" }>, align 1
@vtable.8 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i64 ({ [0 x i8]*, i64 }*)* @"_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17ha7daf7c2b2ea8d27E" to i8*) }>, align 8
@alloc67 = private unnamed_addr constant <{ [16 x i8] }> <{ [16 x i8] c"\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF" }>, align 16
@vtable.b = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h42a39cd9ab169dceE" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E" to i8*) }>, align 8
@vtable.c = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h8d298f77ff4ec3b3E" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E" to i8*) }>, align 8
@alloc462 = private unnamed_addr constant <{ [42 x i8] }> <{ [42 x i8] c"Lazy instance has previously been poisoned" }>, align 1
@alloc463 = private unnamed_addr constant <{ [90 x i8] }> <{ [90 x i8] c"/home/maruyama/.cargo/registry/src/github.com-1ecc6299db9ec823/once_cell-1.13.0/src/lib.rs" }>, align 1
@alloc461 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [90 x i8] }>, <{ [90 x i8] }>* @alloc463, i32 0, i32 0, i32 0), [16 x i8] c"Z\00\00\00\00\00\00\00\CF\04\00\00\19\00\00\00" }>, align 8
@_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE = internal global <{ [16 x i8], [16 x i8], i8* }> <{ [16 x i8] zeroinitializer, [16 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<i64>"*)* @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE to i8*) }>, align 8
@_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE = internal global <{ [16 x i8], [56 x i8], i8* }> <{ [16 x i8] zeroinitializer, [56 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)* @_ZN4core3ops8function6FnOnce9call_once17h792230541602dafdE to i8*) }>, align 8
@alloc70 = private unnamed_addr constant <{ [54 x i8] }> <{ [54 x i8] c"[report_malloc] Failed to convert given name to &str.\0A" }>, align 1
@alloc71 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [54 x i8] }>, <{ [54 x i8] }>* @alloc70, i32 0, i32 0, i32 0), [8 x i8] c"6\00\00\00\00\00\00\00" }>, align 8
@alloc489 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"src/lib.rs" }>, align 1
@alloc466 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00#\00\00\00!\00\00\00" }>, align 8
@alloc468 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00$\00\00\00)\00\00\00" }>, align 8
@alloc254 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"Object id=" }>, align 1
@alloc77 = private unnamed_addr constant <{ [37 x i8] }> <{ [37 x i8] c" is allocated. refcnt=(0 -> 1), addr=" }>, align 1
@alloc78 = private unnamed_addr constant <{ [9 x i8] }> <{ [9 x i8] c", code = " }>, align 1
@alloc225 = private unnamed_addr constant <{ [1 x i8] }> <{ [1 x i8] c"\0A" }>, align 1
@alloc76 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc254, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [37 x i8] }>, <{ [37 x i8] }>* @alloc77, i32 0, i32 0, i32 0), [8 x i8] c"%\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [9 x i8] }>, <{ [9 x i8] }>* @alloc78, i32 0, i32 0, i32 0), [8 x i8] c"\09\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [1 x i8] }>, <{ [1 x i8] }>* @alloc225, i32 0, i32 0, i32 0), [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8
@alloc96 = private unnamed_addr constant <{ [168 x i8] }> <{ [168 x i8] c"\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\04\00\00\00\03\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00" }>, align 8
@alloc470 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00-\00\00\003\00\00\00" }>, align 8
@alloc162 = private unnamed_addr constant <{ [22 x i8] }> <{ [22 x i8] c" is retained. refcnt=(" }>, align 1
@alloc223 = private unnamed_addr constant <{ [4 x i8] }> <{ [4 x i8] c" -> " }>, align 1
@alloc224 = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"), addr=" }>, align 1
@alloc161 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc254, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [22 x i8] }>, <{ [22 x i8] }>* @alloc162, i32 0, i32 0, i32 0), [8 x i8] c"\16\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [4 x i8] }>, <{ [4 x i8] }>* @alloc223, i32 0, i32 0, i32 0), [8 x i8] c"\04\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [8 x i8] }>, <{ [8 x i8] }>* @alloc224, i32 0, i32 0, i32 0), [8 x i8] c"\08\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [1 x i8] }>, <{ [1 x i8] }>* @alloc225, i32 0, i32 0, i32 0), [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8
@alloc247 = private unnamed_addr constant <{ [224 x i8] }> <{ [224 x i8] c"\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\04\00\00\00\03\00\00\00\00\00\00\00" }>, align 8
@alloc249 = private unnamed_addr constant <{ [8 x i8] }> zeroinitializer, align 8
@alloc256 = private unnamed_addr constant <{ [31 x i8] }> <{ [31 x i8] c" whose refcnt zero is retained!" }>, align 1
@alloc255 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc254, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [31 x i8] }>, <{ [31 x i8] }>* @alloc256, i32 0, i32 0, i32 0), [8 x i8] c"\1F\00\00\00\00\00\00\00" }>, align 8
@alloc472 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00C\00\00\00\05\00\00\00" }>, align 8
@alloc474 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00H\00\00\003\00\00\00" }>, align 8
@alloc199 = private unnamed_addr constant <{ [20 x i8] }> <{ [20 x i8] c"Retain of object id=" }>, align 1
@alloc261 = private unnamed_addr constant <{ [50 x i8] }> <{ [50 x i8] c" is reported but it isn't registered to sanitizer." }>, align 1
@alloc200 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [20 x i8] }>, <{ [20 x i8] }>* @alloc199, i32 0, i32 0, i32 0), [8 x i8] c"\14\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc261, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc476 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00I\00\00\00\05\00\00\00" }>, align 8
@alloc478 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00N\00\00\00.\00\00\00" }>, align 8
@alloc266 = private unnamed_addr constant <{ [24 x i8] }> <{ [24 x i8] c"The refcnt of object id=" }>, align 1
@alloc208 = private unnamed_addr constant <{ [37 x i8] }> <{ [37 x i8] c" in report_retain mismatch! reported=" }>, align 1
@alloc269 = private unnamed_addr constant <{ [12 x i8] }> <{ [12 x i8] c", sanitizer=" }>, align 1
@alloc207 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc266, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [37 x i8] }>, <{ [37 x i8] }>* @alloc208, i32 0, i32 0, i32 0), [8 x i8] c"%\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc269, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc480 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00O\00\00\00\05\00\00\00" }>, align 8
@alloc222 = private unnamed_addr constant <{ [22 x i8] }> <{ [22 x i8] c" is released. refcnt=(" }>, align 1
@alloc221 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc254, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [22 x i8] }>, <{ [22 x i8] }>* @alloc222, i32 0, i32 0, i32 0), [8 x i8] c"\16\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [4 x i8] }>, <{ [4 x i8] }>* @alloc223, i32 0, i32 0, i32 0), [8 x i8] c"\04\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [8 x i8] }>, <{ [8 x i8] }>* @alloc224, i32 0, i32 0, i32 0), [8 x i8] c"\08\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [1 x i8] }>, <{ [1 x i8] }>* @alloc225, i32 0, i32 0, i32 0), [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8
@alloc482 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00b\00\00\00\05\00\00\00" }>, align 8
@alloc484 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00g\00\00\002\00\00\00" }>, align 8
@alloc259 = private unnamed_addr constant <{ [21 x i8] }> <{ [21 x i8] c"Release of object id=" }>, align 1
@alloc260 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [21 x i8] }>, <{ [21 x i8] }>* @alloc259, i32 0, i32 0, i32 0), [8 x i8] c"\15\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc261, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc486 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00h\00\00\00\05\00\00\00" }>, align 8
@alloc488 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00m\00\00\00-\00\00\00" }>, align 8
@alloc268 = private unnamed_addr constant <{ [38 x i8] }> <{ [38 x i8] c" in report_release mismatch! reported=" }>, align 1
@alloc267 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc266, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [38 x i8] }>, <{ [38 x i8] }>* @alloc268, i32 0, i32 0, i32 0), [8 x i8] c"&\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc269, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc490 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc489, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00n\00\00\00\05\00\00\00" }>, align 8

; <T as core::any::Any>::type_id
; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind readnone uwtable willreturn
define internal i64 @"_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17ha7daf7c2b2ea8d27E"({ [0 x i8]*, i64 }* noalias nocapture noundef readonly align 8 dereferenceable(16) %self) unnamed_addr #0 {
start:
  ret i64 -5139102199292759541
}

; std::sys_common::backtrace::__rust_end_short_backtrace
; Function Attrs: noinline noreturn nonlazybind uwtable
define internal fastcc void @_ZN3std10sys_common9backtrace26__rust_end_short_backtrace17hea36b766ad666feaE(%"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* noalias nocapture noundef readonly dereferenceable(24) %f) unnamed_addr #1 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_2.sroa.0.0..sroa_idx = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %f, i64 0, i32 0, i32 0
  %_2.sroa.0.0.copyload = load [0 x i8]*, [0 x i8]** %_2.sroa.0.0..sroa_idx, align 8
  %_2.sroa.3.0..sroa_idx3 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %f, i64 0, i32 0, i32 1
  %_2.sroa.3.0.copyload = load i64, i64* %_2.sroa.3.0..sroa_idx3, align 8
  %_2.sroa.4.0..sroa_idx4 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %f, i64 0, i32 1
  %_2.sroa.4.0.copyload = load %"core::panic::location::Location"*, %"core::panic::location::Location"** %_2.sroa.4.0..sroa_idx4, align 8
; call std::panicking::begin_panic::{{closure}}
  tail call fastcc void @"_ZN3std9panicking11begin_panic28_$u7b$$u7b$closure$u7d$$u7d$17h56b3894ae78ba8e2E"([0 x i8]* %_2.sroa.0.0.copyload, i64 %_2.sroa.3.0.copyload, %"core::panic::location::Location"* %_2.sroa.4.0.copyload) #23
  unreachable
}

; std::collections::hash::map::HashMap<K,V,S>::contains_key
; Function Attrs: inlinehint nofree nosync nounwind nonlazybind uwtable
define internal fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h9ac6fd78d11cfe13E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias nocapture noundef readonly align 8 dereferenceable(48) %self, i64 %k.val1) unnamed_addr #2 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !5) #24
  %_4.idx.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1, i32 1, i32 4
  %_4.idx.val.i.i = load i64, i64* %_4.idx.i.i, align 8, !alias.scope !8
  %0 = icmp eq i64 %_4.idx.val.i.i, 0
  br i1 %0, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h3bb1e1c4c67e6a69E.exit", label %bb3.i.i

bb3.i.i:                                          ; preds = %start
  %_4.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1
  %_7.idx.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 0, i32 0
  %_7.idx.val.i.i = load i64, i64* %_7.idx.i.i, align 8, !alias.scope !8
  %_7.idx1.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 0, i32 1
  %_7.idx1.val.i.i = load i64, i64* %_7.idx1.i.i, align 8, !alias.scope !8
  %1 = xor i64 %_7.idx.val.i.i, 8317987319222330741
  %2 = xor i64 %_7.idx1.val.i.i, 7237128888997146477
  %3 = xor i64 %_7.idx.val.i.i, 7816392313619706465
  %4 = xor i64 %_7.idx1.val.i.i, %k.val1
  %5 = xor i64 %4, 8387220255154660723
  %6 = add i64 %2, %1
  %7 = tail call i64 @llvm.fshl.i64(i64 %2, i64 %2, i64 13) #24
  %8 = xor i64 %6, %7
  %9 = tail call i64 @llvm.fshl.i64(i64 %6, i64 %6, i64 32) #24
  %10 = add i64 %5, %3
  %11 = tail call i64 @llvm.fshl.i64(i64 %5, i64 %5, i64 16) #24
  %12 = xor i64 %11, %10
  %13 = add i64 %12, %9
  %14 = tail call i64 @llvm.fshl.i64(i64 %12, i64 %12, i64 21) #24
  %15 = xor i64 %14, %13
  %16 = add i64 %8, %10
  %17 = tail call i64 @llvm.fshl.i64(i64 %8, i64 %8, i64 17) #24
  %18 = xor i64 %16, %17
  %19 = tail call i64 @llvm.fshl.i64(i64 %16, i64 %16, i64 32) #24
  %20 = xor i64 %13, %k.val1
  %21 = xor i64 %15, 576460752303423488
  %22 = add i64 %20, %18
  %23 = tail call i64 @llvm.fshl.i64(i64 %18, i64 %18, i64 13) #24
  %24 = xor i64 %22, %23
  %25 = tail call i64 @llvm.fshl.i64(i64 %22, i64 %22, i64 32) #24
  %26 = add i64 %21, %19
  %27 = tail call i64 @llvm.fshl.i64(i64 %15, i64 %21, i64 16) #24
  %28 = xor i64 %27, %26
  %29 = add i64 %28, %25
  %30 = tail call i64 @llvm.fshl.i64(i64 %28, i64 %28, i64 21) #24
  %31 = xor i64 %30, %29
  %32 = add i64 %26, %24
  %33 = tail call i64 @llvm.fshl.i64(i64 %24, i64 %24, i64 17) #24
  %34 = xor i64 %32, %33
  %35 = tail call i64 @llvm.fshl.i64(i64 %32, i64 %32, i64 32) #24
  %36 = xor i64 %29, 576460752303423488
  %37 = xor i64 %35, 255
  %38 = add i64 %36, %34
  %39 = tail call i64 @llvm.fshl.i64(i64 %34, i64 %34, i64 13) #24
  %40 = xor i64 %38, %39
  %41 = tail call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 32) #24
  %42 = add i64 %31, %37
  %43 = tail call i64 @llvm.fshl.i64(i64 %31, i64 %31, i64 16) #24
  %44 = xor i64 %43, %42
  %45 = add i64 %44, %41
  %46 = tail call i64 @llvm.fshl.i64(i64 %44, i64 %44, i64 21) #24
  %47 = xor i64 %46, %45
  %48 = add i64 %40, %42
  %49 = tail call i64 @llvm.fshl.i64(i64 %40, i64 %40, i64 17) #24
  %50 = xor i64 %48, %49
  %51 = tail call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 32) #24
  %52 = add i64 %50, %45
  %53 = tail call i64 @llvm.fshl.i64(i64 %50, i64 %50, i64 13) #24
  %54 = xor i64 %53, %52
  %55 = tail call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 32) #24
  %56 = add i64 %47, %51
  %57 = tail call i64 @llvm.fshl.i64(i64 %47, i64 %47, i64 16) #24
  %58 = xor i64 %57, %56
  %59 = add i64 %58, %55
  %60 = tail call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 21) #24
  %61 = xor i64 %60, %59
  %62 = add i64 %54, %56
  %63 = tail call i64 @llvm.fshl.i64(i64 %54, i64 %54, i64 17) #24
  %64 = xor i64 %63, %62
  %65 = tail call i64 @llvm.fshl.i64(i64 %62, i64 %62, i64 32) #24
  %66 = add i64 %64, %59
  %67 = tail call i64 @llvm.fshl.i64(i64 %64, i64 %64, i64 13) #24
  %68 = xor i64 %67, %66
  %69 = add i64 %61, %65
  %70 = tail call i64 @llvm.fshl.i64(i64 %61, i64 %61, i64 16) #24
  %71 = xor i64 %70, %69
  %72 = tail call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 21) #24
  %73 = add i64 %68, %69
  %74 = tail call i64 @llvm.fshl.i64(i64 %68, i64 %68, i64 17) #24
  %75 = tail call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 32) #24
  %_17.i.i.i.i.i.i.i = xor i64 %73, %72
  %76 = xor i64 %_17.i.i.i.i.i.i.i, %74
  %77 = xor i64 %76, %75
  tail call void @llvm.experimental.noalias.scope.decl(metadata !9) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !12) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !15) #24
  %top7.i.i.i.i.i.i = lshr i64 %77, 57
  %78 = trunc i64 %top7.i.i.i.i.i.i to i8
  %79 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %_4.i.i to i64*
  %_6.i.i.i.i.i.i = load i64, i64* %79, align 8, !alias.scope !18, !noalias !21
  %80 = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1, i32 1, i32 2
  %self.idx.val.i.i.i.i.i = load i8*, i8** %80, align 8, !alias.scope !23, !noalias !21
  %.0.vec.insert.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %78, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb21.i.i.i.i.i, %bb3.i.i
  %probe_seq.sroa.7.0.i.i.i.i.i = phi i64 [ 0, %bb3.i.i ], [ %93, %bb21.i.i.i.i.i ]
  %.pn.i.i = phi i64 [ %77, %bb3.i.i ], [ %94, %bb21.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i = and i64 %.pn.i.i, %_6.i.i.i.i.i.i
  %81 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i
  %82 = bitcast i8* %81 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i = load <16 x i8>, <16 x i8>* %82, align 1, !noalias !24
  %83 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i
  %84 = bitcast <16 x i1> %83 to i16
  br label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb10.i.i.i.i.i, %bb3.i.i.i.i.i
  %iter.0.i.i.i.i.i = phi i16 [ %84, %bb3.i.i.i.i.i ], [ %_2.i.i.i.i.i.i.i, %bb10.i.i.i.i.i ]
  %85 = icmp eq i16 %iter.0.i.i.i.i.i, 0
  br i1 %85, label %bb12.i.i.i.i.i, label %bb10.i.i.i.i.i

bb12.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %86 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %87 = bitcast <16 x i1> %86 to i16
  %.not.i.i.i.i.i = icmp eq i16 %87, 0
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h3bb1e1c4c67e6a69E.exit"

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %88 = tail call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %88 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %89 = sub i64 0, %index.i.i.i.i.i
  %90 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %89, i32 0
  %91 = getelementptr inbounds i64, i64* %90, i64 -7
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %91, align 8, !noalias !28
  %92 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %k.val1
  br i1 %92, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h3bb1e1c4c67e6a69E.exit", label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %93 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %94 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %93
  br label %bb3.i.i.i.i.i

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h3bb1e1c4c67e6a69E.exit": ; preds = %bb12.i.i.i.i.i, %bb10.i.i.i.i.i, %start
  %.0.i.i = phi i1 [ false, %start ], [ true, %bb10.i.i.i.i.i ], [ false, %bb12.i.i.i.i.i ]
  ret i1 %.0.i.i
}

; std::collections::hash::map::HashMap<K,V,S>::get_mut
; Function Attrs: inlinehint nofree nosync nounwind nonlazybind uwtable
define internal fastcc noundef align 8 dereferenceable_or_null(48) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h1fd66babc11d9351E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias nocapture noundef readonly align 8 dereferenceable(48) %self, i64 %k.val1) unnamed_addr #2 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !31)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !34) #24
  %_4.idx.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1, i32 1, i32 4
  %_4.idx.val.i.i = load i64, i64* %_4.idx.i.i, align 8, !alias.scope !37
  %0 = icmp eq i64 %_4.idx.val.i.i, 0
  br i1 %0, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit", label %bb3.i.i

bb3.i.i:                                          ; preds = %start
  %_4.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1
  %_7.idx.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 0, i32 0
  %_7.idx.val.i.i = load i64, i64* %_7.idx.i.i, align 8, !alias.scope !37
  %_7.idx1.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 0, i32 1
  %_7.idx1.val.i.i = load i64, i64* %_7.idx1.i.i, align 8, !alias.scope !37
  %1 = xor i64 %_7.idx.val.i.i, 8317987319222330741
  %2 = xor i64 %_7.idx1.val.i.i, 7237128888997146477
  %3 = xor i64 %_7.idx.val.i.i, 7816392313619706465
  %4 = xor i64 %_7.idx1.val.i.i, %k.val1
  %5 = xor i64 %4, 8387220255154660723
  %6 = add i64 %2, %1
  %7 = tail call i64 @llvm.fshl.i64(i64 %2, i64 %2, i64 13) #24
  %8 = xor i64 %6, %7
  %9 = tail call i64 @llvm.fshl.i64(i64 %6, i64 %6, i64 32) #24
  %10 = add i64 %5, %3
  %11 = tail call i64 @llvm.fshl.i64(i64 %5, i64 %5, i64 16) #24
  %12 = xor i64 %11, %10
  %13 = add i64 %12, %9
  %14 = tail call i64 @llvm.fshl.i64(i64 %12, i64 %12, i64 21) #24
  %15 = xor i64 %14, %13
  %16 = add i64 %8, %10
  %17 = tail call i64 @llvm.fshl.i64(i64 %8, i64 %8, i64 17) #24
  %18 = xor i64 %16, %17
  %19 = tail call i64 @llvm.fshl.i64(i64 %16, i64 %16, i64 32) #24
  %20 = xor i64 %13, %k.val1
  %21 = xor i64 %15, 576460752303423488
  %22 = add i64 %20, %18
  %23 = tail call i64 @llvm.fshl.i64(i64 %18, i64 %18, i64 13) #24
  %24 = xor i64 %22, %23
  %25 = tail call i64 @llvm.fshl.i64(i64 %22, i64 %22, i64 32) #24
  %26 = add i64 %21, %19
  %27 = tail call i64 @llvm.fshl.i64(i64 %15, i64 %21, i64 16) #24
  %28 = xor i64 %27, %26
  %29 = add i64 %28, %25
  %30 = tail call i64 @llvm.fshl.i64(i64 %28, i64 %28, i64 21) #24
  %31 = xor i64 %30, %29
  %32 = add i64 %26, %24
  %33 = tail call i64 @llvm.fshl.i64(i64 %24, i64 %24, i64 17) #24
  %34 = xor i64 %32, %33
  %35 = tail call i64 @llvm.fshl.i64(i64 %32, i64 %32, i64 32) #24
  %36 = xor i64 %29, 576460752303423488
  %37 = xor i64 %35, 255
  %38 = add i64 %36, %34
  %39 = tail call i64 @llvm.fshl.i64(i64 %34, i64 %34, i64 13) #24
  %40 = xor i64 %38, %39
  %41 = tail call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 32) #24
  %42 = add i64 %31, %37
  %43 = tail call i64 @llvm.fshl.i64(i64 %31, i64 %31, i64 16) #24
  %44 = xor i64 %43, %42
  %45 = add i64 %44, %41
  %46 = tail call i64 @llvm.fshl.i64(i64 %44, i64 %44, i64 21) #24
  %47 = xor i64 %46, %45
  %48 = add i64 %40, %42
  %49 = tail call i64 @llvm.fshl.i64(i64 %40, i64 %40, i64 17) #24
  %50 = xor i64 %48, %49
  %51 = tail call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 32) #24
  %52 = add i64 %50, %45
  %53 = tail call i64 @llvm.fshl.i64(i64 %50, i64 %50, i64 13) #24
  %54 = xor i64 %53, %52
  %55 = tail call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 32) #24
  %56 = add i64 %47, %51
  %57 = tail call i64 @llvm.fshl.i64(i64 %47, i64 %47, i64 16) #24
  %58 = xor i64 %57, %56
  %59 = add i64 %58, %55
  %60 = tail call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 21) #24
  %61 = xor i64 %60, %59
  %62 = add i64 %54, %56
  %63 = tail call i64 @llvm.fshl.i64(i64 %54, i64 %54, i64 17) #24
  %64 = xor i64 %63, %62
  %65 = tail call i64 @llvm.fshl.i64(i64 %62, i64 %62, i64 32) #24
  %66 = add i64 %64, %59
  %67 = tail call i64 @llvm.fshl.i64(i64 %64, i64 %64, i64 13) #24
  %68 = xor i64 %67, %66
  %69 = add i64 %61, %65
  %70 = tail call i64 @llvm.fshl.i64(i64 %61, i64 %61, i64 16) #24
  %71 = xor i64 %70, %69
  %72 = tail call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 21) #24
  %73 = add i64 %68, %69
  %74 = tail call i64 @llvm.fshl.i64(i64 %68, i64 %68, i64 17) #24
  %75 = tail call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 32) #24
  %_17.i.i.i.i.i.i.i = xor i64 %73, %72
  %76 = xor i64 %_17.i.i.i.i.i.i.i, %74
  %77 = xor i64 %76, %75
  tail call void @llvm.experimental.noalias.scope.decl(metadata !38) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !41) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !44) #24
  %top7.i.i.i.i.i.i = lshr i64 %77, 57
  %78 = trunc i64 %top7.i.i.i.i.i.i to i8
  %79 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %_4.i.i to i64*
  %_6.i.i.i.i.i.i = load i64, i64* %79, align 8, !alias.scope !47, !noalias !50
  %80 = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1, i32 1, i32 2
  %self.idx.val.i.i.i.i.i = load i8*, i8** %80, align 8, !alias.scope !52, !noalias !50
  %.0.vec.insert.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %78, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb21.i.i.i.i.i, %bb3.i.i
  %probe_seq.sroa.7.0.i.i.i.i.i = phi i64 [ 0, %bb3.i.i ], [ %93, %bb21.i.i.i.i.i ]
  %.pn.i.i = phi i64 [ %77, %bb3.i.i ], [ %94, %bb21.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i = and i64 %.pn.i.i, %_6.i.i.i.i.i.i
  %81 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i
  %82 = bitcast i8* %81 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i = load <16 x i8>, <16 x i8>* %82, align 1, !noalias !53
  %83 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i
  %84 = bitcast <16 x i1> %83 to i16
  br label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb10.i.i.i.i.i, %bb3.i.i.i.i.i
  %iter.0.i.i.i.i.i = phi i16 [ %84, %bb3.i.i.i.i.i ], [ %_2.i.i.i.i.i.i.i, %bb10.i.i.i.i.i ]
  %85 = icmp eq i16 %iter.0.i.i.i.i.i, 0
  br i1 %85, label %bb12.i.i.i.i.i, label %bb10.i.i.i.i.i

bb12.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %86 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %87 = bitcast <16 x i1> %86 to i16
  %.not.i.i.i.i.i = icmp eq i16 %87, 0
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit"

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %88 = tail call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %88 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %89 = sub i64 0, %index.i.i.i.i.i
  %90 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %89, i32 0
  %91 = getelementptr inbounds i64, i64* %90, i64 -7
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %91, align 8, !noalias !56
  %92 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %k.val1
  br i1 %92, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit.loopexit", label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %93 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %94 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %93
  br label %bb3.i.i.i.i.i

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit.loopexit": ; preds = %bb10.i.i.i.i.i
  %95 = getelementptr inbounds i64, i64* %90, i64 -7
  br label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit"

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit": ; preds = %bb12.i.i.i.i.i, %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit.loopexit", %start
  %.0.i.i = phi i64* [ null, %start ], [ %95, %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE.exit.loopexit" ], [ null, %bb12.i.i.i.i.i ]
  %96 = icmp eq i64* %.0.i.i, null
  %v.i = getelementptr inbounds i64, i64* %.0.i.i, i64 1
  %.0.i = select i1 %96, i64* null, i64* %v.i
  ret i64* %.0.i
}

; std::thread::local::fast::Key<T>::try_initialize
; Function Attrs: noinline nonlazybind uwtable
define internal fastcc noundef align 8 dereferenceable_or_null(16) i64* @"_ZN3std6thread5local4fast12Key$LT$T$GT$14try_initialize17hd4e535fd74b46a6dE"(i64* noalias nocapture noundef align 8 dereferenceable_or_null(24) %init) unnamed_addr #3 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !59)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !62)
  %.not.i.i = icmp eq i64* %init, null
  br i1 %.not.i.i, label %bb5.i.i, label %bb1.i.i

bb1.i.i:                                          ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !65)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !68) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !71) #24
  %_4.sroa.0.0.copyload.i.i = load i64, i64* %init, align 8, !alias.scope !73, !noalias !71
  %0 = getelementptr inbounds i64, i64* %init, i64 1
  %_4.sroa.5.0.copyload.i.i = load i64, i64* %0, align 8, !alias.scope !73, !noalias !71
  %1 = getelementptr inbounds i64, i64* %init, i64 2
  %_4.sroa.6.0.copyload.i.i = load i64, i64* %1, align 8, !alias.scope !73, !noalias !71
  store i64 0, i64* %init, align 8, !alias.scope !76, !noalias !77
  %2 = icmp eq i64 %_4.sroa.0.0.copyload.i.i, 1
  br i1 %2, label %"_ZN3std6thread5local4lazy21LazyKeyInner$LT$T$GT$10initialize17hb28fda0effd918eaE.exit", label %bb5.i.i

bb5.i.i:                                          ; preds = %bb1.i.i, %start
; call std::sys::unix::rand::hashmap_random_keys
  %3 = tail call { i64, i64 } @_ZN3std3sys4unix4rand19hashmap_random_keys17ha4436479ecf804b2E(), !noalias !78
  %.fca.0.extract.i.i = extractvalue { i64, i64 } %3, 0
  %.fca.1.extract.i.i = extractvalue { i64, i64 } %3, 1
  br label %"_ZN3std6thread5local4lazy21LazyKeyInner$LT$T$GT$10initialize17hb28fda0effd918eaE.exit"

"_ZN3std6thread5local4lazy21LazyKeyInner$LT$T$GT$10initialize17hb28fda0effd918eaE.exit": ; preds = %bb1.i.i, %bb5.i.i
  %.sroa.0.0.i.i = phi i64 [ %.fca.0.extract.i.i, %bb5.i.i ], [ %_4.sroa.5.0.copyload.i.i, %bb1.i.i ]
  %.sroa.3.0.i.i = phi i64 [ %.fca.1.extract.i.i, %bb5.i.i ], [ %_4.sroa.6.0.copyload.i.i, %bb1.i.i ]
  store i64 1, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 0), align 8, !alias.scope !79, !noalias !83
  store i64 %.sroa.0.0.i.i, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 0), align 8, !alias.scope !79, !noalias !83
  store i64 %.sroa.3.0.i.i, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 1), align 8, !alias.scope !79, !noalias !83
  ret i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 0)
}

; std::panicking::begin_panic
; Function Attrs: cold noinline noreturn nonlazybind uwtable
define internal fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() unnamed_addr #4 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_3 = alloca %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", align 8
  %0 = bitcast %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3 to i8*
  call void @llvm.lifetime.start.p0i8(i64 24, i8* nonnull %0)
  %1 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 0, i32 0
  store [0 x i8]* bitcast (<{ [42 x i8] }>* @alloc462 to [0 x i8]*), [0 x i8]** %1, align 8
  %2 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 0, i32 1
  store i64 42, i64* %2, align 8
  %3 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 1
  store %"core::panic::location::Location"* bitcast (<{ i8*, [16 x i8] }>* @alloc461 to %"core::panic::location::Location"*), %"core::panic::location::Location"** %3, align 8
; call std::sys_common::backtrace::__rust_end_short_backtrace
  call fastcc void @_ZN3std10sys_common9backtrace26__rust_end_short_backtrace17hea36b766ad666feaE(%"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* noalias nocapture noundef nonnull dereferenceable(24) %_3) #23
  unreachable
}

; std::panicking::begin_panic::{{closure}}
; Function Attrs: inlinehint noreturn nonlazybind uwtable
define internal fastcc void @"_ZN3std9panicking11begin_panic28_$u7b$$u7b$closure$u7d$$u7d$17h56b3894ae78ba8e2E"([0 x i8]* %_1.0.0.0.val, i64 %_1.0.0.1.val, %"core::panic::location::Location"* %_1.0.1.val) unnamed_addr #5 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5 = alloca { i8*, i64 }, align 8
  %0 = bitcast { i8*, i64 }* %_5 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %0)
  %1 = getelementptr [0 x i8], [0 x i8]* %_1.0.0.0.val, i64 0, i64 0
  %.fca.0.gep = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %_5, i64 0, i32 0
  store i8* %1, i8** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %_5, i64 0, i32 1
  store i64 %_1.0.0.1.val, i64* %.fca.1.gep, align 8
  %_2.0 = bitcast { i8*, i64 }* %_5 to {}*
; call std::panicking::rust_panic_with_hook
  call void @_ZN3std9panicking20rust_panic_with_hook17hc82286af2030e925E({}* noundef nonnull align 1 %_2.0, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.0 to [3 x i64]*), i64* noalias noundef readonly align 8 dereferenceable_or_null(48) null, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) %_1.0.1.val, i1 noundef zeroext true) #23
  unreachable
}

; <&T as core::fmt::Debug>::fmt
; Function Attrs: nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hc715f6c95a655b17E"(i64** noalias nocapture noundef readonly align 8 dereferenceable(8) %self, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64) %f) unnamed_addr #6 {
start:
  %_6 = load i64*, i64** %self, align 8, !nonnull !85, !align !86, !noundef !85
; call core::fmt::Formatter::debug_lower_hex
  %_3.i = tail call noundef zeroext i1 @_ZN4core3fmt9Formatter15debug_lower_hex17h50fe8a435241971eE(%"core::fmt::Formatter"* noalias noundef nonnull readonly align 8 dereferenceable(64) %f), !noalias !87
  br i1 %_3.i, label %bb2.i, label %bb4.i

bb4.i:                                            ; preds = %start
; call core::fmt::Formatter::debug_upper_hex
  %_7.i = tail call noundef zeroext i1 @_ZN4core3fmt9Formatter15debug_upper_hex17h3960174cd3e4a3c3E(%"core::fmt::Formatter"* noalias noundef nonnull readonly align 8 dereferenceable(64) %f), !noalias !87
  br i1 %_7.i, label %bb6.i, label %bb8.i

bb2.i:                                            ; preds = %start
; call core::fmt::num::<impl core::fmt::LowerHex for i64>::fmt
  %0 = tail call noundef zeroext i1 @"_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i64$GT$3fmt17ha92a0b3e2a1e9677E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %_6, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f)
  br label %"_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i64$GT$3fmt17h5debc439757ab39aE.exit"

bb8.i:                                            ; preds = %bb4.i
; call core::fmt::num::imp::<impl core::fmt::Display for i64>::fmt
  %1 = tail call noundef zeroext i1 @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %_6, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f)
  br label %"_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i64$GT$3fmt17h5debc439757ab39aE.exit"

bb6.i:                                            ; preds = %bb4.i
; call core::fmt::num::<impl core::fmt::UpperHex for i64>::fmt
  %2 = tail call noundef zeroext i1 @"_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i64$GT$3fmt17hb6321a42a400d3f1E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %_6, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f)
  br label %"_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i64$GT$3fmt17h5debc439757ab39aE.exit"

"_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i64$GT$3fmt17h5debc439757ab39aE.exit": ; preds = %bb2.i, %bb8.i, %bb6.i
  %.0.in.i = phi i1 [ %0, %bb2.i ], [ %2, %bb6.i ], [ %1, %bb8.i ]
  ret i1 %.0.in.i
}

; <&T as core::fmt::Display>::fmt
; Function Attrs: nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h959a2441bf3a547eE"({ [0 x i8]*, i64 }* noalias nocapture noundef readonly align 8 dereferenceable(16) %self, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64) %f) unnamed_addr #6 {
start:
  %0 = getelementptr inbounds { [0 x i8]*, i64 }, { [0 x i8]*, i64 }* %self, i64 0, i32 0
  %_6.0 = load [0 x i8]*, [0 x i8]** %0, align 8, !nonnull !85, !align !90, !noundef !85
  %1 = getelementptr inbounds { [0 x i8]*, i64 }, { [0 x i8]*, i64 }* %self, i64 0, i32 1
  %_6.1 = load i64, i64* %1, align 8
; call <str as core::fmt::Display>::fmt
  %2 = tail call noundef zeroext i1 @"_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17hfa8f7ea124ceedccE"([0 x i8]* noalias noundef nonnull readonly align 1 %_6.0, i64 %_6.1, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f)
  ret i1 %2
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h42a39cd9ab169dceE"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* nocapture readonly %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0.i.i = alloca %"std::sync::mutex::Mutex<i64>", align 8
  tail call void @llvm.experimental.noalias.scope.decl(metadata !91)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !94)
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15.i.i = load i64**, i64*** %0, align 8, !alias.scope !97, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15.i.i to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !98, !noalias !97
  store i64* null, i64** %_15.i.i, align 8, !alias.scope !98, !noalias !97
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20.i.i = bitcast %"std::sync::mutex::Mutex<i64>"* %_5.sroa.0.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !97
  tail call void @llvm.experimental.noalias.scope.decl(metadata !105)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !108)
  %_8.i.i.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"**
  %_9.i.i.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %_8.i.i.i.i, align 8, !alias.scope !111, !noalias !112, !nonnull !85, !align !86, !noundef !85
  %_3.i.i.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* %_9.i.i.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !115, !noalias !118
  store i64* null, i64** %_3.i.i.i.i, align 8, !alias.scope !115, !noalias !118
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i"

bb2.i.i.i.i:                                      ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !118
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<i64>"*)*
  call void %7(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %_5.sroa.0.i.i), !noalias !119
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"**, %"core::option::Option<std::sync::mutex::Mutex<i64>>"*** %8, align 8, !alias.scope !97, !nonnull !85, !align !86, !noundef !85
  %_17.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16.i.i, align 8, !noalias !97
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17.i.i, i64 0, i32 0
  %_2.i16.i.i = load i64, i64* %9, align 8, !range !120, !noalias !97, !noundef !85
  %10 = icmp eq i64 %_2.i16.i.i, 0
  br i1 %10, label %_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17.i.i, i64 0, i32 1
  %12 = bitcast [2 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %bb2.i.bb9_crit_edge.i.i unwind label %cleanup.i.i, !noalias !97

bb2.i.bb9_crit_edge.i.i:                          ; preds = %bb2.i.i.i
  %_22.pre.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16.i.i, align 8, !noalias !97
  br label %_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E.exit

cleanup.i.i:                                      ; preds = %bb2.i.i.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %_20.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16.i.i, align 8, !noalias !97
  %_10.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_20.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx.i.i, align 8, !noalias !97
  %_10.sroa.5.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_20.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast.i.i = bitcast [2 x i64]* %_10.sroa.5.0..sroa_idx.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(16) %_10.sroa.5.0..sroa_cast.i.i, i8* noundef nonnull align 8 dereferenceable(16) %_5.sroa.0.0.sroa_cast20.i.i, i64 16, i1 false), !noalias !97
  resume { i8*, i32 } %13

_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E.exit: ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i", %bb2.i.bb9_crit_edge.i.i
  %_22.i.i = phi %"core::option::Option<std::sync::mutex::Mutex<i64>>"* [ %_22.pre.i.i, %bb2.i.bb9_crit_edge.i.i ], [ %_17.i.i, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i" ]
  %_10.sroa.0.0..sroa_idx2.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_22.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx2.i.i, align 8, !noalias !97
  %_10.sroa.5.0..sroa_idx6.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_22.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast7.i.i = bitcast [2 x i64]* %_10.sroa.5.0..sroa_idx6.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(16) %_10.sroa.5.0..sroa_cast7.i.i, i8* noundef nonnull align 8 dereferenceable(16) %_5.sroa.0.0.sroa_cast20.i.i, i64 16, i1 false), !noalias !97
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !97
  ret i1 true
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h8d298f77ff4ec3b3E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* nocapture readonly %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0.i.i = alloca %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", align 8
  tail call void @llvm.experimental.noalias.scope.decl(metadata !121)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !124)
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15.i.i = load i64**, i64*** %0, align 8, !alias.scope !127, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15.i.i to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !128, !noalias !127
  store i64* null, i64** %_15.i.i, align 8, !alias.scope !128, !noalias !127
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20.i.i = bitcast %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_5.sroa.0.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !127
  tail call void @llvm.experimental.noalias.scope.decl(metadata !135)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !138)
  %_8.i.i.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**
  %_9.i.i.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_8.i.i.i.i, align 8, !alias.scope !141, !noalias !142, !nonnull !85, !align !86, !noundef !85
  %_3.i.i.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_9.i.i.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !145, !noalias !148
  store i64* null, i64** %_3.i.i.i.i, align 8, !alias.scope !145, !noalias !148
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i"

bb2.i.i.i.i:                                      ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !148
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0.i.i), !noalias !149
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !alias.scope !127, !nonnull !85, !align !86, !noundef !85
  %_17.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !127
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 0
  %_2.i16.i.i = load i64, i64* %9, align 8, !range !120, !noalias !127, !noundef !85
  %10 = icmp eq i64 %_2.i16.i.i, 0
  br i1 %10, label %_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1
  %12 = bitcast [7 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i" unwind label %cleanup.i.i.i.i, !noalias !127

cleanup.i.i.i.i:                                  ; preds = %bb2.i.i.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %14 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 1
  %15 = bitcast i64* %14 to %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*
; call core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h1eb938b370d22c57E"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #25, !noalias !127
  %_20.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !127
  %_10.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx.i.i, align 8, !noalias !127
  %_10.sroa.5.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast.i.i = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast.i.i, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20.i.i, i64 56, i1 false), !noalias !127
  resume { i8*, i32 } %13

"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i": ; preds = %bb2.i.i.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 1
  %17 = bitcast i64* %16 to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*
; call core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
  tail call fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %17) #24, !noalias !127
  %_22.pre.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !127
  br label %_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E.exit

_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E.exit: ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i", %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i"
  %_22.i.i = phi %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* [ %_22.pre.i.i, %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i" ], [ %_17.i.i, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i" ]
  %_10.sroa.0.0..sroa_idx2.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_22.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx2.i.i, align 8, !noalias !127
  %_10.sroa.5.0..sroa_idx6.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_22.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast7.i.i = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx6.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast7.i.i, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20.i.i, i64 56, i1 false), !noalias !127
  call void @llvm.lifetime.end.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !127
  ret i1 true
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17h792230541602dafdE(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i.i.i.i.i.i = alloca %"std::thread::local::AccessError", align 1
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  %_2.i = alloca %"std::collections::hash::map::HashMap<i64, ObjectInfo>", align 16
  tail call void @llvm.experimental.noalias.scope.decl(metadata !150)
  %1 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1), !noalias !150
  tail call void @llvm.experimental.noalias.scope.decl(metadata !153)
  %_2.i.i.i.i.i.i.i.i.i.i = load i64, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 0), align 8, !range !120, !noalias !156, !noundef !85
  %trunc.not.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_2.i.i.i.i.i.i.i.i.i.i, 0
  br i1 %trunc.not.i.i.i.i.i.i.i.i.i.i, label %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"

_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i: ; preds = %start
; call std::thread::local::fast::Key<T>::try_initialize
  %2 = tail call fastcc noundef align 8 dereferenceable_or_null(16) i64* @"_ZN3std6thread5local4fast12Key$LT$T$GT$14try_initialize17hd4e535fd74b46a6dE"(i64* noalias noundef align 8 dereferenceable_or_null(24) null), !noalias !163
  %3 = icmp eq i64* %2, null
  br i1 %3, label %bb1.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"

bb1.i.i.i.i.i.i:                                  ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i
  %4 = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 0, i8* nonnull %4), !noalias !164
  %_6.0.i.i.i.i.i.i = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [70 x i8] }>* @alloc365 to [0 x i8]*), i64 70, {}* noundef nonnull align 1 %_6.0.i.i.i.i.i.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.3 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc367 to %"core::panic::location::Location"*)) #23, !noalias !164
  unreachable

"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i": ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, %start
  %.0.i.i2.i.i.i.i.i.i = phi i64* [ %2, %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i ], [ getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 0), %start ]
  %5 = bitcast i64* %.0.i.i2.i.i.i.i.i.i to <2 x i64>*
  %6 = load <2 x i64>, <2 x i64>* %5, align 8, !noalias !163
  %7 = extractelement <2 x i64> %6, i64 0
  %8 = add i64 %7, 1
  store i64 %8, i64* %.0.i.i2.i.i.i.i.i.i, align 8, !alias.scope !165, !noalias !163
  %_2.sroa.7.0..sroa_idx.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 3
  %_2.sroa.7.0..sroa_idx1516.i.i.i = bitcast i64* %_2.sroa.7.0..sroa_idx.i.i.i to i8*
  call void @llvm.memset.p0i8.i64(i8* noundef nonnull align 16 dereferenceable(16) %_2.sroa.7.0..sroa_idx1516.i.i.i, i8 0, i64 16, i1 false) #24, !alias.scope !168, !noalias !150
  %9 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to <2 x i64>*
  store <2 x i64> %6, <2 x i64>* %9, align 16, !alias.scope !168, !noalias !150
  %_2.sroa.5.0..sroa_idx4.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1
  %_2.sroa.5.0..sroa_cast.i.i.i = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %_2.sroa.5.0..sroa_idx4.i.i.i to i64*
  store i64 0, i64* %_2.sroa.5.0..sroa_cast.i.i.i, align 16, !alias.scope !168, !noalias !150
  %_2.sroa.6.0..sroa_idx6.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 2
  store i8* getelementptr inbounds (<{ [16 x i8] }>, <{ [16 x i8] }>* @alloc67, i64 0, i32 0, i64 0), i8** %_2.sroa.6.0..sroa_idx6.i.i.i, align 8, !alias.scope !168, !noalias !150
  tail call void @llvm.experimental.noalias.scope.decl(metadata !171)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !174)
  %10 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %10), !noalias !176
; invoke std::sys_common::mutex::MovableMutex::new
  %11 = invoke i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E()
          to label %bb1.i.i unwind label %cleanup.i.i, !noalias !176

cleanup.i.i:                                      ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"
  %12 = landingpad { i8*, i32 }
          cleanup
  br label %bb6.i.i

bb1.i.i:                                          ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %.0..sroa_idx.i.i, align 4, !noalias !176
; invoke std::sync::poison::Flag::new
  %13 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E.exit" unwind label %cleanup1.i.i, !noalias !176

cleanup1.i.i:                                     ; preds = %bb1.i.i
  %14 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #25
          to label %bb6.i.i unwind label %abort.i.i, !noalias !176

abort.i.i:                                        ; preds = %cleanup1.i.i
  %15 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !176
  unreachable

bb6.i.i:                                          ; preds = %cleanup1.i.i, %cleanup.i.i
  %.pn.i.i = phi { i8*, i32 } [ %14, %cleanup1.i.i ], [ %12, %cleanup.i.i ]
; call core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
  call fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %_2.i) #25, !noalias !177
  resume { i8*, i32 } %.pn.i.i

"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E.exit": ; preds = %bb1.i.i
  %_4.sroa.0.0..sroa_idx26.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 3, i32 0, i32 0
  %_4.sroa.0.0..sroa_idx2627.i.i = bitcast %"hashbrown::map::HashMap<i64, ObjectInfo, std::collections::hash::map::RandomState>"* %_4.sroa.0.0..sroa_idx26.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(48) %_4.sroa.0.0..sroa_idx2627.i.i, i8* noundef nonnull align 16 dereferenceable(48) %1, i64 48, i1 false), !alias.scope !178
  %16 = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %16, align 8, !alias.scope !177, !noalias !174
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %13, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !177, !noalias !174
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %10), !noalias !176
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1), !noalias !150
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  tail call void @llvm.experimental.noalias.scope.decl(metadata !179)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !182)
  %1 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %1), !noalias !185
; call std::sys_common::mutex::MovableMutex::new
  %2 = tail call i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E(), !noalias !185
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %.0..sroa_idx.i.i, align 4, !noalias !185
; invoke std::sync::poison::Flag::new
  %3 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit" unwind label %cleanup1.i.i, !noalias !185

cleanup1.i.i:                                     ; preds = %start
  %4 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #25
          to label %bb5.i.i unwind label %abort.i.i, !noalias !185

abort.i.i:                                        ; preds = %cleanup1.i.i
  %5 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !185
  unreachable

bb5.i.i:                                          ; preds = %cleanup1.i.i
  resume { i8*, i32 } %4

"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit": ; preds = %start
  %6 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %6, align 8, !alias.scope !185
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %3, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !185
  %7 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 3
  store i64 0, i64* %7, align 8, !alias.scope !185
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %1), !noalias !185
  ret void
}

; core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
; Function Attrs: nounwind nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h1eb938b370d22c57E"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nocapture readonly %_1) unnamed_addr #8 {
start:
  %0 = getelementptr %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_1, i64 0, i32 0
; call core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
  tail call fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %0)
  ret void
}

; core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
; Function Attrs: nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !186)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !186, !nonnull !85, !align !86, !noundef !85
  %_5.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i = load i8, i8* %_5.i, align 8, !alias.scope !186
  %_5.not.i.i = icmp eq i8 %_5.val.i, 0
  br i1 %_5.not.i.i, label %bb2.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

bb2.i.i:                                          ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !186
  %_1.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i: ; preds = %bb2.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !186
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %bb5.i.i

bb5.i.i:                                          ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i
  %_6.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i monotonic, align 4, !noalias !186
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i: ; preds = %bb5.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i, %bb2.i.i, %start
  %_5.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i, i32 0 release, align 4, !noalias !186
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i, label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE.exit"

bb2.i.i.i:                                        ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i
  %_2.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i), !noalias !186
  br label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE.exit"

"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, %bb2.i.i.i
  ret void
}

; core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
; Function Attrs: nonlazybind uwtable
define internal void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !189)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !189, !nonnull !85, !align !86, !noundef !85
  %_5.i.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i.i = load i8, i8* %_5.i.i, align 8, !alias.scope !189
  %_5.not.i.i.i = icmp eq i8 %_5.val.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !189
  %_1.i.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !189
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  %_6.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i monotonic, align 4, !noalias !189
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %start
  %_5.i.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i.i, i32 0 release, align 4, !noalias !189
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
  %_2.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i), !noalias !189
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  ret void
}

; core::ptr::drop_in_place<&i64>
; Function Attrs: inlinehint mustprogress nofree norecurse nosync nounwind nonlazybind readnone uwtable willreturn
define internal void @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E"(i64** nocapture readnone %_1) unnamed_addr #9 {
start:
  ret void
}

; core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
; Function Attrs: nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_1) unnamed_addr #6 {
start:
; call <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  tail call void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %_1)
  ret void
}

; core::ptr::drop_in_place<(i64,fixsanitizer::ObjectInfo)>
; Function Attrs: nounwind nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr59drop_in_place$LT$$LP$i64$C$fixsanitizer..ObjectInfo$RP$$GT$17h855e18607bcfb813E"({ i64, %ObjectInfo }* nocapture readonly %_1) unnamed_addr #8 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %.idx.i.i.i = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_1, i64 0, i32 1, i32 3, i32 0, i32 0, i32 0
  %.idx.val.i.i.i = load i8*, i8** %.idx.i.i.i, align 8
  %.idx5.i.i.i = getelementptr { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_1, i64 0, i32 1, i32 3, i32 0, i32 0, i32 1
  %.idx5.val.i.i.i = load i64, i64* %.idx5.i.i.i, align 8
  %_4.i.i.i.i.i.i = icmp eq i64 %.idx5.val.i.i.i, 0
  br i1 %_4.i.i.i.i.i.i, label %"_ZN4core3ptr45drop_in_place$LT$fixsanitizer..ObjectInfo$GT$17h2f8681967b12afc3E.exit", label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i": ; preds = %start
  %0 = icmp ne i8* %.idx.val.i.i.i, null
  tail call void @llvm.assume(i1 %0) #24
  tail call void @__rust_dealloc(i8* nonnull %.idx.val.i.i.i, i64 %.idx5.val.i.i.i, i64 1) #24
  br label %"_ZN4core3ptr45drop_in_place$LT$fixsanitizer..ObjectInfo$GT$17h2f8681967b12afc3E.exit"

"_ZN4core3ptr45drop_in_place$LT$fixsanitizer..ObjectInfo$GT$17h2f8681967b12afc3E.exit": ; preds = %start, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i"
  ret void
}

; core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
; Function Attrs: nounwind nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nocapture readonly %_1) unnamed_addr #8 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %0 = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_1, i64 0, i32 0, i32 1
  tail call void @llvm.experimental.noalias.scope.decl(metadata !192) #24
  %1 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %0 to i64*
  %_2.i.i.i.i = load i64, i64* %1, align 8, !alias.scope !195
  %2 = icmp eq i64 %_2.i.i.i.i, 0
  br i1 %2, label %"_ZN4core3ptr125drop_in_place$LT$hashbrown..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$C$std..collections..hash..map..RandomState$GT$$GT$17h78718f5658623464E.exit", label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !198) #24
  %self.idx.i.i.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_1, i64 0, i32 0, i32 1, i32 1, i32 4
  %self.idx.val.i.i.i.i = load i64, i64* %self.idx.i.i.i.i, align 8, !alias.scope !201
  %3 = icmp eq i64 %self.idx.val.i.i.i.i, 0
  br i1 %3, label %"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i", label %bb6.i.i.i.i

"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i": ; preds = %bb2.i.i.i
  %.pre.i.i.i = add i64 %_2.i.i.i.i, 1
  br label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i"

bb6.i.i.i.i:                                      ; preds = %bb2.i.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !202) #24
  %self.idx.i.i.i.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_1, i64 0, i32 0, i32 1, i32 1, i32 2
  %self.idx.val.i.i.i.i.i = load i8*, i8** %self.idx.i.i.i.i.i, align 8, !alias.scope !205, !noalias !206
  %4 = add i64 %_2.i.i.i.i, 1
  %5 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %4
  %6 = bitcast i8* %self.idx.val.i.i.i.i.i to <16 x i8>*
  %7 = load <16 x i8>, <16 x i8>* %6, align 16, !noalias !208
  %8 = icmp slt <16 x i8> %7, zeroinitializer
  %9 = bitcast <16 x i1> %8 to i16
  %_2.i.i.i.i.i.i.i.i = xor i16 %9, -1
  %10 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 16
  %11 = bitcast i8* %self.idx.val.i.i.i.i.i to i64*
  br label %bb9.i.i.i.i

bb9.i.i.i.i:                                      ; preds = %bb9.i.i.i.i.backedge, %bb6.i.i.i.i
  %iter.sroa.0.0.i.i.i.i = phi i64* [ %11, %bb6.i.i.i.i ], [ %iter.sroa.0.0.sink.i.i.i.i, %bb9.i.i.i.i.backedge ]
  %iter.sroa.6.0.i.i.i.i = phi i8* [ %10, %bb6.i.i.i.i ], [ %iter.sroa.6.263.i.i.i.i, %bb9.i.i.i.i.backedge ]
  %iter.sroa.11.0.i.i.i.i = phi i16 [ %_2.i.i.i.i.i.i.i.i, %bb6.i.i.i.i ], [ %iter.sroa.11.264.i.i.i.i, %bb9.i.i.i.i.backedge ]
  %.not14.i.i.i.i.i.i = icmp eq i16 %iter.sroa.11.0.i.i.i.i, 0
  br i1 %.not14.i.i.i.i.i.i, label %bb6.i.i.i.i.i.i, label %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i"

"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i": ; preds = %bb8.i.i.i.i.i.i
  %_2.i.i6.i.i.le.i.i.i.i = xor i16 %16, -1
  %_4.i.i6.i.i.i.i.i = sub nuw i16 -2, %16
  %_2.i.i7.i.i.i.i.i = and i16 %_4.i.i6.i.i.i.i.i, %_2.i.i6.i.i.le.i.i.i.i
  br label %bb11.i.i.i.i

bb6.i.i.i.i.i.i:                                  ; preds = %bb9.i.i.i.i, %bb8.i.i.i.i.i.i
  %iter.sroa.0.1.i.i.i.i = phi i64* [ %17, %bb8.i.i.i.i.i.i ], [ %iter.sroa.0.0.i.i.i.i, %bb9.i.i.i.i ]
  %12 = phi i8* [ %18, %bb8.i.i.i.i.i.i ], [ %iter.sroa.6.0.i.i.i.i, %bb9.i.i.i.i ]
  %_11.not.i.i.i.i.i.i = icmp ult i8* %12, %5
  br i1 %_11.not.i.i.i.i.i.i, label %bb8.i.i.i.i.i.i, label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i"

bb8.i.i.i.i.i.i:                                  ; preds = %bb6.i.i.i.i.i.i
  %13 = bitcast i8* %12 to <16 x i8>*
  %14 = load <16 x i8>, <16 x i8>* %13, align 16, !noalias !215
  %15 = icmp slt <16 x i8> %14, zeroinitializer
  %16 = bitcast <16 x i1> %15 to i16
  %17 = getelementptr inbounds i64, i64* %iter.sroa.0.1.i.i.i.i, i64 -112
  %18 = getelementptr inbounds i8, i8* %12, i64 16
  %.not.i.i.i.i.i.i = icmp eq i16 %16, -1
  br i1 %.not.i.i.i.i.i.i, label %bb6.i.i.i.i.i.i, label %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i"

"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i": ; preds = %bb9.i.i.i.i
  %_4.i.i.i.i.i.i.i = add i16 %iter.sroa.11.0.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.sroa.11.0.i.i.i.i
  br label %bb11.i.i.i.i

bb11.i.i.i.i:                                     ; preds = %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i", %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i"
  %iter.sroa.0.0.sink.i.i.i.i = phi i64* [ %iter.sroa.0.0.i.i.i.i, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i" ], [ %17, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i" ]
  %iter.sroa.11.0.sink.i.i.i.i = phi i16 [ %iter.sroa.11.0.i.i.i.i, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i" ], [ %_2.i.i6.i.i.le.i.i.i.i, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i" ]
  %iter.sroa.11.264.i.i.i.i = phi i16 [ %_2.i.i.i.i.i.i.i, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i" ], [ %_2.i.i7.i.i.i.i.i, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i" ]
  %iter.sroa.6.263.i.i.i.i = phi i8* [ %iter.sroa.6.0.i.i.i.i, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.i.i.i.i" ], [ %18, %"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E.exit.thread.i.i.i.i" ]
  %_9.val10.i9.i.i.i.i.i = bitcast i64* %iter.sroa.0.0.sink.i.i.i.i to { i64, %ObjectInfo }*
  %19 = tail call i16 @llvm.cttz.i16(i16 %iter.sroa.11.0.sink.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i4.i.i.i.i = zext i16 %19 to i64
  %20 = sub nsw i64 0, %_2.i.i.i.i4.i.i.i.i
  %21 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_9.val10.i9.i.i.i.i.i, i64 %20, i32 0
  %_3.idx.i.i.i.i.i = getelementptr i64, i64* %21, i64 -3
  %22 = bitcast i64* %_3.idx.i.i.i.i.i to i8**
  %_3.idx.val.i.i.i.i.i = load i8*, i8** %22, align 8, !noalias !201
  %23 = getelementptr i64, i64* %21, i64 -2
  %_3.idx1.val.i.i.i.i.i = load i64, i64* %23, align 8, !noalias !201
  %_4.i.i.i.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_3.idx1.val.i.i.i.i.i, 0
  br i1 %_4.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb9.i.i.i.i.backedge, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i.i.i.i.i.i.i": ; preds = %bb11.i.i.i.i
  %24 = icmp ne i8* %_3.idx.val.i.i.i.i.i, null
  tail call void @llvm.assume(i1 %24) #24
  tail call void @__rust_dealloc(i8* nonnull %_3.idx.val.i.i.i.i.i, i64 %_3.idx1.val.i.i.i.i.i, i64 1) #24, !noalias !201
  br label %bb9.i.i.i.i.backedge

bb9.i.i.i.i.backedge:                             ; preds = %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i.i.i.i.i.i.i", %bb11.i.i.i.i
  br label %bb9.i.i.i.i

"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i": ; preds = %bb6.i.i.i.i.i.i, %"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i"
  %.pre-phi.i.i.i = phi i64 [ %.pre.i.i.i, %"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i" ], [ %4, %bb6.i.i.i.i.i.i ]
  tail call void @llvm.experimental.noalias.scope.decl(metadata !224) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !227) #24
  %25 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %.pre-phi.i.i.i, i64 56) #24
  %26 = extractvalue { i64, i1 } %25, 1
  %27 = xor i1 %26, true
  tail call void @llvm.assume(i1 %27) #24
  %28 = extractvalue { i64, i1 } %25, 0
  %29 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %28, i64 15) #24
  %30 = extractvalue { i64, i1 } %29, 1
  %31 = xor i1 %30, true
  tail call void @llvm.assume(i1 %31) #24
  %32 = extractvalue { i64, i1 } %29, 0
  %ctrl_offset.i.i.i.i.i.i = and i64 %32, -16
  %_31.i.i.i.i.i.i = add i64 %_2.i.i.i.i, 17
  %33 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %ctrl_offset.i.i.i.i.i.i, i64 %_31.i.i.i.i.i.i) #24
  %34 = extractvalue { i64, i1 } %33, 1
  %35 = xor i1 %34, true
  tail call void @llvm.assume(i1 %35) #24
  %36 = extractvalue { i64, i1 } %33, 0
  %37 = icmp eq i64 %36, 0
  br i1 %37, label %"_ZN4core3ptr125drop_in_place$LT$hashbrown..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$C$std..collections..hash..map..RandomState$GT$$GT$17h78718f5658623464E.exit", label %bb2.i.i.i.i.i.i

bb2.i.i.i.i.i.i:                                  ; preds = %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i"
  %38 = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_1, i64 0, i32 0, i32 1, i32 1, i32 2
  %_17.i.i.i.i.i = load i8*, i8** %38, align 8, !alias.scope !230, !nonnull !85, !noundef !85
  %39 = sub i64 0, %ctrl_offset.i.i.i.i.i.i
  %40 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i, i64 %39
  tail call void @__rust_dealloc(i8* nonnull %40, i64 %36, i64 16) #24, !noalias !230
  br label %"_ZN4core3ptr125drop_in_place$LT$hashbrown..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$C$std..collections..hash..map..RandomState$GT$$GT$17h78718f5658623464E.exit"

"_ZN4core3ptr125drop_in_place$LT$hashbrown..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$C$std..collections..hash..map..RandomState$GT$$GT$17h78718f5658623464E.exit": ; preds = %start, %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i", %bb2.i.i.i.i.i.i
  ret void
}

; core::panicking::assert_failed
; Function Attrs: cold noreturn nonlazybind uwtable
define internal fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef %kind, i64* noalias noundef readonly align 8 dereferenceable(8) %0, i64* noalias noundef readonly align 8 dereferenceable(8) %1, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef readonly dereferenceable(48) %args, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) %2) unnamed_addr #10 {
start:
  %_12 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %right = alloca i64*, align 8
  %left = alloca i64*, align 8
  store i64* %0, i64** %left, align 8
  store i64* %1, i64** %right, align 8
  %_6.0 = bitcast i64** %left to {}*
  %_9.0 = bitcast i64** %right to {}*
  %3 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_12 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %3)
  %4 = bitcast %"core::option::Option<core::fmt::Arguments>"* %args to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(48) %3, i8* noundef nonnull align 8 dereferenceable(48) %4, i64 48, i1 false)
; call core::panicking::assert_failed_inner
  call void @_ZN4core9panicking19assert_failed_inner17h36469c68b6fc10f1E(i8 noundef %kind, {}* noundef nonnull align 1 %_6.0, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.7 to [3 x i64]*), {}* noundef nonnull align 1 %_9.0, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.7 to [3 x i64]*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_12, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) %2) #23
  unreachable
}

; <std::sync::poison::PoisonError<T> as core::fmt::Debug>::fmt
; Function Attrs: nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN76_$LT$std..sync..poison..PoisonError$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h69df1c324ff6e669E"({ i64*, i8 }* noalias nocapture noundef readonly align 8 dereferenceable(16) %self, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64) %f) unnamed_addr #6 {
start:
  %_4 = alloca %"core::fmt::builders::DebugStruct", align 8
  %0 = bitcast %"core::fmt::builders::DebugStruct"* %_4 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %0)
; call core::fmt::Formatter::debug_struct
  call void @_ZN4core3fmt9Formatter12debug_struct17h65c357ef1edbbc54E(%"core::fmt::builders::DebugStruct"* noalias nocapture noundef nonnull sret(%"core::fmt::builders::DebugStruct") dereferenceable(16) %_4, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f, [0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [11 x i8] }>* @alloc430 to [0 x i8]*), i64 11)
; call core::fmt::builders::DebugStruct::finish_non_exhaustive
  %1 = call noundef zeroext i1 @_ZN4core3fmt8builders11DebugStruct21finish_non_exhaustive17hb4065c184e958738E(%"core::fmt::builders::DebugStruct"* noalias noundef nonnull align 8 dereferenceable(16) %_4)
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %0)
  ret i1 %1
}

; <std::panicking::begin_panic::PanicPayload<A> as core::panic::BoxMeUp>::get
; Function Attrs: nonlazybind uwtable
define internal { {}*, [3 x i64]* } @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17hc9f8af2660d4514aE"({ i8*, i64 }* noalias noundef align 8 dereferenceable(16) %self) unnamed_addr #6 {
start:
  %0 = bitcast { i8*, i64 }* %self to {}**
  %1 = load {}*, {}** %0, align 8
  %2 = icmp eq {}* %1, null
  br i1 %2, label %bb1, label %bb3

bb1:                                              ; preds = %start
; call std::process::abort
  tail call void @_ZN3std7process5abort17h9abe461bf20ade28E() #23
  unreachable

bb3:                                              ; preds = %start
  %_5.0 = bitcast { i8*, i64 }* %self to {}*
  %3 = insertvalue { {}*, [3 x i64]* } undef, {}* %_5.0, 0
  %4 = insertvalue { {}*, [3 x i64]* } %3, [3 x i64]* bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.8 to [3 x i64]*), 1
  ret { {}*, [3 x i64]* } %4
}

; <std::panicking::begin_panic::PanicPayload<A> as core::panic::BoxMeUp>::take_box
; Function Attrs: nonlazybind uwtable
define internal { {}*, [3 x i64]* } @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17h92e001d5e4efd74cE"({ i8*, i64 }* noalias nocapture noundef align 8 dereferenceable(16) %self) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %tmp.sroa.0.0..sroa_idx.i.i.i = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %self, i64 0, i32 0
  %tmp.sroa.0.0.copyload.i.i.i = load i8*, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !231
  %tmp.sroa.4.0..sroa_idx3.i.i.i = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %self, i64 0, i32 1
  %tmp.sroa.4.0.copyload.i.i.i = load i64, i64* %tmp.sroa.4.0..sroa_idx3.i.i.i, align 8, !alias.scope !231
  store i8* null, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !231
  %0 = icmp eq i8* %tmp.sroa.0.0.copyload.i.i.i, null
  br i1 %0, label %bb2, label %bb4

bb2:                                              ; preds = %start
; call std::process::abort
  tail call void @_ZN3std7process5abort17h9abe461bf20ade28E() #23
  unreachable

bb4:                                              ; preds = %start
  %1 = tail call align 8 dereferenceable_or_null(16) i8* @__rust_alloc(i64 16, i64 8) #24, !noalias !236
  %2 = icmp eq i8* %1, null
  br i1 %2, label %bb3.i.i, label %"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit"

bb3.i.i:                                          ; preds = %bb4
; call alloc::alloc::handle_alloc_error
  tail call void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64 16, i64 noundef 8) #23, !noalias !236
  unreachable

"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit": ; preds = %bb4
  %3 = bitcast i8* %1 to i8**
  store i8* %tmp.sroa.0.0.copyload.i.i.i, i8** %3, align 8, !noalias !236
  %4 = getelementptr inbounds i8, i8* %1, i64 8
  %5 = bitcast i8* %4 to i64*
  store i64 %tmp.sroa.4.0.copyload.i.i.i, i64* %5, align 8, !noalias !236
  %_13.0.cast = bitcast i8* %1 to {}*
  %6 = insertvalue { {}*, [3 x i64]* } undef, {}* %_13.0.cast, 0
  %7 = insertvalue { {}*, [3 x i64]* } %6, [3 x i64]* bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.8 to [3 x i64]*), 1
  ret { {}*, [3 x i64]* } %7
}

; hashbrown::raw::RawTable<T,A>::reserve_rehash
; Function Attrs: cold noinline nonlazybind uwtable
define internal fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h37880d6025255f2aE"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias nocapture noundef align 8 dereferenceable(32) %self, i64* noalias noundef readonly align 8 dereferenceable(16) %0) unnamed_addr #11 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !239)
  %1 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 4
  %_9.i = load i64, i64* %1, align 8, !alias.scope !239
  %2 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %_9.i, i64 1) #24
  %3 = extractvalue { i64, i1 } %2, 0
  %4 = extractvalue { i64, i1 } %2, 1
  br i1 %4, label %bb2.i, label %bb4.i

bb2.i:                                            ; preds = %start
; call hashbrown::raw::Fallibility::capacity_overflow
  %5 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !239
  %_13.0.i = extractvalue { i64, i64 } %5, 0
  %_13.1.i = extractvalue { i64, i64 } %5, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb4.i:                                            ; preds = %start
  %6 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self to i64*
  %_16.i = load i64, i64* %6, align 8, !alias.scope !239
  %_2.i.i = icmp ult i64 %_16.i, 8
  %_5.i.i = add i64 %_16.i, 1
  %_4.i.i = lshr i64 %_5.i.i, 3
  %7 = mul nuw i64 %_4.i.i, 7
  %.0.i.i = select i1 %_2.i.i, i64 %_16.i, i64 %7
  %_19.i = lshr i64 %.0.i.i, 1
  %_17.not.i = icmp ugt i64 %3, %_19.i
  br i1 %_17.not.i, label %bb9.i, label %bb7.i

bb9.i:                                            ; preds = %bb4.i
  %_30.i = add nuw i64 %.0.i.i, 1
  %8 = icmp ugt i64 %3, %_30.i
  %.0.sroa.speculated.i.i.i = select i1 %8, i64 %3, i64 %_30.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !242)
  %_5.i.i.i.i.i = icmp ult i64 %.0.sroa.speculated.i.i.i, 8
  br i1 %_5.i.i.i.i.i, label %bb1.i.i.i.i.i, label %bb5.i.i.i.i.i

bb5.i.i.i.i.i:                                    ; preds = %bb9.i
  %9 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %.0.sroa.speculated.i.i.i, i64 8) #24
  %10 = extractvalue { i64, i1 } %9, 1
  br i1 %10, label %bb9.i.i.i.i, label %bb8.i.i.i.i.i

bb1.i.i.i.i.i:                                    ; preds = %bb9.i
  %_8.i.i.i.i.i = icmp ult i64 %.0.sroa.speculated.i.i.i, 4
  %..i.i.i.i.i = select i1 %_8.i.i.i.i.i, i64 4, i64 8
  br label %bb7.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb5.i.i.i.i.i
  %11 = extractvalue { i64, i1 } %9, 0
  %adjusted_cap.i.i.i.i.i = udiv i64 %11, 7
  %p.i.i.i.i.i.i.i = add nsw i64 %adjusted_cap.i.i.i.i.i, -1
  %12 = tail call i64 @llvm.ctlz.i64(i64 %p.i.i.i.i.i.i.i, i1 true) #24, !range !245
  %13 = lshr i64 -1, %12
  %phi.bo.i.i.i.i.i.i = add i64 %13, 1
  br label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb8.i.i.i.i.i, %bb1.i.i.i.i.i
  %.sroa.4.0.i.ph.i.i.i.i = phi i64 [ %phi.bo.i.i.i.i.i.i, %bb8.i.i.i.i.i ], [ %..i.i.i.i.i, %bb1.i.i.i.i.i ]
  %14 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %.sroa.4.0.i.ph.i.i.i.i, i64 56) #24
  %15 = extractvalue { i64, i1 } %14, 1
  br i1 %15, label %bb2.i.i.i.i.i, label %bb9.i.i.i.i.i.i

bb9.i.i.i.i.i.i:                                  ; preds = %bb7.i.i.i.i
  %16 = extractvalue { i64, i1 } %14, 0
  %17 = add nuw i64 %16, 15
  %ctrl_offset.i.i.i.i.i.i = and i64 %17, -16
  %_31.i.i.i.i.i.i = add nuw nsw i64 %.sroa.4.0.i.ph.i.i.i.i, 16
  %18 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %ctrl_offset.i.i.i.i.i.i, i64 %_31.i.i.i.i.i.i) #24
  %19 = extractvalue { i64, i1 } %18, 1
  br i1 %19, label %bb2.i.i.i.i.i, label %bb4.i.i.i.i.i

bb2.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i, %bb7.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %20 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !246
  br label %bb5.i.i

bb4.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i
  %21 = extractvalue { i64, i1 } %18, 0
  %22 = icmp eq i64 %21, 0
  br i1 %22, label %bb13.i.i.i.i, label %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i

_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i: ; preds = %bb4.i.i.i.i.i
  %23 = tail call align 16 i8* @__rust_alloc(i64 %21, i64 16) #24, !noalias !246
  %24 = icmp eq i8* %23, null
  br i1 %24, label %bb15.i.i.i.i.i, label %bb13.i.i.i.i

bb15.i.i.i.i.i:                                   ; preds = %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
; call hashbrown::raw::Fallibility::alloc_err
  %25 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility9alloc_err17h3f1a17e1376e6326E(i1 noundef zeroext true, i64 %21, i64 noundef 16), !noalias !246
  br label %bb5.i.i

bb9.i.i.i.i:                                      ; preds = %bb5.i.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %26 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !253
  br label %bb5.i.i

bb13.i.i.i.i:                                     ; preds = %bb4.i.i.i.i.i, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
  %.sroa.0.0.i.i.i.i.i.i.i.i3 = phi i8* [ %23, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i ], [ inttoptr (i64 16 to i8*), %bb4.i.i.i.i.i ]
  %27 = getelementptr inbounds i8, i8* %.sroa.0.0.i.i.i.i.i.i.i.i3, i64 %ctrl_offset.i.i.i.i.i.i
  %_42.i.i.i.i.i = add nsw i64 %.sroa.4.0.i.ph.i.i.i.i, -1
  %_2.i.i10.i.i.i.i = icmp ult i64 %_42.i.i.i.i.i, 8
  %_4.i.i.i.i.i.i = lshr i64 %.sroa.4.0.i.ph.i.i.i.i, 3
  %28 = mul nuw nsw i64 %_4.i.i.i.i.i.i, 7
  %.0.i.i.i.i.i.i = select i1 %_2.i.i10.i.i.i.i, i64 %_42.i.i.i.i.i, i64 %28
  tail call void @llvm.memset.p0i8.i64(i8* nonnull align 16 %27, i8 -1, i64 %_31.i.i.i.i.i.i, i1 false) #24, !noalias !256
  %29 = sub i64 %.0.i.i.i.i.i.i, %_9.i
  %.not.i.i = icmp eq i64 %_5.i.i, 0
  %30 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %a.i.i.sroa.4.0.copyload.pre.i.i = load i8*, i8** %30, align 8, !alias.scope !257
  br i1 %.not.i.i, label %bb26.thread.i.i, label %bb15.lr.ph.i.i

bb5.i.i:                                          ; preds = %bb9.i.i.i.i, %bb15.i.i.i.i.i, %bb2.i.i.i.i.i
  %.pn.i.pn.i.i.i = phi { i64, i64 } [ %26, %bb9.i.i.i.i ], [ %25, %bb15.i.i.i.i.i ], [ %20, %bb2.i.i.i.i.i ]
  %_7.sroa.7.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 0
  %_7.sroa.13.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb26.thread.i.i:                                  ; preds = %bb13.i.i.i.i
  %31 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !258
  store i8* %27, i8** %30, align 8, !alias.scope !258
  store i64 %29, i64* %31, align 8, !alias.scope !258
  br label %bb2.i.i.i14.i.i

bb15.lr.ph.i.i:                                   ; preds = %bb13.i.i.i.i
  %table.idx.val4.i.cast.i.i = bitcast i8* %a.i.i.sroa.4.0.copyload.pre.i.i to { i64, %ObjectInfo }*
  %32 = bitcast i8* %27 to <16 x i8>*
  %_6.idx.val.i.i.i.i = load i64, i64* %0, align 8
  %33 = getelementptr i64, i64* %0, i64 1
  %_6.idx1.val.i.i.i.i = load i64, i64* %33, align 8
  %34 = xor i64 %_6.idx.val.i.i.i.i, 8317987319222330741
  %35 = xor i64 %_6.idx1.val.i.i.i.i, 7237128888997146477
  %36 = xor i64 %_6.idx.val.i.i.i.i, 7816392313619706465
  %37 = add i64 %35, %34
  %38 = tail call i64 @llvm.fshl.i64(i64 %35, i64 %35, i64 13) #24
  %39 = xor i64 %37, %38
  %40 = tail call i64 @llvm.fshl.i64(i64 %37, i64 %37, i64 32) #24
  %41 = tail call i64 @llvm.fshl.i64(i64 %39, i64 %39, i64 17) #24
  br label %bb15.i.i

bb15.i.i:                                         ; preds = %bb9.backedge.i.i, %bb15.lr.ph.i.i
  %iter.sroa.0.0100.i.i = phi i64 [ 0, %bb15.lr.ph.i.i ], [ %42, %bb9.backedge.i.i ]
  %42 = add nuw i64 %iter.sroa.0.0100.i.i, 1
  %43 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %iter.sroa.0.0100.i.i
  %_29.i.i = load i8, i8* %43, align 1, !noalias !257
  %44 = icmp sgt i8 %_29.i.i, -1
  br i1 %44, label %bb18.i.i, label %bb9.backedge.i.i

bb9.backedge.i.i:                                 ; preds = %bb22.i.i, %bb15.i.i
  %exitcond.not.i.i = icmp eq i64 %iter.sroa.0.0100.i.i, %_16.i
  br i1 %exitcond.not.i.i, label %bb26.i.i, label %bb15.i.i

bb18.i.i:                                         ; preds = %bb15.i.i
  %45 = sub i64 0, %iter.sroa.0.0100.i.i
  %46 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %table.idx.val4.i.cast.i.i, i64 %45, i32 0
  %47 = getelementptr inbounds i64, i64* %46, i64 -7
  %_7.idx.val.i.i.i = load i64, i64* %47, align 8, !alias.scope !265, !noalias !268
  %48 = xor i64 %_7.idx.val.i.i.i, %_6.idx1.val.i.i.i.i
  %49 = xor i64 %48, 8387220255154660723
  %50 = add i64 %49, %36
  %51 = tail call i64 @llvm.fshl.i64(i64 %49, i64 %49, i64 16) #24
  %52 = xor i64 %51, %50
  %53 = add i64 %52, %40
  %54 = tail call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 21) #24
  %55 = xor i64 %54, %53
  %56 = add i64 %39, %50
  %57 = xor i64 %56, %41
  %58 = tail call i64 @llvm.fshl.i64(i64 %56, i64 %56, i64 32) #24
  %59 = xor i64 %53, %_7.idx.val.i.i.i
  %60 = xor i64 %55, 576460752303423488
  %61 = add i64 %59, %57
  %62 = tail call i64 @llvm.fshl.i64(i64 %57, i64 %57, i64 13) #24
  %63 = xor i64 %61, %62
  %64 = tail call i64 @llvm.fshl.i64(i64 %61, i64 %61, i64 32) #24
  %65 = add i64 %60, %58
  %66 = tail call i64 @llvm.fshl.i64(i64 %55, i64 %60, i64 16) #24
  %67 = xor i64 %66, %65
  %68 = add i64 %67, %64
  %69 = tail call i64 @llvm.fshl.i64(i64 %67, i64 %67, i64 21) #24
  %70 = xor i64 %69, %68
  %71 = add i64 %65, %63
  %72 = tail call i64 @llvm.fshl.i64(i64 %63, i64 %63, i64 17) #24
  %73 = xor i64 %71, %72
  %74 = tail call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 32) #24
  %75 = xor i64 %68, 576460752303423488
  %76 = xor i64 %74, 255
  %77 = add i64 %75, %73
  %78 = tail call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 13) #24
  %79 = xor i64 %77, %78
  %80 = tail call i64 @llvm.fshl.i64(i64 %77, i64 %77, i64 32) #24
  %81 = add i64 %70, %76
  %82 = tail call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 16) #24
  %83 = xor i64 %82, %81
  %84 = add i64 %83, %80
  %85 = tail call i64 @llvm.fshl.i64(i64 %83, i64 %83, i64 21) #24
  %86 = xor i64 %85, %84
  %87 = add i64 %79, %81
  %88 = tail call i64 @llvm.fshl.i64(i64 %79, i64 %79, i64 17) #24
  %89 = xor i64 %87, %88
  %90 = tail call i64 @llvm.fshl.i64(i64 %87, i64 %87, i64 32) #24
  %91 = add i64 %89, %84
  %92 = tail call i64 @llvm.fshl.i64(i64 %89, i64 %89, i64 13) #24
  %93 = xor i64 %92, %91
  %94 = tail call i64 @llvm.fshl.i64(i64 %91, i64 %91, i64 32) #24
  %95 = add i64 %86, %90
  %96 = tail call i64 @llvm.fshl.i64(i64 %86, i64 %86, i64 16) #24
  %97 = xor i64 %96, %95
  %98 = add i64 %97, %94
  %99 = tail call i64 @llvm.fshl.i64(i64 %97, i64 %97, i64 21) #24
  %100 = xor i64 %99, %98
  %101 = add i64 %93, %95
  %102 = tail call i64 @llvm.fshl.i64(i64 %93, i64 %93, i64 17) #24
  %103 = xor i64 %102, %101
  %104 = tail call i64 @llvm.fshl.i64(i64 %101, i64 %101, i64 32) #24
  %105 = add i64 %103, %98
  %106 = tail call i64 @llvm.fshl.i64(i64 %103, i64 %103, i64 13) #24
  %107 = xor i64 %106, %105
  %108 = add i64 %100, %104
  %109 = tail call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 16) #24
  %110 = xor i64 %109, %108
  %111 = tail call i64 @llvm.fshl.i64(i64 %110, i64 %110, i64 21) #24
  %112 = add i64 %107, %108
  %113 = tail call i64 @llvm.fshl.i64(i64 %107, i64 %107, i64 17) #24
  %114 = tail call i64 @llvm.fshl.i64(i64 %112, i64 %112, i64 32) #24
  %_17.i.i.i.i.i.i.i.i.i = xor i64 %112, %111
  %115 = xor i64 %_17.i.i.i.i.i.i.i.i.i, %113
  %116 = xor i64 %115, %114
  %_3.i.i.i.i.i = and i64 %116, %_42.i.i.i.i.i
  %117 = getelementptr inbounds i8, i8* %27, i64 %_3.i.i.i.i.i
  %118 = bitcast i8* %117 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %118, align 1, !noalias !272
  %119 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %120 = bitcast <16 x i1> %119 to i16
  %.not23.i.i.i.i = icmp eq i16 %120, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb7.i.i8.i.i:                                     ; preds = %bb17.i.i.i.i, %bb18.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i, %bb18.i.i ], [ %126, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %120, %bb18.i.i ], [ %130, %bb17.i.i.i.i ]
  %121 = tail call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i = zext i16 %121 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_42.i.i.i.i.i
  %122 = getelementptr inbounds i8, i8* %27, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %122, align 1, !noalias !279
  %123 = icmp sgt i8 %_23.i.i.i.i, -1
  br i1 %123, label %bb11.i.i.i.i, label %bb22.i.i

bb17.i.i.i.i:                                     ; preds = %bb18.i.i, %bb17.i.i.i.i
  %probe_seq.sroa.0.025.i.i.i.i = phi i64 [ %126, %bb17.i.i.i.i ], [ %_3.i.i.i.i.i, %bb18.i.i ]
  %probe_seq.sroa.7.024.i.i.i.i = phi i64 [ %124, %bb17.i.i.i.i ], [ 0, %bb18.i.i ]
  %124 = add i64 %probe_seq.sroa.7.024.i.i.i.i, 16
  %125 = add i64 %124, %probe_seq.sroa.0.025.i.i.i.i
  %126 = and i64 %125, %_42.i.i.i.i.i
  %127 = getelementptr inbounds i8, i8* %27, i64 %126
  %128 = bitcast i8* %127 to <16 x i8>*
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %128, align 1, !noalias !272
  %129 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %130 = bitcast <16 x i1> %129 to i16
  %.not.i.i.i.i = icmp eq i16 %130, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i8.i.i
  %131 = load <16 x i8>, <16 x i8>* %32, align 16, !noalias !280
  %132 = icmp slt <16 x i8> %131, zeroinitializer
  %133 = bitcast <16 x i1> %132 to i16
  %134 = tail call i16 @llvm.cttz.i16(i16 %133, i1 true) #24, !range !27
  %_2.i.i.i.i.i = zext i16 %134 to i64
  br label %bb22.i.i

bb22.i.i:                                         ; preds = %bb11.i.i.i.i, %bb7.i.i8.i.i
  %.0.i.i.i.i = phi i64 [ %_2.i.i.i.i.i, %bb11.i.i.i.i ], [ %result.i.i.i.i, %bb7.i.i8.i.i ]
  %top7.i.i.i.i.i = lshr i64 %116, 57
  %135 = trunc i64 %top7.i.i.i.i.i to i8
  %136 = add i64 %.0.i.i.i.i, -16
  %_5.i.i.i9.i.i = and i64 %136, %_42.i.i.i.i.i
  %index2.i.i.i.i.i = add i64 %_5.i.i.i9.i.i, 16
  %137 = getelementptr inbounds i8, i8* %27, i64 %.0.i.i.i.i
  store i8 %135, i8* %137, align 1, !noalias !285
  %138 = getelementptr inbounds i8, i8* %27, i64 %index2.i.i.i.i.i
  store i8 %135, i8* %138, align 1, !noalias !285
  %_12.neg.i.i.i = xor i64 %iter.sroa.0.0100.i.i, -1
  %_11.neg.i.i.i = mul i64 %_12.neg.i.i.i, 56
  %139 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %_11.neg.i.i.i
  %_12.neg.i10.i.i = xor i64 %.0.i.i.i.i, -1
  %_11.neg.i11.i.i = mul i64 %_12.neg.i10.i.i, 56
  %140 = getelementptr inbounds i8, i8* %27, i64 %_11.neg.i11.i.i
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %140, i8* noundef nonnull align 1 dereferenceable(56) %139, i64 56, i1 false) #24, !noalias !257
  br label %bb9.backedge.i.i

bb26.i.i:                                         ; preds = %bb9.backedge.i.i
  %141 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !290
  store i8* %27, i8** %30, align 8, !alias.scope !290
  store i64 %29, i64* %141, align 8, !alias.scope !290
  %142 = icmp eq i64 %_16.i, 0
  br i1 %142, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit", label %bb2.i.i.i14.i.i

bb2.i.i.i14.i.i:                                  ; preds = %bb26.i.i, %bb26.thread.i.i
  %143 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %_5.i.i, i64 56) #24
  %144 = extractvalue { i64, i1 } %143, 1
  %145 = xor i1 %144, true
  tail call void @llvm.assume(i1 %145) #24
  %146 = extractvalue { i64, i1 } %143, 0
  %147 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %146, i64 15) #24
  %148 = extractvalue { i64, i1 } %147, 1
  %149 = xor i1 %148, true
  tail call void @llvm.assume(i1 %149) #24
  %150 = extractvalue { i64, i1 } %147, 0
  %ctrl_offset.i.i.i.i.i.i.i = and i64 %150, -16
  %_31.i.i.i.i.i.i.i = add i64 %_16.i, 17
  %151 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %ctrl_offset.i.i.i.i.i.i.i, i64 %_31.i.i.i.i.i.i.i) #24
  %152 = extractvalue { i64, i1 } %151, 1
  %153 = xor i1 %152, true
  tail call void @llvm.assume(i1 %153) #24
  %154 = extractvalue { i64, i1 } %151, 0
  %155 = icmp eq i64 %154, 0
  br i1 %155, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit", label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb2.i.i.i14.i.i
  %156 = icmp ne i8* %a.i.i.sroa.4.0.copyload.pre.i.i, null
  tail call void @llvm.assume(i1 %156)
  %157 = sub i64 0, %ctrl_offset.i.i.i.i.i.i.i
  %158 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %157
  tail call void @__rust_dealloc(i8* nonnull %158, i64 %154, i64 16) #24, !noalias !293
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb7.i:                                            ; preds = %bb4.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !300)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !303)
  %159 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %self.idx12.val.i.i.i = load i8*, i8** %159, align 8, !alias.scope !306
  %160 = bitcast i8* %self.idx12.val.i.i.i to { i64, %ObjectInfo }*
  br label %bb4.i.i.i

bb4.i.i.i:                                        ; preds = %bb6.i.i.i, %bb7.i
  %iter.sroa.0.0.i.i.i = phi i64 [ 0, %bb7.i ], [ %iter.sroa.0.165.i.i.i, %bb6.i.i.i ]
  %_2.not.i.i.i.i = phi i1 [ false, %bb7.i ], [ true, %bb6.i.i.i ]
  br i1 %_2.not.i.i.i.i, label %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i", label %bb1.i.i.i.i

bb1.i.i.i.i:                                      ; preds = %bb4.i.i.i
  %161 = icmp ult i64 %iter.sroa.0.0.i.i.i, %_5.i.i
  br i1 %161, label %bb6.i.i.i, label %bb8.i.i.i

"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i": ; preds = %bb4.i.i.i
  %162 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %iter.sroa.0.0.i.i.i, i64 15) #24
  %163 = extractvalue { i64, i1 } %162, 0
  %164 = extractvalue { i64, i1 } %162, 1
  %_5.1.not.i.i.i.i.i.i.i.i = xor i1 %164, true
  %165 = icmp ult i64 %163, %_5.i.i
  %or.cond.i.i.i.i.i.i = select i1 %_5.1.not.i.i.i.i.i.i.i.i, i1 %165, i1 false
  br i1 %or.cond.i.i.i.i.i.i, label %bb6.i.i.i, label %bb8.i.i.i

bb8.i.i.i:                                        ; preds = %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i", %bb1.i.i.i.i
  %_25.i.i.i = icmp ult i64 %_5.i.i, 16
  br i1 %_25.i.i.i, label %bb5.i2.i, label %bb5.thread.i.i

bb6.i.i.i:                                        ; preds = %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i", %bb1.i.i.i.i
  %_3.val.i.i.pn.i67.i.i.i = phi i64 [ %163, %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i" ], [ %iter.sroa.0.0.i.i.i, %bb1.i.i.i.i ]
  %iter.sroa.0.165.i.i.i = add nuw i64 %_3.val.i.i.pn.i67.i.i.i, 1
  %166 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_3.val.i.i.pn.i67.i.i.i
  %167 = bitcast i8* %166 to <2 x i64>*
  %168 = bitcast i8* %166 to <16 x i8>*
  %169 = load <16 x i8>, <16 x i8>* %168, align 16, !noalias !307
  %.lobit.i.i.i.i = ashr <16 x i8> %169, <i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7>
  %170 = bitcast <16 x i8> %.lobit.i.i.i.i to <2 x i64>
  %171 = or <2 x i64> %170, <i64 -9187201950435737472, i64 -9187201950435737472>
  store <2 x i64> %171, <2 x i64>* %167, align 16, !noalias !312
  br label %bb4.i.i.i

bb5.thread.i.i:                                   ; preds = %bb8.i.i.i
  %172 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_5.i.i
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(16) %172, i8* noundef nonnull align 1 dereferenceable(16) %self.idx12.val.i.i.i, i64 16, i1 false) #24, !noalias !306
  br label %bb12.lr.ph.i.i

bb5.i2.i:                                         ; preds = %bb8.i.i.i
  %173 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 16
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* nonnull align 1 %173, i8* align 1 %self.idx12.val.i.i.i, i64 %_5.i.i, i1 false) #24, !noalias !306
  %.not.i1.i = icmp eq i64 %_5.i.i, 0
  br i1 %.not.i1.i, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i", label %bb12.lr.ph.i.i

bb12.lr.ph.i.i:                                   ; preds = %bb5.i2.i, %bb5.thread.i.i
  %174 = getelementptr i64, i64* %0, i64 1
  %_6.idx.val.i.i.i10.i = load i64, i64* %0, align 8
  %_6.idx1.val.i.i.i12.i = load i64, i64* %174, align 8
  %175 = xor i64 %_6.idx.val.i.i.i10.i, 8317987319222330741
  %176 = xor i64 %_6.idx1.val.i.i.i12.i, 7237128888997146477
  %177 = xor i64 %_6.idx.val.i.i.i10.i, 7816392313619706465
  %178 = add i64 %176, %175
  %179 = tail call i64 @llvm.fshl.i64(i64 %176, i64 %176, i64 13) #24
  %180 = xor i64 %178, %179
  %181 = tail call i64 @llvm.fshl.i64(i64 %178, i64 %178, i64 32) #24
  %182 = tail call i64 @llvm.fshl.i64(i64 %180, i64 %180, i64 17) #24
  %183 = bitcast i8* %self.idx12.val.i.i.i to <16 x i8>*
  %184 = bitcast i8* %self.idx12.val.i.i.i to { i64, %ObjectInfo }*
  br label %bb12.i.i

bb12.i.i:                                         ; preds = %bb40.i.i, %bb12.lr.ph.i.i
  %table.idx.val4.i43.i.i = phi { i64, %ObjectInfo }* [ %160, %bb12.lr.ph.i.i ], [ %table.idx.val4.i44.i.i, %bb40.i.i ]
  %iter.sroa.0.030.i.i = phi i64 [ 0, %bb12.lr.ph.i.i ], [ %185, %bb40.i.i ]
  %185 = add nuw i64 %iter.sroa.0.030.i.i, 1
  %186 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.030.i.i
  %_23.i.i = load i8, i8* %186, align 1, !noalias !315
  %_22.not.i.i = icmp eq i8 %_23.i.i, -128
  br i1 %_22.not.i.i, label %bb14.i.i, label %bb40.i.i

bb40.i.i:                                         ; preds = %bb34.i.i, %bb27.i.i, %bb12.i.i
  %table.idx.val4.i44.i.i = phi { i64, %ObjectInfo }* [ %160, %bb34.i.i ], [ %184, %bb27.i.i ], [ %table.idx.val4.i43.i.i, %bb12.i.i ]
  %exitcond.not.i3.i = icmp eq i64 %iter.sroa.0.030.i.i, %_16.i
  br i1 %exitcond.not.i3.i, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i", label %bb12.i.i

bb14.i.i:                                         ; preds = %bb12.i.i
  %_12.neg.i.i4.i = xor i64 %iter.sroa.0.030.i.i, -1
  %_11.neg.i.i5.i = mul i64 %_12.neg.i.i4.i, 56
  %187 = getelementptr i8, i8* %self.idx12.val.i.i.i, i64 %_11.neg.i.i5.i
  %188 = sub i64 0, %iter.sroa.0.030.i.i
  %189 = bitcast i8* %187 to <16 x i8>*
  %190 = bitcast i8* %187 to <16 x i8>*
  %191 = getelementptr inbounds i8, i8* %187, i64 16
  %192 = bitcast i8* %191 to <16 x i8>*
  %193 = bitcast i8* %191 to <16 x i8>*
  %194 = getelementptr inbounds i8, i8* %187, i64 32
  %195 = bitcast i8* %194 to <16 x i8>*
  %196 = bitcast i8* %194 to <16 x i8>*
  %197 = getelementptr inbounds i8, i8* %187, i64 48
  %198 = bitcast i8* %197 to <8 x i8>*
  %199 = bitcast i8* %197 to <8 x i8>*
  br label %bb19.i.i

bb19.i.i:                                         ; preds = %vector.body, %bb14.i.i
  %table.idx.val4.i.i.i = phi { i64, %ObjectInfo }* [ %table.idx.val4.i43.i.i, %bb14.i.i ], [ %160, %vector.body ]
  %200 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %table.idx.val4.i.i.i, i64 %188, i32 0
  %201 = getelementptr inbounds i64, i64* %200, i64 -7
  %_7.idx.val.i.i7.i = load i64, i64* %201, align 8, !alias.scope !316, !noalias !319
  %202 = xor i64 %_7.idx.val.i.i7.i, %_6.idx1.val.i.i.i12.i
  %203 = xor i64 %202, 8387220255154660723
  %204 = add i64 %203, %177
  %205 = tail call i64 @llvm.fshl.i64(i64 %203, i64 %203, i64 16) #24
  %206 = xor i64 %205, %204
  %207 = add i64 %206, %181
  %208 = tail call i64 @llvm.fshl.i64(i64 %206, i64 %206, i64 21) #24
  %209 = xor i64 %208, %207
  %210 = add i64 %180, %204
  %211 = xor i64 %210, %182
  %212 = tail call i64 @llvm.fshl.i64(i64 %210, i64 %210, i64 32) #24
  %213 = xor i64 %207, %_7.idx.val.i.i7.i
  %214 = xor i64 %209, 576460752303423488
  %215 = add i64 %213, %211
  %216 = tail call i64 @llvm.fshl.i64(i64 %211, i64 %211, i64 13) #24
  %217 = xor i64 %215, %216
  %218 = tail call i64 @llvm.fshl.i64(i64 %215, i64 %215, i64 32) #24
  %219 = add i64 %214, %212
  %220 = tail call i64 @llvm.fshl.i64(i64 %209, i64 %214, i64 16) #24
  %221 = xor i64 %220, %219
  %222 = add i64 %221, %218
  %223 = tail call i64 @llvm.fshl.i64(i64 %221, i64 %221, i64 21) #24
  %224 = xor i64 %223, %222
  %225 = add i64 %219, %217
  %226 = tail call i64 @llvm.fshl.i64(i64 %217, i64 %217, i64 17) #24
  %227 = xor i64 %225, %226
  %228 = tail call i64 @llvm.fshl.i64(i64 %225, i64 %225, i64 32) #24
  %229 = xor i64 %222, 576460752303423488
  %230 = xor i64 %228, 255
  %231 = add i64 %229, %227
  %232 = tail call i64 @llvm.fshl.i64(i64 %227, i64 %227, i64 13) #24
  %233 = xor i64 %231, %232
  %234 = tail call i64 @llvm.fshl.i64(i64 %231, i64 %231, i64 32) #24
  %235 = add i64 %224, %230
  %236 = tail call i64 @llvm.fshl.i64(i64 %224, i64 %224, i64 16) #24
  %237 = xor i64 %236, %235
  %238 = add i64 %237, %234
  %239 = tail call i64 @llvm.fshl.i64(i64 %237, i64 %237, i64 21) #24
  %240 = xor i64 %239, %238
  %241 = add i64 %233, %235
  %242 = tail call i64 @llvm.fshl.i64(i64 %233, i64 %233, i64 17) #24
  %243 = xor i64 %241, %242
  %244 = tail call i64 @llvm.fshl.i64(i64 %241, i64 %241, i64 32) #24
  %245 = add i64 %243, %238
  %246 = tail call i64 @llvm.fshl.i64(i64 %243, i64 %243, i64 13) #24
  %247 = xor i64 %246, %245
  %248 = tail call i64 @llvm.fshl.i64(i64 %245, i64 %245, i64 32) #24
  %249 = add i64 %240, %244
  %250 = tail call i64 @llvm.fshl.i64(i64 %240, i64 %240, i64 16) #24
  %251 = xor i64 %250, %249
  %252 = add i64 %251, %248
  %253 = tail call i64 @llvm.fshl.i64(i64 %251, i64 %251, i64 21) #24
  %254 = xor i64 %253, %252
  %255 = add i64 %247, %249
  %256 = tail call i64 @llvm.fshl.i64(i64 %247, i64 %247, i64 17) #24
  %257 = xor i64 %256, %255
  %258 = tail call i64 @llvm.fshl.i64(i64 %255, i64 %255, i64 32) #24
  %259 = add i64 %257, %252
  %260 = tail call i64 @llvm.fshl.i64(i64 %257, i64 %257, i64 13) #24
  %261 = xor i64 %260, %259
  %262 = add i64 %254, %258
  %263 = tail call i64 @llvm.fshl.i64(i64 %254, i64 %254, i64 16) #24
  %264 = xor i64 %263, %262
  %265 = tail call i64 @llvm.fshl.i64(i64 %264, i64 %264, i64 21) #24
  %266 = add i64 %261, %262
  %267 = tail call i64 @llvm.fshl.i64(i64 %261, i64 %261, i64 17) #24
  %268 = tail call i64 @llvm.fshl.i64(i64 %266, i64 %266, i64 32) #24
  %_17.i.i.i.i.i.i.i.i13.i = xor i64 %266, %265
  %269 = xor i64 %_17.i.i.i.i.i.i.i.i13.i, %267
  %270 = xor i64 %269, %268
  %_3.i.i3.i.i = and i64 %270, %_16.i
  %271 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_3.i.i3.i.i
  %272 = bitcast i8* %271 to <16 x i8>*
  %.0.copyload.i2122.i.i.i = load <16 x i8>, <16 x i8>* %272, align 1, !noalias !323
  %273 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i, zeroinitializer
  %274 = bitcast <16 x i1> %273 to i16
  %.not23.i.i.i = icmp eq i16 %274, 0
  br i1 %.not23.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb7.i.i.i:                                        ; preds = %bb17.i.i.i, %bb19.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i = phi i64 [ %_3.i.i3.i.i, %bb19.i.i ], [ %280, %bb17.i.i.i ]
  %.lcssa.i.i.i = phi i16 [ %274, %bb19.i.i ], [ %284, %bb17.i.i.i ]
  %275 = tail call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i14.i = zext i16 %275 to i64
  %_17.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i, %_2.i.i.i.i14.i
  %result.i.i.i = and i64 %_17.i.i.i, %_16.i
  %276 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %result.i.i.i
  %_23.i.i.i = load i8, i8* %276, align 1, !noalias !328
  %277 = icmp sgt i8 %_23.i.i.i, -1
  br i1 %277, label %bb11.i.i.i, label %bb24.i.i

bb17.i.i.i:                                       ; preds = %bb19.i.i, %bb17.i.i.i
  %probe_seq.sroa.0.025.i.i.i = phi i64 [ %280, %bb17.i.i.i ], [ %_3.i.i3.i.i, %bb19.i.i ]
  %probe_seq.sroa.7.024.i.i.i = phi i64 [ %278, %bb17.i.i.i ], [ 0, %bb19.i.i ]
  %278 = add i64 %probe_seq.sroa.7.024.i.i.i, 16
  %279 = add i64 %278, %probe_seq.sroa.0.025.i.i.i
  %280 = and i64 %279, %_16.i
  %281 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %280
  %282 = bitcast i8* %281 to <16 x i8>*
  %.0.copyload.i21.i.i.i = load <16 x i8>, <16 x i8>* %282, align 1, !noalias !323
  %283 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i, zeroinitializer
  %284 = bitcast <16 x i1> %283 to i16
  %.not.i.i.i = icmp eq i16 %284, 0
  br i1 %.not.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb11.i.i.i:                                       ; preds = %bb7.i.i.i
  %285 = load <16 x i8>, <16 x i8>* %183, align 16, !noalias !329
  %286 = icmp slt <16 x i8> %285, zeroinitializer
  %287 = bitcast <16 x i1> %286 to i16
  %288 = tail call i16 @llvm.cttz.i16(i16 %287, i1 true) #24, !range !27
  %_2.i.i6.i.i = zext i16 %288 to i64
  br label %bb24.i.i

bb24.i.i:                                         ; preds = %bb11.i.i.i, %bb7.i.i.i
  %.0.i.i.i = phi i64 [ %_2.i.i6.i.i, %bb11.i.i.i ], [ %result.i.i.i, %bb7.i.i.i ]
  %_12.neg.i7.i.i = xor i64 %.0.i.i.i, -1
  %_11.neg.i8.i.i = mul i64 %_12.neg.i7.i.i, 56
  %289 = getelementptr i8, i8* %self.idx12.val.i.i.i, i64 %_11.neg.i8.i.i
  %290 = sub i64 %iter.sroa.0.030.i.i, %_3.i.i3.i.i
  %291 = sub i64 %.0.i.i.i, %_3.i.i3.i.i
  %_3.i612.i.i.i = xor i64 %291, %290
  %.unshifted.i.i.i = and i64 %_3.i612.i.i.i, %_16.i
  %292 = icmp ult i64 %.unshifted.i.i.i, 16
  br i1 %292, label %bb27.i.i, label %bb31.i.i

bb27.i.i:                                         ; preds = %bb24.i.i
  %top7.i.i.i.i = lshr i64 %270, 57
  %293 = trunc i64 %top7.i.i.i.i to i8
  %294 = add i64 %iter.sroa.0.030.i.i, -16
  %_5.i.i.i.i = and i64 %294, %_16.i
  %index2.i.i.i.i = add i64 %_5.i.i.i.i, 16
  %295 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.030.i.i
  store i8 %293, i8* %295, align 1, !noalias !334
  %296 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i
  store i8 %293, i8* %296, align 1, !noalias !334
  br label %bb40.i.i

bb31.i.i:                                         ; preds = %bb24.i.i
  %297 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %.0.i.i.i
  %prev_ctrl.i.i.i = load i8, i8* %297, align 1, !noalias !339
  %top7.i.i.i.i15.i = lshr i64 %270, 57
  %298 = trunc i64 %top7.i.i.i.i15.i to i8
  %299 = add i64 %.0.i.i.i, -16
  %_5.i.i.i.i16.i = and i64 %299, %_16.i
  %index2.i.i.i.i17.i = add i64 %_5.i.i.i.i16.i, 16
  store i8 %298, i8* %297, align 1, !noalias !342
  %300 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i17.i
  store i8 %298, i8* %300, align 1, !noalias !342
  %_73.i.i = icmp eq i8 %prev_ctrl.i.i.i, -1
  br i1 %_73.i.i, label %bb34.i.i, label %vector.body

vector.body:                                      ; preds = %bb31.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !347) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !350) #24
  %wide.load = load <16 x i8>, <16 x i8>* %189, align 1, !alias.scope !347, !noalias !352
  %301 = bitcast i8* %289 to <16 x i8>*
  %wide.load34 = load <16 x i8>, <16 x i8>* %301, align 1, !alias.scope !350, !noalias !353
  store <16 x i8> %wide.load34, <16 x i8>* %190, align 1, !alias.scope !347, !noalias !352
  %302 = bitcast i8* %289 to <16 x i8>*
  store <16 x i8> %wide.load, <16 x i8>* %302, align 1, !alias.scope !350, !noalias !353
  %303 = getelementptr inbounds i8, i8* %289, i64 16
  tail call void @llvm.experimental.noalias.scope.decl(metadata !354) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !356) #24
  %wide.load.1 = load <16 x i8>, <16 x i8>* %192, align 1, !alias.scope !354, !noalias !358
  %304 = bitcast i8* %303 to <16 x i8>*
  %wide.load34.1 = load <16 x i8>, <16 x i8>* %304, align 1, !alias.scope !356, !noalias !359
  store <16 x i8> %wide.load34.1, <16 x i8>* %193, align 1, !alias.scope !354, !noalias !358
  %305 = bitcast i8* %303 to <16 x i8>*
  store <16 x i8> %wide.load.1, <16 x i8>* %305, align 1, !alias.scope !356, !noalias !359
  %306 = getelementptr inbounds i8, i8* %289, i64 32
  tail call void @llvm.experimental.noalias.scope.decl(metadata !360) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !362) #24
  %wide.load.2 = load <16 x i8>, <16 x i8>* %195, align 1, !alias.scope !360, !noalias !364
  %307 = bitcast i8* %306 to <16 x i8>*
  %wide.load34.2 = load <16 x i8>, <16 x i8>* %307, align 1, !alias.scope !362, !noalias !365
  store <16 x i8> %wide.load34.2, <16 x i8>* %196, align 1, !alias.scope !360, !noalias !364
  %308 = bitcast i8* %306 to <16 x i8>*
  store <16 x i8> %wide.load.2, <16 x i8>* %308, align 1, !alias.scope !362, !noalias !365
  %309 = getelementptr inbounds i8, i8* %289, i64 48
  tail call void @llvm.experimental.noalias.scope.decl(metadata !347) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !350) #24
  %wide.load37 = load <8 x i8>, <8 x i8>* %198, align 1, !alias.scope !347, !noalias !352
  %310 = bitcast i8* %309 to <8 x i8>*
  %wide.load38 = load <8 x i8>, <8 x i8>* %310, align 1, !alias.scope !350, !noalias !353
  store <8 x i8> %wide.load38, <8 x i8>* %199, align 1, !alias.scope !347, !noalias !352
  %311 = bitcast i8* %309 to <8 x i8>*
  store <8 x i8> %wide.load37, <8 x i8>* %311, align 1, !alias.scope !350, !noalias !353
  br label %bb19.i.i, !llvm.loop !366

bb34.i.i:                                         ; preds = %bb31.i.i
  %312 = add i64 %iter.sroa.0.030.i.i, -16
  %_5.i.i.i = and i64 %312, %_16.i
  %index2.i.i.i = add i64 %_5.i.i.i, 16
  %313 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.030.i.i
  store i8 -1, i8* %313, align 1, !noalias !369
  %314 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i
  store i8 -1, i8* %314, align 1, !noalias !369
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(56) %289, i8* noundef nonnull align 1 dereferenceable(56) %187, i64 56, i1 false) #24, !noalias !315
  br label %bb40.i.i

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i": ; preds = %bb40.i.i, %bb5.i2.i
  %315 = phi i64 [ 0, %bb5.i2.i ], [ %.0.i.i, %bb40.i.i ]
  %316 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  %317 = sub i64 %315, %_9.i
  store i64 %317, i64* %316, align 8, !alias.scope !315
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit": ; preds = %bb2.i, %bb5.i.i, %bb26.i.i, %bb2.i.i.i14.i.i, %bb2.i.i.i.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i"
  %.sroa.3.0.i = phi i64 [ -9223372036854775807, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i" ], [ %_13.1.i, %bb2.i ], [ %_7.sroa.13.0.i.i.i, %bb5.i.i ], [ -9223372036854775807, %bb26.i.i ], [ -9223372036854775807, %bb2.i.i.i14.i.i ], [ -9223372036854775807, %bb2.i.i.i.i.i.i.i ]
  %.sroa.0.0.i = phi i64 [ undef, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i" ], [ %_13.0.i, %bb2.i ], [ %_7.sroa.7.0.i.i.i, %bb5.i.i ], [ undef, %bb26.i.i ], [ undef, %bb2.i.i.i14.i.i ], [ undef, %bb2.i.i.i.i.i.i.i ]
  %318 = insertvalue { i64, i64 } undef, i64 %.sroa.0.0.i, 0
  %319 = insertvalue { i64, i64 } %318, i64 %.sroa.3.0.i, 1
  ret { i64, i64 } %319
}

; once_cell::imp::OnceCell<T>::initialize
; Function Attrs: cold nonlazybind uwtable
define internal fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h1ed77e854a4795c8E"(i64* noalias noundef readonly align 8 dereferenceable(8) %f) unnamed_addr #12 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %res = alloca %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok", align 1
  %_14 = alloca %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", align 8
  %slot = alloca %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, align 8
  %f1 = alloca i64*, align 8
  %0 = bitcast i64** %f1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %0)
  store i64* %f, i64** %f1, align 8
  %1 = bitcast %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %slot to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %1)
  store %"core::option::Option<std::sync::mutex::Mutex<i64>>"* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 0, i64 8) to %"core::option::Option<std::sync::mutex::Mutex<i64>>"*), %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %slot, align 8
  %2 = bitcast %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14 to i8*
  call void @llvm.lifetime.start.p0i8(i64 24, i8* nonnull %2)
  %3 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 0
  store i64** %f1, i64*** %3, align 8
  %4 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 1
  store %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %slot, %"core::option::Option<std::sync::mutex::Mutex<i64>>"*** %4, align 8
  %5 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 2
  store %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"* %res, %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"** %5, align 8
; call once_cell::imp::initialize_or_wait
  call void @_ZN9once_cell3imp18initialize_or_wait17h9b3310b1603d0203E(%"core::sync::atomic::AtomicUsize"* noundef align 8 dereferenceable(8) bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to %"core::sync::atomic::AtomicUsize"*), i8* noundef nonnull align 1 %2, i8* bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.b to i8*))
  call void @llvm.lifetime.end.p0i8(i64 24, i8* nonnull %2)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %1)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %0)
  ret void
}

; once_cell::imp::OnceCell<T>::initialize
; Function Attrs: cold nonlazybind uwtable
define internal fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef readonly align 8 dereferenceable(8) %f) unnamed_addr #12 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %res = alloca %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok", align 1
  %_14 = alloca %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", align 8
  %slot = alloca %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %f1 = alloca i64*, align 8
  %0 = bitcast i64** %f1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %0)
  store i64* %f, i64** %f1, align 8
  %1 = bitcast %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %slot to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %1)
  store %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %slot, align 8
  %2 = bitcast %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14 to i8*
  call void @llvm.lifetime.start.p0i8(i64 24, i8* nonnull %2)
  %3 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 0
  store i64** %f1, i64*** %3, align 8
  %4 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 1
  store %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %slot, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %4, align 8
  %5 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 2
  store %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"* %res, %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"** %5, align 8
; call once_cell::imp::initialize_or_wait
  call void @_ZN9once_cell3imp18initialize_or_wait17h9b3310b1603d0203E(%"core::sync::atomic::AtomicUsize"* noundef align 8 dereferenceable(8) bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"core::sync::atomic::AtomicUsize"*), i8* noundef nonnull align 1 %2, i8* bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.c to i8*))
  call void @llvm.lifetime.end.p0i8(i64 24, i8* nonnull %2)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %1)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %0)
  ret void
}

; once_cell::imp::OnceCell<T>::initialize::{{closure}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* noalias nocapture noundef readonly align 8 dereferenceable(24) %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0 = alloca %"std::sync::mutex::Mutex<i64>", align 8
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15 = load i64**, i64*** %0, align 8, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15 to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !372
  store i64* null, i64** %_15, align 8, !alias.scope !372
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<i64>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !379)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !382)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %_8.i.i, align 8, !alias.scope !385, !noalias !386, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !389, !noalias !392
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !389, !noalias !392
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !392
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<i64>"*)*
  call void %7(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %_5.sroa.0), !noalias !379
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16 = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"**, %"core::option::Option<std::sync::mutex::Mutex<i64>>"*** %8, align 8, !nonnull !85, !align !86, !noundef !85
  %_17 = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16, align 8
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17, i64 0, i32 0
  %_2.i16 = load i64, i64* %9, align 8, !range !120, !noundef !85
  %10 = icmp eq i64 %_2.i16, 0
  br i1 %10, label %bb9, label %bb2.i

bb2.i:                                            ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17, i64 0, i32 1
  %12 = bitcast [2 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %bb2.i.bb9_crit_edge unwind label %cleanup

bb2.i.bb9_crit_edge:                              ; preds = %bb2.i
  %_22.pre = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16, align 8
  br label %bb9

cleanup:                                          ; preds = %bb2.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %_20 = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16, align 8
  %_10.sroa.0.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_20, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx, align 8
  %_10.sroa.5.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_20, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast = bitcast [2 x i64]* %_10.sroa.5.0..sroa_idx to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(16) %_10.sroa.5.0..sroa_cast, i8* noundef nonnull align 8 dereferenceable(16) %_5.sroa.0.0.sroa_cast20, i64 16, i1 false)
  resume { i8*, i32 } %13

bb9:                                              ; preds = %bb2.i.bb9_crit_edge, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit"
  %_22 = phi %"core::option::Option<std::sync::mutex::Mutex<i64>>"* [ %_22.pre, %bb2.i.bb9_crit_edge ], [ %_17, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit" ]
  %_10.sroa.0.0..sroa_idx2 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_22, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx2, align 8
  %_10.sroa.5.0..sroa_idx6 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_22, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast7 = bitcast [2 x i64]* %_10.sroa.5.0..sroa_idx6 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(16) %_10.sroa.5.0..sroa_cast7, i8* noundef nonnull align 8 dereferenceable(16) %_5.sroa.0.0.sroa_cast20, i64 16, i1 false)
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  ret i1 true
}

; once_cell::imp::OnceCell<T>::initialize::{{closure}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* noalias nocapture noundef readonly align 8 dereferenceable(24) %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0 = alloca %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", align 8
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15 = load i64**, i64*** %0, align 8, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15 to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !393
  store i64* null, i64** %_15, align 8, !alias.scope !393
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !400)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !403)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_8.i.i, align 8, !alias.scope !406, !noalias !407, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !410, !noalias !413
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !410, !noalias !413
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !413
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0), !noalias !400
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !nonnull !85, !align !86, !noundef !85
  %_17 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 0
  %_2.i16 = load i64, i64* %9, align 8, !range !120, !noundef !85
  %10 = icmp eq i64 %_2.i16, 0
  br i1 %10, label %bb9, label %bb2.i

bb2.i:                                            ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1
  %12 = bitcast [7 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i" unwind label %cleanup.i.i

cleanup.i.i:                                      ; preds = %bb2.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %14 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 1
  %15 = bitcast i64* %14 to %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*
; call core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h1eb938b370d22c57E"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #25
  %_20 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %_10.sroa.0.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx, align 8
  %_10.sroa.5.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20, i64 56, i1 false)
  resume { i8*, i32 } %13

"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i": ; preds = %bb2.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 1
  %17 = bitcast i64* %16 to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*
; call core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
  tail call fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %17) #24
  %_22.pre = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  br label %bb9

bb9:                                              ; preds = %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i", %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit"
  %_22 = phi %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* [ %_22.pre, %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i" ], [ %_17, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit" ]
  %_10.sroa.0.0..sroa_idx2 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_22, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx2, align 8
  %_10.sroa.5.0..sroa_idx6 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_22, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast7 = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx6 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast7, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20, i64 56, i1 false)
  call void @llvm.lifetime.end.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  ret i1 true
}

; Function Attrs: nonlazybind uwtable
define i64 @report_malloc(i8* %address, i8* %name) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_23.i.i = alloca { i64, %ObjectInfo }, align 8
  %e.i46 = alloca { i64*, i8 }, align 8
  %this.i.i32 = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %e.i26 = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, align 8
  %e.i = alloca %"core::str::error::Utf8Error", align 8
  %object_table = alloca { i64*, i8 }, align 8
  %_44 = alloca i64, align 8
  %_37 = alloca [3 x { i8*, i64* }], align 8
  %_30 = alloca %"core::fmt::Arguments", align 8
  %guard = alloca { i64*, i8 }, align 8
  %name_c_str1 = alloca { [0 x i8]*, i64 }, align 8
  %_10 = alloca %"core::fmt::Arguments", align 8
  %name_c_str = alloca %"core::result::Result<&str, core::str::error::Utf8Error>", align 8
  %objid = alloca i64, align 8
  %len.i = tail call i64 @strlen(i8* noundef nonnull dereferenceable(1) %name) #24
  %_9.i = add i64 %len.i, 1
  %_2.0.i.i.i = bitcast i8* %name to %"core::ffi::c_str::CStr"*
  %0 = bitcast %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str to i8*
  call void @llvm.lifetime.start.p0i8(i64 24, i8* nonnull %0)
; call core::ffi::c_str::CStr::to_str
  call void @_ZN4core3ffi5c_str4CStr6to_str17haa887525d1060a40E(%"core::result::Result<&str, core::str::error::Utf8Error>"* noalias nocapture noundef nonnull sret(%"core::result::Result<&str, core::str::error::Utf8Error>") dereferenceable(24) %name_c_str, %"core::ffi::c_str::CStr"* noalias noundef nonnull readonly align 1 %_2.0.i.i.i, i64 %_9.i)
  %name_c_str.idx = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 0
  %name_c_str.idx.val = load i64, i64* %name_c_str.idx, align 8
  %.not = icmp eq i64 %name_c_str.idx.val, 0
  br i1 %.not, label %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit", label %bb1.i

bb1.i:                                            ; preds = %start
  %1 = bitcast %"core::fmt::Arguments"* %_10 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1)
  %2 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc71 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %2, align 8, !alias.scope !414, !noalias !417
  %3 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 0, i32 1
  store i64 1, i64* %3, align 8, !alias.scope !414, !noalias !417
  %4 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 1, i32 0
  store i64* null, i64** %4, align 8, !alias.scope !414, !noalias !417
  %5 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{}>* @alloc73 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %5, align 8, !alias.scope !414, !noalias !417
  %6 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 2, i32 1
  store i64 0, i64* %6, align 8, !alias.scope !414, !noalias !417
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_10)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1)
  %7 = bitcast { [0 x i8]*, i64 }* %name_c_str1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %7)
  %_18.sroa.4.0..sroa_idx79 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1
  %_18.sroa.4.0..sroa_cast = bitcast [2 x i64]* %_18.sroa.4.0..sroa_idx79 to [0 x i8]**
  %_18.sroa.4.0.copyload = load [0 x i8]*, [0 x i8]** %_18.sroa.4.0..sroa_cast, align 8
  %_18.sroa.6.0..sroa_idx81 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1, i64 1
  %_18.sroa.6.0.copyload = load i64, i64* %_18.sroa.6.0..sroa_idx81, align 8
  %8 = bitcast %"core::str::error::Utf8Error"* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %8), !noalias !420
  %_18.sroa.4.8..sroa_cast = bitcast %"core::str::error::Utf8Error"* %e.i to [0 x i8]**
  store [0 x i8]* %_18.sroa.4.0.copyload, [0 x i8]** %_18.sroa.4.8..sroa_cast, align 8
  %_18.sroa.6.8..sroa_idx83 = getelementptr inbounds %"core::str::error::Utf8Error", %"core::str::error::Utf8Error"* %e.i, i64 0, i32 1
  %_18.sroa.6.8..sroa_cast = bitcast { i8, i8 }* %_18.sroa.6.8..sroa_idx83 to i64*
  store i64 %_18.sroa.6.0.copyload, i64* %_18.sroa.6.8..sroa_cast, align 8
  %_6.0.i = bitcast %"core::str::error::Utf8Error"* %e.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.5 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc466 to %"core::panic::location::Location"*)) #23, !noalias !420
  unreachable

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit": ; preds = %start
  %9 = bitcast { [0 x i8]*, i64 }* %name_c_str1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %9)
  %_18.sroa.4.0..sroa_idx79108 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1
  %_18.sroa.4.0..sroa_cast109 = bitcast [2 x i64]* %_18.sroa.4.0..sroa_idx79108 to [0 x i8]**
  %_18.sroa.4.0.copyload110 = load [0 x i8]*, [0 x i8]** %_18.sroa.4.0..sroa_cast109, align 8, !nonnull !85
  %_18.sroa.6.0..sroa_idx81111 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1, i64 1
  %_18.sroa.6.0.copyload112 = load i64, i64* %_18.sroa.6.0..sroa_idx81111, align 8
  %.fca.0.gep = getelementptr inbounds { [0 x i8]*, i64 }, { [0 x i8]*, i64 }* %name_c_str1, i64 0, i32 0
  store [0 x i8]* %_18.sroa.4.0.copyload110, [0 x i8]** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { [0 x i8]*, i64 }, { [0 x i8]*, i64 }* %name_c_str1, i64 0, i32 1
  store i64 %_18.sroa.6.0.copyload112, i64* %.fca.1.gep, align 8
  %10 = bitcast { i64*, i8 }* %guard to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %10)
  %11 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %11)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i, align 8
  %12 = load atomic i64, i64* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to i64*) acquire, align 8, !noalias !423
  %13 = icmp eq i64 %12, 2
  br i1 %13, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit"
  %14 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h1ed77e854a4795c8E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %14)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit": ; preds = %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit", %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %11)
  %15 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !428
  %16 = extractvalue { i32, i1 } %15, 1
  br i1 %16, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !428
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
  %17 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !431
  %_1.i.i.i.i.i.i = and i64 %17, 9223372036854775807
  %18 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %18, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %19 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !431
  %phi.bo.i.i.i.i.i = xor i1 %19, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %20 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !431
  %.not118 = icmp eq i8 %20, 0
  br i1 %.not118, label %bb18, label %bb1.i31

bb1.i31:                                          ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %21 = bitcast { i64*, i8 }* %e.i26 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %21), !noalias !434
  %22 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i26, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %22, align 8, !noalias !434
  %23 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i26, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %23, align 8, !noalias !434
  %_6.0.i30 = bitcast { i64*, i8 }* %e.i26 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i30, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc468 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !434

cleanup.i:                                        ; preds = %bb1.i31
  %24 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i26) #25
          to label %common.resume unwind label %abort.i, !noalias !434

unreachable.i:                                    ; preds = %bb1.i31
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %25 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !434
  unreachable

common.resume:                                    ; preds = %bb30, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %24, %cleanup.i ], [ %.pn23, %bb30 ]
  resume { i8*, i32 } %common.resume.op

bb30:                                             ; preds = %cleanup.i52, %cleanup, %bb29
  %.pn23 = phi { i8*, i32 } [ %.pn, %bb29 ], [ %26, %cleanup ], [ %63, %cleanup.i52 ]
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %guard) #25
          to label %common.resume unwind label %abort

cleanup:                                          ; preds = %bb2.i.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb3.i.i.i.i.i.i41, %bb3.i.i.i36, %bb3.i.i.i.i33, %bb18
  %26 = landingpad { i8*, i32 }
          cleanup
  br label %bb30

bb18:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %.fca.0.gep5 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep5, align 8
  %.fca.1.gep7 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep7, align 8
  %27 = bitcast { i64*, i8 }* %guard to %"std::sync::mutex::Mutex<i64>"**
  %28 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 8) to i64*), align 8
  %29 = add i64 %28, 1
  store i64 %29, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 8) to i64*), align 8
  store i64 %29, i64* %objid, align 8
  %30 = bitcast %"core::fmt::Arguments"* %_30 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %30)
  %31 = bitcast [3 x { i8*, i64* }]* %_37 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %31)
  %32 = bitcast i64* %_44 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %32)
  %33 = ptrtoint i8* %address to i64
  store i64 %33, i64* %_44, align 8
  %34 = bitcast [3 x { i8*, i64* }]* %_37 to i64**
  store i64* %objid, i64** %34, align 8
  %35 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_37, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %35, align 8
  %36 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_37, i64 0, i64 1, i32 0
  %37 = bitcast i8** %36 to i64**
  store i64* %_44, i64** %37, align 8
  %38 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_37, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17h0a1d23de10af675eE" to i64*), i64** %38, align 8
  %39 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_37, i64 0, i64 2, i32 0
  %40 = bitcast i8** %39 to { [0 x i8]*, i64 }**
  store { [0 x i8]*, i64 }* %name_c_str1, { [0 x i8]*, i64 }** %40, align 8
  %41 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_37, i64 0, i64 2, i32 1
  store i64* bitcast (i1 ({ [0 x i8]*, i64 }*, %"core::fmt::Formatter"*)* @"_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h959a2441bf3a547eE" to i64*), i64** %41, align 8
  %42 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc76 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %42, align 8, !alias.scope !437, !noalias !440
  %43 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 0, i32 1
  store i64 4, i64* %43, align 8, !alias.scope !437, !noalias !440
  %44 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 1, i32 0
  store i64* bitcast (<{ [168 x i8] }>* @alloc96 to i64*), i64** %44, align 8, !alias.scope !437, !noalias !440
  %45 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 1, i32 1
  store i64 3, i64* %45, align 8, !alias.scope !437, !noalias !440
  %46 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 2, i32 0
  %47 = bitcast [0 x { i8*, i64* }]** %46 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_37, [3 x { i8*, i64* }]** %47, align 8, !alias.scope !437, !noalias !440
  %48 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 2, i32 1
  store i64 3, i64* %48, align 8, !alias.scope !437, !noalias !440
; invoke std::io::stdio::_print
  invoke void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_30)
          to label %bb19 unwind label %cleanup

bb19:                                             ; preds = %bb18
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %30)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %32)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %31)
  %49 = bitcast { i64*, i8 }* %object_table to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %49)
  %50 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i32 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %50)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i32, align 8
  %51 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to i64*) acquire, align 8, !noalias !444
  %52 = icmp eq i64 %51, 2
  br i1 %52, label %bb20, label %bb3.i.i.i.i33

bb3.i.i.i.i33:                                    ; preds = %bb19
  %53 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i32 to i64*
; invoke once_cell::imp::OnceCell<T>::initialize
  invoke fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %53)
          to label %bb20 unwind label %cleanup

bb20:                                             ; preds = %bb19, %bb3.i.i.i.i33
  %_6.i.i.i.i.i.i.i34 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i35 = icmp ne i64 %_6.i.i.i.i.i.i.i34, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i35) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %50)
  %54 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !449
  %55 = extractvalue { i32, i1 } %54, 1
  br i1 %55, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i38, label %bb3.i.i.i36

bb3.i.i.i36:                                      ; preds = %bb20
; invoke std::sys::unix::locks::futex::Mutex::lock_contended
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i38 unwind label %cleanup

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i38: ; preds = %bb3.i.i.i36, %bb20
  %56 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !452
  %_1.i.i.i.i.i.i37 = and i64 %56, 9223372036854775807
  %57 = icmp eq i64 %_1.i.i.i.i.i.i37, 0
  br i1 %57, label %bb21, label %bb3.i.i.i.i.i.i41

bb3.i.i.i.i.i.i41:                                ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i38
; invoke std::panicking::panic_count::is_zero_slow_path
  %58 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc45 unwind label %cleanup

.noexc45:                                         ; preds = %bb3.i.i.i.i.i.i41
  %phi.bo.i.i.i.i.i39 = xor i1 %58, true
  %phi.cast.i.i.i40 = zext i1 %phi.bo.i.i.i.i.i39 to i8
  br label %bb21

bb21:                                             ; preds = %.noexc45, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i38
  %.0.i.i.i.i.i.i42 = phi i8 [ %phi.cast.i.i.i40, %.noexc45 ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i38 ]
  %59 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !452
  %.not119 = icmp eq i8 %59, 0
  br i1 %.not119, label %bb22, label %bb1.i51

bb1.i51:                                          ; preds = %bb21
  %60 = bitcast { i64*, i8 }* %e.i46 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %60), !noalias !455
  %61 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i46, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %61, align 8, !noalias !455
  %62 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i46, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i42, i8* %62, align 8, !noalias !455
  %_6.0.i50 = bitcast { i64*, i8 }* %e.i46 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i50, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc470 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i53 unwind label %cleanup.i52, !noalias !459

cleanup.i52:                                      ; preds = %bb1.i51
  %63 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i46) #25
          to label %bb30 unwind label %abort.i54, !noalias !459

unreachable.i53:                                  ; preds = %bb1.i51
  unreachable

abort.i54:                                        ; preds = %cleanup.i52
  %64 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !459
  unreachable

bb22:                                             ; preds = %bb21
  %.fca.0.gep9 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep9, align 8
  %.fca.1.gep11 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i42, i8* %.fca.1.gep11, align 8
  %_60 = load i64, i64* %objid, align 8
  %_64.0 = load [0 x i8]*, [0 x i8]** %.fca.0.gep, align 8, !nonnull !85, !align !90, !noundef !85
  %_64.1 = load i64, i64* %.fca.1.gep, align 8
  %_6.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_64.1, 0
  br i1 %_6.i.i.i.i.i.i.i.i.i.i, label %bb24, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i": ; preds = %bb22
  %65 = call align 1 i8* @__rust_alloc(i64 %_64.1, i64 1) #24, !noalias !460
  %66 = icmp eq i8* %65, null
  br i1 %66, label %bb23.i.i.i.i.i.i.i.i.i.i, label %bb24

bb23.i.i.i.i.i.i.i.i.i.i:                         ; preds = %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i"
; invoke alloc::alloc::handle_alloc_error
  invoke void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64 %_64.1, i64 noundef 1) #23
          to label %.noexc56 unwind label %cleanup2

.noexc56:                                         ; preds = %bb23.i.i.i.i.i.i.i.i.i.i
  unreachable

bb29:                                             ; preds = %bb21.i.i.i, %cleanup2
  %.pn = phi { i8*, i32 } [ %67, %cleanup2 ], [ %205, %bb21.i.i.i ]
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %object_table) #25
          to label %bb30 unwind label %abort

cleanup2:                                         ; preds = %bb23.i.i.i.i.i.i.i.i.i.i
  %67 = landingpad { i8*, i32 }
          cleanup
  br label %bb29

bb24:                                             ; preds = %bb22, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i"
  %.sroa.0.0.i.i.i.i.i.i.i.i.i.i = phi i8* [ inttoptr (i64 1 to i8*), %bb22 ], [ %65, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i" ]
  %68 = getelementptr [0 x i8], [0 x i8]* %_64.0, i64 0, i64 0
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* nonnull align 1 %.sroa.0.0.i.i.i.i.i.i.i.i.i.i, i8* nonnull align 1 %68, i64 %_64.1, i1 false) #24, !noalias !484
  call void @llvm.experimental.noalias.scope.decl(metadata !485)
  call void @llvm.experimental.noalias.scope.decl(metadata !488)
  %_6.idx.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !491, !noalias !492
  %_6.idx11.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !491, !noalias !492
  %69 = xor i64 %_6.idx.val.i.i, 8317987319222330741
  %70 = xor i64 %_6.idx11.val.i.i, 7237128888997146477
  %71 = xor i64 %_6.idx.val.i.i, 7816392313619706465
  %72 = xor i64 %_60, %_6.idx11.val.i.i
  %73 = xor i64 %72, 8387220255154660723
  %74 = add i64 %70, %69
  %75 = call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 13) #24
  %76 = xor i64 %74, %75
  %77 = call i64 @llvm.fshl.i64(i64 %74, i64 %74, i64 32) #24
  %78 = add i64 %73, %71
  %79 = call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 16) #24
  %80 = xor i64 %79, %78
  %81 = add i64 %80, %77
  %82 = call i64 @llvm.fshl.i64(i64 %80, i64 %80, i64 21) #24
  %83 = xor i64 %82, %81
  %84 = add i64 %76, %78
  %85 = call i64 @llvm.fshl.i64(i64 %76, i64 %76, i64 17) #24
  %86 = xor i64 %84, %85
  %87 = call i64 @llvm.fshl.i64(i64 %84, i64 %84, i64 32) #24
  %88 = xor i64 %81, %_60
  %89 = xor i64 %83, 576460752303423488
  %90 = add i64 %88, %86
  %91 = call i64 @llvm.fshl.i64(i64 %86, i64 %86, i64 13) #24
  %92 = xor i64 %90, %91
  %93 = call i64 @llvm.fshl.i64(i64 %90, i64 %90, i64 32) #24
  %94 = add i64 %89, %87
  %95 = call i64 @llvm.fshl.i64(i64 %83, i64 %89, i64 16) #24
  %96 = xor i64 %95, %94
  %97 = add i64 %96, %93
  %98 = call i64 @llvm.fshl.i64(i64 %96, i64 %96, i64 21) #24
  %99 = xor i64 %98, %97
  %100 = add i64 %94, %92
  %101 = call i64 @llvm.fshl.i64(i64 %92, i64 %92, i64 17) #24
  %102 = xor i64 %100, %101
  %103 = call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 32) #24
  %104 = xor i64 %97, 576460752303423488
  %105 = xor i64 %103, 255
  %106 = add i64 %104, %102
  %107 = call i64 @llvm.fshl.i64(i64 %102, i64 %102, i64 13) #24
  %108 = xor i64 %106, %107
  %109 = call i64 @llvm.fshl.i64(i64 %106, i64 %106, i64 32) #24
  %110 = add i64 %99, %105
  %111 = call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 16) #24
  %112 = xor i64 %111, %110
  %113 = add i64 %112, %109
  %114 = call i64 @llvm.fshl.i64(i64 %112, i64 %112, i64 21) #24
  %115 = xor i64 %114, %113
  %116 = add i64 %108, %110
  %117 = call i64 @llvm.fshl.i64(i64 %108, i64 %108, i64 17) #24
  %118 = xor i64 %116, %117
  %119 = call i64 @llvm.fshl.i64(i64 %116, i64 %116, i64 32) #24
  %120 = add i64 %118, %113
  %121 = call i64 @llvm.fshl.i64(i64 %118, i64 %118, i64 13) #24
  %122 = xor i64 %121, %120
  %123 = call i64 @llvm.fshl.i64(i64 %120, i64 %120, i64 32) #24
  %124 = add i64 %115, %119
  %125 = call i64 @llvm.fshl.i64(i64 %115, i64 %115, i64 16) #24
  %126 = xor i64 %125, %124
  %127 = add i64 %126, %123
  %128 = call i64 @llvm.fshl.i64(i64 %126, i64 %126, i64 21) #24
  %129 = xor i64 %128, %127
  %130 = add i64 %122, %124
  %131 = call i64 @llvm.fshl.i64(i64 %122, i64 %122, i64 17) #24
  %132 = xor i64 %131, %130
  %133 = call i64 @llvm.fshl.i64(i64 %130, i64 %130, i64 32) #24
  %134 = add i64 %132, %127
  %135 = call i64 @llvm.fshl.i64(i64 %132, i64 %132, i64 13) #24
  %136 = xor i64 %135, %134
  %137 = add i64 %129, %133
  %138 = call i64 @llvm.fshl.i64(i64 %129, i64 %129, i64 16) #24
  %139 = xor i64 %138, %137
  %140 = call i64 @llvm.fshl.i64(i64 %139, i64 %139, i64 21) #24
  %141 = add i64 %136, %137
  %142 = call i64 @llvm.fshl.i64(i64 %136, i64 %136, i64 17) #24
  %143 = call i64 @llvm.fshl.i64(i64 %141, i64 %141, i64 32) #24
  %_17.i.i.i.i.i.i.i = xor i64 %141, %140
  %144 = xor i64 %_17.i.i.i.i.i.i.i, %142
  %145 = xor i64 %144, %143
  call void @llvm.experimental.noalias.scope.decl(metadata !497)
  call void @llvm.experimental.noalias.scope.decl(metadata !500) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !503) #24
  %top7.i.i.i.i.i.i = lshr i64 %145, 57
  %146 = trunc i64 %top7.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !506, !noalias !509
  %_3.i.i.i.i.i.i = and i64 %145, %_6.i.i.i.i.i.i
  %self.idx.val.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !511, !noalias !509
  %.0.vec.insert.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %146, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb21.i.i.i.i.i, %bb24
  %probe_seq.sroa.7.0.i.i.i.i.i = phi i64 [ 0, %bb24 ], [ %159, %bb21.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb24 ], [ %161, %bb21.i.i.i.i.i ]
  %147 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i
  %148 = bitcast i8* %147 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i = load <16 x i8>, <16 x i8>* %148, align 1, !noalias !512
  %149 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i
  %150 = bitcast <16 x i1> %149 to i16
  br label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb10.i.i.i.i.i, %bb3.i.i.i.i.i
  %iter.0.i.i.i.i.i = phi i16 [ %150, %bb3.i.i.i.i.i ], [ %_2.i.i.i.i.i.i.i, %bb10.i.i.i.i.i ]
  %151 = icmp eq i16 %iter.0.i.i.i.i.i, 0
  br i1 %151, label %bb12.i.i.i.i.i, label %bb10.i.i.i.i.i

bb12.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %152 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %153 = bitcast <16 x i1> %152 to i16
  %.not.i.i.i.i.i = icmp eq i16 %153, 0
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %bb7.i.i

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %154 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %154 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %155 = sub i64 0, %index.i.i.i.i.i
  %156 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %155, i32 0
  %157 = getelementptr inbounds i64, i64* %156, i64 -7
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %157, align 8, !noalias !515
  %158 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %_60
  br i1 %158, label %bb25, label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %159 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %160 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %159
  %161 = and i64 %160, %_6.i.i.i.i.i.i
  br label %bb3.i.i.i.i.i

bb7.i.i:                                          ; preds = %bb12.i.i.i.i.i
  %162 = bitcast { i64, %ObjectInfo }* %_23.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %162), !noalias !518
  %_70.sroa.0.0..sroa_idx = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 0
  store i64 %_60, i64* %_70.sroa.0.0..sroa_idx, align 8, !noalias !519
  %_70.sroa.6.0..sroa_idx142 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 1
  store i64 %33, i64* %_70.sroa.6.0..sroa_idx142, align 8, !noalias !519
  %_70.sroa.7.0..sroa_idx147 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 2
  store i64 1, i64* %_70.sroa.7.0..sroa_idx147, align 8, !noalias !519
  %_70.sroa.8.0..sroa_idx152 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 3, i32 0, i32 0, i32 0
  store i8* %.sroa.0.0.i.i.i.i.i.i.i.i.i.i, i8** %_70.sroa.8.0..sroa_idx152, align 8, !noalias !519
  %_70.sroa.9.0..sroa_idx157 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 3, i32 0, i32 0, i32 1
  store i64 %_64.1, i64* %_70.sroa.9.0..sroa_idx157, align 8, !noalias !519
  %_70.sroa.10.0..sroa_idx162 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 3, i32 0, i32 1
  store i64 %_64.1, i64* %_70.sroa.10.0..sroa_idx162, align 8, !noalias !519
  %163 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 0
  store i64 %_60, i64* %163, align 8, !noalias !518
  call void @llvm.experimental.noalias.scope.decl(metadata !520)
  %164 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_3.i.i.i.i.i.i
  %165 = bitcast i8* %164 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %165, align 1, !noalias !523
  %166 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %167 = bitcast <16 x i1> %166 to i16
  %.not23.i.i.i.i = icmp eq i16 %167, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb17.i.i.i.i, %bb7.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb7.i.i ], [ %173, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %167, %bb7.i.i ], [ %177, %bb17.i.i.i.i ]
  %168 = call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i = zext i16 %168 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_6.i.i.i.i.i.i
  %169 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %169, align 1, !noalias !530
  %170 = icmp sgt i8 %_23.i.i.i.i, -1
  br i1 %170, label %bb11.i.i.i.i, label %bb2.i.i.i

bb17.i.i.i.i:                                     ; preds = %bb7.i.i, %bb17.i.i.i.i
  %probe_seq.sroa.0.025.i.i.i.i = phi i64 [ %173, %bb17.i.i.i.i ], [ %_3.i.i.i.i.i.i, %bb7.i.i ]
  %probe_seq.sroa.7.024.i.i.i.i = phi i64 [ %171, %bb17.i.i.i.i ], [ 0, %bb7.i.i ]
  %171 = add i64 %probe_seq.sroa.7.024.i.i.i.i, 16
  %172 = add i64 %171, %probe_seq.sroa.0.025.i.i.i.i
  %173 = and i64 %172, %_6.i.i.i.i.i.i
  %174 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %173
  %175 = bitcast i8* %174 to <16 x i8>*
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %175, align 1, !noalias !523
  %176 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %177 = bitcast <16 x i1> %176 to i16
  %.not.i.i.i.i = icmp eq i16 %177, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i.i.i
  %178 = bitcast i8* %self.idx.val.i.i.i.i.i to <16 x i8>*
  %179 = load <16 x i8>, <16 x i8>* %178, align 16, !noalias !531
  %180 = icmp slt <16 x i8> %179, zeroinitializer
  %181 = bitcast <16 x i1> %180 to i16
  %182 = call i16 @llvm.cttz.i16(i16 %181, i1 true) #24, !range !27
  %_2.i.i.i.i.i = zext i16 %182 to i64
  %.phi.trans.insert.i.i.i = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_2.i.i.i.i.i
  %old_ctrl.pre.i.i.i = load i8, i8* %.phi.trans.insert.i.i.i, align 1, !noalias !536
  br label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %bb11.i.i.i.i, %bb7.i.i.i.i
  %old_ctrl.i.i.i = phi i8 [ %old_ctrl.pre.i.i.i, %bb11.i.i.i.i ], [ %_23.i.i.i.i, %bb7.i.i.i.i ]
  %.0.i.i.i.i = phi i64 [ %_2.i.i.i.i.i, %bb11.i.i.i.i ], [ %result.i.i.i.i, %bb7.i.i.i.i ]
  %_14.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !537, !noalias !538
  %183 = icmp eq i64 %_14.i.i.i, 0
  %_2.i.i.i.i = and i8 %old_ctrl.i.i.i, 1
  %184 = icmp ne i8 %_2.i.i.i.i, 0
  %or.cond.i.i.i = select i1 %183, i1 %184, i1 false
  br i1 %or.cond.i.i.i, label %bb1.i.i.i.i, label %bb25.thread

bb1.i.i.i.i:                                      ; preds = %bb2.i.i.i
; invoke hashbrown::raw::RawTable<T,A>::reserve_rehash
  %185 = invoke fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h37880d6025255f2aE"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias noundef nonnull align 8 dereferenceable(32) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"*), i64* noalias noundef nonnull readonly align 8 dereferenceable(16) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to i64*))
          to label %bb9.i.i.i unwind label %bb21.i.i.i, !noalias !539

bb9.i.i.i:                                        ; preds = %bb1.i.i.i.i
  %.fca.1.extract.i.i.i.i = extractvalue { i64, i64 } %185, 1
  %.not.i1.i.i.i = icmp eq i64 %.fca.1.extract.i.i.i.i, -9223372036854775807
  call void @llvm.assume(i1 %.not.i1.i.i.i)
  call void @llvm.experimental.noalias.scope.decl(metadata !540)
  %_6.i.i3.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !543, !noalias !538
  %_3.i.i4.i.i.i = and i64 %_6.i.i3.i.i.i, %145
  %self.idx12.val.i6.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !546, !noalias !538
  %186 = getelementptr inbounds i8, i8* %self.idx12.val.i6.i.i.i, i64 %_3.i.i4.i.i.i
  %187 = bitcast i8* %186 to <16 x i8>*
  %.0.copyload.i2122.i7.i.i.i = load <16 x i8>, <16 x i8>* %187, align 1, !noalias !547
  %188 = icmp slt <16 x i8> %.0.copyload.i2122.i7.i.i.i, zeroinitializer
  %189 = bitcast <16 x i1> %188 to i16
  %.not23.i8.i.i.i = icmp eq i16 %189, 0
  br i1 %.not23.i8.i.i.i, label %bb17.i20.i.i.i, label %bb7.i15.i.i.i

bb7.i15.i.i.i:                                    ; preds = %bb17.i20.i.i.i, %bb9.i.i.i
  %probe_seq.sroa.0.0.lcssa.i9.i.i.i = phi i64 [ %_3.i.i4.i.i.i, %bb9.i.i.i ], [ %195, %bb17.i20.i.i.i ]
  %.lcssa.i10.i.i.i = phi i16 [ %189, %bb9.i.i.i ], [ %199, %bb17.i20.i.i.i ]
  %190 = call i16 @llvm.cttz.i16(i16 %.lcssa.i10.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i11.i.i.i = zext i16 %190 to i64
  %_17.i12.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i9.i.i.i, %_2.i.i.i11.i.i.i
  %result.i13.i.i.i = and i64 %_17.i12.i.i.i, %_6.i.i3.i.i.i
  %191 = getelementptr inbounds i8, i8* %self.idx12.val.i6.i.i.i, i64 %result.i13.i.i.i
  %_23.i14.i.i.i = load i8, i8* %191, align 1, !noalias !550
  %192 = icmp sgt i8 %_23.i14.i.i.i, -1
  br i1 %192, label %bb11.i22.i.i.i, label %bb25.thread

bb17.i20.i.i.i:                                   ; preds = %bb9.i.i.i, %bb17.i20.i.i.i
  %probe_seq.sroa.0.025.i16.i.i.i = phi i64 [ %195, %bb17.i20.i.i.i ], [ %_3.i.i4.i.i.i, %bb9.i.i.i ]
  %probe_seq.sroa.7.024.i17.i.i.i = phi i64 [ %193, %bb17.i20.i.i.i ], [ 0, %bb9.i.i.i ]
  %193 = add i64 %probe_seq.sroa.7.024.i17.i.i.i, 16
  %194 = add i64 %193, %probe_seq.sroa.0.025.i16.i.i.i
  %195 = and i64 %194, %_6.i.i3.i.i.i
  %196 = getelementptr inbounds i8, i8* %self.idx12.val.i6.i.i.i, i64 %195
  %197 = bitcast i8* %196 to <16 x i8>*
  %.0.copyload.i21.i18.i.i.i = load <16 x i8>, <16 x i8>* %197, align 1, !noalias !547
  %198 = icmp slt <16 x i8> %.0.copyload.i21.i18.i.i.i, zeroinitializer
  %199 = bitcast <16 x i1> %198 to i16
  %.not.i19.i.i.i = icmp eq i16 %199, 0
  br i1 %.not.i19.i.i.i, label %bb17.i20.i.i.i, label %bb7.i15.i.i.i

bb11.i22.i.i.i:                                   ; preds = %bb7.i15.i.i.i
  %200 = bitcast i8* %self.idx12.val.i6.i.i.i to <16 x i8>*
  %201 = load <16 x i8>, <16 x i8>* %200, align 16, !noalias !551
  %202 = icmp slt <16 x i8> %201, zeroinitializer
  %203 = bitcast <16 x i1> %202 to i16
  %204 = call i16 @llvm.cttz.i16(i16 %203, i1 true) #24, !range !27
  %_2.i.i21.i.i.i = zext i16 %204 to i64
  br label %bb25.thread

bb21.i.i.i:                                       ; preds = %bb1.i.i.i.i
  %205 = landingpad { i8*, i32 }
          cleanup
; call core::ptr::drop_in_place<(i64,fixsanitizer::ObjectInfo)>
  call fastcc void @"_ZN4core3ptr59drop_in_place$LT$$LP$i64$C$fixsanitizer..ObjectInfo$RP$$GT$17h855e18607bcfb813E"({ i64, %ObjectInfo }* nonnull %_23.i.i) #25, !noalias !556
  br label %bb29

bb25.thread:                                      ; preds = %bb2.i.i.i, %bb7.i15.i.i.i, %bb11.i22.i.i.i
  %self.idx1.val.i.i.i.i.i.i = phi i8* [ %self.idx.val.i.i.i.i.i, %bb2.i.i.i ], [ %self.idx12.val.i6.i.i.i, %bb11.i22.i.i.i ], [ %self.idx12.val.i6.i.i.i, %bb7.i15.i.i.i ]
  %_8.i.i.i.i.i.i = phi i64 [ %_6.i.i.i.i.i.i, %bb2.i.i.i ], [ %_6.i.i3.i.i.i, %bb11.i22.i.i.i ], [ %_6.i.i3.i.i.i, %bb7.i15.i.i.i ]
  %index.0.i.i.i = phi i64 [ %.0.i.i.i.i, %bb2.i.i.i ], [ %_2.i.i21.i.i.i, %bb11.i22.i.i.i ], [ %result.i13.i.i.i, %bb7.i15.i.i.i ]
  %self.idx.val28.i.i.i = bitcast i8* %self.idx1.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  call void @llvm.experimental.noalias.scope.decl(metadata !557)
  %sext.i.i.i.i = sub nsw i8 0, %_2.i.i.i.i
  %_5.neg.i.i.i.i = sext i8 %sext.i.i.i.i to i64
  %206 = add i64 %index.0.i.i.i, -16
  %_5.i.i.i.i.i.i = and i64 %206, %_8.i.i.i.i.i.i
  %index2.i.i.i.i.i.i = add i64 %_5.i.i.i.i.i.i, 16
  %207 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index.0.i.i.i
  store i8 %146, i8* %207, align 1, !noalias !560
  %208 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i
  store i8 %146, i8* %208, align 1, !noalias !560
  %209 = load <2 x i64>, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !565, !noalias !538
  %210 = insertelement <2 x i64> <i64 poison, i64 1>, i64 %_5.neg.i.i.i.i, i64 0
  %211 = add <2 x i64> %209, %210
  store <2 x i64> %211, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !565, !noalias !538
  %212 = sub i64 0, %index.0.i.i.i
  %213 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %self.idx.val28.i.i.i, i64 %212, i32 0
  %214 = getelementptr inbounds i64, i64* %213, i64 -7
  %215 = bitcast i64* %214 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %215, i8* noundef nonnull align 8 dereferenceable(56) %162, i64 56, i1 false), !noalias !556
  call void @llvm.lifetime.end.p0i8(i64 56, i8* nonnull %162), !noalias !518
  br label %bb26

bb25:                                             ; preds = %bb10.i.i.i.i.i
  %_65.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx100 = getelementptr inbounds i64, i64* %156, i64 -3
  %_65.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast = bitcast i64* %_65.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx100 to {}**
  %_65.sroa.3.0.copyload = load {}*, {}** %_65.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast, align 8, !noalias !566
  %_65.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx102 = getelementptr inbounds i64, i64* %156, i64 -2
  %_65.sroa.5.0.copyload = load i64, i64* %_65.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx102, align 8, !noalias !566
  %_70.sroa.0.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx = getelementptr inbounds i64, i64* %156, i64 -6
  store i64 %_60, i64* %_70.sroa.0.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx, align 8, !noalias !519
  %_70.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx143 = getelementptr inbounds i64, i64* %156, i64 -5
  store i64 %33, i64* %_70.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx143, align 8, !noalias !519
  %_70.sroa.7.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx148 = getelementptr inbounds i64, i64* %156, i64 -4
  store i64 1, i64* %_70.sroa.7.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx148, align 8, !noalias !519
  %_70.sroa.8.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast = bitcast i64* %_65.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx100 to i8**
  store i8* %.sroa.0.0.i.i.i.i.i.i.i.i.i.i, i8** %_70.sroa.8.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast, align 8, !noalias !519
  store i64 %_64.1, i64* %_65.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx102, align 8, !noalias !519
  %_70.sroa.10.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx163 = getelementptr inbounds i64, i64* %156, i64 -1
  store i64 %_64.1, i64* %_70.sroa.10.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx163, align 8, !noalias !519
  %216 = icmp eq {}* %_65.sroa.3.0.copyload, null
  %_4.i.i.i.i.i.i.i60 = icmp eq i64 %_65.sroa.5.0.copyload, 0
  %or.cond = select i1 %216, i1 true, i1 %_4.i.i.i.i.i.i.i60
  br i1 %or.cond, label %bb26, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i": ; preds = %bb25
  %217 = bitcast {}* %_65.sroa.3.0.copyload to i8*
  call void @__rust_dealloc(i8* nonnull %217, i64 %_65.sroa.5.0.copyload, i64 1) #24
  br label %bb26

bb26:                                             ; preds = %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i", %bb25, %bb25.thread
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i42, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i62, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i62:                                      ; preds = %bb26
  %218 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !567
  %_1.i.i.i.i.i.i61 = and i64 %218, 9223372036854775807
  %219 = icmp eq i64 %_1.i.i.i.i.i.i61, 0
  br i1 %219, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i62
; invoke std::panicking::panic_count::is_zero_slow_path
  %220 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc63 unwind label %cleanup

.noexc63:                                         ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  br i1 %220, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %.noexc63
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !567
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %.noexc63, %bb2.i.i.i62, %bb26
  %221 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !567
  %222 = icmp eq i32 %221, 2
  br i1 %222, label %bb2.i.i.i.i, label %bb27

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; invoke std::sys::unix::locks::futex::Mutex::wake
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %bb27 unwind label %cleanup

abort:                                            ; preds = %bb30, %bb29
  %223 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26
  unreachable

bb27:                                             ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %49)
  call void @llvm.experimental.noalias.scope.decl(metadata !570)
  %_8.i.i65 = load %"std::sync::mutex::Mutex<i64>"*, %"std::sync::mutex::Mutex<i64>"** %27, align 8, !alias.scope !570, !nonnull !85, !align !86, !noundef !85
  %_5.val.i.i67 = load i8, i8* %.fca.1.gep7, align 8, !alias.scope !570
  %_5.not.i.i.i68 = icmp eq i8 %_5.val.i.i67, 0
  br i1 %_5.not.i.i.i68, label %bb2.i.i.i70, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75

bb2.i.i.i70:                                      ; preds = %bb27
  %224 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !570
  %_1.i.i.i.i.i.i69 = and i64 %224, 9223372036854775807
  %225 = icmp eq i64 %_1.i.i.i.i.i.i69, 0
  br i1 %225, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i71

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i71: ; preds = %bb2.i.i.i70
; call std::panicking::panic_count::is_zero_slow_path
  %226 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !570
  br i1 %226, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75, label %bb5.i.i.i73

bb5.i.i.i73:                                      ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i71
  %_6.i.i.i.i72 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i65, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i72 monotonic, align 4, !noalias !570
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75: ; preds = %bb5.i.i.i73, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i71, %bb2.i.i.i70, %bb27
  %_5.i.i.i.i.i74 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i65, i64 0, i32 0, i32 0, i32 0, i32 0
  %227 = atomicrmw xchg i32* %_5.i.i.i.i.i74, i32 0 release, align 4, !noalias !570
  %228 = icmp eq i32 %227, 2
  br i1 %228, label %bb2.i.i.i.i77, label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

bb2.i.i.i.i77:                                    ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75
  %_2.i.i.i76 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i65, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i76), !noalias !570
  br label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i75, %bb2.i.i.i.i77
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %10)
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %9)
  call void @llvm.lifetime.end.p0i8(i64 24, i8* nonnull %0)
  %229 = load i64, i64* %objid, align 8
  ret i64 %229
}

; Function Attrs: nonlazybind uwtable
define void @report_retain(i8* %address, i64 %0, i64 %1) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %_113 = alloca [3 x { i8*, i64* }], align 8
  %_105 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %_79 = alloca [1 x { i8*, i64* }], align 8
  %_72 = alloca %"core::fmt::Arguments", align 8
  %object_table = alloca { i64*, i8 }, align 8
  %_55 = alloca [1 x { i8*, i64* }], align 8
  %_47 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %_27 = alloca i64, align 8
  %_22 = alloca i64, align 8
  %_12 = alloca [4 x { i8*, i64* }], align 8
  %_5 = alloca %"core::fmt::Arguments", align 8
  %refcnt = alloca i64, align 8
  %obj_id = alloca i64, align 8
  store i64 %0, i64* %obj_id, align 8
  store i64 %1, i64* %refcnt, align 8
  %2 = bitcast %"core::fmt::Arguments"* %_5 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %2)
  %3 = bitcast [4 x { i8*, i64* }]* %_12 to i8*
  call void @llvm.lifetime.start.p0i8(i64 64, i8* nonnull %3)
  %4 = bitcast i64* %_22 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %4)
  %5 = add i64 %1, 1
  store i64 %5, i64* %_22, align 8
  %6 = bitcast i64* %_27 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %6)
  %7 = ptrtoint i8* %address to i64
  store i64 %7, i64* %_27, align 8
  %8 = bitcast [4 x { i8*, i64* }]* %_12 to i64**
  store i64* %obj_id, i64** %8, align 8
  %9 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %9, align 8
  %10 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 1, i32 0
  %11 = bitcast i8** %10 to i64**
  store i64* %refcnt, i64** %11, align 8
  %12 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %12, align 8
  %13 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 2, i32 0
  %14 = bitcast i8** %13 to i64**
  store i64* %_22, i64** %14, align 8
  %15 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %15, align 8
  %16 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 3, i32 0
  %17 = bitcast i8** %16 to i64**
  store i64* %_27, i64** %17, align 8
  %18 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 3, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17h0a1d23de10af675eE" to i64*), i64** %18, align 8
  %19 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc161 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %19, align 8, !alias.scope !573, !noalias !576
  %20 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 0, i32 1
  store i64 5, i64* %20, align 8, !alias.scope !573, !noalias !576
  %21 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 0
  store i64* bitcast (<{ [224 x i8] }>* @alloc247 to i64*), i64** %21, align 8, !alias.scope !573, !noalias !576
  %22 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 1
  store i64 4, i64* %22, align 8, !alias.scope !573, !noalias !576
  %23 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 0
  %24 = bitcast [0 x { i8*, i64* }]** %23 to [4 x { i8*, i64* }]**
  store [4 x { i8*, i64* }]* %_12, [4 x { i8*, i64* }]** %24, align 8, !alias.scope !573, !noalias !576
  %25 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 1
  store i64 4, i64* %25, align 8, !alias.scope !573, !noalias !576
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_5)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %2)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %6)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %4)
  call void @llvm.lifetime.end.p0i8(i64 64, i8* nonnull %3)
  %_39 = load i64, i64* %refcnt, align 8
  %_38 = icmp eq i64 %_39, 0
  br i1 %_38, label %bb8, label %bb11

bb11:                                             ; preds = %start
  %26 = bitcast { i64*, i8 }* %object_table to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %26)
  %27 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %27)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %28 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to i64*) acquire, align 8, !noalias !580
  %29 = icmp eq i64 %28, 2
  br i1 %29, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb11
  %30 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %30)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit": ; preds = %bb11, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %27)
  %31 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !585
  %32 = extractvalue { i32, i1 } %31, 1
  br i1 %32, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !585
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
  %33 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !588
  %_1.i.i.i.i.i.i = and i64 %33, 9223372036854775807
  %34 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %34, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %35 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !588
  %phi.bo.i.i.i.i.i = xor i1 %35, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %36 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !588
  %.not = icmp eq i8 %36, 0
  br i1 %.not, label %bb15, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %37 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %37), !noalias !591
  %38 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %38, align 8, !noalias !591
  %39 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %39, align 8, !noalias !591
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc474 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !595

cleanup.i:                                        ; preds = %bb1.i
  %40 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i) #25
          to label %common.resume unwind label %abort.i, !noalias !595

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %41 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !595
  unreachable

common.resume:                                    ; preds = %cleanup, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %40, %cleanup.i ], [ %47, %cleanup ]
  resume { i8*, i32 } %common.resume.op

bb8:                                              ; preds = %start
  %42 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_47 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %42)
  %43 = bitcast [1 x { i8*, i64* }]* %_55 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %43)
  %44 = bitcast [1 x { i8*, i64* }]* %_55 to i64**
  store i64* %obj_id, i64** %44, align 8
  %45 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_55, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %45, align 8
  %_48.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_47 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc255 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_48.sroa.0.0..sroa_cast, align 8
  %_48.sroa.4.0..sroa_idx24 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 0
  store i64 2, i64* %_48.sroa.4.0..sroa_idx24, align 8
  %_48.sroa.5.0..sroa_idx26 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 1
  %_48.sroa.5.0..sroa_cast = bitcast i64* %_48.sroa.5.0..sroa_idx26 to i64**
  store i64* null, i64** %_48.sroa.5.0..sroa_cast, align 8
  %_48.sroa.630.0..sroa_idx31 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 3
  %46 = bitcast i64* %_48.sroa.630.0..sroa_idx31 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_55, [1 x { i8*, i64* }]** %46, align 8
  %_48.sroa.7.0..sroa_idx33 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 4
  store i64 1, i64* %_48.sroa.7.0..sroa_idx33, align 8
; call core::panicking::assert_failed
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc249 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_47, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc472 to %"core::panic::location::Location"*)) #23
  unreachable

cleanup:                                          ; preds = %bb1.i21, %bb28, %bb19
  %47 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %object_table) #25
          to label %common.resume unwind label %abort

bb15:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !596
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_66 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h9ac6fd78d11cfe13E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  br i1 %_66, label %bb21, label %bb19

bb19:                                             ; preds = %bb15
  %48 = bitcast %"core::fmt::Arguments"* %_72 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %48)
  %49 = bitcast [1 x { i8*, i64* }]* %_79 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %49)
  %50 = bitcast [1 x { i8*, i64* }]* %_79 to i64**
  store i64* %obj_id, i64** %50, align 8
  %51 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_79, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %51, align 8
  %52 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc200 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %52, align 8, !alias.scope !599, !noalias !602
  %53 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 0, i32 1
  store i64 2, i64* %53, align 8, !alias.scope !599, !noalias !602
  %54 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 1, i32 0
  store i64* null, i64** %54, align 8, !alias.scope !599, !noalias !602
  %55 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 0
  %56 = bitcast [0 x { i8*, i64* }]** %55 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_79, [1 x { i8*, i64* }]** %56, align 8, !alias.scope !599, !noalias !602
  %57 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 1
  store i64 1, i64* %57, align 8, !alias.scope !599, !noalias !602
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_72, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc476 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb28, %bb19
  unreachable

bb21:                                             ; preds = %bb15
  %obj_id.val19 = load i64, i64* %obj_id, align 8, !alias.scope !596
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_84 = call fastcc noundef align 8 dereferenceable_or_null(48) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h1fd66babc11d9351E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val19)
  %58 = icmp eq i64* %_84, null
  br i1 %58, label %bb1.i21, label %bb23

bb1.i21:                                          ; preds = %bb21
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc406 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc478 to %"core::panic::location::Location"*)) #23
          to label %.noexc unwind label %cleanup

.noexc:                                           ; preds = %bb1.i21
  unreachable

bb23:                                             ; preds = %bb21
  %59 = getelementptr inbounds i64, i64* %_84, i64 2
  %_97 = load i64, i64* %59, align 8
  %_98 = load i64, i64* %refcnt, align 8
  %_96.not = icmp eq i64 %_97, %_98
  br i1 %_96.not, label %bb29, label %bb28

bb29:                                             ; preds = %bb23
  %60 = add i64 %_97, 1
  store i64 %60, i64* %59, align 8
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb29
  %61 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !605
  %_1.i.i.i.i.i.i22 = and i64 %61, 9223372036854775807
  %62 = icmp eq i64 %_1.i.i.i.i.i.i22, 0
  br i1 %62, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %63 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !605
  br i1 %63, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !605
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb29
  %64 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !605
  %65 = icmp eq i32 %64, 2
  br i1 %65, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !605
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %26)
  ret void

bb28:                                             ; preds = %bb23
  %66 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_105 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %66)
  %67 = bitcast [3 x { i8*, i64* }]* %_113 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %67)
  %68 = bitcast [3 x { i8*, i64* }]* %_113 to i64**
  store i64* %obj_id, i64** %68, align 8
  %69 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %69, align 8
  %70 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 1, i32 0
  %71 = bitcast i8** %70 to i64**
  store i64* %refcnt, i64** %71, align 8
  %72 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %72, align 8
  %73 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 2, i32 0
  %74 = bitcast i8** %73 to i64**
  store i64* %59, i64** %74, align 8
  %75 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %75, align 8
  %_106.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_105 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc207 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_106.sroa.0.0..sroa_cast, align 8
  %_106.sroa.4.0..sroa_idx40 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 0
  store i64 3, i64* %_106.sroa.4.0..sroa_idx40, align 8
  %_106.sroa.5.0..sroa_idx42 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 1
  %_106.sroa.5.0..sroa_cast = bitcast i64* %_106.sroa.5.0..sroa_idx42 to i64**
  store i64* null, i64** %_106.sroa.5.0..sroa_cast, align 8
  %_106.sroa.646.0..sroa_idx47 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 3
  %76 = bitcast i64* %_106.sroa.646.0..sroa_idx47 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_113, [3 x { i8*, i64* }]** %76, align 8
  %_106.sroa.7.0..sroa_idx49 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 4
  store i64 3, i64* %_106.sroa.7.0..sroa_idx49, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %59, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_105, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc480 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

abort:                                            ; preds = %cleanup
  %77 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26
  unreachable
}

; Function Attrs: nonlazybind uwtable
define void @report_release(i8* %address, i64 %0, i64 %1) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %_113 = alloca [3 x { i8*, i64* }], align 8
  %_105 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %_79 = alloca [1 x { i8*, i64* }], align 8
  %_72 = alloca %"core::fmt::Arguments", align 8
  %object_info = alloca { i64*, i8 }, align 8
  %_55 = alloca [1 x { i8*, i64* }], align 8
  %_47 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %_27 = alloca i64, align 8
  %_22 = alloca i64, align 8
  %_12 = alloca [4 x { i8*, i64* }], align 8
  %_5 = alloca %"core::fmt::Arguments", align 8
  %refcnt = alloca i64, align 8
  %obj_id = alloca i64, align 8
  store i64 %0, i64* %obj_id, align 8
  store i64 %1, i64* %refcnt, align 8
  %2 = bitcast %"core::fmt::Arguments"* %_5 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %2)
  %3 = bitcast [4 x { i8*, i64* }]* %_12 to i8*
  call void @llvm.lifetime.start.p0i8(i64 64, i8* nonnull %3)
  %4 = bitcast i64* %_22 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %4)
  %5 = add i64 %1, -1
  store i64 %5, i64* %_22, align 8
  %6 = bitcast i64* %_27 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %6)
  %7 = ptrtoint i8* %address to i64
  store i64 %7, i64* %_27, align 8
  %8 = bitcast [4 x { i8*, i64* }]* %_12 to i64**
  store i64* %obj_id, i64** %8, align 8
  %9 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %9, align 8
  %10 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 1, i32 0
  %11 = bitcast i8** %10 to i64**
  store i64* %refcnt, i64** %11, align 8
  %12 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %12, align 8
  %13 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 2, i32 0
  %14 = bitcast i8** %13 to i64**
  store i64* %_22, i64** %14, align 8
  %15 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %15, align 8
  %16 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 3, i32 0
  %17 = bitcast i8** %16 to i64**
  store i64* %_27, i64** %17, align 8
  %18 = getelementptr inbounds [4 x { i8*, i64* }], [4 x { i8*, i64* }]* %_12, i64 0, i64 3, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17h0a1d23de10af675eE" to i64*), i64** %18, align 8
  %19 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc221 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %19, align 8, !alias.scope !608, !noalias !611
  %20 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 0, i32 1
  store i64 5, i64* %20, align 8, !alias.scope !608, !noalias !611
  %21 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 0
  store i64* bitcast (<{ [224 x i8] }>* @alloc247 to i64*), i64** %21, align 8, !alias.scope !608, !noalias !611
  %22 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 1
  store i64 4, i64* %22, align 8, !alias.scope !608, !noalias !611
  %23 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 0
  %24 = bitcast [0 x { i8*, i64* }]** %23 to [4 x { i8*, i64* }]**
  store [4 x { i8*, i64* }]* %_12, [4 x { i8*, i64* }]** %24, align 8, !alias.scope !608, !noalias !611
  %25 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 1
  store i64 4, i64* %25, align 8, !alias.scope !608, !noalias !611
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_5)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %2)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %6)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %4)
  call void @llvm.lifetime.end.p0i8(i64 64, i8* nonnull %3)
  %_39 = load i64, i64* %refcnt, align 8
  %_38 = icmp eq i64 %_39, 0
  br i1 %_38, label %bb8, label %bb11

bb11:                                             ; preds = %start
  %26 = bitcast { i64*, i8 }* %object_info to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %26)
  %27 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %27)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %28 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to i64*) acquire, align 8, !noalias !615
  %29 = icmp eq i64 %28, 2
  br i1 %29, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb11
  %30 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %30)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit": ; preds = %bb11, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %27)
  %31 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !620
  %32 = extractvalue { i32, i1 } %31, 1
  br i1 %32, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !620
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
  %33 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !623
  %_1.i.i.i.i.i.i = and i64 %33, 9223372036854775807
  %34 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %34, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %35 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !623
  %phi.bo.i.i.i.i.i = xor i1 %35, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %36 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !623
  %.not = icmp eq i8 %36, 0
  br i1 %.not, label %bb15, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %37 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %37), !noalias !626
  %38 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %38, align 8, !noalias !626
  %39 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %39, align 8, !noalias !626
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc484 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !630

cleanup.i:                                        ; preds = %bb1.i
  %40 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i) #25
          to label %common.resume unwind label %abort.i, !noalias !630

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %41 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !630
  unreachable

common.resume:                                    ; preds = %cleanup, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %40, %cleanup.i ], [ %47, %cleanup ]
  resume { i8*, i32 } %common.resume.op

bb8:                                              ; preds = %start
  %42 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_47 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %42)
  %43 = bitcast [1 x { i8*, i64* }]* %_55 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %43)
  %44 = bitcast [1 x { i8*, i64* }]* %_55 to i64**
  store i64* %obj_id, i64** %44, align 8
  %45 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_55, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %45, align 8
  %_48.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_47 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc255 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_48.sroa.0.0..sroa_cast, align 8
  %_48.sroa.4.0..sroa_idx30 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 0
  store i64 2, i64* %_48.sroa.4.0..sroa_idx30, align 8
  %_48.sroa.5.0..sroa_idx32 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 1
  %_48.sroa.5.0..sroa_cast = bitcast i64* %_48.sroa.5.0..sroa_idx32 to i64**
  store i64* null, i64** %_48.sroa.5.0..sroa_cast, align 8
  %_48.sroa.636.0..sroa_idx37 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 3
  %46 = bitcast i64* %_48.sroa.636.0..sroa_idx37 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_55, [1 x { i8*, i64* }]** %46, align 8
  %_48.sroa.7.0..sroa_idx39 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 4
  store i64 1, i64* %_48.sroa.7.0..sroa_idx39, align 8
; call core::panicking::assert_failed
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc249 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_47, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc482 to %"core::panic::location::Location"*)) #23
  unreachable

cleanup:                                          ; preds = %bb1.i22, %bb28, %bb19
  %47 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %object_info) #25
          to label %common.resume unwind label %abort

bb15:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !596
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_66 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h9ac6fd78d11cfe13E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  br i1 %_66, label %bb21, label %bb19

bb19:                                             ; preds = %bb15
  %48 = bitcast %"core::fmt::Arguments"* %_72 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %48)
  %49 = bitcast [1 x { i8*, i64* }]* %_79 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %49)
  %50 = bitcast [1 x { i8*, i64* }]* %_79 to i64**
  store i64* %obj_id, i64** %50, align 8
  %51 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_79, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %51, align 8
  %52 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc260 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %52, align 8, !alias.scope !631, !noalias !634
  %53 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 0, i32 1
  store i64 2, i64* %53, align 8, !alias.scope !631, !noalias !634
  %54 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 1, i32 0
  store i64* null, i64** %54, align 8, !alias.scope !631, !noalias !634
  %55 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 0
  %56 = bitcast [0 x { i8*, i64* }]** %55 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_79, [1 x { i8*, i64* }]** %56, align 8, !alias.scope !631, !noalias !634
  %57 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 1
  store i64 1, i64* %57, align 8, !alias.scope !631, !noalias !634
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_72, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc486 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb28, %bb19
  unreachable

bb21:                                             ; preds = %bb15
  %obj_id.val19 = load i64, i64* %obj_id, align 8, !alias.scope !596
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_84 = call fastcc noundef align 8 dereferenceable_or_null(48) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h1fd66babc11d9351E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val19)
  %58 = icmp eq i64* %_84, null
  br i1 %58, label %bb1.i22, label %bb23

bb1.i22:                                          ; preds = %bb21
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc406 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc488 to %"core::panic::location::Location"*)) #23
          to label %.noexc unwind label %cleanup

.noexc:                                           ; preds = %bb1.i22
  unreachable

bb23:                                             ; preds = %bb21
  %59 = getelementptr inbounds i64, i64* %_84, i64 2
  %_97 = load i64, i64* %59, align 8
  %_98 = load i64, i64* %refcnt, align 8
  %_96.not = icmp eq i64 %_97, %_98
  br i1 %_96.not, label %bb29, label %bb28

bb29:                                             ; preds = %bb23
  %60 = add i64 %_97, -1
  store i64 %60, i64* %59, align 8
  %61 = icmp eq i64 %60, 0
  br i1 %61, label %bb32, label %bb35

bb28:                                             ; preds = %bb23
  %62 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_105 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %62)
  %63 = bitcast [3 x { i8*, i64* }]* %_113 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %63)
  %64 = bitcast [3 x { i8*, i64* }]* %_113 to i64**
  store i64* %obj_id, i64** %64, align 8
  %65 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %65, align 8
  %66 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 1, i32 0
  %67 = bitcast i8** %66 to i64**
  store i64* %refcnt, i64** %67, align 8
  %68 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %68, align 8
  %69 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 2, i32 0
  %70 = bitcast i8** %69 to i64**
  store i64* %59, i64** %70, align 8
  %71 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_113, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %71, align 8
  %_106.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_105 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc267 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_106.sroa.0.0..sroa_cast, align 8
  %_106.sroa.4.0..sroa_idx46 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 0
  store i64 3, i64* %_106.sroa.4.0..sroa_idx46, align 8
  %_106.sroa.5.0..sroa_idx48 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 1
  %_106.sroa.5.0..sroa_cast = bitcast i64* %_106.sroa.5.0..sroa_idx48 to i64**
  store i64* null, i64** %_106.sroa.5.0..sroa_cast, align 8
  %_106.sroa.652.0..sroa_idx53 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 3
  %72 = bitcast i64* %_106.sroa.652.0..sroa_idx53 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_113, [3 x { i8*, i64* }]** %72, align 8
  %_106.sroa.7.0..sroa_idx55 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 4
  store i64 3, i64* %_106.sroa.7.0..sroa_idx55, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %59, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_105, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc490 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

bb35:                                             ; preds = %bb12.i.i.i.i.i.i, %bb2.i, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i", %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i", %bb29
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb35
  %73 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !637
  %_1.i.i.i.i.i.i25 = and i64 %73, 9223372036854775807
  %74 = icmp eq i64 %_1.i.i.i.i.i.i25, 0
  br i1 %74, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %75 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !637
  br i1 %75, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !637
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb35
  %76 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !637
  %77 = icmp eq i32 %76, 2
  br i1 %77, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !637
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %26)
  ret void

bb32:                                             ; preds = %bb29
  %obj_id.val20 = load i64, i64* %obj_id, align 8, !alias.scope !596
  call void @llvm.experimental.noalias.scope.decl(metadata !640)
  call void @llvm.experimental.noalias.scope.decl(metadata !643) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !646) #24
  %_5.idx.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !649, !noalias !650
  %_5.idx1.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !649, !noalias !650
  %78 = xor i64 %_5.idx.val.i.i.i, 8317987319222330741
  %79 = xor i64 %_5.idx1.val.i.i.i, 7237128888997146477
  %80 = xor i64 %_5.idx.val.i.i.i, 7816392313619706465
  %81 = xor i64 %obj_id.val20, %_5.idx1.val.i.i.i
  %82 = xor i64 %81, 8387220255154660723
  %83 = add i64 %79, %78
  %84 = call i64 @llvm.fshl.i64(i64 %79, i64 %79, i64 13) #24
  %85 = xor i64 %83, %84
  %86 = call i64 @llvm.fshl.i64(i64 %83, i64 %83, i64 32) #24
  %87 = add i64 %82, %80
  %88 = call i64 @llvm.fshl.i64(i64 %82, i64 %82, i64 16) #24
  %89 = xor i64 %88, %87
  %90 = add i64 %89, %86
  %91 = call i64 @llvm.fshl.i64(i64 %89, i64 %89, i64 21) #24
  %92 = xor i64 %91, %90
  %93 = add i64 %85, %87
  %94 = call i64 @llvm.fshl.i64(i64 %85, i64 %85, i64 17) #24
  %95 = xor i64 %93, %94
  %96 = call i64 @llvm.fshl.i64(i64 %93, i64 %93, i64 32) #24
  %97 = xor i64 %90, %obj_id.val20
  %98 = xor i64 %92, 576460752303423488
  %99 = add i64 %97, %95
  %100 = call i64 @llvm.fshl.i64(i64 %95, i64 %95, i64 13) #24
  %101 = xor i64 %99, %100
  %102 = call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 32) #24
  %103 = add i64 %98, %96
  %104 = call i64 @llvm.fshl.i64(i64 %92, i64 %98, i64 16) #24
  %105 = xor i64 %104, %103
  %106 = add i64 %105, %102
  %107 = call i64 @llvm.fshl.i64(i64 %105, i64 %105, i64 21) #24
  %108 = xor i64 %107, %106
  %109 = add i64 %103, %101
  %110 = call i64 @llvm.fshl.i64(i64 %101, i64 %101, i64 17) #24
  %111 = xor i64 %109, %110
  %112 = call i64 @llvm.fshl.i64(i64 %109, i64 %109, i64 32) #24
  %113 = xor i64 %106, 576460752303423488
  %114 = xor i64 %112, 255
  %115 = add i64 %113, %111
  %116 = call i64 @llvm.fshl.i64(i64 %111, i64 %111, i64 13) #24
  %117 = xor i64 %115, %116
  %118 = call i64 @llvm.fshl.i64(i64 %115, i64 %115, i64 32) #24
  %119 = add i64 %108, %114
  %120 = call i64 @llvm.fshl.i64(i64 %108, i64 %108, i64 16) #24
  %121 = xor i64 %120, %119
  %122 = add i64 %121, %118
  %123 = call i64 @llvm.fshl.i64(i64 %121, i64 %121, i64 21) #24
  %124 = xor i64 %123, %122
  %125 = add i64 %117, %119
  %126 = call i64 @llvm.fshl.i64(i64 %117, i64 %117, i64 17) #24
  %127 = xor i64 %125, %126
  %128 = call i64 @llvm.fshl.i64(i64 %125, i64 %125, i64 32) #24
  %129 = add i64 %127, %122
  %130 = call i64 @llvm.fshl.i64(i64 %127, i64 %127, i64 13) #24
  %131 = xor i64 %130, %129
  %132 = call i64 @llvm.fshl.i64(i64 %129, i64 %129, i64 32) #24
  %133 = add i64 %124, %128
  %134 = call i64 @llvm.fshl.i64(i64 %124, i64 %124, i64 16) #24
  %135 = xor i64 %134, %133
  %136 = add i64 %135, %132
  %137 = call i64 @llvm.fshl.i64(i64 %135, i64 %135, i64 21) #24
  %138 = xor i64 %137, %136
  %139 = add i64 %131, %133
  %140 = call i64 @llvm.fshl.i64(i64 %131, i64 %131, i64 17) #24
  %141 = xor i64 %140, %139
  %142 = call i64 @llvm.fshl.i64(i64 %139, i64 %139, i64 32) #24
  %143 = add i64 %141, %136
  %144 = call i64 @llvm.fshl.i64(i64 %141, i64 %141, i64 13) #24
  %145 = xor i64 %144, %143
  %146 = add i64 %138, %142
  %147 = call i64 @llvm.fshl.i64(i64 %138, i64 %138, i64 16) #24
  %148 = xor i64 %147, %146
  %149 = call i64 @llvm.fshl.i64(i64 %148, i64 %148, i64 21) #24
  %150 = add i64 %145, %146
  %151 = call i64 @llvm.fshl.i64(i64 %145, i64 %145, i64 17) #24
  %152 = call i64 @llvm.fshl.i64(i64 %150, i64 %150, i64 32) #24
  %_17.i.i.i.i.i.i.i.i = xor i64 %150, %149
  %153 = xor i64 %_17.i.i.i.i.i.i.i.i, %151
  %154 = xor i64 %153, %152
  call void @llvm.experimental.noalias.scope.decl(metadata !654) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !657) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !660) #24
  %top7.i.i.i.i.i.i.i = lshr i64 %154, 57
  %155 = trunc i64 %top7.i.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i.i26 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !663, !noalias !666
  %self.idx.val.i.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !669, !noalias !666
  %.0.vec.insert.i.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %155, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i.i27

bb3.i.i.i.i.i.i27:                                ; preds = %bb21.i.i.i.i.i.i, %bb32
  %probe_seq.sroa.7.0.i.i.i.i.i.i = phi i64 [ 0, %bb32 ], [ %168, %bb21.i.i.i.i.i.i ]
  %.pn.i.i.i = phi i64 [ %154, %bb32 ], [ %169, %bb21.i.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i.i = and i64 %.pn.i.i.i, %_6.i.i.i.i.i.i.i26
  %156 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i.i
  %157 = bitcast i8* %156 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %157, align 1, !noalias !670
  %158 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i.i
  %159 = bitcast <16 x i1> %158 to i16
  br label %bb8.i.i.i.i.i.i

bb8.i.i.i.i.i.i:                                  ; preds = %bb10.i.i.i.i.i.i, %bb3.i.i.i.i.i.i27
  %iter.0.i.i.i.i.i.i = phi i16 [ %159, %bb3.i.i.i.i.i.i27 ], [ %_2.i.i.i.i.i.i.i.i, %bb10.i.i.i.i.i.i ]
  %160 = icmp eq i16 %iter.0.i.i.i.i.i.i, 0
  br i1 %160, label %bb12.i.i.i.i.i.i, label %bb10.i.i.i.i.i.i

bb12.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %161 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %162 = bitcast <16 x i1> %161 to i16
  %.not.i.i.i.i.i.i = icmp eq i16 %162, 0
  br i1 %.not.i.i.i.i.i.i, label %bb21.i.i.i.i.i.i, label %bb35

bb10.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %163 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i.i = zext i16 %163 to i64
  %_4.i.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i.i
  %_25.i.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i.i = and i64 %_25.i.i.i.i.i.i, %_6.i.i.i.i.i.i.i26
  %164 = sub i64 0, %index.i.i.i.i.i.i
  %165 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i.i, i64 %164, i32 0
  %166 = getelementptr inbounds i64, i64* %165, i64 -7
  %_6.idx.val.i.i.i.i.i.i.i = load i64, i64* %166, align 8, !noalias !673
  %167 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i.i, %obj_id.val20
  br i1 %167, label %bb4.i.i.i.i, label %bb8.i.i.i.i.i.i

bb21.i.i.i.i.i.i:                                 ; preds = %bb12.i.i.i.i.i.i
  %168 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i.i, 16
  %169 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %168
  br label %bb3.i.i.i.i.i.i27

bb4.i.i.i.i:                                      ; preds = %bb10.i.i.i.i.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !676) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !679) #24
  %170 = ptrtoint i8* %self.idx.val.i.i.i.i.i.i to i64
  %171 = ptrtoint i64* %165 to i64
  %172 = sub i64 %170, %171
  %173 = sdiv exact i64 %172, 56
  call void @llvm.experimental.noalias.scope.decl(metadata !682) #24
  %174 = add nsw i64 %173, -16
  %index_before.i.i.i.i.i.i.i = and i64 %174, %_6.i.i.i.i.i.i.i26
  %175 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index_before.i.i.i.i.i.i.i
  %176 = bitcast i8* %175 to <16 x i8>*
  %.0.copyload.i17.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %176, align 1, !noalias !685
  %177 = icmp eq <16 x i8> %.0.copyload.i17.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %178 = bitcast <16 x i1> %177 to i16
  %179 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %173
  %180 = bitcast i8* %179 to <16 x i8>*
  %.0.copyload.i418.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %180, align 1, !noalias !689
  %181 = icmp eq <16 x i8> %.0.copyload.i418.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %182 = bitcast <16 x i1> %181 to i16
  %183 = call i16 @llvm.ctlz.i16(i16 %178, i1 false) #24, !range !27
  %184 = call i16 @llvm.cttz.i16(i16 %182, i1 false) #24, !range !27
  %narrow.i.i.i.i.i.i.i = add nuw nsw i16 %184, %183
  %_20.i.i.i.i.i.i.i = icmp ugt i16 %narrow.i.i.i.i.i.i.i, 15
  br i1 %_20.i.i.i.i.i.i.i, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i", label %bb11.i.i.i.i.i.i.i

bb11.i.i.i.i.i.i.i:                               ; preds = %bb4.i.i.i.i
  %185 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !692, !noalias !693
  %186 = add i64 %185, 1
  store i64 %186, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !692, !noalias !693
  br label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i"

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i": ; preds = %bb11.i.i.i.i.i.i.i, %bb4.i.i.i.i
  %.sink20.i.i.i.i.i.i.i = phi i8 [ -1, %bb11.i.i.i.i.i.i.i ], [ -128, %bb4.i.i.i.i ]
  %index2.i.i.i.i.i.i.i.i = add i64 %index_before.i.i.i.i.i.i.i, 16
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %179, align 1, !noalias !694
  %187 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i.i.i
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %187, align 1, !noalias !694
  %188 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !692, !noalias !693
  %189 = add i64 %188, -1
  store i64 %189, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !692, !noalias !693
  %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_idx5.i.i = getelementptr inbounds i64, i64* %165, i64 -3
  %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i = bitcast i64* %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_idx5.i.i to {}**
  %_3.sroa.4.0.copyload.i.i = load {}*, {}** %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i, align 8, !noalias !695
  %190 = icmp eq {}* %_3.sroa.4.0.copyload.i.i, null
  br i1 %190, label %bb35, label %bb2.i

bb2.i:                                            ; preds = %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i"
  %_124.sroa.5.32._3.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i.sroa_idx = getelementptr inbounds i64, i64* %165, i64 -2
  %_124.sroa.5.32.copyload = load i64, i64* %_124.sroa.5.32._3.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i.sroa_idx, align 8, !noalias !696
  %_4.i.i.i.i.i.i.i = icmp eq i64 %_124.sroa.5.32.copyload, 0
  br i1 %_4.i.i.i.i.i.i.i, label %bb35, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i": ; preds = %bb2.i
  %191 = bitcast {}* %_3.sroa.4.0.copyload.i.i to i8*
  call void @__rust_dealloc(i8* nonnull %191, i64 %_124.sroa.5.32.copyload, i64 1) #24
  br label %bb35

abort:                                            ; preds = %cleanup
  %192 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26
  unreachable
}

; Function Attrs: nonlazybind uwtable
declare noundef i32 @rust_eh_personality(i32, i32 noundef, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*) unnamed_addr #6

; Function Attrs: argmemonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.lifetime.start.p0i8(i64 immarg, i8* nocapture) #13

; Function Attrs: argmemonly mustprogress nofree nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #14

; Function Attrs: argmemonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.lifetime.end.p0i8(i64 immarg, i8* nocapture) #13

; std::sys::unix::rand::hashmap_random_keys
; Function Attrs: nonlazybind uwtable
declare { i64, i64 } @_ZN3std3sys4unix4rand19hashmap_random_keys17ha4436479ecf804b2E() unnamed_addr #6

; std::sys::unix::locks::futex::Mutex::lock_contended
; Function Attrs: cold nonlazybind uwtable
declare void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef align 4 dereferenceable(4)) unnamed_addr #12

; std::sys::unix::locks::futex::Mutex::wake
; Function Attrs: cold nonlazybind uwtable
declare void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef align 4 dereferenceable(4)) unnamed_addr #12

; std::sys_common::mutex::MovableMutex::new
; Function Attrs: nonlazybind uwtable
declare i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E() unnamed_addr #6

; std::sync::poison::Flag::new
; Function Attrs: nonlazybind uwtable
declare i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E() unnamed_addr #6

; core::panicking::panic_no_unwind
; Function Attrs: cold noinline noreturn nounwind nonlazybind uwtable
declare void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() unnamed_addr #15

; std::panicking::rust_panic_with_hook
; Function Attrs: noreturn nonlazybind uwtable
declare void @_ZN3std9panicking20rust_panic_with_hook17hc82286af2030e925E({}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), i64* noalias noundef readonly align 8 dereferenceable_or_null(48), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24), i1 noundef zeroext) unnamed_addr #16

; std::panicking::panic_count::is_zero_slow_path
; Function Attrs: cold noinline nonlazybind uwtable
declare noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E() unnamed_addr #11

; <str as core::fmt::Display>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17hfa8f7ea124ceedccE"([0 x i8]* noalias noundef nonnull readonly align 1, i64, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; Function Attrs: argmemonly mustprogress nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i64(i8* nocapture writeonly, i8, i64, i1 immarg) #17

; Function Attrs: argmemonly mustprogress nofree nounwind willreturn
declare void @llvm.memmove.p0i8.p0i8.i64(i8* nocapture writeonly, i8* nocapture readonly, i64, i1 immarg) #14

; Function Attrs: argmemonly mustprogress nofree nounwind nonlazybind readonly uwtable willreturn
declare i64 @strlen(i8* nocapture) unnamed_addr #18

; core::fmt::num::imp::<impl core::fmt::Display for i64>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E"(i64* noalias noundef readonly align 8 dereferenceable(8), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::fmt::num::<impl core::fmt::UpperHex for usize>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17h0a1d23de10af675eE"(i64* noalias noundef readonly align 8 dereferenceable(8), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::fmt::Formatter::debug_lower_hex
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @_ZN4core3fmt9Formatter15debug_lower_hex17h50fe8a435241971eE(%"core::fmt::Formatter"* noalias noundef readonly align 8 dereferenceable(64)) unnamed_addr #6

; core::fmt::num::<impl core::fmt::LowerHex for i64>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$i64$GT$3fmt17ha92a0b3e2a1e9677E"(i64* noalias noundef readonly align 8 dereferenceable(8), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::fmt::Formatter::debug_upper_hex
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @_ZN4core3fmt9Formatter15debug_upper_hex17h3960174cd3e4a3c3E(%"core::fmt::Formatter"* noalias noundef readonly align 8 dereferenceable(64)) unnamed_addr #6

; core::fmt::num::<impl core::fmt::UpperHex for i64>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN4core3fmt3num53_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$i64$GT$3fmt17hb6321a42a400d3f1E"(i64* noalias noundef readonly align 8 dereferenceable(8), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::panicking::panic_fmt
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef dereferenceable(48), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #4

; Function Attrs: inaccessiblememonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.assume(i1 noundef) #19

; core::panicking::panic
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1, i64, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #4

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i16 @llvm.ctlz.i16(i16, i1 immarg) #20

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i16 @llvm.cttz.i16(i16, i1 immarg) #20

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.fshl.i64(i64, i64, i64) #20

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i64, i1 } @llvm.uadd.with.overflow.i64(i64, i64) #20

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i64, i1 } @llvm.umul.with.overflow.i64(i64, i64) #20

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.ctlz.i64(i64, i1 immarg) #20

; <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
; Function Attrs: nonlazybind uwtable
declare void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef align 4 dereferenceable(4)) unnamed_addr #6

; <std::thread::local::AccessError as core::fmt::Debug>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h514ef917cd5ecc1bE"(%"std::thread::local::AccessError"* noalias noundef nonnull readonly align 1, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::result::unwrap_failed
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1, i64, {}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #4

; <core::str::error::Utf8Error as core::fmt::Debug>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN64_$LT$core..str..error..Utf8Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h864a228d6ab6973cE"(%"core::str::error::Utf8Error"* noalias noundef readonly align 8 dereferenceable(16), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::panicking::assert_failed_inner
; Function Attrs: noreturn nonlazybind uwtable
declare void @_ZN4core9panicking19assert_failed_inner17h36469c68b6fc10f1E(i8 noundef, {}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), {}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef dereferenceable(48), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #16

; alloc::alloc::handle_alloc_error
; Function Attrs: cold noreturn nonlazybind uwtable
declare void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64, i64 noundef) unnamed_addr #10

; Function Attrs: nofree nounwind nonlazybind uwtable
declare noalias i8* @__rust_alloc(i64, i64) unnamed_addr #21

; Function Attrs: nounwind nonlazybind uwtable
declare void @__rust_dealloc(i8*, i64, i64) unnamed_addr #8

; core::fmt::Formatter::debug_struct
; Function Attrs: nonlazybind uwtable
declare void @_ZN4core3fmt9Formatter12debug_struct17h65c357ef1edbbc54E(%"core::fmt::builders::DebugStruct"* noalias nocapture noundef sret(%"core::fmt::builders::DebugStruct") dereferenceable(16), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64), [0 x i8]* noalias noundef nonnull readonly align 1, i64) unnamed_addr #6

; core::fmt::builders::DebugStruct::finish_non_exhaustive
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @_ZN4core3fmt8builders11DebugStruct21finish_non_exhaustive17hb4065c184e958738E(%"core::fmt::builders::DebugStruct"* noalias noundef align 8 dereferenceable(16)) unnamed_addr #6

; std::process::abort
; Function Attrs: cold noreturn nonlazybind uwtable
declare void @_ZN3std7process5abort17h9abe461bf20ade28E() unnamed_addr #10

; hashbrown::raw::Fallibility::capacity_overflow
; Function Attrs: nonlazybind uwtable
declare { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext) unnamed_addr #6

; hashbrown::raw::Fallibility::alloc_err
; Function Attrs: nonlazybind uwtable
declare { i64, i64 } @_ZN9hashbrown3raw11Fallibility9alloc_err17h3f1a17e1376e6326E(i1 noundef zeroext, i64, i64 noundef) unnamed_addr #6

; once_cell::imp::initialize_or_wait
; Function Attrs: noinline nonlazybind uwtable
declare void @_ZN9once_cell3imp18initialize_or_wait17h9b3310b1603d0203E(%"core::sync::atomic::AtomicUsize"* noundef align 8 dereferenceable(8), i8* noundef align 1, i8*) unnamed_addr #3

; core::ffi::c_str::CStr::to_str
; Function Attrs: nonlazybind uwtable
declare void @_ZN4core3ffi5c_str4CStr6to_str17haa887525d1060a40E(%"core::result::Result<&str, core::str::error::Utf8Error>"* noalias nocapture noundef sret(%"core::result::Result<&str, core::str::error::Utf8Error>") dereferenceable(24), %"core::ffi::c_str::CStr"* noalias noundef nonnull readonly align 1, i64) unnamed_addr #6

; std::io::stdio::_print
; Function Attrs: nonlazybind uwtable
declare void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef dereferenceable(48)) unnamed_addr #6

; Function Attrs: inaccessiblememonly nofree nosync nounwind willreturn
declare void @llvm.experimental.noalias.scope.decl(metadata) #22

attributes #0 = { mustprogress nofree norecurse nosync nounwind nonlazybind readnone uwtable willreturn "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #1 = { noinline noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #2 = { inlinehint nofree nosync nounwind nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #3 = { noinline nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #4 = { cold noinline noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #5 = { inlinehint noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #6 = { nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #7 = { inlinehint nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #8 = { nounwind nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #9 = { inlinehint mustprogress nofree norecurse nosync nounwind nonlazybind readnone uwtable willreturn "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #10 = { cold noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #11 = { cold noinline nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #12 = { cold nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #13 = { argmemonly mustprogress nofree nosync nounwind willreturn }
attributes #14 = { argmemonly mustprogress nofree nounwind willreturn }
attributes #15 = { cold noinline noreturn nounwind nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #16 = { noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #17 = { argmemonly mustprogress nofree nounwind willreturn writeonly }
attributes #18 = { argmemonly mustprogress nofree nounwind nonlazybind readonly uwtable willreturn "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #19 = { inaccessiblememonly mustprogress nofree nosync nounwind willreturn }
attributes #20 = { mustprogress nofree nosync nounwind readnone speculatable willreturn }
attributes #21 = { nofree nounwind nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #22 = { inaccessiblememonly nofree nosync nounwind willreturn }
attributes #23 = { noreturn }
attributes #24 = { nounwind }
attributes #25 = { noinline }
attributes #26 = { noinline noreturn nounwind }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 7, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{!3}
!3 = distinct !{!3, !4, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h3bb1e1c4c67e6a69E: %self"}
!4 = distinct !{!4, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h3bb1e1c4c67e6a69E"}
!5 = !{!6}
!6 = distinct !{!6, !7, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$9get_inner17hb28eb85a0b499d75E: %self"}
!7 = distinct !{!7, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$9get_inner17hb28eb85a0b499d75E"}
!8 = !{!6, !3}
!9 = !{!10}
!10 = distinct !{!10, !11, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$3get17hc8d30a44e7acc255E: %self"}
!11 = distinct !{!11, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$3get17hc8d30a44e7acc255E"}
!12 = !{!13}
!13 = distinct !{!13, !14, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: %self"}
!14 = distinct !{!14, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E"}
!15 = !{!16}
!16 = distinct !{!16, !17, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!17 = distinct !{!17, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!18 = !{!19, !16, !13, !10, !6, !3}
!19 = distinct !{!19, !20, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!20 = distinct !{!20, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!21 = !{!22}
!22 = distinct !{!22, !14, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: argument 1"}
!23 = !{!13, !10, !6, !3}
!24 = !{!25, !16, !13, !22, !10, !6, !3}
!25 = distinct !{!25, !26, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!26 = distinct !{!26, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!27 = !{i16 0, i16 17}
!28 = !{!29, !16, !13, !22, !10, !6, !3}
!29 = distinct !{!29, !30, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E: %_1"}
!30 = distinct !{!30, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E"}
!31 = !{!32}
!32 = distinct !{!32, !33, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE: %self"}
!33 = distinct !{!33, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17h7f36be79e3d8a2acE"}
!34 = !{!35}
!35 = distinct !{!35, !36, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$13get_inner_mut17h2450f9c8f44ee0d3E: %self"}
!36 = distinct !{!36, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$13get_inner_mut17h2450f9c8f44ee0d3E"}
!37 = !{!35, !32}
!38 = !{!39}
!39 = distinct !{!39, !40, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h96cd7094a0a5915dE: %self"}
!40 = distinct !{!40, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h96cd7094a0a5915dE"}
!41 = !{!42}
!42 = distinct !{!42, !43, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: %self"}
!43 = distinct !{!43, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E"}
!44 = !{!45}
!45 = distinct !{!45, !46, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!46 = distinct !{!46, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!47 = !{!48, !45, !42, !39, !35, !32}
!48 = distinct !{!48, !49, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!49 = distinct !{!49, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!50 = !{!51}
!51 = distinct !{!51, !43, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: argument 1"}
!52 = !{!42, !39, !35, !32}
!53 = !{!54, !45, !42, !51, !39, !35, !32}
!54 = distinct !{!54, !55, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!55 = distinct !{!55, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!56 = !{!57, !45, !42, !51, !39, !35, !32}
!57 = distinct !{!57, !58, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E: %_1"}
!58 = distinct !{!58, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E"}
!59 = !{!60}
!60 = distinct !{!60, !61, !"_ZN3std6thread5local4lazy21LazyKeyInner$LT$T$GT$10initialize17hb28fda0effd918eaE: %init"}
!61 = distinct !{!61, !"_ZN3std6thread5local4lazy21LazyKeyInner$LT$T$GT$10initialize17hb28fda0effd918eaE"}
!62 = !{!63}
!63 = distinct !{!63, !64, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit28_$u7b$$u7b$closure$u7d$$u7d$17h409f968442c2a6c4E: argument 0"}
!64 = distinct !{!64, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit28_$u7b$$u7b$closure$u7d$$u7d$17h409f968442c2a6c4E"}
!65 = !{!66}
!66 = distinct !{!66, !67, !"_ZN4core6option15Option$LT$T$GT$4take17h0cdc848c9dfd8888E: argument 0"}
!67 = distinct !{!67, !"_ZN4core6option15Option$LT$T$GT$4take17h0cdc848c9dfd8888E"}
!68 = !{!69}
!69 = distinct !{!69, !70, !"_ZN4core3mem7replace17h703a176b838a7681E: %result"}
!70 = distinct !{!70, !"_ZN4core3mem7replace17h703a176b838a7681E"}
!71 = !{!72}
!72 = distinct !{!72, !70, !"_ZN4core3mem7replace17h703a176b838a7681E: %src"}
!73 = !{!69, !74, !66, !75, !63, !60}
!74 = distinct !{!74, !70, !"_ZN4core3mem7replace17h703a176b838a7681E: %dest"}
!75 = distinct !{!75, !67, !"_ZN4core6option15Option$LT$T$GT$4take17h0cdc848c9dfd8888E: %self"}
!76 = !{!74, !72, !75, !63, !60}
!77 = !{!69, !66}
!78 = !{!63, !60}
!79 = !{!80, !82}
!80 = distinct !{!80, !81, !"_ZN4core3mem7replace17h703a176b838a7681E: %dest"}
!81 = distinct !{!81, !"_ZN4core3mem7replace17h703a176b838a7681E"}
!82 = distinct !{!82, !81, !"_ZN4core3mem7replace17h703a176b838a7681E: %src"}
!83 = !{!84, !60}
!84 = distinct !{!84, !81, !"_ZN4core3mem7replace17h703a176b838a7681E: %result"}
!85 = !{}
!86 = !{i64 8}
!87 = !{!88}
!88 = distinct !{!88, !89, !"_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i64$GT$3fmt17h5debc439757ab39aE: %self"}
!89 = distinct !{!89, !"_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$i64$GT$3fmt17h5debc439757ab39aE"}
!90 = !{i64 1}
!91 = !{!92}
!92 = distinct !{!92, !93, !"_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E: %_1"}
!93 = distinct !{!93, !"_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E"}
!94 = !{!95}
!95 = distinct !{!95, !96, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E: %_1"}
!96 = distinct !{!96, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E"}
!97 = !{!95, !92}
!98 = !{!99, !101, !103}
!99 = distinct !{!99, !100, !"_ZN4core3mem7replace17ha318695de15894dbE: %dest"}
!100 = distinct !{!100, !"_ZN4core3mem7replace17ha318695de15894dbE"}
!101 = distinct !{!101, !102, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E: %self"}
!102 = distinct !{!102, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E"}
!103 = distinct !{!103, !104, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E: %val"}
!104 = distinct !{!104, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E"}
!105 = !{!106}
!106 = distinct !{!106, !107, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: %_1"}
!107 = distinct !{!107, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE"}
!108 = !{!109}
!109 = distinct !{!109, !110, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: %_1"}
!110 = distinct !{!110, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE"}
!111 = !{!109, !106}
!112 = !{!113, !114, !95, !92}
!113 = distinct !{!113, !110, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: argument 0"}
!114 = distinct !{!114, !107, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: argument 0"}
!115 = !{!116}
!116 = distinct !{!116, !117, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E: %dest"}
!117 = distinct !{!117, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E"}
!118 = !{!113, !109, !114, !106, !95, !92}
!119 = !{!106, !95, !92}
!120 = !{i64 0, i64 2}
!121 = !{!122}
!122 = distinct !{!122, !123, !"_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E: %_1"}
!123 = distinct !{!123, !"_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E"}
!124 = !{!125}
!125 = distinct !{!125, !126, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E: %_1"}
!126 = distinct !{!126, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E"}
!127 = !{!125, !122}
!128 = !{!129, !131, !133}
!129 = distinct !{!129, !130, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E: %dest"}
!130 = distinct !{!130, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E"}
!131 = distinct !{!131, !132, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E: %self"}
!132 = distinct !{!132, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E"}
!133 = distinct !{!133, !134, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE: %val"}
!134 = distinct !{!134, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE"}
!135 = !{!136}
!136 = distinct !{!136, !137, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: %_1"}
!137 = distinct !{!137, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE"}
!138 = !{!139}
!139 = distinct !{!139, !140, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: %_1"}
!140 = distinct !{!140, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E"}
!141 = !{!139, !136}
!142 = !{!143, !144, !125, !122}
!143 = distinct !{!143, !140, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: argument 0"}
!144 = distinct !{!144, !137, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: argument 0"}
!145 = !{!146}
!146 = distinct !{!146, !147, !"_ZN4core3mem7replace17h17668aa3bb646e28E: %dest"}
!147 = distinct !{!147, !"_ZN4core3mem7replace17h17668aa3bb646e28E"}
!148 = !{!143, !139, !144, !136, !125, !122}
!149 = !{!136, !125, !122}
!150 = !{!151}
!151 = distinct !{!151, !152, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E: argument 0"}
!152 = distinct !{!152, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E"}
!153 = !{!154}
!154 = distinct !{!154, !155, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE: argument 0"}
!155 = distinct !{!155, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE"}
!156 = !{!157, !159, !161, !154, !151}
!157 = distinct !{!157, !158, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE: %init"}
!158 = distinct !{!158, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE"}
!159 = distinct !{!159, !160, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E: %init"}
!160 = distinct !{!160, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E"}
!161 = distinct !{!161, !162, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE: argument 0"}
!162 = distinct !{!162, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE"}
!163 = !{!161, !154, !151}
!164 = !{!154, !151}
!165 = !{!166}
!166 = distinct !{!166, !167, !"_ZN4core3mem7replace17h3116444c89fcbd6bE: %dest"}
!167 = distinct !{!167, !"_ZN4core3mem7replace17h3116444c89fcbd6bE"}
!168 = !{!169, !154}
!169 = distinct !{!169, !170, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17h09e7fd16abe92fafE: argument 0"}
!170 = distinct !{!170, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17h09e7fd16abe92fafE"}
!171 = !{!172}
!172 = distinct !{!172, !173, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17h5647cc520582ff0bE: argument 0"}
!173 = distinct !{!173, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17h5647cc520582ff0bE"}
!174 = !{!175}
!175 = distinct !{!175, !173, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17h5647cc520582ff0bE: %t"}
!176 = !{!172, !175, !151}
!177 = !{!172, !151}
!178 = !{!172, !175}
!179 = !{!180}
!180 = distinct !{!180, !181, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE: argument 0"}
!181 = distinct !{!181, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE"}
!182 = !{!183}
!183 = distinct !{!183, !184, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E: argument 0"}
!184 = distinct !{!184, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E"}
!185 = !{!183, !180}
!186 = !{!187}
!187 = distinct !{!187, !188, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!188 = distinct !{!188, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!189 = !{!190}
!190 = distinct !{!190, !191, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!191 = distinct !{!191, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!192 = !{!193}
!193 = distinct !{!193, !194, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h525f6dd284d29203E: %self"}
!194 = distinct !{!194, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h525f6dd284d29203E"}
!195 = !{!196, !193}
!196 = distinct !{!196, !197, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!197 = distinct !{!197, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!198 = !{!199}
!199 = distinct !{!199, !200, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E: %self"}
!200 = distinct !{!200, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E"}
!201 = !{!199, !193}
!202 = !{!203}
!203 = distinct !{!203, !204, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4iter17hea862c4ee711fef1E: %self"}
!204 = distinct !{!204, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4iter17hea862c4ee711fef1E"}
!205 = !{!203, !199, !193}
!206 = !{!207}
!207 = distinct !{!207, !204, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4iter17hea862c4ee711fef1E: argument 0"}
!208 = !{!209, !211, !213, !207, !203, !199, !193}
!209 = distinct !{!209, !210, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!210 = distinct !{!210, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!211 = distinct !{!211, !212, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!212 = distinct !{!212, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!213 = distinct !{!213, !214, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17h3a8faabbbff5cd00E: argument 0"}
!214 = distinct !{!214, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17h3a8faabbbff5cd00E"}
!215 = !{!216, !218, !220, !222, !199, !193}
!216 = distinct !{!216, !217, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!217 = distinct !{!217, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!218 = distinct !{!218, !219, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!219 = distinct !{!219, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!220 = distinct !{!220, !221, !"_ZN96_$LT$hashbrown..raw..RawIterRange$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h05cca4a540c158cfE: %self"}
!221 = distinct !{!221, !"_ZN96_$LT$hashbrown..raw..RawIterRange$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h05cca4a540c158cfE"}
!222 = distinct !{!222, !223, !"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E: %self"}
!223 = distinct !{!223, !"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E"}
!224 = !{!225}
!225 = distinct !{!225, !226, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he13d6557b60c3d5dE: %self"}
!226 = distinct !{!226, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he13d6557b60c3d5dE"}
!227 = !{!228}
!228 = distinct !{!228, !229, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!229 = distinct !{!229, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!230 = !{!228, !225, !193}
!231 = !{!232, !234}
!232 = distinct !{!232, !233, !"_ZN4core3mem7replace17h788e58c37a635438E: %dest"}
!233 = distinct !{!233, !"_ZN4core3mem7replace17h788e58c37a635438E"}
!234 = distinct !{!234, !235, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE: %self"}
!235 = distinct !{!235, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE"}
!236 = !{!237}
!237 = distinct !{!237, !238, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE: %x.0"}
!238 = distinct !{!238, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE"}
!239 = !{!240}
!240 = distinct !{!240, !241, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E: %self"}
!241 = distinct !{!241, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E"}
!242 = !{!243}
!243 = distinct !{!243, !244, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E: %self"}
!244 = distinct !{!244, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E"}
!245 = !{i64 0, i64 65}
!246 = !{!247, !249, !251, !243, !240}
!247 = distinct !{!247, !248, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE: argument 0"}
!248 = distinct !{!248, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE"}
!249 = distinct !{!249, !250, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E: argument 0"}
!250 = distinct !{!250, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E"}
!251 = distinct !{!251, !252, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E: argument 0"}
!252 = distinct !{!252, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E"}
!253 = !{!254, !249, !251, !243, !240}
!254 = distinct !{!254, !255, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E: argument 0"}
!255 = distinct !{!255, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E"}
!256 = !{!249, !251, !243, !240}
!257 = !{!243, !240}
!258 = !{!259, !261, !262, !264, !243, !240}
!259 = distinct !{!259, !260, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %x"}
!260 = distinct !{!260, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E"}
!261 = distinct !{!261, !260, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y:thread"}
!262 = distinct !{!262, !263, !"_ZN4core3mem4swap17h8292e61c571debd1E: %x"}
!263 = distinct !{!263, !"_ZN4core3mem4swap17h8292e61c571debd1E"}
!264 = distinct !{!264, !263, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y:thread"}
!265 = !{!266}
!266 = distinct !{!266, !267, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!267 = distinct !{!267, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!268 = !{!269, !271, !243, !240}
!269 = distinct !{!269, !270, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %_1"}
!270 = distinct !{!270, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E"}
!271 = distinct !{!271, !270, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %table"}
!272 = !{!273, !275, !277, !243, !240}
!273 = distinct !{!273, !274, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!274 = distinct !{!274, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!275 = distinct !{!275, !276, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!276 = distinct !{!276, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!277 = distinct !{!277, !278, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E: %self"}
!278 = distinct !{!278, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E"}
!279 = !{!275, !277, !243, !240}
!280 = !{!281, !283, !275, !277, !243, !240}
!281 = distinct !{!281, !282, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!282 = distinct !{!282, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!283 = distinct !{!283, !284, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!284 = distinct !{!284, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!285 = !{!286, !288, !277, !243, !240}
!286 = distinct !{!286, !287, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!287 = distinct !{!287, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!288 = distinct !{!288, !289, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!289 = distinct !{!289, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!290 = !{!259, !291, !262, !292, !243, !240}
!291 = distinct !{!291, !260, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y"}
!292 = distinct !{!292, !263, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y"}
!293 = !{!294, !296, !298, !243, !240}
!294 = distinct !{!294, !295, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!295 = distinct !{!295, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!296 = distinct !{!296, !297, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E: %self_"}
!297 = distinct !{!297, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E"}
!298 = distinct !{!298, !299, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E: %self"}
!299 = distinct !{!299, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E"}
!300 = !{!301}
!301 = distinct !{!301, !302, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E: %self"}
!302 = distinct !{!302, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E"}
!303 = !{!304}
!304 = distinct !{!304, !305, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E: %self"}
!305 = distinct !{!305, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E"}
!306 = !{!304, !301, !240}
!307 = !{!308, !310, !304, !301, !240}
!308 = distinct !{!308, !309, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!309 = distinct !{!309, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!310 = distinct !{!310, !311, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!311 = distinct !{!311, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!312 = !{!313, !304, !301, !240}
!313 = distinct !{!313, !314, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE: %a"}
!314 = distinct !{!314, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE"}
!315 = !{!301, !240}
!316 = !{!317}
!317 = distinct !{!317, !318, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!318 = distinct !{!318, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!319 = !{!320, !322, !301, !240}
!320 = distinct !{!320, !321, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %_1"}
!321 = distinct !{!321, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E"}
!322 = distinct !{!322, !321, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %table"}
!323 = !{!324, !326, !301, !240}
!324 = distinct !{!324, !325, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!325 = distinct !{!325, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!326 = distinct !{!326, !327, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!327 = distinct !{!327, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!328 = !{!326, !301, !240}
!329 = !{!330, !332, !326, !301, !240}
!330 = distinct !{!330, !331, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!331 = distinct !{!331, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!332 = distinct !{!332, !333, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!333 = distinct !{!333, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!334 = !{!335, !337, !301, !240}
!335 = distinct !{!335, !336, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!336 = distinct !{!336, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!337 = distinct !{!337, !338, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!338 = distinct !{!338, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!339 = !{!340, !301, !240}
!340 = distinct !{!340, !341, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E: %self"}
!341 = distinct !{!341, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E"}
!342 = !{!343, !345, !340, !301, !240}
!343 = distinct !{!343, !344, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!344 = distinct !{!344, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!345 = distinct !{!345, !346, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!346 = distinct !{!346, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!347 = !{!348}
!348 = distinct !{!348, !349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x"}
!349 = distinct !{!349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE"}
!350 = !{!351}
!351 = distinct !{!351, !349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y"}
!352 = !{!351, !301, !240}
!353 = !{!348, !301, !240}
!354 = !{!355}
!355 = distinct !{!355, !349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It1"}
!356 = !{!357}
!357 = distinct !{!357, !349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It1"}
!358 = !{!357, !301, !240}
!359 = !{!355, !301, !240}
!360 = !{!361}
!361 = distinct !{!361, !349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It2"}
!362 = !{!363}
!363 = distinct !{!363, !349, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It2"}
!364 = !{!363, !301, !240}
!365 = !{!361, !301, !240}
!366 = distinct !{!366, !367, !368}
!367 = !{!"llvm.loop.isvectorized", i32 1}
!368 = !{!"llvm.loop.unroll.runtime.disable"}
!369 = !{!370, !301, !240}
!370 = distinct !{!370, !371, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!371 = distinct !{!371, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!372 = !{!373, !375, !377}
!373 = distinct !{!373, !374, !"_ZN4core3mem7replace17ha318695de15894dbE: %dest"}
!374 = distinct !{!374, !"_ZN4core3mem7replace17ha318695de15894dbE"}
!375 = distinct !{!375, !376, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E: %self"}
!376 = distinct !{!376, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E"}
!377 = distinct !{!377, !378, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E: %val"}
!378 = distinct !{!378, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E"}
!379 = !{!380}
!380 = distinct !{!380, !381, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: %_1"}
!381 = distinct !{!381, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE"}
!382 = !{!383}
!383 = distinct !{!383, !384, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: %_1"}
!384 = distinct !{!384, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE"}
!385 = !{!383, !380}
!386 = !{!387, !388}
!387 = distinct !{!387, !384, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: argument 0"}
!388 = distinct !{!388, !381, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: argument 0"}
!389 = !{!390}
!390 = distinct !{!390, !391, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E: %dest"}
!391 = distinct !{!391, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E"}
!392 = !{!387, !383, !388, !380}
!393 = !{!394, !396, !398}
!394 = distinct !{!394, !395, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E: %dest"}
!395 = distinct !{!395, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E"}
!396 = distinct !{!396, !397, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E: %self"}
!397 = distinct !{!397, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E"}
!398 = distinct !{!398, !399, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE: %val"}
!399 = distinct !{!399, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE"}
!400 = !{!401}
!401 = distinct !{!401, !402, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: %_1"}
!402 = distinct !{!402, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE"}
!403 = !{!404}
!404 = distinct !{!404, !405, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: %_1"}
!405 = distinct !{!405, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E"}
!406 = !{!404, !401}
!407 = !{!408, !409}
!408 = distinct !{!408, !405, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: argument 0"}
!409 = distinct !{!409, !402, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: argument 0"}
!410 = !{!411}
!411 = distinct !{!411, !412, !"_ZN4core3mem7replace17h17668aa3bb646e28E: %dest"}
!412 = distinct !{!412, !"_ZN4core3mem7replace17h17668aa3bb646e28E"}
!413 = !{!408, !404, !409, !401}
!414 = !{!415}
!415 = distinct !{!415, !416, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!416 = distinct !{!416, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!417 = !{!418, !419}
!418 = distinct !{!418, !416, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!419 = distinct !{!419, !416, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!420 = !{!421}
!421 = distinct !{!421, !422, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E: %self"}
!422 = distinct !{!422, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E"}
!423 = !{!424, !426}
!424 = distinct !{!424, !425, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E: %f"}
!425 = distinct !{!425, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E"}
!426 = distinct !{!426, !427, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E: %f"}
!427 = distinct !{!427, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E"}
!428 = !{!429}
!429 = distinct !{!429, !430, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE: argument 0"}
!430 = distinct !{!430, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE"}
!431 = !{!432, !429}
!432 = distinct !{!432, !433, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E: argument 0"}
!433 = distinct !{!433, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E"}
!434 = !{!435}
!435 = distinct !{!435, !436, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E: %self"}
!436 = distinct !{!436, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E"}
!437 = !{!438}
!438 = distinct !{!438, !439, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: argument 0"}
!439 = distinct !{!439, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E"}
!440 = !{!441, !442, !443}
!441 = distinct !{!441, !439, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %pieces.0"}
!442 = distinct !{!442, !439, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %args.0"}
!443 = distinct !{!443, !439, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %fmt.0"}
!444 = !{!445, !447}
!445 = distinct !{!445, !446, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE: %f"}
!446 = distinct !{!446, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE"}
!447 = distinct !{!447, !448, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E: %f"}
!448 = distinct !{!448, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E"}
!449 = !{!450}
!450 = distinct !{!450, !451, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE: argument 0"}
!451 = distinct !{!451, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE"}
!452 = !{!453, !450}
!453 = distinct !{!453, !454, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E: argument 0"}
!454 = distinct !{!454, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E"}
!455 = !{!456, !458}
!456 = distinct !{!456, !457, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: %self"}
!457 = distinct !{!457, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE"}
!458 = distinct !{!458, !457, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: argument 1"}
!459 = !{!456}
!460 = !{!461, !463, !465, !466, !468, !469, !471, !472, !474, !475, !477, !478, !480, !481, !483}
!461 = distinct !{!461, !462, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16with_capacity_in17haf58c241a925526dE: argument 0"}
!462 = distinct !{!462, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16with_capacity_in17haf58c241a925526dE"}
!463 = distinct !{!463, !464, !"_ZN52_$LT$T$u20$as$u20$alloc..slice..hack..ConvertVec$GT$6to_vec17h53aee583d85922ecE: %v"}
!464 = distinct !{!464, !"_ZN52_$LT$T$u20$as$u20$alloc..slice..hack..ConvertVec$GT$6to_vec17h53aee583d85922ecE"}
!465 = distinct !{!465, !464, !"_ZN52_$LT$T$u20$as$u20$alloc..slice..hack..ConvertVec$GT$6to_vec17h53aee583d85922ecE: %s.0"}
!466 = distinct !{!466, !467, !"_ZN5alloc5slice4hack6to_vec17h9d653acab8d582dcE: argument 0"}
!467 = distinct !{!467, !"_ZN5alloc5slice4hack6to_vec17h9d653acab8d582dcE"}
!468 = distinct !{!468, !467, !"_ZN5alloc5slice4hack6to_vec17h9d653acab8d582dcE: %s.0"}
!469 = distinct !{!469, !470, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$9to_vec_in17hcb2720fd082a03b1E: argument 0"}
!470 = distinct !{!470, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$9to_vec_in17hcb2720fd082a03b1E"}
!471 = distinct !{!471, !470, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$9to_vec_in17hcb2720fd082a03b1E: %self.0"}
!472 = distinct !{!472, !473, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6to_vec17ha27e4e65413e47a6E: argument 0"}
!473 = distinct !{!473, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6to_vec17ha27e4e65413e47a6E"}
!474 = distinct !{!474, !473, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6to_vec17ha27e4e65413e47a6E: %self.0"}
!475 = distinct !{!475, !476, !"_ZN5alloc5slice64_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$$u5b$T$u5d$$GT$8to_owned17h826e2cc3001afcccE: argument 0"}
!476 = distinct !{!476, !"_ZN5alloc5slice64_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$$u5b$T$u5d$$GT$8to_owned17h826e2cc3001afcccE"}
!477 = distinct !{!477, !476, !"_ZN5alloc5slice64_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$$u5b$T$u5d$$GT$8to_owned17h826e2cc3001afcccE: %self.0"}
!478 = distinct !{!478, !479, !"_ZN5alloc3str56_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$str$GT$8to_owned17h0326c85be227b8e5E: argument 0"}
!479 = distinct !{!479, !"_ZN5alloc3str56_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$str$GT$8to_owned17h0326c85be227b8e5E"}
!480 = distinct !{!480, !479, !"_ZN5alloc3str56_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$str$GT$8to_owned17h0326c85be227b8e5E: %self.0"}
!481 = distinct !{!481, !482, !"_ZN76_$LT$alloc..string..String$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h28e78c83f20c9950E: argument 0"}
!482 = distinct !{!482, !"_ZN76_$LT$alloc..string..String$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h28e78c83f20c9950E"}
!483 = distinct !{!483, !482, !"_ZN76_$LT$alloc..string..String$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h28e78c83f20c9950E: %s.0"}
!484 = !{!463, !466, !469, !472, !475, !478, !481}
!485 = !{!486}
!486 = distinct !{!486, !487, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE: %self"}
!487 = distinct !{!487, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE"}
!488 = !{!489}
!489 = distinct !{!489, !490, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E: %self"}
!490 = distinct !{!490, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E"}
!491 = !{!489, !486}
!492 = !{!493, !494, !495, !496}
!493 = distinct !{!493, !490, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E: argument 0"}
!494 = distinct !{!494, !490, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E: %v"}
!495 = distinct !{!495, !487, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE: argument 0"}
!496 = distinct !{!496, !487, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE: %v"}
!497 = !{!498}
!498 = distinct !{!498, !499, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h96cd7094a0a5915dE: %self"}
!499 = distinct !{!499, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h96cd7094a0a5915dE"}
!500 = !{!501}
!501 = distinct !{!501, !502, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: %self"}
!502 = distinct !{!502, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E"}
!503 = !{!504}
!504 = distinct !{!504, !505, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!505 = distinct !{!505, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!506 = !{!507, !504, !501, !498, !489, !486}
!507 = distinct !{!507, !508, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!508 = distinct !{!508, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!509 = !{!510, !493, !494, !495, !496}
!510 = distinct !{!510, !502, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: argument 1"}
!511 = !{!501, !498, !489, !486}
!512 = !{!513, !504, !501, !510, !498, !493, !489, !494, !495, !486, !496}
!513 = distinct !{!513, !514, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!514 = distinct !{!514, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!515 = !{!516, !504, !501, !510, !498, !493, !489, !494, !495, !486, !496}
!516 = distinct !{!516, !517, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E: %_1"}
!517 = distinct !{!517, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E"}
!518 = !{!493, !489, !494, !495, !486, !496}
!519 = !{!495, !486}
!520 = !{!521}
!521 = distinct !{!521, !522, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE: %self"}
!522 = distinct !{!522, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE"}
!523 = !{!524, !526, !521, !528, !529, !493, !489, !494, !495, !486, !496}
!524 = distinct !{!524, !525, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!525 = distinct !{!525, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!526 = distinct !{!526, !527, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!527 = distinct !{!527, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!528 = distinct !{!528, !522, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE: %value"}
!529 = distinct !{!529, !522, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE: %hasher"}
!530 = !{!526, !521, !528, !529, !493, !489, !494, !495, !486, !496}
!531 = !{!532, !534, !526, !521, !528, !529, !493, !489, !494, !495, !486, !496}
!532 = distinct !{!532, !533, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!533 = distinct !{!533, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!534 = distinct !{!534, !535, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!535 = distinct !{!535, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!536 = !{!521, !528, !529, !493, !489, !494, !495, !486, !496}
!537 = !{!521, !489, !486}
!538 = !{!528, !529, !493, !494, !495, !496}
!539 = !{!528, !493, !494, !495, !496}
!540 = !{!541}
!541 = distinct !{!541, !542, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!542 = distinct !{!542, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!543 = !{!544, !541, !521, !489, !486}
!544 = distinct !{!544, !545, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!545 = distinct !{!545, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!546 = !{!541, !521, !489, !486}
!547 = !{!548, !541, !521, !528, !493, !494, !495, !496}
!548 = distinct !{!548, !549, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!549 = distinct !{!549, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!550 = !{!541, !521, !528, !493, !494, !495, !496}
!551 = !{!552, !554, !541, !521, !528, !493, !494, !495, !496}
!552 = distinct !{!552, !553, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!553 = distinct !{!553, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!554 = distinct !{!554, !555, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!555 = distinct !{!555, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!556 = !{!521, !493, !494, !495, !496}
!557 = !{!558}
!558 = distinct !{!558, !559, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E: %self"}
!559 = distinct !{!559, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E"}
!560 = !{!561, !563, !558, !521, !528, !493, !494, !495, !496}
!561 = distinct !{!561, !562, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!562 = distinct !{!562, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!563 = distinct !{!563, !564, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!564 = distinct !{!564, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!565 = !{!558, !521, !489, !486}
!566 = !{!489, !494, !486, !496}
!567 = !{!568}
!568 = distinct !{!568, !569, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!569 = distinct !{!569, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!570 = !{!571}
!571 = distinct !{!571, !572, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE: %self"}
!572 = distinct !{!572, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE"}
!573 = !{!574}
!574 = distinct !{!574, !575, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: argument 0"}
!575 = distinct !{!575, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E"}
!576 = !{!577, !578, !579}
!577 = distinct !{!577, !575, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %pieces.0"}
!578 = distinct !{!578, !575, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %args.0"}
!579 = distinct !{!579, !575, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %fmt.0"}
!580 = !{!581, !583}
!581 = distinct !{!581, !582, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE: %f"}
!582 = distinct !{!582, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE"}
!583 = distinct !{!583, !584, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E: %f"}
!584 = distinct !{!584, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E"}
!585 = !{!586}
!586 = distinct !{!586, !587, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE: argument 0"}
!587 = distinct !{!587, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE"}
!588 = !{!589, !586}
!589 = distinct !{!589, !590, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E: argument 0"}
!590 = distinct !{!590, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E"}
!591 = !{!592, !594}
!592 = distinct !{!592, !593, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: %self"}
!593 = distinct !{!593, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE"}
!594 = distinct !{!594, !593, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: argument 1"}
!595 = !{!592}
!596 = !{!597}
!597 = distinct !{!597, !598, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!598 = distinct !{!598, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!599 = !{!600}
!600 = distinct !{!600, !601, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!601 = distinct !{!601, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!602 = !{!603, !604}
!603 = distinct !{!603, !601, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!604 = distinct !{!604, !601, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!605 = !{!606}
!606 = distinct !{!606, !607, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!607 = distinct !{!607, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!608 = !{!609}
!609 = distinct !{!609, !610, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: argument 0"}
!610 = distinct !{!610, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E"}
!611 = !{!612, !613, !614}
!612 = distinct !{!612, !610, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %pieces.0"}
!613 = distinct !{!613, !610, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %args.0"}
!614 = distinct !{!614, !610, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %fmt.0"}
!615 = !{!616, !618}
!616 = distinct !{!616, !617, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE: %f"}
!617 = distinct !{!617, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE"}
!618 = distinct !{!618, !619, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E: %f"}
!619 = distinct !{!619, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E"}
!620 = !{!621}
!621 = distinct !{!621, !622, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE: argument 0"}
!622 = distinct !{!622, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE"}
!623 = !{!624, !621}
!624 = distinct !{!624, !625, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E: argument 0"}
!625 = distinct !{!625, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E"}
!626 = !{!627, !629}
!627 = distinct !{!627, !628, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: %self"}
!628 = distinct !{!628, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE"}
!629 = distinct !{!629, !628, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: argument 1"}
!630 = !{!627}
!631 = !{!632}
!632 = distinct !{!632, !633, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!633 = distinct !{!633, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!634 = !{!635, !636}
!635 = distinct !{!635, !633, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!636 = distinct !{!636, !633, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!637 = !{!638}
!638 = distinct !{!638, !639, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!639 = distinct !{!639, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!640 = !{!641}
!641 = distinct !{!641, !642, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h9c40df2332e3f4a7E: %self"}
!642 = distinct !{!642, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h9c40df2332e3f4a7E"}
!643 = !{!644}
!644 = distinct !{!644, !645, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h3e472d875cf1033bE: %self"}
!645 = distinct !{!645, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h3e472d875cf1033bE"}
!646 = !{!647}
!647 = distinct !{!647, !648, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE: %self"}
!648 = distinct !{!648, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE"}
!649 = !{!647, !644, !641}
!650 = !{!651, !652, !653}
!651 = distinct !{!651, !648, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE: argument 0"}
!652 = distinct !{!652, !645, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h3e472d875cf1033bE: argument 0"}
!653 = distinct !{!653, !642, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h9c40df2332e3f4a7E: argument 0"}
!654 = !{!655}
!655 = distinct !{!655, !656, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17h4afae7353d3cefa4E: %self"}
!656 = distinct !{!656, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17h4afae7353d3cefa4E"}
!657 = !{!658}
!658 = distinct !{!658, !659, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: %self"}
!659 = distinct !{!659, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E"}
!660 = !{!661}
!661 = distinct !{!661, !662, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!662 = distinct !{!662, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!663 = !{!664, !661, !658, !655, !647, !644, !641}
!664 = distinct !{!664, !665, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!665 = distinct !{!665, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!666 = !{!667, !668, !651, !652, !653}
!667 = distinct !{!667, !659, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: argument 1"}
!668 = distinct !{!668, !656, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17h4afae7353d3cefa4E: argument 0"}
!669 = !{!658, !655, !647, !644, !641}
!670 = !{!671, !661, !658, !667, !668, !655, !651, !647, !652, !644, !653, !641}
!671 = distinct !{!671, !672, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!672 = distinct !{!672, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!673 = !{!674, !661, !658, !667, !668, !655, !651, !647, !652, !644, !653, !641}
!674 = distinct !{!674, !675, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E: %_1"}
!675 = distinct !{!675, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E"}
!676 = !{!677}
!677 = distinct !{!677, !678, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17ha6f0eafe2ff00441E: %self"}
!678 = distinct !{!678, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17ha6f0eafe2ff00441E"}
!679 = !{!680}
!680 = distinct !{!680, !681, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h4aa5880891f88a93E: %self"}
!681 = distinct !{!681, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h4aa5880891f88a93E"}
!682 = !{!683}
!683 = distinct !{!683, !684, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E: %self"}
!684 = distinct !{!684, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E"}
!685 = !{!686, !683, !680, !688, !677, !668, !655, !651, !647, !652, !644, !653, !641}
!686 = distinct !{!686, !687, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!687 = distinct !{!687, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!688 = distinct !{!688, !678, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17ha6f0eafe2ff00441E: argument 0"}
!689 = !{!690, !683, !680, !688, !677, !668, !655, !651, !647, !652, !644, !653, !641}
!690 = distinct !{!690, !691, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!691 = distinct !{!691, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!692 = !{!683, !680, !677, !655, !647, !644, !641}
!693 = !{!688, !668, !651, !652, !653}
!694 = !{!683, !680, !688, !677, !668, !655, !651, !647, !652, !644, !653, !641}
!695 = !{!655, !647, !652, !644, !653, !641}
!696 = !{!644, !641}
