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
@alloc377 = private unnamed_addr constant <{ [70 x i8] }> <{ [70 x i8] c"cannot access a Thread Local Storage value during or after destruction" }>, align 1
@alloc380 = private unnamed_addr constant <{ [79 x i8] }> <{ [79 x i8] c"/rustc/a8314ef7d0ec7b75c336af2c9857bfaf43002bfc/library/std/src/thread/local.rs" }>, align 1
@alloc379 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [79 x i8] }>, <{ [79 x i8] }>* @alloc380, i32 0, i32 0, i32 0), [16 x i8] c"O\00\00\00\00\00\00\00\A5\01\00\00\1A\00\00\00" }>, align 8
@vtable.0 = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17h92e001d5e4efd74cE" to i8*), i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17hc9f8af2660d4514aE" to i8*) }>, align 8
@_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E = external local_unnamed_addr global %"core::sync::atomic::AtomicUsize"
@alloc73 = private unnamed_addr constant <{}> zeroinitializer, align 8
@alloc418 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Option::unwrap()` on a `None` value" }>, align 1
@vtable.3 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00", i8* bitcast (i1 (%"std::thread::local::AccessError"*, %"core::fmt::Formatter"*)* @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h514ef917cd5ecc1bE" to i8*) }>, align 8
@alloc430 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@vtable.5 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"core::str::error::Utf8Error"*, %"core::fmt::Formatter"*)* @"_ZN64_$LT$core..str..error..Utf8Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h864a228d6ab6973cE" to i8*) }>, align 8
@vtable.6 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void ({ i64*, i8 }*)* @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 ({ i64*, i8 }*, %"core::fmt::Formatter"*)* @"_ZN76_$LT$std..sync..poison..PoisonError$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h69df1c324ff6e669E" to i8*) }>, align 8
@vtable.7 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (i64**, %"core::fmt::Formatter"*)* @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hc715f6c95a655b17E" to i8*) }>, align 8
@alloc442 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"PoisonError" }>, align 1
@vtable.8 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i64 ({ [0 x i8]*, i64 }*)* @"_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17ha7daf7c2b2ea8d27E" to i8*) }>, align 8
@alloc67 = private unnamed_addr constant <{ [16 x i8] }> <{ [16 x i8] c"\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF" }>, align 16
@vtable.b = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h42a39cd9ab169dceE" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E" to i8*) }>, align 8
@vtable.c = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h8d298f77ff4ec3b3E" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E" to i8*) }>, align 8
@alloc474 = private unnamed_addr constant <{ [42 x i8] }> <{ [42 x i8] c"Lazy instance has previously been poisoned" }>, align 1
@alloc475 = private unnamed_addr constant <{ [90 x i8] }> <{ [90 x i8] c"/home/maruyama/.cargo/registry/src/github.com-1ecc6299db9ec823/once_cell-1.13.0/src/lib.rs" }>, align 1
@alloc473 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [90 x i8] }>, <{ [90 x i8] }>* @alloc475, i32 0, i32 0, i32 0), [16 x i8] c"Z\00\00\00\00\00\00\00\CF\04\00\00\19\00\00\00" }>, align 8
@_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE = internal global <{ [16 x i8], [16 x i8], i8* }> <{ [16 x i8] zeroinitializer, [16 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<i64>"*)* @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE to i8*) }>, align 8
@_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE = internal global <{ [16 x i8], [56 x i8], i8* }> <{ [16 x i8] zeroinitializer, [56 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)* @_ZN4core3ops8function6FnOnce9call_once17h792230541602dafdE to i8*) }>, align 8
@alloc70 = private unnamed_addr constant <{ [54 x i8] }> <{ [54 x i8] c"[report_malloc] Failed to convert given name to &str.\0A" }>, align 1
@alloc71 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [54 x i8] }>, <{ [54 x i8] }>* @alloc70, i32 0, i32 0, i32 0), [8 x i8] c"6\00\00\00\00\00\00\00" }>, align 8
@alloc501 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"src/lib.rs" }>, align 1
@alloc478 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00#\00\00\00!\00\00\00" }>, align 8
@alloc480 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00$\00\00\00)\00\00\00" }>, align 8
@alloc482 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00-\00\00\003\00\00\00" }>, align 8
@alloc226 = private unnamed_addr constant <{ [8 x i8] }> zeroinitializer, align 8
@alloc231 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"Object id=" }>, align 1
@alloc233 = private unnamed_addr constant <{ [31 x i8] }> <{ [31 x i8] c" whose refcnt zero is retained!" }>, align 1
@alloc232 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc231, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [31 x i8] }>, <{ [31 x i8] }>* @alloc233, i32 0, i32 0, i32 0), [8 x i8] c"\1F\00\00\00\00\00\00\00" }>, align 8
@alloc484 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00:\00\00\00\05\00\00\00" }>, align 8
@alloc486 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00?\00\00\003\00\00\00" }>, align 8
@alloc170 = private unnamed_addr constant <{ [20 x i8] }> <{ [20 x i8] c"Retain of object id=" }>, align 1
@alloc238 = private unnamed_addr constant <{ [50 x i8] }> <{ [50 x i8] c" is reported but it isn't registered to sanitizer." }>, align 1
@alloc171 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [20 x i8] }>, <{ [20 x i8] }>* @alloc170, i32 0, i32 0, i32 0), [8 x i8] c"\14\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc238, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc488 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00@\00\00\00\05\00\00\00" }>, align 8
@alloc490 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00E\00\00\00.\00\00\00" }>, align 8
@alloc243 = private unnamed_addr constant <{ [24 x i8] }> <{ [24 x i8] c"The refcnt of object id=" }>, align 1
@alloc179 = private unnamed_addr constant <{ [37 x i8] }> <{ [37 x i8] c" in report_retain mismatch! reported=" }>, align 1
@alloc246 = private unnamed_addr constant <{ [12 x i8] }> <{ [12 x i8] c", sanitizer=" }>, align 1
@alloc178 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc243, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [37 x i8] }>, <{ [37 x i8] }>* @alloc179, i32 0, i32 0, i32 0), [8 x i8] c"%\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc246, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc492 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00F\00\00\00\05\00\00\00" }>, align 8
@alloc494 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00Z\00\00\00\05\00\00\00" }>, align 8
@alloc496 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00_\00\00\002\00\00\00" }>, align 8
@alloc236 = private unnamed_addr constant <{ [21 x i8] }> <{ [21 x i8] c"Release of object id=" }>, align 1
@alloc237 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [21 x i8] }>, <{ [21 x i8] }>* @alloc236, i32 0, i32 0, i32 0), [8 x i8] c"\15\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc238, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc498 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00`\00\00\00\05\00\00\00" }>, align 8
@alloc500 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00e\00\00\00-\00\00\00" }>, align 8
@alloc245 = private unnamed_addr constant <{ [38 x i8] }> <{ [38 x i8] c" in report_release mismatch! reported=" }>, align 1
@alloc244 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc243, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [38 x i8] }>, <{ [38 x i8] }>* @alloc245, i32 0, i32 0, i32 0), [8 x i8] c"&\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc246, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc502 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc501, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00f\00\00\00\05\00\00\00" }>, align 8

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
  store [0 x i8]* bitcast (<{ [42 x i8] }>* @alloc474 to [0 x i8]*), [0 x i8]** %1, align 8
  %2 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 0, i32 1
  store i64 42, i64* %2, align 8
  %3 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 1
  store %"core::panic::location::Location"* bitcast (<{ i8*, [16 x i8] }>* @alloc473 to %"core::panic::location::Location"*), %"core::panic::location::Location"** %3, align 8
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

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h42a39cd9ab169dceE"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* nocapture readonly %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0.i.i = alloca %"std::sync::mutex::Mutex<i64>", align 8
  tail call void @llvm.experimental.noalias.scope.decl(metadata !90)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !93)
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15.i.i = load i64**, i64*** %0, align 8, !alias.scope !96, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15.i.i to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !97, !noalias !96
  store i64* null, i64** %_15.i.i, align 8, !alias.scope !97, !noalias !96
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20.i.i = bitcast %"std::sync::mutex::Mutex<i64>"* %_5.sroa.0.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !96
  tail call void @llvm.experimental.noalias.scope.decl(metadata !104)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !107)
  %_8.i.i.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"**
  %_9.i.i.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %_8.i.i.i.i, align 8, !alias.scope !110, !noalias !111, !nonnull !85, !align !86, !noundef !85
  %_3.i.i.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* %_9.i.i.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !114, !noalias !117
  store i64* null, i64** %_3.i.i.i.i, align 8, !alias.scope !114, !noalias !117
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i"

bb2.i.i.i.i:                                      ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !117
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<i64>"*)*
  call void %7(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %_5.sroa.0.i.i), !noalias !118
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"**, %"core::option::Option<std::sync::mutex::Mutex<i64>>"*** %8, align 8, !alias.scope !96, !nonnull !85, !align !86, !noundef !85
  %_17.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16.i.i, align 8, !noalias !96
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17.i.i, i64 0, i32 0
  %_2.i16.i.i = load i64, i64* %9, align 8, !range !119, !noalias !96, !noundef !85
  %10 = icmp eq i64 %_2.i16.i.i, 0
  br i1 %10, label %_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17.i.i, i64 0, i32 1
  %12 = bitcast [2 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %bb2.i.bb9_crit_edge.i.i unwind label %cleanup.i.i, !noalias !96

bb2.i.bb9_crit_edge.i.i:                          ; preds = %bb2.i.i.i
  %_22.pre.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16.i.i, align 8, !noalias !96
  br label %_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E.exit

cleanup.i.i:                                      ; preds = %bb2.i.i.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %_20.i.i = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16.i.i, align 8, !noalias !96
  %_10.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_20.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx.i.i, align 8, !noalias !96
  %_10.sroa.5.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_20.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast.i.i = bitcast [2 x i64]* %_10.sroa.5.0..sroa_idx.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(16) %_10.sroa.5.0..sroa_cast.i.i, i8* noundef nonnull align 8 dereferenceable(16) %_5.sroa.0.0.sroa_cast20.i.i, i64 16, i1 false), !noalias !96
  resume { i8*, i32 } %13

_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E.exit: ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i", %bb2.i.bb9_crit_edge.i.i
  %_22.i.i = phi %"core::option::Option<std::sync::mutex::Mutex<i64>>"* [ %_22.pre.i.i, %bb2.i.bb9_crit_edge.i.i ], [ %_17.i.i, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit.i.i" ]
  %_10.sroa.0.0..sroa_idx2.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_22.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx2.i.i, align 8, !noalias !96
  %_10.sroa.5.0..sroa_idx6.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_22.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast7.i.i = bitcast [2 x i64]* %_10.sroa.5.0..sroa_idx6.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(16) %_10.sroa.5.0..sroa_cast7.i.i, i8* noundef nonnull align 8 dereferenceable(16) %_5.sroa.0.0.sroa_cast20.i.i, i64 16, i1 false), !noalias !96
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !96
  ret i1 true
}

; core::ops::function::FnOnce::call_once{{vtable.shim}}
; Function Attrs: inlinehint nonlazybind uwtable
define internal noundef zeroext i1 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h8d298f77ff4ec3b3E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* nocapture readonly %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0.i.i = alloca %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", align 8
  tail call void @llvm.experimental.noalias.scope.decl(metadata !120)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !123)
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15.i.i = load i64**, i64*** %0, align 8, !alias.scope !126, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15.i.i to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !127, !noalias !126
  store i64* null, i64** %_15.i.i, align 8, !alias.scope !127, !noalias !126
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20.i.i = bitcast %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_5.sroa.0.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !126
  tail call void @llvm.experimental.noalias.scope.decl(metadata !134)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !137)
  %_8.i.i.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**
  %_9.i.i.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_8.i.i.i.i, align 8, !alias.scope !140, !noalias !141, !nonnull !85, !align !86, !noundef !85
  %_3.i.i.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_9.i.i.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !144, !noalias !147
  store i64* null, i64** %_3.i.i.i.i, align 8, !alias.scope !144, !noalias !147
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i"

bb2.i.i.i.i:                                      ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !147
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0.i.i), !noalias !148
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !alias.scope !126, !nonnull !85, !align !86, !noundef !85
  %_17.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !126
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 0
  %_2.i16.i.i = load i64, i64* %9, align 8, !range !119, !noalias !126, !noundef !85
  %10 = icmp eq i64 %_2.i16.i.i, 0
  br i1 %10, label %_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1
  %12 = bitcast [7 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i" unwind label %cleanup.i.i.i.i, !noalias !126

cleanup.i.i.i.i:                                  ; preds = %bb2.i.i.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %14 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 1
  %15 = bitcast i64* %14 to %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*
; call core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h1eb938b370d22c57E"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #25, !noalias !126
  %_20.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !126
  %_10.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx.i.i, align 8, !noalias !126
  %_10.sroa.5.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast.i.i = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast.i.i, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20.i.i, i64 56, i1 false), !noalias !126
  resume { i8*, i32 } %13

"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i": ; preds = %bb2.i.i.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 1
  %17 = bitcast i64* %16 to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*
; call core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
  tail call fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %17) #24, !noalias !126
  %_22.pre.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !126
  br label %_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E.exit

_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E.exit: ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i", %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i"
  %_22.i.i = phi %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* [ %_22.pre.i.i, %"_ZN4core3ptr126drop_in_place$LT$std..sync..mutex..Mutex$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17habcaf67af3fc0740E.exit.i.i.i" ], [ %_17.i.i, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit.i.i" ]
  %_10.sroa.0.0..sroa_idx2.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_22.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx2.i.i, align 8, !noalias !126
  %_10.sroa.5.0..sroa_idx6.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_22.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast7.i.i = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx6.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast7.i.i, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20.i.i, i64 56, i1 false), !noalias !126
  call void @llvm.lifetime.end.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20.i.i), !noalias !126
  ret i1 true
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17h792230541602dafdE(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i.i.i.i.i.i = alloca %"std::thread::local::AccessError", align 1
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  %_2.i = alloca %"std::collections::hash::map::HashMap<i64, ObjectInfo>", align 16
  tail call void @llvm.experimental.noalias.scope.decl(metadata !149)
  %1 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1), !noalias !149
  tail call void @llvm.experimental.noalias.scope.decl(metadata !152)
  %_2.i.i.i.i.i.i.i.i.i.i = load i64, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 0), align 8, !range !119, !noalias !155, !noundef !85
  %trunc.not.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_2.i.i.i.i.i.i.i.i.i.i, 0
  br i1 %trunc.not.i.i.i.i.i.i.i.i.i.i, label %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"

_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i: ; preds = %start
; call std::thread::local::fast::Key<T>::try_initialize
  %2 = tail call fastcc noundef align 8 dereferenceable_or_null(16) i64* @"_ZN3std6thread5local4fast12Key$LT$T$GT$14try_initialize17hd4e535fd74b46a6dE"(i64* noalias noundef align 8 dereferenceable_or_null(24) null), !noalias !162
  %3 = icmp eq i64* %2, null
  br i1 %3, label %bb1.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"

bb1.i.i.i.i.i.i:                                  ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i
  %4 = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 0, i8* nonnull %4), !noalias !163
  %_6.0.i.i.i.i.i.i = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [70 x i8] }>* @alloc377 to [0 x i8]*), i64 70, {}* noundef nonnull align 1 %_6.0.i.i.i.i.i.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.3 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc379 to %"core::panic::location::Location"*)) #23, !noalias !163
  unreachable

"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i": ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, %start
  %.0.i.i2.i.i.i.i.i.i = phi i64* [ %2, %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i ], [ getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 0), %start ]
  %5 = bitcast i64* %.0.i.i2.i.i.i.i.i.i to <2 x i64>*
  %6 = load <2 x i64>, <2 x i64>* %5, align 8, !noalias !162
  %7 = extractelement <2 x i64> %6, i64 0
  %8 = add i64 %7, 1
  store i64 %8, i64* %.0.i.i2.i.i.i.i.i.i, align 8, !alias.scope !164, !noalias !162
  %_2.sroa.7.0..sroa_idx.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 3
  %_2.sroa.7.0..sroa_idx1516.i.i.i = bitcast i64* %_2.sroa.7.0..sroa_idx.i.i.i to i8*
  call void @llvm.memset.p0i8.i64(i8* noundef nonnull align 16 dereferenceable(16) %_2.sroa.7.0..sroa_idx1516.i.i.i, i8 0, i64 16, i1 false) #24, !alias.scope !167, !noalias !149
  %9 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to <2 x i64>*
  store <2 x i64> %6, <2 x i64>* %9, align 16, !alias.scope !167, !noalias !149
  %_2.sroa.5.0..sroa_idx4.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1
  %_2.sroa.5.0..sroa_cast.i.i.i = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %_2.sroa.5.0..sroa_idx4.i.i.i to i64*
  store i64 0, i64* %_2.sroa.5.0..sroa_cast.i.i.i, align 16, !alias.scope !167, !noalias !149
  %_2.sroa.6.0..sroa_idx6.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 2
  store i8* getelementptr inbounds (<{ [16 x i8] }>, <{ [16 x i8] }>* @alloc67, i64 0, i32 0, i64 0), i8** %_2.sroa.6.0..sroa_idx6.i.i.i, align 8, !alias.scope !167, !noalias !149
  tail call void @llvm.experimental.noalias.scope.decl(metadata !170)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !173)
  %10 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %10), !noalias !175
; invoke std::sys_common::mutex::MovableMutex::new
  %11 = invoke i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E()
          to label %bb1.i.i unwind label %cleanup.i.i, !noalias !175

cleanup.i.i:                                      ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"
  %12 = landingpad { i8*, i32 }
          cleanup
  br label %bb6.i.i

bb1.i.i:                                          ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE.exit.i"
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %.0..sroa_idx.i.i, align 4, !noalias !175
; invoke std::sync::poison::Flag::new
  %13 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E.exit" unwind label %cleanup1.i.i, !noalias !175

cleanup1.i.i:                                     ; preds = %bb1.i.i
  %14 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #25
          to label %bb6.i.i unwind label %abort.i.i, !noalias !175

abort.i.i:                                        ; preds = %cleanup1.i.i
  %15 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !175
  unreachable

bb6.i.i:                                          ; preds = %cleanup1.i.i, %cleanup.i.i
  %.pn.i.i = phi { i8*, i32 } [ %14, %cleanup1.i.i ], [ %12, %cleanup.i.i ]
; call core::ptr::drop_in_place<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>
  call fastcc void @"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17hb2a8f8b98a871ef9E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %_2.i) #25, !noalias !176
  resume { i8*, i32 } %.pn.i.i

"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E.exit": ; preds = %bb1.i.i
  %_4.sroa.0.0..sroa_idx26.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 3, i32 0, i32 0
  %_4.sroa.0.0..sroa_idx2627.i.i = bitcast %"hashbrown::map::HashMap<i64, ObjectInfo, std::collections::hash::map::RandomState>"* %_4.sroa.0.0..sroa_idx26.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(48) %_4.sroa.0.0..sroa_idx2627.i.i, i8* noundef nonnull align 16 dereferenceable(48) %1, i64 48, i1 false), !alias.scope !177
  %16 = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %16, align 8, !alias.scope !176, !noalias !173
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %13, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !176, !noalias !173
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %10), !noalias !175
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1), !noalias !149
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  tail call void @llvm.experimental.noalias.scope.decl(metadata !178)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !181)
  %1 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %1), !noalias !184
; call std::sys_common::mutex::MovableMutex::new
  %2 = tail call i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E(), !noalias !184
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %.0..sroa_idx.i.i, align 4, !noalias !184
; invoke std::sync::poison::Flag::new
  %3 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit" unwind label %cleanup1.i.i, !noalias !184

cleanup1.i.i:                                     ; preds = %start
  %4 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #25
          to label %bb5.i.i unwind label %abort.i.i, !noalias !184

abort.i.i:                                        ; preds = %cleanup1.i.i
  %5 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !184
  unreachable

bb5.i.i:                                          ; preds = %cleanup1.i.i
  resume { i8*, i32 } %4

"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit": ; preds = %start
  %6 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %6, align 8, !alias.scope !184
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %3, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !184
  %7 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 3
  store i64 0, i64* %7, align 8, !alias.scope !184
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %1), !noalias !184
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !185)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !185, !nonnull !85, !align !86, !noundef !85
  %_5.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i = load i8, i8* %_5.i, align 8, !alias.scope !185
  %_5.not.i.i = icmp eq i8 %_5.val.i, 0
  br i1 %_5.not.i.i, label %bb2.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

bb2.i.i:                                          ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !185
  %_1.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i: ; preds = %bb2.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !185
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %bb5.i.i

bb5.i.i:                                          ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i
  %_6.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i monotonic, align 4, !noalias !185
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i: ; preds = %bb5.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i, %bb2.i.i, %start
  %_5.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i, i32 0 release, align 4, !noalias !185
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i, label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE.exit"

bb2.i.i.i:                                        ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i
  %_2.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i), !noalias !185
  br label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE.exit"

"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, %bb2.i.i.i
  ret void
}

; core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
; Function Attrs: nonlazybind uwtable
define internal void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !188)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !188, !nonnull !85, !align !86, !noundef !85
  %_5.i.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i.i = load i8, i8* %_5.i.i, align 8, !alias.scope !188
  %_5.not.i.i.i = icmp eq i8 %_5.val.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !188
  %_1.i.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !188
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  %_6.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i monotonic, align 4, !noalias !188
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %start
  %_5.i.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i.i, i32 0 release, align 4, !noalias !188
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
  %_2.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i), !noalias !188
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !191) #24
  %1 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %0 to i64*
  %_2.i.i.i.i = load i64, i64* %1, align 8, !alias.scope !194
  %2 = icmp eq i64 %_2.i.i.i.i, 0
  br i1 %2, label %"_ZN4core3ptr125drop_in_place$LT$hashbrown..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$C$std..collections..hash..map..RandomState$GT$$GT$17h78718f5658623464E.exit", label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !197) #24
  %self.idx.i.i.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_1, i64 0, i32 0, i32 1, i32 1, i32 4
  %self.idx.val.i.i.i.i = load i64, i64* %self.idx.i.i.i.i, align 8, !alias.scope !200
  %3 = icmp eq i64 %self.idx.val.i.i.i.i, 0
  br i1 %3, label %"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i", label %bb6.i.i.i.i

"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i": ; preds = %bb2.i.i.i
  %.pre.i.i.i = add i64 %_2.i.i.i.i, 1
  br label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i"

bb6.i.i.i.i:                                      ; preds = %bb2.i.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !201) #24
  %self.idx.i.i.i.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_1, i64 0, i32 0, i32 1, i32 1, i32 2
  %self.idx.val.i.i.i.i.i = load i8*, i8** %self.idx.i.i.i.i.i, align 8, !alias.scope !204, !noalias !205
  %4 = add i64 %_2.i.i.i.i, 1
  %5 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %4
  %6 = bitcast i8* %self.idx.val.i.i.i.i.i to <16 x i8>*
  %7 = load <16 x i8>, <16 x i8>* %6, align 16, !noalias !207
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
  %14 = load <16 x i8>, <16 x i8>* %13, align 16, !noalias !214
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
  %_3.idx.val.i.i.i.i.i = load i8*, i8** %22, align 8, !noalias !200
  %23 = getelementptr i64, i64* %21, i64 -2
  %_3.idx1.val.i.i.i.i.i = load i64, i64* %23, align 8, !noalias !200
  %_4.i.i.i.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_3.idx1.val.i.i.i.i.i, 0
  br i1 %_4.i.i.i.i.i.i.i.i.i.i.i.i.i, label %bb9.i.i.i.i.backedge, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i.i.i.i.i.i.i": ; preds = %bb11.i.i.i.i
  %24 = icmp ne i8* %_3.idx.val.i.i.i.i.i, null
  tail call void @llvm.assume(i1 %24) #24
  tail call void @__rust_dealloc(i8* nonnull %_3.idx.val.i.i.i.i.i, i64 %_3.idx1.val.i.i.i.i.i, i64 1) #24, !noalias !200
  br label %bb9.i.i.i.i.backedge

bb9.i.i.i.i.backedge:                             ; preds = %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i.i.i.i.i.i.i", %bb11.i.i.i.i
  br label %bb9.i.i.i.i

"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit.i.i.i": ; preds = %bb6.i.i.i.i.i.i, %"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i"
  %.pre-phi.i.i.i = phi i64 [ %.pre.i.i.i, %"bb2._ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E.exit_crit_edge.i.i.i" ], [ %4, %bb6.i.i.i.i.i.i ]
  tail call void @llvm.experimental.noalias.scope.decl(metadata !223) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !226) #24
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
  %_17.i.i.i.i.i = load i8*, i8** %38, align 8, !alias.scope !229, !nonnull !85, !noundef !85
  %39 = sub i64 0, %ctrl_offset.i.i.i.i.i.i
  %40 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i, i64 %39
  tail call void @__rust_dealloc(i8* nonnull %40, i64 %36, i64 16) #24, !noalias !229
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
  call void @_ZN4core3fmt9Formatter12debug_struct17h65c357ef1edbbc54E(%"core::fmt::builders::DebugStruct"* noalias nocapture noundef nonnull sret(%"core::fmt::builders::DebugStruct") dereferenceable(16) %_4, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f, [0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [11 x i8] }>* @alloc442 to [0 x i8]*), i64 11)
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
  %tmp.sroa.0.0.copyload.i.i.i = load i8*, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !230
  %tmp.sroa.4.0..sroa_idx3.i.i.i = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %self, i64 0, i32 1
  %tmp.sroa.4.0.copyload.i.i.i = load i64, i64* %tmp.sroa.4.0..sroa_idx3.i.i.i, align 8, !alias.scope !230
  store i8* null, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !230
  %0 = icmp eq i8* %tmp.sroa.0.0.copyload.i.i.i, null
  br i1 %0, label %bb2, label %bb4

bb2:                                              ; preds = %start
; call std::process::abort
  tail call void @_ZN3std7process5abort17h9abe461bf20ade28E() #23
  unreachable

bb4:                                              ; preds = %start
  %1 = tail call align 8 dereferenceable_or_null(16) i8* @__rust_alloc(i64 16, i64 8) #24, !noalias !235
  %2 = icmp eq i8* %1, null
  br i1 %2, label %bb3.i.i, label %"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit"

bb3.i.i:                                          ; preds = %bb4
; call alloc::alloc::handle_alloc_error
  tail call void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64 16, i64 noundef 8) #23, !noalias !235
  unreachable

"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit": ; preds = %bb4
  %3 = bitcast i8* %1 to i8**
  store i8* %tmp.sroa.0.0.copyload.i.i.i, i8** %3, align 8, !noalias !235
  %4 = getelementptr inbounds i8, i8* %1, i64 8
  %5 = bitcast i8* %4 to i64*
  store i64 %tmp.sroa.4.0.copyload.i.i.i, i64* %5, align 8, !noalias !235
  %_13.0.cast = bitcast i8* %1 to {}*
  %6 = insertvalue { {}*, [3 x i64]* } undef, {}* %_13.0.cast, 0
  %7 = insertvalue { {}*, [3 x i64]* } %6, [3 x i64]* bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.8 to [3 x i64]*), 1
  ret { {}*, [3 x i64]* } %7
}

; hashbrown::raw::RawTable<T,A>::reserve_rehash
; Function Attrs: cold noinline nonlazybind uwtable
define internal fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h37880d6025255f2aE"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias nocapture noundef align 8 dereferenceable(32) %self, i64* noalias noundef readonly align 8 dereferenceable(16) %0) unnamed_addr #11 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !238)
  %1 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 4
  %_9.i = load i64, i64* %1, align 8, !alias.scope !238
  %2 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %_9.i, i64 1) #24
  %3 = extractvalue { i64, i1 } %2, 0
  %4 = extractvalue { i64, i1 } %2, 1
  br i1 %4, label %bb2.i, label %bb4.i

bb2.i:                                            ; preds = %start
; call hashbrown::raw::Fallibility::capacity_overflow
  %5 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !238
  %_13.0.i = extractvalue { i64, i64 } %5, 0
  %_13.1.i = extractvalue { i64, i64 } %5, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb4.i:                                            ; preds = %start
  %6 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self to i64*
  %_16.i = load i64, i64* %6, align 8, !alias.scope !238
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !241)
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
  %12 = tail call i64 @llvm.ctlz.i64(i64 %p.i.i.i.i.i.i.i, i1 true) #24, !range !244
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
  %20 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !245
  br label %bb5.i.i

bb4.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i
  %21 = extractvalue { i64, i1 } %18, 0
  %22 = icmp eq i64 %21, 0
  br i1 %22, label %bb13.i.i.i.i, label %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i

_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i: ; preds = %bb4.i.i.i.i.i
  %23 = tail call align 16 i8* @__rust_alloc(i64 %21, i64 16) #24, !noalias !245
  %24 = icmp eq i8* %23, null
  br i1 %24, label %bb15.i.i.i.i.i, label %bb13.i.i.i.i

bb15.i.i.i.i.i:                                   ; preds = %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
; call hashbrown::raw::Fallibility::alloc_err
  %25 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility9alloc_err17h3f1a17e1376e6326E(i1 noundef zeroext true, i64 %21, i64 noundef 16), !noalias !245
  br label %bb5.i.i

bb9.i.i.i.i:                                      ; preds = %bb5.i.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %26 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !252
  br label %bb5.i.i

bb13.i.i.i.i:                                     ; preds = %bb4.i.i.i.i.i, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
  %.sroa.0.0.i.i.i.i.i.i.i.i3 = phi i8* [ %23, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i ], [ inttoptr (i64 16 to i8*), %bb4.i.i.i.i.i ]
  %27 = getelementptr inbounds i8, i8* %.sroa.0.0.i.i.i.i.i.i.i.i3, i64 %ctrl_offset.i.i.i.i.i.i
  %_42.i.i.i.i.i = add nsw i64 %.sroa.4.0.i.ph.i.i.i.i, -1
  %_2.i.i10.i.i.i.i = icmp ult i64 %_42.i.i.i.i.i, 8
  %_4.i.i.i.i.i.i = lshr i64 %.sroa.4.0.i.ph.i.i.i.i, 3
  %28 = mul nuw nsw i64 %_4.i.i.i.i.i.i, 7
  %.0.i.i.i.i.i.i = select i1 %_2.i.i10.i.i.i.i, i64 %_42.i.i.i.i.i, i64 %28
  tail call void @llvm.memset.p0i8.i64(i8* nonnull align 16 %27, i8 -1, i64 %_31.i.i.i.i.i.i, i1 false) #24, !noalias !255
  %29 = sub i64 %.0.i.i.i.i.i.i, %_9.i
  %.not.i.i = icmp eq i64 %_5.i.i, 0
  %30 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %a.i.i.sroa.4.0.copyload.pre.i.i = load i8*, i8** %30, align 8, !alias.scope !256
  br i1 %.not.i.i, label %bb26.thread.i.i, label %bb15.lr.ph.i.i

bb5.i.i:                                          ; preds = %bb9.i.i.i.i, %bb15.i.i.i.i.i, %bb2.i.i.i.i.i
  %.pn.i.pn.i.i.i = phi { i64, i64 } [ %26, %bb9.i.i.i.i ], [ %25, %bb15.i.i.i.i.i ], [ %20, %bb2.i.i.i.i.i ]
  %_7.sroa.7.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 0
  %_7.sroa.13.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb26.thread.i.i:                                  ; preds = %bb13.i.i.i.i
  %31 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !257
  store i8* %27, i8** %30, align 8, !alias.scope !257
  store i64 %29, i64* %31, align 8, !alias.scope !257
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
  %_29.i.i = load i8, i8* %43, align 1, !noalias !256
  %44 = icmp sgt i8 %_29.i.i, -1
  br i1 %44, label %bb18.i.i, label %bb9.backedge.i.i

bb9.backedge.i.i:                                 ; preds = %bb22.i.i, %bb15.i.i
  %exitcond.not.i.i = icmp eq i64 %iter.sroa.0.0100.i.i, %_16.i
  br i1 %exitcond.not.i.i, label %bb26.i.i, label %bb15.i.i

bb18.i.i:                                         ; preds = %bb15.i.i
  %45 = sub i64 0, %iter.sroa.0.0100.i.i
  %46 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %table.idx.val4.i.cast.i.i, i64 %45, i32 0
  %47 = getelementptr inbounds i64, i64* %46, i64 -7
  %_7.idx.val.i.i.i = load i64, i64* %47, align 8, !alias.scope !264, !noalias !267
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
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %118, align 1, !noalias !271
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
  %_23.i.i.i.i = load i8, i8* %122, align 1, !noalias !278
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
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %128, align 1, !noalias !271
  %129 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %130 = bitcast <16 x i1> %129 to i16
  %.not.i.i.i.i = icmp eq i16 %130, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i8.i.i
  %131 = load <16 x i8>, <16 x i8>* %32, align 16, !noalias !279
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
  store i8 %135, i8* %137, align 1, !noalias !284
  %138 = getelementptr inbounds i8, i8* %27, i64 %index2.i.i.i.i.i
  store i8 %135, i8* %138, align 1, !noalias !284
  %_12.neg.i.i.i = xor i64 %iter.sroa.0.0100.i.i, -1
  %_11.neg.i.i.i = mul i64 %_12.neg.i.i.i, 56
  %139 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %_11.neg.i.i.i
  %_12.neg.i10.i.i = xor i64 %.0.i.i.i.i, -1
  %_11.neg.i11.i.i = mul i64 %_12.neg.i10.i.i, 56
  %140 = getelementptr inbounds i8, i8* %27, i64 %_11.neg.i11.i.i
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %140, i8* noundef nonnull align 1 dereferenceable(56) %139, i64 56, i1 false) #24, !noalias !256
  br label %bb9.backedge.i.i

bb26.i.i:                                         ; preds = %bb9.backedge.i.i
  %141 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !289
  store i8* %27, i8** %30, align 8, !alias.scope !289
  store i64 %29, i64* %141, align 8, !alias.scope !289
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
  tail call void @__rust_dealloc(i8* nonnull %158, i64 %154, i64 16) #24, !noalias !292
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb7.i:                                            ; preds = %bb4.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !299)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !302)
  %159 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %self.idx12.val.i.i.i = load i8*, i8** %159, align 8, !alias.scope !305
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
  %169 = load <16 x i8>, <16 x i8>* %168, align 16, !noalias !306
  %.lobit.i.i.i.i = ashr <16 x i8> %169, <i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7>
  %170 = bitcast <16 x i8> %.lobit.i.i.i.i to <2 x i64>
  %171 = or <2 x i64> %170, <i64 -9187201950435737472, i64 -9187201950435737472>
  store <2 x i64> %171, <2 x i64>* %167, align 16, !noalias !311
  br label %bb4.i.i.i

bb5.thread.i.i:                                   ; preds = %bb8.i.i.i
  %172 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_5.i.i
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(16) %172, i8* noundef nonnull align 1 dereferenceable(16) %self.idx12.val.i.i.i, i64 16, i1 false) #24, !noalias !305
  br label %bb12.lr.ph.i.i

bb5.i2.i:                                         ; preds = %bb8.i.i.i
  %173 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 16
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* nonnull align 1 %173, i8* align 1 %self.idx12.val.i.i.i, i64 %_5.i.i, i1 false) #24, !noalias !305
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
  %_23.i.i = load i8, i8* %186, align 1, !noalias !314
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
  %_7.idx.val.i.i7.i = load i64, i64* %201, align 8, !alias.scope !315, !noalias !318
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
  %.0.copyload.i2122.i.i.i = load <16 x i8>, <16 x i8>* %272, align 1, !noalias !322
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
  %_23.i.i.i = load i8, i8* %276, align 1, !noalias !327
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
  %.0.copyload.i21.i.i.i = load <16 x i8>, <16 x i8>* %282, align 1, !noalias !322
  %283 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i, zeroinitializer
  %284 = bitcast <16 x i1> %283 to i16
  %.not.i.i.i = icmp eq i16 %284, 0
  br i1 %.not.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb11.i.i.i:                                       ; preds = %bb7.i.i.i
  %285 = load <16 x i8>, <16 x i8>* %183, align 16, !noalias !328
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
  store i8 %293, i8* %295, align 1, !noalias !333
  %296 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i
  store i8 %293, i8* %296, align 1, !noalias !333
  br label %bb40.i.i

bb31.i.i:                                         ; preds = %bb24.i.i
  %297 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %.0.i.i.i
  %prev_ctrl.i.i.i = load i8, i8* %297, align 1, !noalias !338
  %top7.i.i.i.i15.i = lshr i64 %270, 57
  %298 = trunc i64 %top7.i.i.i.i15.i to i8
  %299 = add i64 %.0.i.i.i, -16
  %_5.i.i.i.i16.i = and i64 %299, %_16.i
  %index2.i.i.i.i17.i = add i64 %_5.i.i.i.i16.i, 16
  store i8 %298, i8* %297, align 1, !noalias !341
  %300 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i17.i
  store i8 %298, i8* %300, align 1, !noalias !341
  %_73.i.i = icmp eq i8 %prev_ctrl.i.i.i, -1
  br i1 %_73.i.i, label %bb34.i.i, label %vector.body

vector.body:                                      ; preds = %bb31.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !346) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !349) #24
  %wide.load = load <16 x i8>, <16 x i8>* %189, align 1, !alias.scope !346, !noalias !351
  %301 = bitcast i8* %289 to <16 x i8>*
  %wide.load34 = load <16 x i8>, <16 x i8>* %301, align 1, !alias.scope !349, !noalias !352
  store <16 x i8> %wide.load34, <16 x i8>* %190, align 1, !alias.scope !346, !noalias !351
  %302 = bitcast i8* %289 to <16 x i8>*
  store <16 x i8> %wide.load, <16 x i8>* %302, align 1, !alias.scope !349, !noalias !352
  %303 = getelementptr inbounds i8, i8* %289, i64 16
  tail call void @llvm.experimental.noalias.scope.decl(metadata !353) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !355) #24
  %wide.load.1 = load <16 x i8>, <16 x i8>* %192, align 1, !alias.scope !353, !noalias !357
  %304 = bitcast i8* %303 to <16 x i8>*
  %wide.load34.1 = load <16 x i8>, <16 x i8>* %304, align 1, !alias.scope !355, !noalias !358
  store <16 x i8> %wide.load34.1, <16 x i8>* %193, align 1, !alias.scope !353, !noalias !357
  %305 = bitcast i8* %303 to <16 x i8>*
  store <16 x i8> %wide.load.1, <16 x i8>* %305, align 1, !alias.scope !355, !noalias !358
  %306 = getelementptr inbounds i8, i8* %289, i64 32
  tail call void @llvm.experimental.noalias.scope.decl(metadata !359) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !361) #24
  %wide.load.2 = load <16 x i8>, <16 x i8>* %195, align 1, !alias.scope !359, !noalias !363
  %307 = bitcast i8* %306 to <16 x i8>*
  %wide.load34.2 = load <16 x i8>, <16 x i8>* %307, align 1, !alias.scope !361, !noalias !364
  store <16 x i8> %wide.load34.2, <16 x i8>* %196, align 1, !alias.scope !359, !noalias !363
  %308 = bitcast i8* %306 to <16 x i8>*
  store <16 x i8> %wide.load.2, <16 x i8>* %308, align 1, !alias.scope !361, !noalias !364
  %309 = getelementptr inbounds i8, i8* %289, i64 48
  tail call void @llvm.experimental.noalias.scope.decl(metadata !346) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !349) #24
  %wide.load37 = load <8 x i8>, <8 x i8>* %198, align 1, !alias.scope !346, !noalias !351
  %310 = bitcast i8* %309 to <8 x i8>*
  %wide.load38 = load <8 x i8>, <8 x i8>* %310, align 1, !alias.scope !349, !noalias !352
  store <8 x i8> %wide.load38, <8 x i8>* %199, align 1, !alias.scope !346, !noalias !351
  %311 = bitcast i8* %309 to <8 x i8>*
  store <8 x i8> %wide.load37, <8 x i8>* %311, align 1, !alias.scope !349, !noalias !352
  br label %bb19.i.i, !llvm.loop !365

bb34.i.i:                                         ; preds = %bb31.i.i
  %312 = add i64 %iter.sroa.0.030.i.i, -16
  %_5.i.i.i = and i64 %312, %_16.i
  %index2.i.i.i = add i64 %_5.i.i.i, 16
  %313 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.030.i.i
  store i8 -1, i8* %313, align 1, !noalias !368
  %314 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i
  store i8 -1, i8* %314, align 1, !noalias !368
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(56) %289, i8* noundef nonnull align 1 dereferenceable(56) %187, i64 56, i1 false) #24, !noalias !314
  br label %bb40.i.i

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i": ; preds = %bb40.i.i, %bb5.i2.i
  %315 = phi i64 [ 0, %bb5.i2.i ], [ %.0.i.i, %bb40.i.i ]
  %316 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  %317 = sub i64 %315, %_9.i
  store i64 %317, i64* %316, align 8, !alias.scope !314
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
  %2 = load i64, i64* %1, align 8, !alias.scope !371
  store i64* null, i64** %_15, align 8, !alias.scope !371
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<i64>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !378)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !381)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %_8.i.i, align 8, !alias.scope !384, !noalias !385, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !388, !noalias !391
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !388, !noalias !391
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !391
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<i64>"*)*
  call void %7(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %_5.sroa.0), !noalias !378
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16 = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"**, %"core::option::Option<std::sync::mutex::Mutex<i64>>"*** %8, align 8, !nonnull !85, !align !86, !noundef !85
  %_17 = load %"core::option::Option<std::sync::mutex::Mutex<i64>>"*, %"core::option::Option<std::sync::mutex::Mutex<i64>>"** %_16, align 8
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<i64>>", %"core::option::Option<std::sync::mutex::Mutex<i64>>"* %_17, i64 0, i32 0
  %_2.i16 = load i64, i64* %9, align 8, !range !119, !noundef !85
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
  %2 = load i64, i64* %1, align 8, !alias.scope !392
  store i64* null, i64** %_15, align 8, !alias.scope !392
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !399)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !402)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_8.i.i, align 8, !alias.scope !405, !noalias !406, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !409, !noalias !412
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !409, !noalias !412
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !412
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0), !noalias !399
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !nonnull !85, !align !86, !noundef !85
  %_17 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 0
  %_2.i16 = load i64, i64* %9, align 8, !range !119, !noundef !85
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
  %e.i41 = alloca { i64*, i8 }, align 8
  %this.i.i27 = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %e.i21 = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, align 8
  %e.i = alloca %"core::str::error::Utf8Error", align 8
  %object_table = alloca { i64*, i8 }, align 8
  %guard = alloca { i64*, i8 }, align 8
  %_10 = alloca %"core::fmt::Arguments", align 8
  %name_c_str = alloca %"core::result::Result<&str, core::str::error::Utf8Error>", align 8
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc71 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %2, align 8, !alias.scope !413, !noalias !416
  %3 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 0, i32 1
  store i64 1, i64* %3, align 8, !alias.scope !413, !noalias !416
  %4 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 1, i32 0
  store i64* null, i64** %4, align 8, !alias.scope !413, !noalias !416
  %5 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{}>* @alloc73 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %5, align 8, !alias.scope !413, !noalias !416
  %6 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 2, i32 1
  store i64 0, i64* %6, align 8, !alias.scope !413, !noalias !416
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_10)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1)
  %_18.sroa.4.0..sroa_idx74 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1
  %_18.sroa.4.0..sroa_cast = bitcast [2 x i64]* %_18.sroa.4.0..sroa_idx74 to [0 x i8]**
  %_18.sroa.4.0.copyload = load [0 x i8]*, [0 x i8]** %_18.sroa.4.0..sroa_cast, align 8
  %_18.sroa.6.0..sroa_idx76 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1, i64 1
  %_18.sroa.6.0.copyload = load i64, i64* %_18.sroa.6.0..sroa_idx76, align 8
  %7 = bitcast %"core::str::error::Utf8Error"* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %7), !noalias !419
  %_18.sroa.4.8..sroa_cast = bitcast %"core::str::error::Utf8Error"* %e.i to [0 x i8]**
  store [0 x i8]* %_18.sroa.4.0.copyload, [0 x i8]** %_18.sroa.4.8..sroa_cast, align 8
  %_18.sroa.6.8..sroa_idx78 = getelementptr inbounds %"core::str::error::Utf8Error", %"core::str::error::Utf8Error"* %e.i, i64 0, i32 1
  %_18.sroa.6.8..sroa_cast = bitcast { i8, i8 }* %_18.sroa.6.8..sroa_idx78 to i64*
  store i64 %_18.sroa.6.0.copyload, i64* %_18.sroa.6.8..sroa_cast, align 8
  %_6.0.i = bitcast %"core::str::error::Utf8Error"* %e.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc430 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.5 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc478 to %"core::panic::location::Location"*)) #23, !noalias !419
  unreachable

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit": ; preds = %start
  %_18.sroa.4.0..sroa_idx74103 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1
  %_18.sroa.4.0..sroa_cast104 = bitcast [2 x i64]* %_18.sroa.4.0..sroa_idx74103 to [0 x i8]**
  %_18.sroa.4.0.copyload105 = load [0 x i8]*, [0 x i8]** %_18.sroa.4.0..sroa_cast104, align 8, !nonnull !85
  %_18.sroa.6.0..sroa_idx76106 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1, i64 1
  %_18.sroa.6.0.copyload107 = load i64, i64* %_18.sroa.6.0..sroa_idx76106, align 8
  %8 = bitcast { i64*, i8 }* %guard to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %8)
  %9 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %9)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i, align 8
  %10 = load atomic i64, i64* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to i64*) acquire, align 8, !noalias !422
  %11 = icmp eq i64 %10, 2
  br i1 %11, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit"
  %12 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h1ed77e854a4795c8E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %12)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit": ; preds = %"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit", %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %9)
  %13 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !427
  %14 = extractvalue { i32, i1 } %13, 1
  br i1 %14, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !427
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
  %15 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !430
  %_1.i.i.i.i.i.i = and i64 %15, 9223372036854775807
  %16 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %16, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %17 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !430
  %phi.bo.i.i.i.i.i = xor i1 %17, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %18 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !430
  %.not113 = icmp eq i8 %18, 0
  br i1 %.not113, label %bb13, label %bb1.i26

bb1.i26:                                          ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %19 = bitcast { i64*, i8 }* %e.i21 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %19), !noalias !433
  %20 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i21, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %20, align 8, !noalias !433
  %21 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i21, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %21, align 8, !noalias !433
  %_6.0.i25 = bitcast { i64*, i8 }* %e.i21 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc430 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i25, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc480 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !433

cleanup.i:                                        ; preds = %bb1.i26
  %22 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i21) #25
          to label %common.resume unwind label %abort.i, !noalias !433

unreachable.i:                                    ; preds = %bb1.i26
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %23 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !433
  unreachable

common.resume:                                    ; preds = %bb24, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %22, %cleanup.i ], [ %.pn18, %bb24 ]
  resume { i8*, i32 } %common.resume.op

bb24:                                             ; preds = %cleanup.i47, %cleanup, %bb23
  %.pn18 = phi { i8*, i32 } [ %.pn, %bb23 ], [ %24, %cleanup ], [ %42, %cleanup.i47 ]
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %guard) #25
          to label %common.resume unwind label %abort

cleanup:                                          ; preds = %bb2.i.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb3.i.i.i.i.i.i36, %bb3.i.i.i31, %bb3.i.i.i.i28
  %24 = landingpad { i8*, i32 }
          cleanup
  br label %bb24

bb13:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %25 = bitcast { i64*, i8 }* %guard to %"std::sync::mutex::Mutex<i64>"**
  %26 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 8) to i64*), align 8
  %27 = add i64 %26, 1
  store i64 %27, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 8) to i64*), align 8
  %28 = bitcast { i64*, i8 }* %object_table to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %28)
  %29 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i27 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %29)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i27, align 8
  %30 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to i64*) acquire, align 8, !noalias !436
  %31 = icmp eq i64 %30, 2
  br i1 %31, label %bb14, label %bb3.i.i.i.i28

bb3.i.i.i.i28:                                    ; preds = %bb13
  %32 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i27 to i64*
; invoke once_cell::imp::OnceCell<T>::initialize
  invoke fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %32)
          to label %bb14 unwind label %cleanup

bb14:                                             ; preds = %bb13, %bb3.i.i.i.i28
  %_6.i.i.i.i.i.i.i29 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i30 = icmp ne i64 %_6.i.i.i.i.i.i.i29, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i30) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %29)
  %33 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !441
  %34 = extractvalue { i32, i1 } %33, 1
  br i1 %34, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i33, label %bb3.i.i.i31

bb3.i.i.i31:                                      ; preds = %bb14
; invoke std::sys::unix::locks::futex::Mutex::lock_contended
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i33 unwind label %cleanup

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i33: ; preds = %bb3.i.i.i31, %bb14
  %35 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !444
  %_1.i.i.i.i.i.i32 = and i64 %35, 9223372036854775807
  %36 = icmp eq i64 %_1.i.i.i.i.i.i32, 0
  br i1 %36, label %bb15, label %bb3.i.i.i.i.i.i36

bb3.i.i.i.i.i.i36:                                ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i33
; invoke std::panicking::panic_count::is_zero_slow_path
  %37 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc40 unwind label %cleanup

.noexc40:                                         ; preds = %bb3.i.i.i.i.i.i36
  %phi.bo.i.i.i.i.i34 = xor i1 %37, true
  %phi.cast.i.i.i35 = zext i1 %phi.bo.i.i.i.i.i34 to i8
  br label %bb15

bb15:                                             ; preds = %.noexc40, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i33
  %.0.i.i.i.i.i.i37 = phi i8 [ %phi.cast.i.i.i35, %.noexc40 ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i33 ]
  %38 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !444
  %.not114 = icmp eq i8 %38, 0
  br i1 %.not114, label %bb16, label %bb1.i46

bb1.i46:                                          ; preds = %bb15
  %39 = bitcast { i64*, i8 }* %e.i41 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %39), !noalias !447
  %40 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i41, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %40, align 8, !noalias !447
  %41 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i41, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i37, i8* %41, align 8, !noalias !447
  %_6.0.i45 = bitcast { i64*, i8 }* %e.i41 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc430 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i45, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc482 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i48 unwind label %cleanup.i47, !noalias !451

cleanup.i47:                                      ; preds = %bb1.i46
  %42 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i41) #25
          to label %bb24 unwind label %abort.i49, !noalias !451

unreachable.i48:                                  ; preds = %bb1.i46
  unreachable

abort.i49:                                        ; preds = %cleanup.i47
  %43 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !451
  unreachable

bb16:                                             ; preds = %bb15
  %.fca.0.gep6 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep6, align 8
  %.fca.1.gep8 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i37, i8* %.fca.1.gep8, align 8
  %_6.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_18.sroa.6.0.copyload107, 0
  br i1 %_6.i.i.i.i.i.i.i.i.i.i, label %bb18, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i": ; preds = %bb16
  %44 = call align 1 i8* @__rust_alloc(i64 %_18.sroa.6.0.copyload107, i64 1) #24, !noalias !452
  %45 = icmp eq i8* %44, null
  br i1 %45, label %bb23.i.i.i.i.i.i.i.i.i.i, label %bb18

bb23.i.i.i.i.i.i.i.i.i.i:                         ; preds = %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i"
; invoke alloc::alloc::handle_alloc_error
  invoke void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64 %_18.sroa.6.0.copyload107, i64 noundef 1) #23
          to label %.noexc51 unwind label %cleanup3

.noexc51:                                         ; preds = %bb23.i.i.i.i.i.i.i.i.i.i
  unreachable

bb23:                                             ; preds = %bb21.i.i.i, %cleanup3
  %.pn = phi { i8*, i32 } [ %46, %cleanup3 ], [ %184, %bb21.i.i.i ]
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %object_table) #25
          to label %bb24 unwind label %abort

cleanup3:                                         ; preds = %bb23.i.i.i.i.i.i.i.i.i.i
  %46 = landingpad { i8*, i32 }
          cleanup
  br label %bb23

bb18:                                             ; preds = %bb16, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i"
  %.sroa.0.0.i.i.i.i.i.i.i.i.i.i = phi i8* [ inttoptr (i64 1 to i8*), %bb16 ], [ %44, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$8allocate17hc2161512132c4323E.exit.i.i.i.i.i.i.i.i.i.i" ]
  %47 = getelementptr [0 x i8], [0 x i8]* %_18.sroa.4.0.copyload105, i64 0, i64 0
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* nonnull align 1 %.sroa.0.0.i.i.i.i.i.i.i.i.i.i, i8* nonnull align 1 %47, i64 %_18.sroa.6.0.copyload107, i1 false) #24, !noalias !476
  %_37 = ptrtoint i8* %address to i64
  call void @llvm.experimental.noalias.scope.decl(metadata !477)
  call void @llvm.experimental.noalias.scope.decl(metadata !480)
  %_6.idx.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !483, !noalias !484
  %_6.idx11.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !483, !noalias !484
  %48 = xor i64 %_6.idx.val.i.i, 8317987319222330741
  %49 = xor i64 %_6.idx11.val.i.i, 7237128888997146477
  %50 = xor i64 %_6.idx.val.i.i, 7816392313619706465
  %51 = xor i64 %27, %_6.idx11.val.i.i
  %52 = xor i64 %51, 8387220255154660723
  %53 = add i64 %49, %48
  %54 = call i64 @llvm.fshl.i64(i64 %49, i64 %49, i64 13) #24
  %55 = xor i64 %53, %54
  %56 = call i64 @llvm.fshl.i64(i64 %53, i64 %53, i64 32) #24
  %57 = add i64 %52, %50
  %58 = call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 16) #24
  %59 = xor i64 %58, %57
  %60 = add i64 %59, %56
  %61 = call i64 @llvm.fshl.i64(i64 %59, i64 %59, i64 21) #24
  %62 = xor i64 %61, %60
  %63 = add i64 %55, %57
  %64 = call i64 @llvm.fshl.i64(i64 %55, i64 %55, i64 17) #24
  %65 = xor i64 %63, %64
  %66 = call i64 @llvm.fshl.i64(i64 %63, i64 %63, i64 32) #24
  %67 = xor i64 %60, %27
  %68 = xor i64 %62, 576460752303423488
  %69 = add i64 %67, %65
  %70 = call i64 @llvm.fshl.i64(i64 %65, i64 %65, i64 13) #24
  %71 = xor i64 %69, %70
  %72 = call i64 @llvm.fshl.i64(i64 %69, i64 %69, i64 32) #24
  %73 = add i64 %68, %66
  %74 = call i64 @llvm.fshl.i64(i64 %62, i64 %68, i64 16) #24
  %75 = xor i64 %74, %73
  %76 = add i64 %75, %72
  %77 = call i64 @llvm.fshl.i64(i64 %75, i64 %75, i64 21) #24
  %78 = xor i64 %77, %76
  %79 = add i64 %73, %71
  %80 = call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 17) #24
  %81 = xor i64 %79, %80
  %82 = call i64 @llvm.fshl.i64(i64 %79, i64 %79, i64 32) #24
  %83 = xor i64 %76, 576460752303423488
  %84 = xor i64 %82, 255
  %85 = add i64 %83, %81
  %86 = call i64 @llvm.fshl.i64(i64 %81, i64 %81, i64 13) #24
  %87 = xor i64 %85, %86
  %88 = call i64 @llvm.fshl.i64(i64 %85, i64 %85, i64 32) #24
  %89 = add i64 %78, %84
  %90 = call i64 @llvm.fshl.i64(i64 %78, i64 %78, i64 16) #24
  %91 = xor i64 %90, %89
  %92 = add i64 %91, %88
  %93 = call i64 @llvm.fshl.i64(i64 %91, i64 %91, i64 21) #24
  %94 = xor i64 %93, %92
  %95 = add i64 %87, %89
  %96 = call i64 @llvm.fshl.i64(i64 %87, i64 %87, i64 17) #24
  %97 = xor i64 %95, %96
  %98 = call i64 @llvm.fshl.i64(i64 %95, i64 %95, i64 32) #24
  %99 = add i64 %97, %92
  %100 = call i64 @llvm.fshl.i64(i64 %97, i64 %97, i64 13) #24
  %101 = xor i64 %100, %99
  %102 = call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 32) #24
  %103 = add i64 %94, %98
  %104 = call i64 @llvm.fshl.i64(i64 %94, i64 %94, i64 16) #24
  %105 = xor i64 %104, %103
  %106 = add i64 %105, %102
  %107 = call i64 @llvm.fshl.i64(i64 %105, i64 %105, i64 21) #24
  %108 = xor i64 %107, %106
  %109 = add i64 %101, %103
  %110 = call i64 @llvm.fshl.i64(i64 %101, i64 %101, i64 17) #24
  %111 = xor i64 %110, %109
  %112 = call i64 @llvm.fshl.i64(i64 %109, i64 %109, i64 32) #24
  %113 = add i64 %111, %106
  %114 = call i64 @llvm.fshl.i64(i64 %111, i64 %111, i64 13) #24
  %115 = xor i64 %114, %113
  %116 = add i64 %108, %112
  %117 = call i64 @llvm.fshl.i64(i64 %108, i64 %108, i64 16) #24
  %118 = xor i64 %117, %116
  %119 = call i64 @llvm.fshl.i64(i64 %118, i64 %118, i64 21) #24
  %120 = add i64 %115, %116
  %121 = call i64 @llvm.fshl.i64(i64 %115, i64 %115, i64 17) #24
  %122 = call i64 @llvm.fshl.i64(i64 %120, i64 %120, i64 32) #24
  %_17.i.i.i.i.i.i.i = xor i64 %120, %119
  %123 = xor i64 %_17.i.i.i.i.i.i.i, %121
  %124 = xor i64 %123, %122
  call void @llvm.experimental.noalias.scope.decl(metadata !489)
  call void @llvm.experimental.noalias.scope.decl(metadata !492) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !495) #24
  %top7.i.i.i.i.i.i = lshr i64 %124, 57
  %125 = trunc i64 %top7.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !498, !noalias !501
  %_3.i.i.i.i.i.i = and i64 %124, %_6.i.i.i.i.i.i
  %self.idx.val.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !503, !noalias !501
  %.0.vec.insert.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %125, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb21.i.i.i.i.i, %bb18
  %probe_seq.sroa.7.0.i.i.i.i.i = phi i64 [ 0, %bb18 ], [ %138, %bb21.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb18 ], [ %140, %bb21.i.i.i.i.i ]
  %126 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i
  %127 = bitcast i8* %126 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i = load <16 x i8>, <16 x i8>* %127, align 1, !noalias !504
  %128 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i
  %129 = bitcast <16 x i1> %128 to i16
  br label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb10.i.i.i.i.i, %bb3.i.i.i.i.i
  %iter.0.i.i.i.i.i = phi i16 [ %129, %bb3.i.i.i.i.i ], [ %_2.i.i.i.i.i.i.i, %bb10.i.i.i.i.i ]
  %130 = icmp eq i16 %iter.0.i.i.i.i.i, 0
  br i1 %130, label %bb12.i.i.i.i.i, label %bb10.i.i.i.i.i

bb12.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %131 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %132 = bitcast <16 x i1> %131 to i16
  %.not.i.i.i.i.i = icmp eq i16 %132, 0
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %bb7.i.i

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %133 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %133 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %134 = sub i64 0, %index.i.i.i.i.i
  %135 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %134, i32 0
  %136 = getelementptr inbounds i64, i64* %135, i64 -7
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %136, align 8, !noalias !507
  %137 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %27
  br i1 %137, label %bb19, label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %138 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %139 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %138
  %140 = and i64 %139, %_6.i.i.i.i.i.i
  br label %bb3.i.i.i.i.i

bb7.i.i:                                          ; preds = %bb12.i.i.i.i.i
  %141 = bitcast { i64, %ObjectInfo }* %_23.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %141), !noalias !510
  %_46.sroa.0.0..sroa_idx = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 0
  store i64 %27, i64* %_46.sroa.0.0..sroa_idx, align 8, !noalias !511
  %_46.sroa.6.0..sroa_idx137 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 1
  store i64 %_37, i64* %_46.sroa.6.0..sroa_idx137, align 8, !noalias !511
  %_46.sroa.7.0..sroa_idx142 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 2
  store i64 1, i64* %_46.sroa.7.0..sroa_idx142, align 8, !noalias !511
  %_46.sroa.8.0..sroa_idx147 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 3, i32 0, i32 0, i32 0
  store i8* %.sroa.0.0.i.i.i.i.i.i.i.i.i.i, i8** %_46.sroa.8.0..sroa_idx147, align 8, !noalias !511
  %_46.sroa.9.0..sroa_idx152 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 3, i32 0, i32 0, i32 1
  store i64 %_18.sroa.6.0.copyload107, i64* %_46.sroa.9.0..sroa_idx152, align 8, !noalias !511
  %_46.sroa.10.0..sroa_idx157 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 1, i32 3, i32 0, i32 1
  store i64 %_18.sroa.6.0.copyload107, i64* %_46.sroa.10.0..sroa_idx157, align 8, !noalias !511
  %142 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_23.i.i, i64 0, i32 0
  store i64 %27, i64* %142, align 8, !noalias !510
  call void @llvm.experimental.noalias.scope.decl(metadata !512)
  %143 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_3.i.i.i.i.i.i
  %144 = bitcast i8* %143 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %144, align 1, !noalias !515
  %145 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %146 = bitcast <16 x i1> %145 to i16
  %.not23.i.i.i.i = icmp eq i16 %146, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb17.i.i.i.i, %bb7.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb7.i.i ], [ %152, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %146, %bb7.i.i ], [ %156, %bb17.i.i.i.i ]
  %147 = call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i = zext i16 %147 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_6.i.i.i.i.i.i
  %148 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %148, align 1, !noalias !522
  %149 = icmp sgt i8 %_23.i.i.i.i, -1
  br i1 %149, label %bb11.i.i.i.i, label %bb2.i.i.i

bb17.i.i.i.i:                                     ; preds = %bb7.i.i, %bb17.i.i.i.i
  %probe_seq.sroa.0.025.i.i.i.i = phi i64 [ %152, %bb17.i.i.i.i ], [ %_3.i.i.i.i.i.i, %bb7.i.i ]
  %probe_seq.sroa.7.024.i.i.i.i = phi i64 [ %150, %bb17.i.i.i.i ], [ 0, %bb7.i.i ]
  %150 = add i64 %probe_seq.sroa.7.024.i.i.i.i, 16
  %151 = add i64 %150, %probe_seq.sroa.0.025.i.i.i.i
  %152 = and i64 %151, %_6.i.i.i.i.i.i
  %153 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %152
  %154 = bitcast i8* %153 to <16 x i8>*
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %154, align 1, !noalias !515
  %155 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %156 = bitcast <16 x i1> %155 to i16
  %.not.i.i.i.i = icmp eq i16 %156, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i.i.i
  %157 = bitcast i8* %self.idx.val.i.i.i.i.i to <16 x i8>*
  %158 = load <16 x i8>, <16 x i8>* %157, align 16, !noalias !523
  %159 = icmp slt <16 x i8> %158, zeroinitializer
  %160 = bitcast <16 x i1> %159 to i16
  %161 = call i16 @llvm.cttz.i16(i16 %160, i1 true) #24, !range !27
  %_2.i.i.i.i.i = zext i16 %161 to i64
  %.phi.trans.insert.i.i.i = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_2.i.i.i.i.i
  %old_ctrl.pre.i.i.i = load i8, i8* %.phi.trans.insert.i.i.i, align 1, !noalias !528
  br label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %bb11.i.i.i.i, %bb7.i.i.i.i
  %old_ctrl.i.i.i = phi i8 [ %old_ctrl.pre.i.i.i, %bb11.i.i.i.i ], [ %_23.i.i.i.i, %bb7.i.i.i.i ]
  %.0.i.i.i.i = phi i64 [ %_2.i.i.i.i.i, %bb11.i.i.i.i ], [ %result.i.i.i.i, %bb7.i.i.i.i ]
  %_14.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !529, !noalias !530
  %162 = icmp eq i64 %_14.i.i.i, 0
  %_2.i.i.i.i = and i8 %old_ctrl.i.i.i, 1
  %163 = icmp ne i8 %_2.i.i.i.i, 0
  %or.cond.i.i.i = select i1 %162, i1 %163, i1 false
  br i1 %or.cond.i.i.i, label %bb1.i.i.i.i, label %bb19.thread

bb1.i.i.i.i:                                      ; preds = %bb2.i.i.i
; invoke hashbrown::raw::RawTable<T,A>::reserve_rehash
  %164 = invoke fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h37880d6025255f2aE"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias noundef nonnull align 8 dereferenceable(32) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"*), i64* noalias noundef nonnull readonly align 8 dereferenceable(16) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to i64*))
          to label %bb9.i.i.i unwind label %bb21.i.i.i, !noalias !531

bb9.i.i.i:                                        ; preds = %bb1.i.i.i.i
  %.fca.1.extract.i.i.i.i = extractvalue { i64, i64 } %164, 1
  %.not.i1.i.i.i = icmp eq i64 %.fca.1.extract.i.i.i.i, -9223372036854775807
  call void @llvm.assume(i1 %.not.i1.i.i.i)
  call void @llvm.experimental.noalias.scope.decl(metadata !532)
  %_6.i.i3.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !535, !noalias !530
  %_3.i.i4.i.i.i = and i64 %_6.i.i3.i.i.i, %124
  %self.idx12.val.i6.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !538, !noalias !530
  %165 = getelementptr inbounds i8, i8* %self.idx12.val.i6.i.i.i, i64 %_3.i.i4.i.i.i
  %166 = bitcast i8* %165 to <16 x i8>*
  %.0.copyload.i2122.i7.i.i.i = load <16 x i8>, <16 x i8>* %166, align 1, !noalias !539
  %167 = icmp slt <16 x i8> %.0.copyload.i2122.i7.i.i.i, zeroinitializer
  %168 = bitcast <16 x i1> %167 to i16
  %.not23.i8.i.i.i = icmp eq i16 %168, 0
  br i1 %.not23.i8.i.i.i, label %bb17.i20.i.i.i, label %bb7.i15.i.i.i

bb7.i15.i.i.i:                                    ; preds = %bb17.i20.i.i.i, %bb9.i.i.i
  %probe_seq.sroa.0.0.lcssa.i9.i.i.i = phi i64 [ %_3.i.i4.i.i.i, %bb9.i.i.i ], [ %174, %bb17.i20.i.i.i ]
  %.lcssa.i10.i.i.i = phi i16 [ %168, %bb9.i.i.i ], [ %178, %bb17.i20.i.i.i ]
  %169 = call i16 @llvm.cttz.i16(i16 %.lcssa.i10.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i11.i.i.i = zext i16 %169 to i64
  %_17.i12.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i9.i.i.i, %_2.i.i.i11.i.i.i
  %result.i13.i.i.i = and i64 %_17.i12.i.i.i, %_6.i.i3.i.i.i
  %170 = getelementptr inbounds i8, i8* %self.idx12.val.i6.i.i.i, i64 %result.i13.i.i.i
  %_23.i14.i.i.i = load i8, i8* %170, align 1, !noalias !542
  %171 = icmp sgt i8 %_23.i14.i.i.i, -1
  br i1 %171, label %bb11.i22.i.i.i, label %bb19.thread

bb17.i20.i.i.i:                                   ; preds = %bb9.i.i.i, %bb17.i20.i.i.i
  %probe_seq.sroa.0.025.i16.i.i.i = phi i64 [ %174, %bb17.i20.i.i.i ], [ %_3.i.i4.i.i.i, %bb9.i.i.i ]
  %probe_seq.sroa.7.024.i17.i.i.i = phi i64 [ %172, %bb17.i20.i.i.i ], [ 0, %bb9.i.i.i ]
  %172 = add i64 %probe_seq.sroa.7.024.i17.i.i.i, 16
  %173 = add i64 %172, %probe_seq.sroa.0.025.i16.i.i.i
  %174 = and i64 %173, %_6.i.i3.i.i.i
  %175 = getelementptr inbounds i8, i8* %self.idx12.val.i6.i.i.i, i64 %174
  %176 = bitcast i8* %175 to <16 x i8>*
  %.0.copyload.i21.i18.i.i.i = load <16 x i8>, <16 x i8>* %176, align 1, !noalias !539
  %177 = icmp slt <16 x i8> %.0.copyload.i21.i18.i.i.i, zeroinitializer
  %178 = bitcast <16 x i1> %177 to i16
  %.not.i19.i.i.i = icmp eq i16 %178, 0
  br i1 %.not.i19.i.i.i, label %bb17.i20.i.i.i, label %bb7.i15.i.i.i

bb11.i22.i.i.i:                                   ; preds = %bb7.i15.i.i.i
  %179 = bitcast i8* %self.idx12.val.i6.i.i.i to <16 x i8>*
  %180 = load <16 x i8>, <16 x i8>* %179, align 16, !noalias !543
  %181 = icmp slt <16 x i8> %180, zeroinitializer
  %182 = bitcast <16 x i1> %181 to i16
  %183 = call i16 @llvm.cttz.i16(i16 %182, i1 true) #24, !range !27
  %_2.i.i21.i.i.i = zext i16 %183 to i64
  br label %bb19.thread

bb21.i.i.i:                                       ; preds = %bb1.i.i.i.i
  %184 = landingpad { i8*, i32 }
          cleanup
; call core::ptr::drop_in_place<(i64,fixsanitizer::ObjectInfo)>
  call fastcc void @"_ZN4core3ptr59drop_in_place$LT$$LP$i64$C$fixsanitizer..ObjectInfo$RP$$GT$17h855e18607bcfb813E"({ i64, %ObjectInfo }* nonnull %_23.i.i) #25, !noalias !548
  br label %bb23

bb19.thread:                                      ; preds = %bb2.i.i.i, %bb7.i15.i.i.i, %bb11.i22.i.i.i
  %self.idx1.val.i.i.i.i.i.i = phi i8* [ %self.idx.val.i.i.i.i.i, %bb2.i.i.i ], [ %self.idx12.val.i6.i.i.i, %bb11.i22.i.i.i ], [ %self.idx12.val.i6.i.i.i, %bb7.i15.i.i.i ]
  %_8.i.i.i.i.i.i = phi i64 [ %_6.i.i.i.i.i.i, %bb2.i.i.i ], [ %_6.i.i3.i.i.i, %bb11.i22.i.i.i ], [ %_6.i.i3.i.i.i, %bb7.i15.i.i.i ]
  %index.0.i.i.i = phi i64 [ %.0.i.i.i.i, %bb2.i.i.i ], [ %_2.i.i21.i.i.i, %bb11.i22.i.i.i ], [ %result.i13.i.i.i, %bb7.i15.i.i.i ]
  %self.idx.val28.i.i.i = bitcast i8* %self.idx1.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  call void @llvm.experimental.noalias.scope.decl(metadata !549)
  %sext.i.i.i.i = sub nsw i8 0, %_2.i.i.i.i
  %_5.neg.i.i.i.i = sext i8 %sext.i.i.i.i to i64
  %185 = add i64 %index.0.i.i.i, -16
  %_5.i.i.i.i.i.i = and i64 %185, %_8.i.i.i.i.i.i
  %index2.i.i.i.i.i.i = add i64 %_5.i.i.i.i.i.i, 16
  %186 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index.0.i.i.i
  store i8 %125, i8* %186, align 1, !noalias !552
  %187 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i
  store i8 %125, i8* %187, align 1, !noalias !552
  %188 = load <2 x i64>, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !557, !noalias !530
  %189 = insertelement <2 x i64> <i64 poison, i64 1>, i64 %_5.neg.i.i.i.i, i64 0
  %190 = add <2 x i64> %188, %189
  store <2 x i64> %190, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !557, !noalias !530
  %191 = sub i64 0, %index.0.i.i.i
  %192 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %self.idx.val28.i.i.i, i64 %191, i32 0
  %193 = getelementptr inbounds i64, i64* %192, i64 -7
  %194 = bitcast i64* %193 to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %194, i8* noundef nonnull align 8 dereferenceable(56) %141, i64 56, i1 false), !noalias !548
  call void @llvm.lifetime.end.p0i8(i64 56, i8* nonnull %141), !noalias !510
  br label %bb20

bb19:                                             ; preds = %bb10.i.i.i.i.i
  %_41.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx95 = getelementptr inbounds i64, i64* %135, i64 -3
  %_41.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast = bitcast i64* %_41.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx95 to {}**
  %_41.sroa.3.0.copyload = load {}*, {}** %_41.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast, align 8, !noalias !558
  %_41.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx97 = getelementptr inbounds i64, i64* %135, i64 -2
  %_41.sroa.5.0.copyload = load i64, i64* %_41.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx97, align 8, !noalias !558
  %_46.sroa.0.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx = getelementptr inbounds i64, i64* %135, i64 -6
  store i64 %27, i64* %_46.sroa.0.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx, align 8, !noalias !511
  %_46.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx138 = getelementptr inbounds i64, i64* %135, i64 -5
  store i64 %_37, i64* %_46.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx138, align 8, !noalias !511
  %_46.sroa.7.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx143 = getelementptr inbounds i64, i64* %135, i64 -4
  store i64 1, i64* %_46.sroa.7.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx143, align 8, !noalias !511
  %_46.sroa.8.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast = bitcast i64* %_41.sroa.3.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx95 to i8**
  store i8* %.sroa.0.0.i.i.i.i.i.i.i.i.i.i, i8** %_46.sroa.8.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_cast, align 8, !noalias !511
  store i64 %_18.sroa.6.0.copyload107, i64* %_41.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx97, align 8, !noalias !511
  %_46.sroa.10.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx158 = getelementptr inbounds i64, i64* %135, i64 -1
  store i64 %_18.sroa.6.0.copyload107, i64* %_46.sroa.10.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.sroa_idx158, align 8, !noalias !511
  %195 = icmp eq {}* %_41.sroa.3.0.copyload, null
  %_4.i.i.i.i.i.i.i55 = icmp eq i64 %_41.sroa.5.0.copyload, 0
  %or.cond = select i1 %195, i1 true, i1 %_4.i.i.i.i.i.i.i55
  br i1 %or.cond, label %bb20, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i": ; preds = %bb19
  %196 = bitcast {}* %_41.sroa.3.0.copyload to i8*
  call void @__rust_dealloc(i8* nonnull %196, i64 %_41.sroa.5.0.copyload, i64 1) #24
  br label %bb20

bb20:                                             ; preds = %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i", %bb19, %bb19.thread
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i37, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i57, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i57:                                      ; preds = %bb20
  %197 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !559
  %_1.i.i.i.i.i.i56 = and i64 %197, 9223372036854775807
  %198 = icmp eq i64 %_1.i.i.i.i.i.i56, 0
  br i1 %198, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i57
; invoke std::panicking::panic_count::is_zero_slow_path
  %199 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc58 unwind label %cleanup

.noexc58:                                         ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  br i1 %199, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %.noexc58
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !559
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %.noexc58, %bb2.i.i.i57, %bb20
  %200 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !559
  %201 = icmp eq i32 %200, 2
  br i1 %201, label %bb2.i.i.i.i, label %bb21

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; invoke std::sys::unix::locks::futex::Mutex::wake
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %bb21 unwind label %cleanup

abort:                                            ; preds = %bb24, %bb23
  %202 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26
  unreachable

bb21:                                             ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %28)
  call void @llvm.experimental.noalias.scope.decl(metadata !562)
  %_8.i.i60 = load %"std::sync::mutex::Mutex<i64>"*, %"std::sync::mutex::Mutex<i64>"** %25, align 8, !alias.scope !562, !nonnull !85, !align !86, !noundef !85
  %_5.val.i.i62 = load i8, i8* %.fca.1.gep, align 8, !alias.scope !562
  %_5.not.i.i.i63 = icmp eq i8 %_5.val.i.i62, 0
  br i1 %_5.not.i.i.i63, label %bb2.i.i.i65, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70

bb2.i.i.i65:                                      ; preds = %bb21
  %203 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !562
  %_1.i.i.i.i.i.i64 = and i64 %203, 9223372036854775807
  %204 = icmp eq i64 %_1.i.i.i.i.i.i64, 0
  br i1 %204, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66: ; preds = %bb2.i.i.i65
; call std::panicking::panic_count::is_zero_slow_path
  %205 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !562
  br i1 %205, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70, label %bb5.i.i.i68

bb5.i.i.i68:                                      ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66
  %_6.i.i.i.i67 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i60, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i67 monotonic, align 4, !noalias !562
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70: ; preds = %bb5.i.i.i68, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66, %bb2.i.i.i65, %bb21
  %_5.i.i.i.i.i69 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i60, i64 0, i32 0, i32 0, i32 0, i32 0
  %206 = atomicrmw xchg i32* %_5.i.i.i.i.i69, i32 0 release, align 4, !noalias !562
  %207 = icmp eq i32 %206, 2
  br i1 %207, label %bb2.i.i.i.i72, label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

bb2.i.i.i.i72:                                    ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70
  %_2.i.i.i71 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i60, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i71), !noalias !562
  br label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70, %bb2.i.i.i.i72
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %8)
  call void @llvm.lifetime.end.p0i8(i64 24, i8* nonnull %0)
  ret i64 %27
}

; Function Attrs: nonlazybind uwtable
define void @report_retain(i8* nocapture readnone %address, i64 %0, i64 %1) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %_84 = alloca [3 x { i8*, i64* }], align 8
  %_76 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %_50 = alloca [1 x { i8*, i64* }], align 8
  %_43 = alloca %"core::fmt::Arguments", align 8
  %object_table = alloca { i64*, i8 }, align 8
  %_26 = alloca [1 x { i8*, i64* }], align 8
  %_18 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %refcnt = alloca i64, align 8
  %obj_id = alloca i64, align 8
  store i64 %0, i64* %obj_id, align 8
  store i64 %1, i64* %refcnt, align 8
  %_9 = icmp eq i64 %1, 0
  br i1 %_9, label %bb1, label %bb4

bb4:                                              ; preds = %start
  %2 = bitcast { i64*, i8 }* %object_table to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %2)
  %3 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %3)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %4 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to i64*) acquire, align 8, !noalias !565
  %5 = icmp eq i64 %4, 2
  br i1 %5, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb4
  %6 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %6)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit": ; preds = %bb4, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3)
  %7 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !570
  %8 = extractvalue { i32, i1 } %7, 1
  br i1 %8, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !570
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
  %9 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !573
  %_1.i.i.i.i.i.i = and i64 %9, 9223372036854775807
  %10 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %10, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %11 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !573
  %phi.bo.i.i.i.i.i = xor i1 %11, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %12 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !573
  %.not = icmp eq i8 %12, 0
  br i1 %.not, label %bb8, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %13 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %13), !noalias !576
  %14 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %14, align 8, !noalias !576
  %15 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %15, align 8, !noalias !576
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc430 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc486 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !580

cleanup.i:                                        ; preds = %bb1.i
  %16 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i) #25
          to label %common.resume unwind label %abort.i, !noalias !580

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %17 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !580
  unreachable

common.resume:                                    ; preds = %cleanup, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %16, %cleanup.i ], [ %23, %cleanup ]
  resume { i8*, i32 } %common.resume.op

bb1:                                              ; preds = %start
  %18 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_18 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %18)
  %19 = bitcast [1 x { i8*, i64* }]* %_26 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %19)
  %20 = bitcast [1 x { i8*, i64* }]* %_26 to i64**
  store i64* %obj_id, i64** %20, align 8
  %21 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_26, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %21, align 8
  %_19.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_18 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc232 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_19.sroa.0.0..sroa_cast, align 8
  %_19.sroa.4.0..sroa_idx20 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 0
  store i64 2, i64* %_19.sroa.4.0..sroa_idx20, align 8
  %_19.sroa.5.0..sroa_idx22 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 1
  %_19.sroa.5.0..sroa_cast = bitcast i64* %_19.sroa.5.0..sroa_idx22 to i64**
  store i64* null, i64** %_19.sroa.5.0..sroa_cast, align 8
  %_19.sroa.626.0..sroa_idx27 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 3
  %22 = bitcast i64* %_19.sroa.626.0..sroa_idx27 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_26, [1 x { i8*, i64* }]** %22, align 8
  %_19.sroa.7.0..sroa_idx29 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 4
  store i64 1, i64* %_19.sroa.7.0..sroa_idx29, align 8
; call core::panicking::assert_failed
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc226 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_18, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc484 to %"core::panic::location::Location"*)) #23
  unreachable

cleanup:                                          ; preds = %bb1.i17, %bb21, %bb12
  %23 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %object_table) #25
          to label %common.resume unwind label %abort

bb8:                                              ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !581
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_37 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h9ac6fd78d11cfe13E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  br i1 %_37, label %bb14, label %bb12

bb12:                                             ; preds = %bb8
  %24 = bitcast %"core::fmt::Arguments"* %_43 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %24)
  %25 = bitcast [1 x { i8*, i64* }]* %_50 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %25)
  %26 = bitcast [1 x { i8*, i64* }]* %_50 to i64**
  store i64* %obj_id, i64** %26, align 8
  %27 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_50, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %27, align 8
  %28 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc171 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %28, align 8, !alias.scope !584, !noalias !587
  %29 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 0, i32 1
  store i64 2, i64* %29, align 8, !alias.scope !584, !noalias !587
  %30 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 1, i32 0
  store i64* null, i64** %30, align 8, !alias.scope !584, !noalias !587
  %31 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 0
  %32 = bitcast [0 x { i8*, i64* }]** %31 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_50, [1 x { i8*, i64* }]** %32, align 8, !alias.scope !584, !noalias !587
  %33 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 1
  store i64 1, i64* %33, align 8, !alias.scope !584, !noalias !587
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_43, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc488 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb21, %bb12
  unreachable

bb14:                                             ; preds = %bb8
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_55 = call fastcc noundef align 8 dereferenceable_or_null(48) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h1fd66babc11d9351E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  %34 = icmp eq i64* %_55, null
  br i1 %34, label %bb1.i17, label %bb16

bb1.i17:                                          ; preds = %bb14
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc490 to %"core::panic::location::Location"*)) #23
          to label %.noexc unwind label %cleanup

.noexc:                                           ; preds = %bb1.i17
  unreachable

bb16:                                             ; preds = %bb14
  %35 = getelementptr inbounds i64, i64* %_55, i64 2
  %_68 = load i64, i64* %35, align 8
  %_69 = load i64, i64* %refcnt, align 8
  %_67.not = icmp eq i64 %_68, %_69
  br i1 %_67.not, label %bb22, label %bb21

bb22:                                             ; preds = %bb16
  %36 = add i64 %_68, 1
  store i64 %36, i64* %35, align 8
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb22
  %37 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !590
  %_1.i.i.i.i.i.i18 = and i64 %37, 9223372036854775807
  %38 = icmp eq i64 %_1.i.i.i.i.i.i18, 0
  br i1 %38, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %39 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !590
  br i1 %39, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !590
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb22
  %40 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !590
  %41 = icmp eq i32 %40, 2
  br i1 %41, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !590
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %2)
  ret void

bb21:                                             ; preds = %bb16
  %42 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %42)
  %43 = bitcast [3 x { i8*, i64* }]* %_84 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %43)
  %44 = bitcast [3 x { i8*, i64* }]* %_84 to i64**
  store i64* %obj_id, i64** %44, align 8
  %45 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %45, align 8
  %46 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 0
  %47 = bitcast i8** %46 to i64**
  store i64* %refcnt, i64** %47, align 8
  %48 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %48, align 8
  %49 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 0
  %50 = bitcast i8** %49 to i64**
  store i64* %35, i64** %50, align 8
  %51 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %51, align 8
  %_77.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc178 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_77.sroa.0.0..sroa_cast, align 8
  %_77.sroa.4.0..sroa_idx36 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 0
  store i64 3, i64* %_77.sroa.4.0..sroa_idx36, align 8
  %_77.sroa.5.0..sroa_idx38 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 1
  %_77.sroa.5.0..sroa_cast = bitcast i64* %_77.sroa.5.0..sroa_idx38 to i64**
  store i64* null, i64** %_77.sroa.5.0..sroa_cast, align 8
  %_77.sroa.642.0..sroa_idx43 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 3
  %52 = bitcast i64* %_77.sroa.642.0..sroa_idx43 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_84, [3 x { i8*, i64* }]** %52, align 8
  %_77.sroa.7.0..sroa_idx45 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 4
  store i64 3, i64* %_77.sroa.7.0..sroa_idx45, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %35, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_76, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc492 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

abort:                                            ; preds = %cleanup
  %53 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26
  unreachable
}

; Function Attrs: nonlazybind uwtable
define void @report_release(i8* nocapture readnone %address, i64 %0, i64 %1) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %_84 = alloca [3 x { i8*, i64* }], align 8
  %_76 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %_50 = alloca [1 x { i8*, i64* }], align 8
  %_43 = alloca %"core::fmt::Arguments", align 8
  %object_info = alloca { i64*, i8 }, align 8
  %_26 = alloca [1 x { i8*, i64* }], align 8
  %_18 = alloca %"core::option::Option<core::fmt::Arguments>", align 8
  %refcnt = alloca i64, align 8
  %obj_id = alloca i64, align 8
  store i64 %0, i64* %obj_id, align 8
  store i64 %1, i64* %refcnt, align 8
  %_9 = icmp eq i64 %1, 0
  br i1 %_9, label %bb1, label %bb4

bb4:                                              ; preds = %start
  %2 = bitcast { i64*, i8 }* %object_info to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %2)
  %3 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %3)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %4 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE to i64*) acquire, align 8, !noalias !593
  %5 = icmp eq i64 %4, 2
  br i1 %5, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb4
  %6 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h69f4c2431493b8a0E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %6)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit": ; preds = %bb4, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3)
  %7 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !598
  %8 = extractvalue { i32, i1 } %7, 1
  br i1 %8, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !598
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17hb0e82ab36dbea5e2E.exit"
  %9 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !601
  %_1.i.i.i.i.i.i = and i64 %9, 9223372036854775807
  %10 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %10, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %11 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !601
  %phi.bo.i.i.i.i.i = xor i1 %11, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %12 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !601
  %.not = icmp eq i8 %12, 0
  br i1 %.not, label %bb8, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %13 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %13), !noalias !604
  %14 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %14, align 8, !noalias !604
  %15 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %15, align 8, !noalias !604
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc430 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc496 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !608

cleanup.i:                                        ; preds = %bb1.i
  %16 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h787ca9f1f012f374E"({ i64*, i8 }* nonnull %e.i) #25
          to label %common.resume unwind label %abort.i, !noalias !608

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %17 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !608
  unreachable

common.resume:                                    ; preds = %cleanup, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %16, %cleanup.i ], [ %23, %cleanup ]
  resume { i8*, i32 } %common.resume.op

bb1:                                              ; preds = %start
  %18 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_18 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %18)
  %19 = bitcast [1 x { i8*, i64* }]* %_26 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %19)
  %20 = bitcast [1 x { i8*, i64* }]* %_26 to i64**
  store i64* %obj_id, i64** %20, align 8
  %21 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_26, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %21, align 8
  %_19.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_18 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc232 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_19.sroa.0.0..sroa_cast, align 8
  %_19.sroa.4.0..sroa_idx26 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 0
  store i64 2, i64* %_19.sroa.4.0..sroa_idx26, align 8
  %_19.sroa.5.0..sroa_idx28 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 1
  %_19.sroa.5.0..sroa_cast = bitcast i64* %_19.sroa.5.0..sroa_idx28 to i64**
  store i64* null, i64** %_19.sroa.5.0..sroa_cast, align 8
  %_19.sroa.632.0..sroa_idx33 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 3
  %22 = bitcast i64* %_19.sroa.632.0..sroa_idx33 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_26, [1 x { i8*, i64* }]** %22, align 8
  %_19.sroa.7.0..sroa_idx35 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 4
  store i64 1, i64* %_19.sroa.7.0..sroa_idx35, align 8
; call core::panicking::assert_failed
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc226 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_18, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc494 to %"core::panic::location::Location"*)) #23
  unreachable

cleanup:                                          ; preds = %bb1.i18, %bb21, %bb12
  %23 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE"({ i64*, i8 }* nonnull %object_info) #25
          to label %common.resume unwind label %abort

bb8:                                              ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !581
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_37 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h9ac6fd78d11cfe13E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  br i1 %_37, label %bb14, label %bb12

bb12:                                             ; preds = %bb8
  %24 = bitcast %"core::fmt::Arguments"* %_43 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %24)
  %25 = bitcast [1 x { i8*, i64* }]* %_50 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %25)
  %26 = bitcast [1 x { i8*, i64* }]* %_50 to i64**
  store i64* %obj_id, i64** %26, align 8
  %27 = getelementptr inbounds [1 x { i8*, i64* }], [1 x { i8*, i64* }]* %_50, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %27, align 8
  %28 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc237 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %28, align 8, !alias.scope !609, !noalias !612
  %29 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 0, i32 1
  store i64 2, i64* %29, align 8, !alias.scope !609, !noalias !612
  %30 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 1, i32 0
  store i64* null, i64** %30, align 8, !alias.scope !609, !noalias !612
  %31 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 0
  %32 = bitcast [0 x { i8*, i64* }]** %31 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_50, [1 x { i8*, i64* }]** %32, align 8, !alias.scope !609, !noalias !612
  %33 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 1
  store i64 1, i64* %33, align 8, !alias.scope !609, !noalias !612
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_43, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc498 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb21, %bb12
  unreachable

bb14:                                             ; preds = %bb8
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_55 = call fastcc noundef align 8 dereferenceable_or_null(48) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h1fd66babc11d9351E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  %34 = icmp eq i64* %_55, null
  br i1 %34, label %bb1.i18, label %bb16

bb1.i18:                                          ; preds = %bb14
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc418 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc500 to %"core::panic::location::Location"*)) #23
          to label %.noexc unwind label %cleanup

.noexc:                                           ; preds = %bb1.i18
  unreachable

bb16:                                             ; preds = %bb14
  %35 = getelementptr inbounds i64, i64* %_55, i64 2
  %_68 = load i64, i64* %35, align 8
  %_69 = load i64, i64* %refcnt, align 8
  %_67.not = icmp eq i64 %_68, %_69
  br i1 %_67.not, label %bb22, label %bb21

bb22:                                             ; preds = %bb16
  %36 = add i64 %_68, -1
  store i64 %36, i64* %35, align 8
  %37 = icmp eq i64 %36, 0
  br i1 %37, label %bb24, label %bb28

bb21:                                             ; preds = %bb16
  %38 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %38)
  %39 = bitcast [3 x { i8*, i64* }]* %_84 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %39)
  %40 = bitcast [3 x { i8*, i64* }]* %_84 to i64**
  store i64* %obj_id, i64** %40, align 8
  %41 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %41, align 8
  %42 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 0
  %43 = bitcast i8** %42 to i64**
  store i64* %refcnt, i64** %43, align 8
  %44 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %44, align 8
  %45 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 0
  %46 = bitcast i8** %45 to i64**
  store i64* %35, i64** %46, align 8
  %47 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %47, align 8
  %_77.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc244 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_77.sroa.0.0..sroa_cast, align 8
  %_77.sroa.4.0..sroa_idx42 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 0
  store i64 3, i64* %_77.sroa.4.0..sroa_idx42, align 8
  %_77.sroa.5.0..sroa_idx44 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 1
  %_77.sroa.5.0..sroa_cast = bitcast i64* %_77.sroa.5.0..sroa_idx44 to i64**
  store i64* null, i64** %_77.sroa.5.0..sroa_cast, align 8
  %_77.sroa.648.0..sroa_idx49 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 3
  %48 = bitcast i64* %_77.sroa.648.0..sroa_idx49 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_84, [3 x { i8*, i64* }]** %48, align 8
  %_77.sroa.7.0..sroa_idx51 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 4
  store i64 3, i64* %_77.sroa.7.0..sroa_idx51, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %35, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_76, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc502 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

bb28:                                             ; preds = %bb12.i.i.i.i.i.i, %bb2.i, %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i", %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i", %bb22
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb28
  %49 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !615
  %_1.i.i.i.i.i.i21 = and i64 %49, 9223372036854775807
  %50 = icmp eq i64 %_1.i.i.i.i.i.i21, 0
  br i1 %50, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %51 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !615
  br i1 %51, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !615
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb28
  %52 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !615
  %53 = icmp eq i32 %52, 2
  br i1 %53, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !615
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h29708ecb7f63c8ebE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %2)
  ret void

bb24:                                             ; preds = %bb22
  call void @llvm.experimental.noalias.scope.decl(metadata !618)
  call void @llvm.experimental.noalias.scope.decl(metadata !621) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !624) #24
  %_5.idx.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !627, !noalias !628
  %_5.idx1.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !627, !noalias !628
  %54 = xor i64 %_5.idx.val.i.i.i, 8317987319222330741
  %55 = xor i64 %_5.idx1.val.i.i.i, 7237128888997146477
  %56 = xor i64 %_5.idx.val.i.i.i, 7816392313619706465
  %57 = xor i64 %obj_id.val, %_5.idx1.val.i.i.i
  %58 = xor i64 %57, 8387220255154660723
  %59 = add i64 %55, %54
  %60 = call i64 @llvm.fshl.i64(i64 %55, i64 %55, i64 13) #24
  %61 = xor i64 %59, %60
  %62 = call i64 @llvm.fshl.i64(i64 %59, i64 %59, i64 32) #24
  %63 = add i64 %58, %56
  %64 = call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 16) #24
  %65 = xor i64 %64, %63
  %66 = add i64 %65, %62
  %67 = call i64 @llvm.fshl.i64(i64 %65, i64 %65, i64 21) #24
  %68 = xor i64 %67, %66
  %69 = add i64 %61, %63
  %70 = call i64 @llvm.fshl.i64(i64 %61, i64 %61, i64 17) #24
  %71 = xor i64 %69, %70
  %72 = call i64 @llvm.fshl.i64(i64 %69, i64 %69, i64 32) #24
  %73 = xor i64 %66, %obj_id.val
  %74 = xor i64 %68, 576460752303423488
  %75 = add i64 %73, %71
  %76 = call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 13) #24
  %77 = xor i64 %75, %76
  %78 = call i64 @llvm.fshl.i64(i64 %75, i64 %75, i64 32) #24
  %79 = add i64 %74, %72
  %80 = call i64 @llvm.fshl.i64(i64 %68, i64 %74, i64 16) #24
  %81 = xor i64 %80, %79
  %82 = add i64 %81, %78
  %83 = call i64 @llvm.fshl.i64(i64 %81, i64 %81, i64 21) #24
  %84 = xor i64 %83, %82
  %85 = add i64 %79, %77
  %86 = call i64 @llvm.fshl.i64(i64 %77, i64 %77, i64 17) #24
  %87 = xor i64 %85, %86
  %88 = call i64 @llvm.fshl.i64(i64 %85, i64 %85, i64 32) #24
  %89 = xor i64 %82, 576460752303423488
  %90 = xor i64 %88, 255
  %91 = add i64 %89, %87
  %92 = call i64 @llvm.fshl.i64(i64 %87, i64 %87, i64 13) #24
  %93 = xor i64 %91, %92
  %94 = call i64 @llvm.fshl.i64(i64 %91, i64 %91, i64 32) #24
  %95 = add i64 %84, %90
  %96 = call i64 @llvm.fshl.i64(i64 %84, i64 %84, i64 16) #24
  %97 = xor i64 %96, %95
  %98 = add i64 %97, %94
  %99 = call i64 @llvm.fshl.i64(i64 %97, i64 %97, i64 21) #24
  %100 = xor i64 %99, %98
  %101 = add i64 %93, %95
  %102 = call i64 @llvm.fshl.i64(i64 %93, i64 %93, i64 17) #24
  %103 = xor i64 %101, %102
  %104 = call i64 @llvm.fshl.i64(i64 %101, i64 %101, i64 32) #24
  %105 = add i64 %103, %98
  %106 = call i64 @llvm.fshl.i64(i64 %103, i64 %103, i64 13) #24
  %107 = xor i64 %106, %105
  %108 = call i64 @llvm.fshl.i64(i64 %105, i64 %105, i64 32) #24
  %109 = add i64 %100, %104
  %110 = call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 16) #24
  %111 = xor i64 %110, %109
  %112 = add i64 %111, %108
  %113 = call i64 @llvm.fshl.i64(i64 %111, i64 %111, i64 21) #24
  %114 = xor i64 %113, %112
  %115 = add i64 %107, %109
  %116 = call i64 @llvm.fshl.i64(i64 %107, i64 %107, i64 17) #24
  %117 = xor i64 %116, %115
  %118 = call i64 @llvm.fshl.i64(i64 %115, i64 %115, i64 32) #24
  %119 = add i64 %117, %112
  %120 = call i64 @llvm.fshl.i64(i64 %117, i64 %117, i64 13) #24
  %121 = xor i64 %120, %119
  %122 = add i64 %114, %118
  %123 = call i64 @llvm.fshl.i64(i64 %114, i64 %114, i64 16) #24
  %124 = xor i64 %123, %122
  %125 = call i64 @llvm.fshl.i64(i64 %124, i64 %124, i64 21) #24
  %126 = add i64 %121, %122
  %127 = call i64 @llvm.fshl.i64(i64 %121, i64 %121, i64 17) #24
  %128 = call i64 @llvm.fshl.i64(i64 %126, i64 %126, i64 32) #24
  %_17.i.i.i.i.i.i.i.i = xor i64 %126, %125
  %129 = xor i64 %_17.i.i.i.i.i.i.i.i, %127
  %130 = xor i64 %129, %128
  call void @llvm.experimental.noalias.scope.decl(metadata !632) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !635) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !638) #24
  %top7.i.i.i.i.i.i.i = lshr i64 %130, 57
  %131 = trunc i64 %top7.i.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i.i22 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !641, !noalias !644
  %self.idx.val.i.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !647, !noalias !644
  %.0.vec.insert.i.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %131, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i.i23

bb3.i.i.i.i.i.i23:                                ; preds = %bb21.i.i.i.i.i.i, %bb24
  %probe_seq.sroa.7.0.i.i.i.i.i.i = phi i64 [ 0, %bb24 ], [ %144, %bb21.i.i.i.i.i.i ]
  %.pn.i.i.i = phi i64 [ %130, %bb24 ], [ %145, %bb21.i.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i.i = and i64 %.pn.i.i.i, %_6.i.i.i.i.i.i.i22
  %132 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i.i
  %133 = bitcast i8* %132 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %133, align 1, !noalias !648
  %134 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i.i
  %135 = bitcast <16 x i1> %134 to i16
  br label %bb8.i.i.i.i.i.i

bb8.i.i.i.i.i.i:                                  ; preds = %bb10.i.i.i.i.i.i, %bb3.i.i.i.i.i.i23
  %iter.0.i.i.i.i.i.i = phi i16 [ %135, %bb3.i.i.i.i.i.i23 ], [ %_2.i.i.i.i.i.i.i.i, %bb10.i.i.i.i.i.i ]
  %136 = icmp eq i16 %iter.0.i.i.i.i.i.i, 0
  br i1 %136, label %bb12.i.i.i.i.i.i, label %bb10.i.i.i.i.i.i

bb12.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %137 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %138 = bitcast <16 x i1> %137 to i16
  %.not.i.i.i.i.i.i = icmp eq i16 %138, 0
  br i1 %.not.i.i.i.i.i.i, label %bb21.i.i.i.i.i.i, label %bb28

bb10.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %139 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i.i = zext i16 %139 to i64
  %_4.i.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i.i
  %_25.i.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i.i = and i64 %_25.i.i.i.i.i.i, %_6.i.i.i.i.i.i.i22
  %140 = sub i64 0, %index.i.i.i.i.i.i
  %141 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i.i, i64 %140, i32 0
  %142 = getelementptr inbounds i64, i64* %141, i64 -7
  %_6.idx.val.i.i.i.i.i.i.i = load i64, i64* %142, align 8, !noalias !651
  %143 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i.i, %obj_id.val
  br i1 %143, label %bb4.i.i.i.i, label %bb8.i.i.i.i.i.i

bb21.i.i.i.i.i.i:                                 ; preds = %bb12.i.i.i.i.i.i
  %144 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i.i, 16
  %145 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %144
  br label %bb3.i.i.i.i.i.i23

bb4.i.i.i.i:                                      ; preds = %bb10.i.i.i.i.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !654) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !657) #24
  %146 = ptrtoint i8* %self.idx.val.i.i.i.i.i.i to i64
  %147 = ptrtoint i64* %141 to i64
  %148 = sub i64 %146, %147
  %149 = sdiv exact i64 %148, 56
  call void @llvm.experimental.noalias.scope.decl(metadata !660) #24
  %150 = add nsw i64 %149, -16
  %index_before.i.i.i.i.i.i.i = and i64 %150, %_6.i.i.i.i.i.i.i22
  %151 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index_before.i.i.i.i.i.i.i
  %152 = bitcast i8* %151 to <16 x i8>*
  %.0.copyload.i17.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %152, align 1, !noalias !663
  %153 = icmp eq <16 x i8> %.0.copyload.i17.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %154 = bitcast <16 x i1> %153 to i16
  %155 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %149
  %156 = bitcast i8* %155 to <16 x i8>*
  %.0.copyload.i418.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %156, align 1, !noalias !667
  %157 = icmp eq <16 x i8> %.0.copyload.i418.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %158 = bitcast <16 x i1> %157 to i16
  %159 = call i16 @llvm.ctlz.i16(i16 %154, i1 false) #24, !range !27
  %160 = call i16 @llvm.cttz.i16(i16 %158, i1 false) #24, !range !27
  %narrow.i.i.i.i.i.i.i = add nuw nsw i16 %160, %159
  %_20.i.i.i.i.i.i.i = icmp ugt i16 %narrow.i.i.i.i.i.i.i, 15
  br i1 %_20.i.i.i.i.i.i.i, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i", label %bb11.i.i.i.i.i.i.i

bb11.i.i.i.i.i.i.i:                               ; preds = %bb4.i.i.i.i
  %161 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !670, !noalias !671
  %162 = add i64 %161, 1
  store i64 %162, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !670, !noalias !671
  br label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i"

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i": ; preds = %bb11.i.i.i.i.i.i.i, %bb4.i.i.i.i
  %.sink20.i.i.i.i.i.i.i = phi i8 [ -1, %bb11.i.i.i.i.i.i.i ], [ -128, %bb4.i.i.i.i ]
  %index2.i.i.i.i.i.i.i.i = add i64 %index_before.i.i.i.i.i.i.i, 16
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %155, align 1, !noalias !672
  %163 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i.i.i
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %163, align 1, !noalias !672
  %164 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !670, !noalias !671
  %165 = add i64 %164, -1
  store i64 %165, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h0a5fd7fdc7eef49cE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !670, !noalias !671
  %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_idx5.i.i = getelementptr inbounds i64, i64* %141, i64 -3
  %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i = bitcast i64* %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_idx5.i.i to {}**
  %_3.sroa.4.0.copyload.i.i = load {}*, {}** %_3.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i, align 8, !noalias !673
  %166 = icmp eq {}* %_3.sroa.4.0.copyload.i.i, null
  br i1 %166, label %bb28, label %bb2.i

bb2.i:                                            ; preds = %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE.exit.i.i"
  %_95.sroa.5.32._3.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i.sroa_idx = getelementptr inbounds i64, i64* %141, i64 -2
  %_95.sroa.5.32.copyload = load i64, i64* %_95.sroa.5.32._3.sroa.6.0.tmp.sroa.0.0..sroa_cast3.i.i.i.i.i.i.sroa_cast.i.i.sroa_idx, align 8, !noalias !674
  %_4.i.i.i.i.i.i.i = icmp eq i64 %_95.sroa.5.32.copyload, 0
  br i1 %_4.i.i.i.i.i.i.i, label %bb28, label %"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i"

"_ZN63_$LT$alloc..alloc..Global$u20$as$u20$core..alloc..Allocator$GT$10deallocate17h7f67acca890379e8E.exit.i.i.i.i.i.i": ; preds = %bb2.i
  %167 = bitcast {}* %_3.sroa.4.0.copyload.i.i to i8*
  call void @__rust_dealloc(i8* nonnull %167, i64 %_95.sroa.5.32.copyload, i64 1) #24
  br label %bb28

abort:                                            ; preds = %cleanup
  %168 = landingpad { i8*, i32 }
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

; Function Attrs: argmemonly mustprogress nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i64(i8* nocapture writeonly, i8, i64, i1 immarg) #17

; Function Attrs: argmemonly mustprogress nofree nounwind willreturn
declare void @llvm.memmove.p0i8.p0i8.i64(i8* nocapture writeonly, i8* nocapture readonly, i64, i1 immarg) #14

; Function Attrs: argmemonly mustprogress nofree nounwind nonlazybind readonly uwtable willreturn
declare i64 @strlen(i8* nocapture) unnamed_addr #18

; core::fmt::num::imp::<impl core::fmt::Display for i64>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E"(i64* noalias noundef readonly align 8 dereferenceable(8), %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

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
!90 = !{!91}
!91 = distinct !{!91, !92, !"_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E: %_1"}
!92 = distinct !{!92, !"_ZN4core3ops8function6FnOnce9call_once17h29f0bc10cf72e0f0E"}
!93 = !{!94}
!94 = distinct !{!94, !95, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E: %_1"}
!95 = distinct !{!95, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E"}
!96 = !{!94, !91}
!97 = !{!98, !100, !102}
!98 = distinct !{!98, !99, !"_ZN4core3mem7replace17ha318695de15894dbE: %dest"}
!99 = distinct !{!99, !"_ZN4core3mem7replace17ha318695de15894dbE"}
!100 = distinct !{!100, !101, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E: %self"}
!101 = distinct !{!101, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E"}
!102 = distinct !{!102, !103, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E: %val"}
!103 = distinct !{!103, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E"}
!104 = !{!105}
!105 = distinct !{!105, !106, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: %_1"}
!106 = distinct !{!106, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE"}
!107 = !{!108}
!108 = distinct !{!108, !109, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: %_1"}
!109 = distinct !{!109, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE"}
!110 = !{!108, !105}
!111 = !{!112, !113, !94, !91}
!112 = distinct !{!112, !109, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: argument 0"}
!113 = distinct !{!113, !106, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: argument 0"}
!114 = !{!115}
!115 = distinct !{!115, !116, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E: %dest"}
!116 = distinct !{!116, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E"}
!117 = !{!112, !108, !113, !105, !94, !91}
!118 = !{!105, !94, !91}
!119 = !{i64 0, i64 2}
!120 = !{!121}
!121 = distinct !{!121, !122, !"_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E: %_1"}
!122 = distinct !{!122, !"_ZN4core3ops8function6FnOnce9call_once17hb7d8c5c4f646cc95E"}
!123 = !{!124}
!124 = distinct !{!124, !125, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E: %_1"}
!125 = distinct !{!125, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17ha6bbd41d656cadb0E"}
!126 = !{!124, !121}
!127 = !{!128, !130, !132}
!128 = distinct !{!128, !129, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E: %dest"}
!129 = distinct !{!129, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E"}
!130 = distinct !{!130, !131, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E: %self"}
!131 = distinct !{!131, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E"}
!132 = distinct !{!132, !133, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE: %val"}
!133 = distinct !{!133, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE"}
!134 = !{!135}
!135 = distinct !{!135, !136, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: %_1"}
!136 = distinct !{!136, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE"}
!137 = !{!138}
!138 = distinct !{!138, !139, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: %_1"}
!139 = distinct !{!139, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E"}
!140 = !{!138, !135}
!141 = !{!142, !143, !124, !121}
!142 = distinct !{!142, !139, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: argument 0"}
!143 = distinct !{!143, !136, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: argument 0"}
!144 = !{!145}
!145 = distinct !{!145, !146, !"_ZN4core3mem7replace17h17668aa3bb646e28E: %dest"}
!146 = distinct !{!146, !"_ZN4core3mem7replace17h17668aa3bb646e28E"}
!147 = !{!142, !138, !143, !135, !124, !121}
!148 = !{!135, !124, !121}
!149 = !{!150}
!150 = distinct !{!150, !151, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E: argument 0"}
!151 = distinct !{!151, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h501149ac3ee65ba3E"}
!152 = !{!153}
!153 = distinct !{!153, !154, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE: argument 0"}
!154 = distinct !{!154, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h467fd19500e5bbbaE"}
!155 = !{!156, !158, !160, !153, !150}
!156 = distinct !{!156, !157, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE: %init"}
!157 = distinct !{!157, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE"}
!158 = distinct !{!158, !159, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E: %init"}
!159 = distinct !{!159, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E"}
!160 = distinct !{!160, !161, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE: argument 0"}
!161 = distinct !{!161, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE"}
!162 = !{!160, !153, !150}
!163 = !{!153, !150}
!164 = !{!165}
!165 = distinct !{!165, !166, !"_ZN4core3mem7replace17h3116444c89fcbd6bE: %dest"}
!166 = distinct !{!166, !"_ZN4core3mem7replace17h3116444c89fcbd6bE"}
!167 = !{!168, !153}
!168 = distinct !{!168, !169, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17h09e7fd16abe92fafE: argument 0"}
!169 = distinct !{!169, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17h09e7fd16abe92fafE"}
!170 = !{!171}
!171 = distinct !{!171, !172, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17h5647cc520582ff0bE: argument 0"}
!172 = distinct !{!172, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17h5647cc520582ff0bE"}
!173 = !{!174}
!174 = distinct !{!174, !172, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17h5647cc520582ff0bE: %t"}
!175 = !{!171, !174, !150}
!176 = !{!171, !150}
!177 = !{!171, !174}
!178 = !{!179}
!179 = distinct !{!179, !180, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE: argument 0"}
!180 = distinct !{!180, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE"}
!181 = !{!182}
!182 = distinct !{!182, !183, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E: argument 0"}
!183 = distinct !{!183, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E"}
!184 = !{!182, !179}
!185 = !{!186}
!186 = distinct !{!186, !187, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!187 = distinct !{!187, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!188 = !{!189}
!189 = distinct !{!189, !190, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!190 = distinct !{!190, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!191 = !{!192}
!192 = distinct !{!192, !193, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h525f6dd284d29203E: %self"}
!193 = distinct !{!193, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h525f6dd284d29203E"}
!194 = !{!195, !192}
!195 = distinct !{!195, !196, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!196 = distinct !{!196, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!197 = !{!198}
!198 = distinct !{!198, !199, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E: %self"}
!199 = distinct !{!199, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13drop_elements17he092f6f78011ec17E"}
!200 = !{!198, !192}
!201 = !{!202}
!202 = distinct !{!202, !203, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4iter17hea862c4ee711fef1E: %self"}
!203 = distinct !{!203, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4iter17hea862c4ee711fef1E"}
!204 = !{!202, !198, !192}
!205 = !{!206}
!206 = distinct !{!206, !203, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4iter17hea862c4ee711fef1E: argument 0"}
!207 = !{!208, !210, !212, !206, !202, !198, !192}
!208 = distinct !{!208, !209, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!209 = distinct !{!209, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!210 = distinct !{!210, !211, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!211 = distinct !{!211, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!212 = distinct !{!212, !213, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17h3a8faabbbff5cd00E: argument 0"}
!213 = distinct !{!213, !"_ZN9hashbrown3raw21RawIterRange$LT$T$GT$3new17h3a8faabbbff5cd00E"}
!214 = !{!215, !217, !219, !221, !198, !192}
!215 = distinct !{!215, !216, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!216 = distinct !{!216, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!217 = distinct !{!217, !218, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!218 = distinct !{!218, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!219 = distinct !{!219, !220, !"_ZN96_$LT$hashbrown..raw..RawIterRange$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h05cca4a540c158cfE: %self"}
!220 = distinct !{!220, !"_ZN96_$LT$hashbrown..raw..RawIterRange$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17h05cca4a540c158cfE"}
!221 = distinct !{!221, !222, !"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E: %self"}
!222 = distinct !{!222, !"_ZN91_$LT$hashbrown..raw..RawIter$LT$T$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc54d60bc04ebad82E"}
!223 = !{!224}
!224 = distinct !{!224, !225, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he13d6557b60c3d5dE: %self"}
!225 = distinct !{!225, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he13d6557b60c3d5dE"}
!226 = !{!227}
!227 = distinct !{!227, !228, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!228 = distinct !{!228, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!229 = !{!227, !224, !192}
!230 = !{!231, !233}
!231 = distinct !{!231, !232, !"_ZN4core3mem7replace17h788e58c37a635438E: %dest"}
!232 = distinct !{!232, !"_ZN4core3mem7replace17h788e58c37a635438E"}
!233 = distinct !{!233, !234, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE: %self"}
!234 = distinct !{!234, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE"}
!235 = !{!236}
!236 = distinct !{!236, !237, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE: %x.0"}
!237 = distinct !{!237, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE"}
!238 = !{!239}
!239 = distinct !{!239, !240, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E: %self"}
!240 = distinct !{!240, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E"}
!241 = !{!242}
!242 = distinct !{!242, !243, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E: %self"}
!243 = distinct !{!243, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E"}
!244 = !{i64 0, i64 65}
!245 = !{!246, !248, !250, !242, !239}
!246 = distinct !{!246, !247, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE: argument 0"}
!247 = distinct !{!247, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE"}
!248 = distinct !{!248, !249, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E: argument 0"}
!249 = distinct !{!249, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E"}
!250 = distinct !{!250, !251, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E: argument 0"}
!251 = distinct !{!251, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E"}
!252 = !{!253, !248, !250, !242, !239}
!253 = distinct !{!253, !254, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E: argument 0"}
!254 = distinct !{!254, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E"}
!255 = !{!248, !250, !242, !239}
!256 = !{!242, !239}
!257 = !{!258, !260, !261, !263, !242, !239}
!258 = distinct !{!258, !259, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %x"}
!259 = distinct !{!259, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E"}
!260 = distinct !{!260, !259, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y:thread"}
!261 = distinct !{!261, !262, !"_ZN4core3mem4swap17h8292e61c571debd1E: %x"}
!262 = distinct !{!262, !"_ZN4core3mem4swap17h8292e61c571debd1E"}
!263 = distinct !{!263, !262, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y:thread"}
!264 = !{!265}
!265 = distinct !{!265, !266, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!266 = distinct !{!266, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!267 = !{!268, !270, !242, !239}
!268 = distinct !{!268, !269, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %_1"}
!269 = distinct !{!269, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E"}
!270 = distinct !{!270, !269, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %table"}
!271 = !{!272, !274, !276, !242, !239}
!272 = distinct !{!272, !273, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!273 = distinct !{!273, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!274 = distinct !{!274, !275, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!275 = distinct !{!275, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!276 = distinct !{!276, !277, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E: %self"}
!277 = distinct !{!277, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E"}
!278 = !{!274, !276, !242, !239}
!279 = !{!280, !282, !274, !276, !242, !239}
!280 = distinct !{!280, !281, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!281 = distinct !{!281, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!282 = distinct !{!282, !283, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!283 = distinct !{!283, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!284 = !{!285, !287, !276, !242, !239}
!285 = distinct !{!285, !286, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!286 = distinct !{!286, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!287 = distinct !{!287, !288, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!288 = distinct !{!288, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!289 = !{!258, !290, !261, !291, !242, !239}
!290 = distinct !{!290, !259, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y"}
!291 = distinct !{!291, !262, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y"}
!292 = !{!293, !295, !297, !242, !239}
!293 = distinct !{!293, !294, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!294 = distinct !{!294, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!295 = distinct !{!295, !296, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E: %self_"}
!296 = distinct !{!296, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E"}
!297 = distinct !{!297, !298, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E: %self"}
!298 = distinct !{!298, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E"}
!299 = !{!300}
!300 = distinct !{!300, !301, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E: %self"}
!301 = distinct !{!301, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E"}
!302 = !{!303}
!303 = distinct !{!303, !304, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E: %self"}
!304 = distinct !{!304, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E"}
!305 = !{!303, !300, !239}
!306 = !{!307, !309, !303, !300, !239}
!307 = distinct !{!307, !308, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!308 = distinct !{!308, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!309 = distinct !{!309, !310, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!310 = distinct !{!310, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!311 = !{!312, !303, !300, !239}
!312 = distinct !{!312, !313, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE: %a"}
!313 = distinct !{!313, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE"}
!314 = !{!300, !239}
!315 = !{!316}
!316 = distinct !{!316, !317, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!317 = distinct !{!317, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!318 = !{!319, !321, !300, !239}
!319 = distinct !{!319, !320, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %_1"}
!320 = distinct !{!320, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E"}
!321 = distinct !{!321, !320, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17h5e48384fd06b21d3E: %table"}
!322 = !{!323, !325, !300, !239}
!323 = distinct !{!323, !324, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!324 = distinct !{!324, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!325 = distinct !{!325, !326, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!326 = distinct !{!326, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!327 = !{!325, !300, !239}
!328 = !{!329, !331, !325, !300, !239}
!329 = distinct !{!329, !330, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!330 = distinct !{!330, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!331 = distinct !{!331, !332, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!332 = distinct !{!332, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!333 = !{!334, !336, !300, !239}
!334 = distinct !{!334, !335, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!335 = distinct !{!335, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!336 = distinct !{!336, !337, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!337 = distinct !{!337, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!338 = !{!339, !300, !239}
!339 = distinct !{!339, !340, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E: %self"}
!340 = distinct !{!340, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E"}
!341 = !{!342, !344, !339, !300, !239}
!342 = distinct !{!342, !343, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!343 = distinct !{!343, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!344 = distinct !{!344, !345, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!345 = distinct !{!345, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!346 = !{!347}
!347 = distinct !{!347, !348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x"}
!348 = distinct !{!348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE"}
!349 = !{!350}
!350 = distinct !{!350, !348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y"}
!351 = !{!350, !300, !239}
!352 = !{!347, !300, !239}
!353 = !{!354}
!354 = distinct !{!354, !348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It1"}
!355 = !{!356}
!356 = distinct !{!356, !348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It1"}
!357 = !{!356, !300, !239}
!358 = !{!354, !300, !239}
!359 = !{!360}
!360 = distinct !{!360, !348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It2"}
!361 = !{!362}
!362 = distinct !{!362, !348, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It2"}
!363 = !{!362, !300, !239}
!364 = !{!360, !300, !239}
!365 = distinct !{!365, !366, !367}
!366 = !{!"llvm.loop.isvectorized", i32 1}
!367 = !{!"llvm.loop.unroll.runtime.disable"}
!368 = !{!369, !300, !239}
!369 = distinct !{!369, !370, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!370 = distinct !{!370, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!371 = !{!372, !374, !376}
!372 = distinct !{!372, !373, !"_ZN4core3mem7replace17ha318695de15894dbE: %dest"}
!373 = distinct !{!373, !"_ZN4core3mem7replace17ha318695de15894dbE"}
!374 = distinct !{!374, !375, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E: %self"}
!375 = distinct !{!375, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E"}
!376 = distinct !{!376, !377, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E: %val"}
!377 = distinct !{!377, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E"}
!378 = !{!379}
!379 = distinct !{!379, !380, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: %_1"}
!380 = distinct !{!380, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE"}
!381 = !{!382}
!382 = distinct !{!382, !383, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: %_1"}
!383 = distinct !{!383, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE"}
!384 = !{!382, !379}
!385 = !{!386, !387}
!386 = distinct !{!386, !383, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: argument 0"}
!387 = distinct !{!387, !380, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: argument 0"}
!388 = !{!389}
!389 = distinct !{!389, !390, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E: %dest"}
!390 = distinct !{!390, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E"}
!391 = !{!386, !382, !387, !379}
!392 = !{!393, !395, !397}
!393 = distinct !{!393, !394, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E: %dest"}
!394 = distinct !{!394, !"_ZN4core3mem7replace17h8cca4baf101fbcf1E"}
!395 = distinct !{!395, !396, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E: %self"}
!396 = distinct !{!396, !"_ZN4core6option15Option$LT$T$GT$4take17h58c01ba554c42930E"}
!397 = distinct !{!397, !398, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE: %val"}
!398 = distinct !{!398, !"_ZN9once_cell14take_unchecked17h4fe05cc2bcf0106bE"}
!399 = !{!400}
!400 = distinct !{!400, !401, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: %_1"}
!401 = distinct !{!401, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE"}
!402 = !{!403}
!403 = distinct !{!403, !404, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: %_1"}
!404 = distinct !{!404, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E"}
!405 = !{!403, !400}
!406 = !{!407, !408}
!407 = distinct !{!407, !404, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h45e7aa0961f37934E: argument 0"}
!408 = distinct !{!408, !401, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17h7ed2501f12947bafE: argument 0"}
!409 = !{!410}
!410 = distinct !{!410, !411, !"_ZN4core3mem7replace17h17668aa3bb646e28E: %dest"}
!411 = distinct !{!411, !"_ZN4core3mem7replace17h17668aa3bb646e28E"}
!412 = !{!407, !403, !408, !400}
!413 = !{!414}
!414 = distinct !{!414, !415, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!415 = distinct !{!415, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!416 = !{!417, !418}
!417 = distinct !{!417, !415, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!418 = distinct !{!418, !415, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!419 = !{!420}
!420 = distinct !{!420, !421, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E: %self"}
!421 = distinct !{!421, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E"}
!422 = !{!423, !425}
!423 = distinct !{!423, !424, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E: %f"}
!424 = distinct !{!424, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E"}
!425 = distinct !{!425, !426, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E: %f"}
!426 = distinct !{!426, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E"}
!427 = !{!428}
!428 = distinct !{!428, !429, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE: argument 0"}
!429 = distinct !{!429, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE"}
!430 = !{!431, !428}
!431 = distinct !{!431, !432, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E: argument 0"}
!432 = distinct !{!432, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E"}
!433 = !{!434}
!434 = distinct !{!434, !435, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E: %self"}
!435 = distinct !{!435, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E"}
!436 = !{!437, !439}
!437 = distinct !{!437, !438, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE: %f"}
!438 = distinct !{!438, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE"}
!439 = distinct !{!439, !440, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E: %f"}
!440 = distinct !{!440, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E"}
!441 = !{!442}
!442 = distinct !{!442, !443, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE: argument 0"}
!443 = distinct !{!443, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE"}
!444 = !{!445, !442}
!445 = distinct !{!445, !446, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E: argument 0"}
!446 = distinct !{!446, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E"}
!447 = !{!448, !450}
!448 = distinct !{!448, !449, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: %self"}
!449 = distinct !{!449, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE"}
!450 = distinct !{!450, !449, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: argument 1"}
!451 = !{!448}
!452 = !{!453, !455, !457, !458, !460, !461, !463, !464, !466, !467, !469, !470, !472, !473, !475}
!453 = distinct !{!453, !454, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16with_capacity_in17haf58c241a925526dE: argument 0"}
!454 = distinct !{!454, !"_ZN5alloc3vec16Vec$LT$T$C$A$GT$16with_capacity_in17haf58c241a925526dE"}
!455 = distinct !{!455, !456, !"_ZN52_$LT$T$u20$as$u20$alloc..slice..hack..ConvertVec$GT$6to_vec17h53aee583d85922ecE: %v"}
!456 = distinct !{!456, !"_ZN52_$LT$T$u20$as$u20$alloc..slice..hack..ConvertVec$GT$6to_vec17h53aee583d85922ecE"}
!457 = distinct !{!457, !456, !"_ZN52_$LT$T$u20$as$u20$alloc..slice..hack..ConvertVec$GT$6to_vec17h53aee583d85922ecE: %s.0"}
!458 = distinct !{!458, !459, !"_ZN5alloc5slice4hack6to_vec17h9d653acab8d582dcE: argument 0"}
!459 = distinct !{!459, !"_ZN5alloc5slice4hack6to_vec17h9d653acab8d582dcE"}
!460 = distinct !{!460, !459, !"_ZN5alloc5slice4hack6to_vec17h9d653acab8d582dcE: %s.0"}
!461 = distinct !{!461, !462, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$9to_vec_in17hcb2720fd082a03b1E: argument 0"}
!462 = distinct !{!462, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$9to_vec_in17hcb2720fd082a03b1E"}
!463 = distinct !{!463, !462, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$9to_vec_in17hcb2720fd082a03b1E: %self.0"}
!464 = distinct !{!464, !465, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6to_vec17ha27e4e65413e47a6E: argument 0"}
!465 = distinct !{!465, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6to_vec17ha27e4e65413e47a6E"}
!466 = distinct !{!466, !465, !"_ZN5alloc5slice29_$LT$impl$u20$$u5b$T$u5d$$GT$6to_vec17ha27e4e65413e47a6E: %self.0"}
!467 = distinct !{!467, !468, !"_ZN5alloc5slice64_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$$u5b$T$u5d$$GT$8to_owned17h826e2cc3001afcccE: argument 0"}
!468 = distinct !{!468, !"_ZN5alloc5slice64_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$$u5b$T$u5d$$GT$8to_owned17h826e2cc3001afcccE"}
!469 = distinct !{!469, !468, !"_ZN5alloc5slice64_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$$u5b$T$u5d$$GT$8to_owned17h826e2cc3001afcccE: %self.0"}
!470 = distinct !{!470, !471, !"_ZN5alloc3str56_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$str$GT$8to_owned17h0326c85be227b8e5E: argument 0"}
!471 = distinct !{!471, !"_ZN5alloc3str56_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$str$GT$8to_owned17h0326c85be227b8e5E"}
!472 = distinct !{!472, !471, !"_ZN5alloc3str56_$LT$impl$u20$alloc..borrow..ToOwned$u20$for$u20$str$GT$8to_owned17h0326c85be227b8e5E: %self.0"}
!473 = distinct !{!473, !474, !"_ZN76_$LT$alloc..string..String$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h28e78c83f20c9950E: argument 0"}
!474 = distinct !{!474, !"_ZN76_$LT$alloc..string..String$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h28e78c83f20c9950E"}
!475 = distinct !{!475, !474, !"_ZN76_$LT$alloc..string..String$u20$as$u20$core..convert..From$LT$$RF$str$GT$$GT$4from17h28e78c83f20c9950E: %s.0"}
!476 = !{!455, !458, !461, !464, !467, !470, !473}
!477 = !{!478}
!478 = distinct !{!478, !479, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE: %self"}
!479 = distinct !{!479, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE"}
!480 = !{!481}
!481 = distinct !{!481, !482, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E: %self"}
!482 = distinct !{!482, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E"}
!483 = !{!481, !478}
!484 = !{!485, !486, !487, !488}
!485 = distinct !{!485, !482, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E: argument 0"}
!486 = distinct !{!486, !482, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17h5a46c16749a32080E: %v"}
!487 = distinct !{!487, !479, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE: argument 0"}
!488 = distinct !{!488, !479, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hf55a14824918ed1eE: %v"}
!489 = !{!490}
!490 = distinct !{!490, !491, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h96cd7094a0a5915dE: %self"}
!491 = distinct !{!491, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h96cd7094a0a5915dE"}
!492 = !{!493}
!493 = distinct !{!493, !494, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: %self"}
!494 = distinct !{!494, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E"}
!495 = !{!496}
!496 = distinct !{!496, !497, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!497 = distinct !{!497, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!498 = !{!499, !496, !493, !490, !481, !478}
!499 = distinct !{!499, !500, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!500 = distinct !{!500, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!501 = !{!502, !485, !486, !487, !488}
!502 = distinct !{!502, !494, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: argument 1"}
!503 = !{!493, !490, !481, !478}
!504 = !{!505, !496, !493, !502, !490, !485, !481, !486, !487, !478, !488}
!505 = distinct !{!505, !506, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!506 = distinct !{!506, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!507 = !{!508, !496, !493, !502, !490, !485, !481, !486, !487, !478, !488}
!508 = distinct !{!508, !509, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E: %_1"}
!509 = distinct !{!509, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E"}
!510 = !{!485, !481, !486, !487, !478, !488}
!511 = !{!487, !478}
!512 = !{!513}
!513 = distinct !{!513, !514, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE: %self"}
!514 = distinct !{!514, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE"}
!515 = !{!516, !518, !513, !520, !521, !485, !481, !486, !487, !478, !488}
!516 = distinct !{!516, !517, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!517 = distinct !{!517, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!518 = distinct !{!518, !519, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!519 = distinct !{!519, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!520 = distinct !{!520, !514, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE: %value"}
!521 = distinct !{!521, !514, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17h5a4c6771c72e69fdE: %hasher"}
!522 = !{!518, !513, !520, !521, !485, !481, !486, !487, !478, !488}
!523 = !{!524, !526, !518, !513, !520, !521, !485, !481, !486, !487, !478, !488}
!524 = distinct !{!524, !525, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!525 = distinct !{!525, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!526 = distinct !{!526, !527, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!527 = distinct !{!527, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!528 = !{!513, !520, !521, !485, !481, !486, !487, !478, !488}
!529 = !{!513, !481, !478}
!530 = !{!520, !521, !485, !486, !487, !488}
!531 = !{!520, !485, !486, !487, !488}
!532 = !{!533}
!533 = distinct !{!533, !534, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!534 = distinct !{!534, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!535 = !{!536, !533, !513, !481, !478}
!536 = distinct !{!536, !537, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!537 = distinct !{!537, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!538 = !{!533, !513, !481, !478}
!539 = !{!540, !533, !513, !520, !485, !486, !487, !488}
!540 = distinct !{!540, !541, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!541 = distinct !{!541, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!542 = !{!533, !513, !520, !485, !486, !487, !488}
!543 = !{!544, !546, !533, !513, !520, !485, !486, !487, !488}
!544 = distinct !{!544, !545, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!545 = distinct !{!545, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!546 = distinct !{!546, !547, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!547 = distinct !{!547, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!548 = !{!513, !485, !486, !487, !488}
!549 = !{!550}
!550 = distinct !{!550, !551, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E: %self"}
!551 = distinct !{!551, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E"}
!552 = !{!553, !555, !550, !513, !520, !485, !486, !487, !488}
!553 = distinct !{!553, !554, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!554 = distinct !{!554, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!555 = distinct !{!555, !556, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!556 = distinct !{!556, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!557 = !{!550, !513, !481, !478}
!558 = !{!481, !486, !478, !488}
!559 = !{!560}
!560 = distinct !{!560, !561, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!561 = distinct !{!561, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!562 = !{!563}
!563 = distinct !{!563, !564, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE: %self"}
!564 = distinct !{!564, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE"}
!565 = !{!566, !568}
!566 = distinct !{!566, !567, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE: %f"}
!567 = distinct !{!567, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE"}
!568 = distinct !{!568, !569, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E: %f"}
!569 = distinct !{!569, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E"}
!570 = !{!571}
!571 = distinct !{!571, !572, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE: argument 0"}
!572 = distinct !{!572, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE"}
!573 = !{!574, !571}
!574 = distinct !{!574, !575, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E: argument 0"}
!575 = distinct !{!575, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E"}
!576 = !{!577, !579}
!577 = distinct !{!577, !578, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: %self"}
!578 = distinct !{!578, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE"}
!579 = distinct !{!579, !578, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: argument 1"}
!580 = !{!577}
!581 = !{!582}
!582 = distinct !{!582, !583, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!583 = distinct !{!583, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!584 = !{!585}
!585 = distinct !{!585, !586, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!586 = distinct !{!586, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!587 = !{!588, !589}
!588 = distinct !{!588, !586, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!589 = distinct !{!589, !586, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!590 = !{!591}
!591 = distinct !{!591, !592, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!592 = distinct !{!592, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!593 = !{!594, !596}
!594 = distinct !{!594, !595, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE: %f"}
!595 = distinct !{!595, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17h85566ba017d8be8bE"}
!596 = distinct !{!596, !597, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E: %f"}
!597 = distinct !{!597, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17hc0eef1a4845b5272E"}
!598 = !{!599}
!599 = distinct !{!599, !600, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE: argument 0"}
!600 = distinct !{!600, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17h446a98d5168371bcE"}
!601 = !{!602, !599}
!602 = distinct !{!602, !603, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E: argument 0"}
!603 = distinct !{!603, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h8759ab031ee54877E"}
!604 = !{!605, !607}
!605 = distinct !{!605, !606, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: %self"}
!606 = distinct !{!606, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE"}
!607 = distinct !{!607, !606, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h66132e22800570feE: argument 1"}
!608 = !{!605}
!609 = !{!610}
!610 = distinct !{!610, !611, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!611 = distinct !{!611, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!612 = !{!613, !614}
!613 = distinct !{!613, !611, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!614 = distinct !{!614, !611, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!615 = !{!616}
!616 = distinct !{!616, !617, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE: %self"}
!617 = distinct !{!617, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hc5af6ff738cf760dE"}
!618 = !{!619}
!619 = distinct !{!619, !620, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h9c40df2332e3f4a7E: %self"}
!620 = distinct !{!620, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h9c40df2332e3f4a7E"}
!621 = !{!622}
!622 = distinct !{!622, !623, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h3e472d875cf1033bE: %self"}
!623 = distinct !{!623, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h3e472d875cf1033bE"}
!624 = !{!625}
!625 = distinct !{!625, !626, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE: %self"}
!626 = distinct !{!626, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE"}
!627 = !{!625, !622, !619}
!628 = !{!629, !630, !631}
!629 = distinct !{!629, !626, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h2e8f2adc9def066bE: argument 0"}
!630 = distinct !{!630, !623, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h3e472d875cf1033bE: argument 0"}
!631 = distinct !{!631, !620, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h9c40df2332e3f4a7E: argument 0"}
!632 = !{!633}
!633 = distinct !{!633, !634, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17h4afae7353d3cefa4E: %self"}
!634 = distinct !{!634, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17h4afae7353d3cefa4E"}
!635 = !{!636}
!636 = distinct !{!636, !637, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: %self"}
!637 = distinct !{!637, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E"}
!638 = !{!639}
!639 = distinct !{!639, !640, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!640 = distinct !{!640, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!641 = !{!642, !639, !636, !633, !625, !622, !619}
!642 = distinct !{!642, !643, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!643 = distinct !{!643, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!644 = !{!645, !646, !629, !630, !631}
!645 = distinct !{!645, !637, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h398d51ca79baad78E: argument 1"}
!646 = distinct !{!646, !634, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17h4afae7353d3cefa4E: argument 0"}
!647 = !{!636, !633, !625, !622, !619}
!648 = !{!649, !639, !636, !645, !646, !633, !629, !625, !630, !622, !631, !619}
!649 = distinct !{!649, !650, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!650 = distinct !{!650, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!651 = !{!652, !639, !636, !645, !646, !633, !629, !625, !630, !622, !631, !619}
!652 = distinct !{!652, !653, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E: %_1"}
!653 = distinct !{!653, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17heabe2bd78d4b31a3E"}
!654 = !{!655}
!655 = distinct !{!655, !656, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17ha6f0eafe2ff00441E: %self"}
!656 = distinct !{!656, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17ha6f0eafe2ff00441E"}
!657 = !{!658}
!658 = distinct !{!658, !659, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h4aa5880891f88a93E: %self"}
!659 = distinct !{!659, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h4aa5880891f88a93E"}
!660 = !{!661}
!661 = distinct !{!661, !662, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E: %self"}
!662 = distinct !{!662, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E"}
!663 = !{!664, !661, !658, !666, !655, !646, !633, !629, !625, !630, !622, !631, !619}
!664 = distinct !{!664, !665, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!665 = distinct !{!665, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!666 = distinct !{!666, !656, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17ha6f0eafe2ff00441E: argument 0"}
!667 = !{!668, !661, !658, !666, !655, !646, !633, !629, !625, !630, !622, !631, !619}
!668 = distinct !{!668, !669, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!669 = distinct !{!669, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!670 = !{!661, !658, !655, !633, !625, !622, !619}
!671 = !{!666, !646, !629, !630, !631}
!672 = !{!661, !658, !666, !655, !646, !633, !629, !625, !630, !622, !631, !619}
!673 = !{!633, !625, !630, !622, !631, !619}
!674 = !{!622, !619}
