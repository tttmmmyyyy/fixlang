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
%ObjectInfo = type { i64, i64, i64 }
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
@alloc371 = private unnamed_addr constant <{ [70 x i8] }> <{ [70 x i8] c"cannot access a Thread Local Storage value during or after destruction" }>, align 1
@alloc374 = private unnamed_addr constant <{ [79 x i8] }> <{ [79 x i8] c"/rustc/a8314ef7d0ec7b75c336af2c9857bfaf43002bfc/library/std/src/thread/local.rs" }>, align 1
@alloc373 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [79 x i8] }>, <{ [79 x i8] }>* @alloc374, i32 0, i32 0, i32 0), [16 x i8] c"O\00\00\00\00\00\00\00\A5\01\00\00\1A\00\00\00" }>, align 8
@vtable.0 = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17h92e001d5e4efd74cE" to i8*), i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17hc9f8af2660d4514aE" to i8*) }>, align 8
@_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E = external local_unnamed_addr global %"core::sync::atomic::AtomicUsize"
@alloc73 = private unnamed_addr constant <{}> zeroinitializer, align 8
@alloc410 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Option::unwrap()` on a `None` value" }>, align 1
@vtable.3 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00", i8* bitcast (i1 (%"std::thread::local::AccessError"*, %"core::fmt::Formatter"*)* @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h514ef917cd5ecc1bE" to i8*) }>, align 8
@alloc422 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@vtable.4 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"core::str::error::Utf8Error"*, %"core::fmt::Formatter"*)* @"_ZN64_$LT$core..str..error..Utf8Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h864a228d6ab6973cE" to i8*) }>, align 8
@vtable.6 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void ({ i64*, i8 }*)* @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 ({ i64*, i8 }*, %"core::fmt::Formatter"*)* @"_ZN76_$LT$std..sync..poison..PoisonError$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h0cd32de15374fa48E" to i8*) }>, align 8
@vtable.7 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (i64**, %"core::fmt::Formatter"*)* @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hc715f6c95a655b17E" to i8*) }>, align 8
@alloc432 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"PoisonError" }>, align 1
@vtable.8 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i64 ({ [0 x i8]*, i64 }*)* @"_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17ha7daf7c2b2ea8d27E" to i8*) }>, align 8
@alloc67 = private unnamed_addr constant <{ [16 x i8] }> <{ [16 x i8] c"\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF" }>, align 16
@vtable.b = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h42a39cd9ab169dceE" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E" to i8*) }>, align 8
@vtable.c = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hcf0b305cdf28ac00E" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E" to i8*) }>, align 8
@alloc464 = private unnamed_addr constant <{ [42 x i8] }> <{ [42 x i8] c"Lazy instance has previously been poisoned" }>, align 1
@alloc465 = private unnamed_addr constant <{ [90 x i8] }> <{ [90 x i8] c"/home/maruyama/.cargo/registry/src/github.com-1ecc6299db9ec823/once_cell-1.13.0/src/lib.rs" }>, align 1
@alloc463 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [90 x i8] }>, <{ [90 x i8] }>* @alloc465, i32 0, i32 0, i32 0), [16 x i8] c"Z\00\00\00\00\00\00\00\CF\04\00\00\19\00\00\00" }>, align 8
@_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE = internal global <{ [16 x i8], [16 x i8], i8* }> <{ [16 x i8] zeroinitializer, [16 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<i64>"*)* @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE to i8*) }>, align 8
@_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE = internal global <{ [16 x i8], [56 x i8], i8* }> <{ [16 x i8] zeroinitializer, [56 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)* @_ZN4core3ops8function6FnOnce9call_once17hd20ed85d13df1445E to i8*) }>, align 8
@alloc70 = private unnamed_addr constant <{ [54 x i8] }> <{ [54 x i8] c"[report_malloc] Failed to convert given name to &str.\0A" }>, align 1
@alloc71 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [54 x i8] }>, <{ [54 x i8] }>* @alloc70, i32 0, i32 0, i32 0), [8 x i8] c"6\00\00\00\00\00\00\00" }>, align 8
@alloc491 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"src/lib.rs" }>, align 1
@alloc468 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00\22\00\00\00!\00\00\00" }>, align 8
@alloc470 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00#\00\00\00)\00\00\00" }>, align 8
@alloc260 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"Object id=" }>, align 1
@alloc77 = private unnamed_addr constant <{ [37 x i8] }> <{ [37 x i8] c" is allocated. refcnt=(0 -> 1), addr=" }>, align 1
@alloc78 = private unnamed_addr constant <{ [7 x i8] }> <{ [7 x i8] c", name=" }>, align 1
@alloc231 = private unnamed_addr constant <{ [1 x i8] }> <{ [1 x i8] c"\0A" }>, align 1
@alloc76 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc260, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [37 x i8] }>, <{ [37 x i8] }>* @alloc77, i32 0, i32 0, i32 0), [8 x i8] c"%\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [7 x i8] }>, <{ [7 x i8] }>* @alloc78, i32 0, i32 0, i32 0), [8 x i8] c"\07\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [1 x i8] }>, <{ [1 x i8] }>* @alloc231, i32 0, i32 0, i32 0), [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8
@alloc96 = private unnamed_addr constant <{ [168 x i8] }> <{ [168 x i8] c"\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\04\00\00\00\03\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00" }>, align 8
@alloc472 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00,\00\00\003\00\00\00" }>, align 8
@alloc168 = private unnamed_addr constant <{ [22 x i8] }> <{ [22 x i8] c" is retained. refcnt=(" }>, align 1
@alloc229 = private unnamed_addr constant <{ [4 x i8] }> <{ [4 x i8] c" -> " }>, align 1
@alloc230 = private unnamed_addr constant <{ [8 x i8] }> <{ [8 x i8] c"), addr=" }>, align 1
@alloc167 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc260, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [22 x i8] }>, <{ [22 x i8] }>* @alloc168, i32 0, i32 0, i32 0), [8 x i8] c"\16\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [4 x i8] }>, <{ [4 x i8] }>* @alloc229, i32 0, i32 0, i32 0), [8 x i8] c"\04\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [8 x i8] }>, <{ [8 x i8] }>* @alloc230, i32 0, i32 0, i32 0), [8 x i8] c"\08\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [1 x i8] }>, <{ [1 x i8] }>* @alloc231, i32 0, i32 0, i32 0), [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8
@alloc253 = private unnamed_addr constant <{ [224 x i8] }> <{ [224 x i8] c"\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\03\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00 \00\00\00\04\00\00\00\03\00\00\00\00\00\00\00" }>, align 8
@alloc255 = private unnamed_addr constant <{ [8 x i8] }> zeroinitializer, align 8
@alloc262 = private unnamed_addr constant <{ [31 x i8] }> <{ [31 x i8] c" whose refcnt zero is retained!" }>, align 1
@alloc261 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc260, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [31 x i8] }>, <{ [31 x i8] }>* @alloc262, i32 0, i32 0, i32 0), [8 x i8] c"\1F\00\00\00\00\00\00\00" }>, align 8
@alloc474 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00A\00\00\00\05\00\00\00" }>, align 8
@alloc476 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00F\00\00\003\00\00\00" }>, align 8
@alloc205 = private unnamed_addr constant <{ [20 x i8] }> <{ [20 x i8] c"Retain of object id=" }>, align 1
@alloc267 = private unnamed_addr constant <{ [50 x i8] }> <{ [50 x i8] c" is reported but it isn't registered to sanitizer." }>, align 1
@alloc206 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [20 x i8] }>, <{ [20 x i8] }>* @alloc205, i32 0, i32 0, i32 0), [8 x i8] c"\14\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc267, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc478 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00G\00\00\00\05\00\00\00" }>, align 8
@alloc480 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00L\00\00\00.\00\00\00" }>, align 8
@alloc272 = private unnamed_addr constant <{ [24 x i8] }> <{ [24 x i8] c"The refcnt of object id=" }>, align 1
@alloc214 = private unnamed_addr constant <{ [37 x i8] }> <{ [37 x i8] c" in report_retain mismatch! reported=" }>, align 1
@alloc275 = private unnamed_addr constant <{ [12 x i8] }> <{ [12 x i8] c", sanitizer=" }>, align 1
@alloc213 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc272, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [37 x i8] }>, <{ [37 x i8] }>* @alloc214, i32 0, i32 0, i32 0), [8 x i8] c"%\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc275, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc482 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00M\00\00\00\05\00\00\00" }>, align 8
@alloc228 = private unnamed_addr constant <{ [22 x i8] }> <{ [22 x i8] c" is released. refcnt=(" }>, align 1
@alloc227 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc260, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [22 x i8] }>, <{ [22 x i8] }>* @alloc228, i32 0, i32 0, i32 0), [8 x i8] c"\16\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [4 x i8] }>, <{ [4 x i8] }>* @alloc229, i32 0, i32 0, i32 0), [8 x i8] c"\04\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [8 x i8] }>, <{ [8 x i8] }>* @alloc230, i32 0, i32 0, i32 0), [8 x i8] c"\08\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [1 x i8] }>, <{ [1 x i8] }>* @alloc231, i32 0, i32 0, i32 0), [8 x i8] c"\01\00\00\00\00\00\00\00" }>, align 8
@alloc484 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00`\00\00\00\05\00\00\00" }>, align 8
@alloc486 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00e\00\00\002\00\00\00" }>, align 8
@alloc265 = private unnamed_addr constant <{ [21 x i8] }> <{ [21 x i8] c"Release of object id=" }>, align 1
@alloc266 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [21 x i8] }>, <{ [21 x i8] }>* @alloc265, i32 0, i32 0, i32 0), [8 x i8] c"\15\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc267, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc488 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00f\00\00\00\05\00\00\00" }>, align 8
@alloc490 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00k\00\00\00-\00\00\00" }>, align 8
@alloc274 = private unnamed_addr constant <{ [38 x i8] }> <{ [38 x i8] c" in report_release mismatch! reported=" }>, align 1
@alloc273 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc272, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [38 x i8] }>, <{ [38 x i8] }>* @alloc274, i32 0, i32 0, i32 0), [8 x i8] c"&\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc275, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc492 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc491, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00l\00\00\00\05\00\00\00" }>, align 8

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
define internal fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h7c6dbde3483cee85E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias nocapture noundef readonly align 8 dereferenceable(48) %self, i64 %k.val1) unnamed_addr #2 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !5) #24
  %_4.idx.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1, i32 1, i32 4
  %_4.idx.val.i.i = load i64, i64* %_4.idx.i.i, align 8, !alias.scope !8
  %0 = icmp eq i64 %_4.idx.val.i.i, 0
  br i1 %0, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h7f4c47b100c2fe02E.exit", label %bb3.i.i

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
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h7f4c47b100c2fe02E.exit"

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %88 = tail call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %88 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %89 = sub i64 0, %index.i.i.i.i.i
  %90 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %89, i32 0
  %91 = getelementptr inbounds i64, i64* %90, i64 -4
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %91, align 8, !noalias !28
  %92 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %k.val1
  br i1 %92, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h7f4c47b100c2fe02E.exit", label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %93 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %94 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %93
  br label %bb3.i.i.i.i.i

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h7f4c47b100c2fe02E.exit": ; preds = %bb12.i.i.i.i.i, %bb10.i.i.i.i.i, %start
  %.0.i.i = phi i1 [ false, %start ], [ true, %bb10.i.i.i.i.i ], [ false, %bb12.i.i.i.i.i ]
  ret i1 %.0.i.i
}

; std::collections::hash::map::HashMap<K,V,S>::get_mut
; Function Attrs: inlinehint nofree nosync nounwind nonlazybind uwtable
define internal fastcc noundef align 8 dereferenceable_or_null(24) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h806e044307d21e0aE"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias nocapture noundef readonly align 8 dereferenceable(48) %self, i64 %k.val1) unnamed_addr #2 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !31)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !34) #24
  %_4.idx.i.i = getelementptr %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %self, i64 0, i32 0, i32 1, i32 1, i32 4
  %_4.idx.val.i.i = load i64, i64* %_4.idx.i.i, align 8, !alias.scope !37
  %0 = icmp eq i64 %_4.idx.val.i.i, 0
  br i1 %0, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit", label %bb3.i.i

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
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit"

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %88 = tail call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %88 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %89 = sub i64 0, %index.i.i.i.i.i
  %90 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %89, i32 0
  %91 = getelementptr inbounds i64, i64* %90, i64 -4
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %91, align 8, !noalias !56
  %92 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %k.val1
  br i1 %92, label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit.loopexit", label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %93 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %94 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %93
  br label %bb3.i.i.i.i.i

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit.loopexit": ; preds = %bb10.i.i.i.i.i
  %95 = getelementptr inbounds i64, i64* %90, i64 -4
  br label %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit"

"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit": ; preds = %bb12.i.i.i.i.i, %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit.loopexit", %start
  %.0.i.i = phi i64* [ null, %start ], [ %95, %"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E.exit.loopexit" ], [ null, %bb12.i.i.i.i.i ]
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
  store [0 x i8]* bitcast (<{ [42 x i8] }>* @alloc464 to [0 x i8]*), [0 x i8]** %1, align 8
  %2 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 0, i32 1
  store i64 42, i64* %2, align 8
  %3 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 1
  store %"core::panic::location::Location"* bitcast (<{ i8*, [16 x i8] }>* @alloc463 to %"core::panic::location::Location"*), %"core::panic::location::Location"** %3, align 8
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
define internal noundef zeroext i1 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hcf0b305cdf28ac00E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* nocapture readonly %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
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
  br i1 %6, label %bb2.i.i.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i"

bb2.i.i.i.i:                                      ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !148
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0.i.i), !noalias !149
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !alias.scope !127, !nonnull !85, !align !86, !noundef !85
  %_17.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !127
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 0
  %_2.i16.i.i = load i64, i64* %9, align 8, !range !120, !noalias !127, !noundef !85
  %10 = icmp eq i64 %_2.i16.i.i, 0
  br i1 %10, label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1
  %12 = bitcast [7 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %bb4.i.i.i.i unwind label %cleanup.i.i.i.i, !noalias !127

cleanup.i.i.i.i:                                  ; preds = %bb2.i.i.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %14 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 1
  %15 = bitcast i64* %14 to %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*
; call core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #25, !noalias !127
  %_20.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !127
  %_10.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx.i.i, align 8, !noalias !127
  %_10.sroa.5.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast.i.i = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast.i.i, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20.i.i, i64 56, i1 false), !noalias !127
  resume { i8*, i32 } %13

bb4.i.i.i.i:                                      ; preds = %bb2.i.i.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 3
  tail call void @llvm.experimental.noalias.scope.decl(metadata !150) #24
  %_2.i.i.i.i.i.i.i.i.i.i = load i64, i64* %16, align 8, !alias.scope !153, !noalias !127
  %17 = icmp eq i64 %_2.i.i.i.i.i.i.i.i.i.i, 0
  br i1 %17, label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit, label %bb2.i.i.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i.i.i:                            ; preds = %bb4.i.i.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !156) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !159) #24
  %18 = add i64 %_2.i.i.i.i.i.i.i.i.i.i, 1
  %19 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %18, i64 32) #24
  %20 = extractvalue { i64, i1 } %19, 1
  %21 = xor i1 %20, true
  tail call void @llvm.assume(i1 %21) #24
  %22 = extractvalue { i64, i1 } %19, 0
  %_31.i.i.i.i.i.i.i.i.i.i.i.i = add i64 %_2.i.i.i.i.i.i.i.i.i.i, 17
  %23 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %22, i64 %_31.i.i.i.i.i.i.i.i.i.i.i.i) #24
  %24 = extractvalue { i64, i1 } %23, 1
  %25 = xor i1 %24, true
  tail call void @llvm.assume(i1 %25) #24
  %26 = extractvalue { i64, i1 } %23, 0
  %27 = icmp eq i64 %26, 0
  br i1 %27, label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit, label %bb2.i.i.i.i.i.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i.i.i.i.i.i:                      ; preds = %bb2.i.i.i.i.i.i.i.i.i
  %28 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 4
  %29 = bitcast i64* %28 to i8**
  %_17.i.i.i.i.i.i.i.i.i.i.i = load i8*, i8** %29, align 8, !alias.scope !162, !noalias !127, !nonnull !85, !noundef !85
  %30 = sub i64 0, %22
  %31 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i.i.i.i.i.i.i, i64 %30
  tail call void @__rust_dealloc(i8* nonnull %31, i64 %26, i64 16) #24, !noalias !163
  br label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit

_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit: ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i", %bb4.i.i.i.i, %bb2.i.i.i.i.i.i.i.i.i, %bb2.i.i.i.i.i.i.i.i.i.i.i.i
  %_22.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !127
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
define internal void @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  tail call void @llvm.experimental.noalias.scope.decl(metadata !164)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !167)
  %1 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %1), !noalias !170
; call std::sys_common::mutex::MovableMutex::new
  %2 = tail call i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E(), !noalias !170
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %.0..sroa_idx.i.i, align 4, !noalias !170
; invoke std::sync::poison::Flag::new
  %3 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit" unwind label %cleanup1.i.i, !noalias !170

cleanup1.i.i:                                     ; preds = %start
  %4 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #25
          to label %bb5.i.i unwind label %abort.i.i, !noalias !170

abort.i.i:                                        ; preds = %cleanup1.i.i
  %5 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !170
  unreachable

bb5.i.i:                                          ; preds = %cleanup1.i.i
  resume { i8*, i32 } %4

"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit": ; preds = %start
  %6 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %6, align 8, !alias.scope !170
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %3, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !170
  %7 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 3
  store i64 0, i64* %7, align 8, !alias.scope !170
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %1), !noalias !170
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17hd20ed85d13df1445E(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i.i.i.i.i.i = alloca %"std::thread::local::AccessError", align 1
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  %_2.i = alloca %"std::collections::hash::map::HashMap<i64, ObjectInfo>", align 16
  tail call void @llvm.experimental.noalias.scope.decl(metadata !171)
  %1 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1), !noalias !171
  tail call void @llvm.experimental.noalias.scope.decl(metadata !174)
  %_2.i.i.i.i.i.i.i.i.i.i = load i64, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 0), align 8, !range !120, !noalias !177, !noundef !85
  %trunc.not.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_2.i.i.i.i.i.i.i.i.i.i, 0
  br i1 %trunc.not.i.i.i.i.i.i.i.i.i.i, label %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"

_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i: ; preds = %start
; call std::thread::local::fast::Key<T>::try_initialize
  %2 = tail call fastcc noundef align 8 dereferenceable_or_null(16) i64* @"_ZN3std6thread5local4fast12Key$LT$T$GT$14try_initialize17hd4e535fd74b46a6dE"(i64* noalias noundef align 8 dereferenceable_or_null(24) null), !noalias !184
  %3 = icmp eq i64* %2, null
  br i1 %3, label %bb1.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"

bb1.i.i.i.i.i.i:                                  ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i
  %4 = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 0, i8* nonnull %4), !noalias !185
  %_6.0.i.i.i.i.i.i = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [70 x i8] }>* @alloc371 to [0 x i8]*), i64 70, {}* noundef nonnull align 1 %_6.0.i.i.i.i.i.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.3 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc373 to %"core::panic::location::Location"*)) #23, !noalias !185
  unreachable

"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i": ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, %start
  %.0.i.i2.i.i.i.i.i.i = phi i64* [ %2, %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i ], [ getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 0), %start ]
  %5 = bitcast i64* %.0.i.i2.i.i.i.i.i.i to <2 x i64>*
  %6 = load <2 x i64>, <2 x i64>* %5, align 8, !noalias !184
  %7 = extractelement <2 x i64> %6, i64 0
  %8 = add i64 %7, 1
  store i64 %8, i64* %.0.i.i2.i.i.i.i.i.i, align 8, !alias.scope !186, !noalias !184
  %_2.sroa.7.0..sroa_idx.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 3
  %_2.sroa.7.0..sroa_idx1516.i.i.i = bitcast i64* %_2.sroa.7.0..sroa_idx.i.i.i to i8*
  call void @llvm.memset.p0i8.i64(i8* noundef nonnull align 16 dereferenceable(16) %_2.sroa.7.0..sroa_idx1516.i.i.i, i8 0, i64 16, i1 false) #24, !alias.scope !189, !noalias !171
  %9 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to <2 x i64>*
  store <2 x i64> %6, <2 x i64>* %9, align 16, !alias.scope !189, !noalias !171
  %_2.sroa.5.0..sroa_idx4.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1
  %_2.sroa.5.0..sroa_cast.i.i.i = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %_2.sroa.5.0..sroa_idx4.i.i.i to i64*
  store i64 0, i64* %_2.sroa.5.0..sroa_cast.i.i.i, align 16, !alias.scope !189, !noalias !171
  %_2.sroa.6.0..sroa_idx6.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 2
  store i8* getelementptr inbounds (<{ [16 x i8] }>, <{ [16 x i8] }>* @alloc67, i64 0, i32 0, i64 0), i8** %_2.sroa.6.0..sroa_idx6.i.i.i, align 8, !alias.scope !189, !noalias !171
  tail call void @llvm.experimental.noalias.scope.decl(metadata !192)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !195)
  %10 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %10), !noalias !197
; invoke std::sys_common::mutex::MovableMutex::new
  %11 = invoke i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E()
          to label %bb1.i.i unwind label %cleanup.i.i, !noalias !197

cleanup.i.i:                                      ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"
  %12 = landingpad { i8*, i32 }
          cleanup
  br label %bb6.i.i

bb1.i.i:                                          ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %.0..sroa_idx.i.i, align 4, !noalias !197
; invoke std::sync::poison::Flag::new
  %13 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E.exit" unwind label %cleanup1.i.i, !noalias !197

cleanup1.i.i:                                     ; preds = %bb1.i.i
  %14 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #25
          to label %bb6.i.i unwind label %abort.i.i, !noalias !197

abort.i.i:                                        ; preds = %cleanup1.i.i
  %15 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !197
  unreachable

bb6.i.i:                                          ; preds = %cleanup1.i.i, %cleanup.i.i
  %.pn.i.i = phi { i8*, i32 } [ %14, %cleanup1.i.i ], [ %12, %cleanup.i.i ]
  call fastcc void bitcast (void (%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)* @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE" to void (%"std::collections::hash::map::HashMap<i64, ObjectInfo>"*)*)(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %_2.i) #25, !noalias !198
  resume { i8*, i32 } %.pn.i.i

"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E.exit": ; preds = %bb1.i.i
  %_4.sroa.0.0..sroa_idx26.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 3, i32 0, i32 0
  %_4.sroa.0.0..sroa_idx2627.i.i = bitcast %"hashbrown::map::HashMap<i64, ObjectInfo, std::collections::hash::map::RandomState>"* %_4.sroa.0.0..sroa_idx26.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(48) %_4.sroa.0.0..sroa_idx2627.i.i, i8* noundef nonnull align 16 dereferenceable(48) %1, i64 48, i1 false), !alias.scope !199
  %16 = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %16, align 8, !alias.scope !198, !noalias !195
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %13, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !198, !noalias !195
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %10), !noalias !197
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1), !noalias !171
  ret void
}

; core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
; Function Attrs: nounwind nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nocapture readonly %_1) unnamed_addr #8 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %0 = getelementptr inbounds %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_1, i64 0, i32 0, i32 0, i32 1
  tail call void @llvm.experimental.noalias.scope.decl(metadata !200) #24
  %1 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %0 to i64*
  %_2.i.i.i.i.i = load i64, i64* %1, align 8, !alias.scope !203
  %2 = icmp eq i64 %_2.i.i.i.i.i, 0
  br i1 %2, label %"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit", label %bb2.i.i.i.i

bb2.i.i.i.i:                                      ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !206) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !209) #24
  %3 = add i64 %_2.i.i.i.i.i, 1
  %4 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %3, i64 32) #24
  %5 = extractvalue { i64, i1 } %4, 1
  %6 = xor i1 %5, true
  tail call void @llvm.assume(i1 %6) #24
  %7 = extractvalue { i64, i1 } %4, 0
  %_31.i.i.i.i.i.i.i = add i64 %_2.i.i.i.i.i, 17
  %8 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %7, i64 %_31.i.i.i.i.i.i.i) #24
  %9 = extractvalue { i64, i1 } %8, 1
  %10 = xor i1 %9, true
  tail call void @llvm.assume(i1 %10) #24
  %11 = extractvalue { i64, i1 } %8, 0
  %12 = icmp eq i64 %11, 0
  br i1 %12, label %"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit", label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb2.i.i.i.i
  %13 = getelementptr inbounds %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_1, i64 0, i32 0, i32 0, i32 1, i32 1, i32 2
  %_17.i.i.i.i.i.i = load i8*, i8** %13, align 8, !alias.scope !212, !nonnull !85, !noundef !85
  %14 = sub i64 0, %7
  %15 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i.i, i64 %14
  tail call void @__rust_dealloc(i8* nonnull %15, i64 %11, i64 16) #24, !noalias !212
  br label %"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit"

"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit": ; preds = %start, %bb2.i.i.i.i, %bb2.i.i.i.i.i.i.i
  ret void
}

; core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
; Function Attrs: nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !213)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !213, !nonnull !85, !align !86, !noundef !85
  %_5.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i = load i8, i8* %_5.i, align 8, !alias.scope !213
  %_5.not.i.i = icmp eq i8 %_5.val.i, 0
  br i1 %_5.not.i.i, label %bb2.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

bb2.i.i:                                          ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !213
  %_1.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i: ; preds = %bb2.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !213
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %bb5.i.i

bb5.i.i:                                          ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i
  %_6.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i monotonic, align 4, !noalias !213
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i: ; preds = %bb5.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i, %bb2.i.i, %start
  %_5.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i, i32 0 release, align 4, !noalias !213
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i, label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE.exit"

bb2.i.i.i:                                        ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i
  %_2.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i), !noalias !213
  br label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE.exit"

"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, %bb2.i.i.i
  ret void
}

; core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
; Function Attrs: nonlazybind uwtable
define internal void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !216)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !216, !nonnull !85, !align !86, !noundef !85
  %_5.i.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i.i = load i8, i8* %_5.i.i, align 8, !alias.scope !216
  %_5.not.i.i.i = icmp eq i8 %_5.val.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !216
  %_1.i.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !216
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  %_6.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i monotonic, align 4, !noalias !216
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %start
  %_5.i.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i.i, i32 0 release, align 4, !noalias !216
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
  %_2.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i), !noalias !216
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
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
define internal noundef zeroext i1 @"_ZN76_$LT$std..sync..poison..PoisonError$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h0cd32de15374fa48E"({ i64*, i8 }* noalias nocapture noundef readonly align 8 dereferenceable(16) %self, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64) %f) unnamed_addr #6 {
start:
  %_4 = alloca %"core::fmt::builders::DebugStruct", align 8
  %0 = bitcast %"core::fmt::builders::DebugStruct"* %_4 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %0)
; call core::fmt::Formatter::debug_struct
  call void @_ZN4core3fmt9Formatter12debug_struct17h65c357ef1edbbc54E(%"core::fmt::builders::DebugStruct"* noalias nocapture noundef nonnull sret(%"core::fmt::builders::DebugStruct") dereferenceable(16) %_4, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f, [0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [11 x i8] }>* @alloc432 to [0 x i8]*), i64 11)
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
  %tmp.sroa.0.0.copyload.i.i.i = load i8*, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !219
  %tmp.sroa.4.0..sroa_idx3.i.i.i = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %self, i64 0, i32 1
  %tmp.sroa.4.0.copyload.i.i.i = load i64, i64* %tmp.sroa.4.0..sroa_idx3.i.i.i, align 8, !alias.scope !219
  store i8* null, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !219
  %0 = icmp eq i8* %tmp.sroa.0.0.copyload.i.i.i, null
  br i1 %0, label %bb2, label %bb4

bb2:                                              ; preds = %start
; call std::process::abort
  tail call void @_ZN3std7process5abort17h9abe461bf20ade28E() #23
  unreachable

bb4:                                              ; preds = %start
  %1 = tail call align 8 dereferenceable_or_null(16) i8* @__rust_alloc(i64 16, i64 8) #24, !noalias !224
  %2 = icmp eq i8* %1, null
  br i1 %2, label %bb3.i.i, label %"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit"

bb3.i.i:                                          ; preds = %bb4
; call alloc::alloc::handle_alloc_error
  tail call void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64 16, i64 noundef 8) #23, !noalias !224
  unreachable

"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit": ; preds = %bb4
  %3 = bitcast i8* %1 to i8**
  store i8* %tmp.sroa.0.0.copyload.i.i.i, i8** %3, align 8, !noalias !224
  %4 = getelementptr inbounds i8, i8* %1, i64 8
  %5 = bitcast i8* %4 to i64*
  store i64 %tmp.sroa.4.0.copyload.i.i.i, i64* %5, align 8, !noalias !224
  %_13.0.cast = bitcast i8* %1 to {}*
  %6 = insertvalue { {}*, [3 x i64]* } undef, {}* %_13.0.cast, 0
  %7 = insertvalue { {}*, [3 x i64]* } %6, [3 x i64]* bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.8 to [3 x i64]*), 1
  ret { {}*, [3 x i64]* } %7
}

; hashbrown::raw::RawTable<T,A>::reserve_rehash
; Function Attrs: cold noinline nonlazybind uwtable
define internal fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h320d5dd485a72968E"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias nocapture noundef align 8 dereferenceable(32) %self, i64* noalias noundef readonly align 8 dereferenceable(16) %0) unnamed_addr #11 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !227)
  %1 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 4
  %_9.i = load i64, i64* %1, align 8, !alias.scope !227
  %2 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %_9.i, i64 1) #24
  %3 = extractvalue { i64, i1 } %2, 0
  %4 = extractvalue { i64, i1 } %2, 1
  br i1 %4, label %bb2.i, label %bb4.i

bb2.i:                                            ; preds = %start
; call hashbrown::raw::Fallibility::capacity_overflow
  %5 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !227
  %_13.0.i = extractvalue { i64, i64 } %5, 0
  %_13.1.i = extractvalue { i64, i64 } %5, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb4.i:                                            ; preds = %start
  %6 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self to i64*
  %_16.i = load i64, i64* %6, align 8, !alias.scope !227
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !230)
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
  %12 = tail call i64 @llvm.ctlz.i64(i64 %p.i.i.i.i.i.i.i, i1 true) #24, !range !233
  %13 = lshr i64 -1, %12
  %phi.bo.i.i.i.i.i.i = add i64 %13, 1
  br label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb8.i.i.i.i.i, %bb1.i.i.i.i.i
  %.sroa.4.0.i.ph.i.i.i.i = phi i64 [ %phi.bo.i.i.i.i.i.i, %bb8.i.i.i.i.i ], [ %..i.i.i.i.i, %bb1.i.i.i.i.i ]
  %14 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %.sroa.4.0.i.ph.i.i.i.i, i64 32) #24
  %15 = extractvalue { i64, i1 } %14, 1
  br i1 %15, label %bb2.i.i.i.i.i, label %bb9.i.i.i.i.i.i

bb9.i.i.i.i.i.i:                                  ; preds = %bb7.i.i.i.i
  %16 = extractvalue { i64, i1 } %14, 0
  %_31.i.i.i.i.i.i = add nuw nsw i64 %.sroa.4.0.i.ph.i.i.i.i, 16
  %17 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %16, i64 %_31.i.i.i.i.i.i) #24
  %18 = extractvalue { i64, i1 } %17, 1
  br i1 %18, label %bb2.i.i.i.i.i, label %bb4.i.i.i.i.i

bb2.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i, %bb7.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %19 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !234
  br label %bb5.i.i

bb4.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i
  %20 = extractvalue { i64, i1 } %17, 0
  %21 = icmp eq i64 %20, 0
  br i1 %21, label %bb13.i.i.i.i, label %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i

_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i: ; preds = %bb4.i.i.i.i.i
  %22 = tail call align 16 i8* @__rust_alloc(i64 %20, i64 16) #24, !noalias !234
  %23 = icmp eq i8* %22, null
  br i1 %23, label %bb15.i.i.i.i.i, label %bb13.i.i.i.i

bb15.i.i.i.i.i:                                   ; preds = %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
; call hashbrown::raw::Fallibility::alloc_err
  %24 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility9alloc_err17h3f1a17e1376e6326E(i1 noundef zeroext true, i64 %20, i64 noundef 16), !noalias !234
  br label %bb5.i.i

bb9.i.i.i.i:                                      ; preds = %bb5.i.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %25 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !241
  br label %bb5.i.i

bb13.i.i.i.i:                                     ; preds = %bb4.i.i.i.i.i, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
  %.sroa.0.0.i.i.i.i.i.i.i.i3 = phi i8* [ %22, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i ], [ inttoptr (i64 16 to i8*), %bb4.i.i.i.i.i ]
  %26 = getelementptr inbounds i8, i8* %.sroa.0.0.i.i.i.i.i.i.i.i3, i64 %16
  %_42.i.i.i.i.i = add nsw i64 %.sroa.4.0.i.ph.i.i.i.i, -1
  %_2.i.i10.i.i.i.i = icmp ult i64 %_42.i.i.i.i.i, 8
  %_4.i.i.i.i.i.i = lshr i64 %.sroa.4.0.i.ph.i.i.i.i, 3
  %27 = mul nuw nsw i64 %_4.i.i.i.i.i.i, 7
  %.0.i.i.i.i.i.i = select i1 %_2.i.i10.i.i.i.i, i64 %_42.i.i.i.i.i, i64 %27
  tail call void @llvm.memset.p0i8.i64(i8* nonnull align 16 %26, i8 -1, i64 %_31.i.i.i.i.i.i, i1 false) #24, !noalias !244
  %28 = sub i64 %.0.i.i.i.i.i.i, %_9.i
  %.not.i.i = icmp eq i64 %_5.i.i, 0
  %29 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %a.i.i.sroa.4.0.copyload.pre.i.i = load i8*, i8** %29, align 8, !alias.scope !245
  br i1 %.not.i.i, label %bb26.thread.i.i, label %bb15.lr.ph.i.i

bb5.i.i:                                          ; preds = %bb9.i.i.i.i, %bb15.i.i.i.i.i, %bb2.i.i.i.i.i
  %.pn.i.pn.i.i.i = phi { i64, i64 } [ %25, %bb9.i.i.i.i ], [ %24, %bb15.i.i.i.i.i ], [ %19, %bb2.i.i.i.i.i ]
  %_7.sroa.7.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 0
  %_7.sroa.13.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb26.thread.i.i:                                  ; preds = %bb13.i.i.i.i
  %30 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !246
  store i8* %26, i8** %29, align 8, !alias.scope !246
  store i64 %28, i64* %30, align 8, !alias.scope !246
  br label %bb2.i.i.i14.i.i

bb15.lr.ph.i.i:                                   ; preds = %bb13.i.i.i.i
  %table.idx.val4.i.cast.i.i = bitcast i8* %a.i.i.sroa.4.0.copyload.pre.i.i to { i64, %ObjectInfo }*
  %31 = bitcast i8* %26 to <16 x i8>*
  %_6.idx.val.i.i.i.i = load i64, i64* %0, align 8
  %32 = getelementptr i64, i64* %0, i64 1
  %_6.idx1.val.i.i.i.i = load i64, i64* %32, align 8
  %33 = xor i64 %_6.idx.val.i.i.i.i, 8317987319222330741
  %34 = xor i64 %_6.idx1.val.i.i.i.i, 7237128888997146477
  %35 = xor i64 %_6.idx.val.i.i.i.i, 7816392313619706465
  %36 = add i64 %34, %33
  %37 = tail call i64 @llvm.fshl.i64(i64 %34, i64 %34, i64 13) #24
  %38 = xor i64 %36, %37
  %39 = tail call i64 @llvm.fshl.i64(i64 %36, i64 %36, i64 32) #24
  %40 = tail call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 17) #24
  br label %bb15.i.i

bb15.i.i:                                         ; preds = %bb9.backedge.i.i, %bb15.lr.ph.i.i
  %iter.sroa.0.0100.i.i = phi i64 [ 0, %bb15.lr.ph.i.i ], [ %41, %bb9.backedge.i.i ]
  %41 = add nuw i64 %iter.sroa.0.0100.i.i, 1
  %42 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %iter.sroa.0.0100.i.i
  %_29.i.i = load i8, i8* %42, align 1, !noalias !245
  %43 = icmp sgt i8 %_29.i.i, -1
  br i1 %43, label %bb18.i.i, label %bb9.backedge.i.i

bb9.backedge.i.i:                                 ; preds = %bb22.i.i, %bb15.i.i
  %exitcond.not.i.i = icmp eq i64 %iter.sroa.0.0100.i.i, %_16.i
  br i1 %exitcond.not.i.i, label %bb26.i.i, label %bb15.i.i

bb18.i.i:                                         ; preds = %bb15.i.i
  %44 = sub i64 0, %iter.sroa.0.0100.i.i
  %45 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %table.idx.val4.i.cast.i.i, i64 %44, i32 0
  %46 = getelementptr inbounds i64, i64* %45, i64 -4
  %_7.idx.val.i.i.i = load i64, i64* %46, align 8, !alias.scope !253, !noalias !256
  %47 = xor i64 %_7.idx.val.i.i.i, %_6.idx1.val.i.i.i.i
  %48 = xor i64 %47, 8387220255154660723
  %49 = add i64 %48, %35
  %50 = tail call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 16) #24
  %51 = xor i64 %50, %49
  %52 = add i64 %51, %39
  %53 = tail call i64 @llvm.fshl.i64(i64 %51, i64 %51, i64 21) #24
  %54 = xor i64 %53, %52
  %55 = add i64 %38, %49
  %56 = xor i64 %55, %40
  %57 = tail call i64 @llvm.fshl.i64(i64 %55, i64 %55, i64 32) #24
  %58 = xor i64 %52, %_7.idx.val.i.i.i
  %59 = xor i64 %54, 576460752303423488
  %60 = add i64 %58, %56
  %61 = tail call i64 @llvm.fshl.i64(i64 %56, i64 %56, i64 13) #24
  %62 = xor i64 %60, %61
  %63 = tail call i64 @llvm.fshl.i64(i64 %60, i64 %60, i64 32) #24
  %64 = add i64 %59, %57
  %65 = tail call i64 @llvm.fshl.i64(i64 %54, i64 %59, i64 16) #24
  %66 = xor i64 %65, %64
  %67 = add i64 %66, %63
  %68 = tail call i64 @llvm.fshl.i64(i64 %66, i64 %66, i64 21) #24
  %69 = xor i64 %68, %67
  %70 = add i64 %64, %62
  %71 = tail call i64 @llvm.fshl.i64(i64 %62, i64 %62, i64 17) #24
  %72 = xor i64 %70, %71
  %73 = tail call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 32) #24
  %74 = xor i64 %67, 576460752303423488
  %75 = xor i64 %73, 255
  %76 = add i64 %74, %72
  %77 = tail call i64 @llvm.fshl.i64(i64 %72, i64 %72, i64 13) #24
  %78 = xor i64 %76, %77
  %79 = tail call i64 @llvm.fshl.i64(i64 %76, i64 %76, i64 32) #24
  %80 = add i64 %69, %75
  %81 = tail call i64 @llvm.fshl.i64(i64 %69, i64 %69, i64 16) #24
  %82 = xor i64 %81, %80
  %83 = add i64 %82, %79
  %84 = tail call i64 @llvm.fshl.i64(i64 %82, i64 %82, i64 21) #24
  %85 = xor i64 %84, %83
  %86 = add i64 %78, %80
  %87 = tail call i64 @llvm.fshl.i64(i64 %78, i64 %78, i64 17) #24
  %88 = xor i64 %86, %87
  %89 = tail call i64 @llvm.fshl.i64(i64 %86, i64 %86, i64 32) #24
  %90 = add i64 %88, %83
  %91 = tail call i64 @llvm.fshl.i64(i64 %88, i64 %88, i64 13) #24
  %92 = xor i64 %91, %90
  %93 = tail call i64 @llvm.fshl.i64(i64 %90, i64 %90, i64 32) #24
  %94 = add i64 %85, %89
  %95 = tail call i64 @llvm.fshl.i64(i64 %85, i64 %85, i64 16) #24
  %96 = xor i64 %95, %94
  %97 = add i64 %96, %93
  %98 = tail call i64 @llvm.fshl.i64(i64 %96, i64 %96, i64 21) #24
  %99 = xor i64 %98, %97
  %100 = add i64 %92, %94
  %101 = tail call i64 @llvm.fshl.i64(i64 %92, i64 %92, i64 17) #24
  %102 = xor i64 %101, %100
  %103 = tail call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 32) #24
  %104 = add i64 %102, %97
  %105 = tail call i64 @llvm.fshl.i64(i64 %102, i64 %102, i64 13) #24
  %106 = xor i64 %105, %104
  %107 = add i64 %99, %103
  %108 = tail call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 16) #24
  %109 = xor i64 %108, %107
  %110 = tail call i64 @llvm.fshl.i64(i64 %109, i64 %109, i64 21) #24
  %111 = add i64 %106, %107
  %112 = tail call i64 @llvm.fshl.i64(i64 %106, i64 %106, i64 17) #24
  %113 = tail call i64 @llvm.fshl.i64(i64 %111, i64 %111, i64 32) #24
  %_17.i.i.i.i.i.i.i.i.i = xor i64 %111, %110
  %114 = xor i64 %_17.i.i.i.i.i.i.i.i.i, %112
  %115 = xor i64 %114, %113
  %_3.i.i.i.i.i = and i64 %115, %_42.i.i.i.i.i
  %116 = getelementptr inbounds i8, i8* %26, i64 %_3.i.i.i.i.i
  %117 = bitcast i8* %116 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %117, align 1, !noalias !260
  %118 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %119 = bitcast <16 x i1> %118 to i16
  %.not23.i.i.i.i = icmp eq i16 %119, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb7.i.i8.i.i:                                     ; preds = %bb17.i.i.i.i, %bb18.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i, %bb18.i.i ], [ %125, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %119, %bb18.i.i ], [ %129, %bb17.i.i.i.i ]
  %120 = tail call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i = zext i16 %120 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_42.i.i.i.i.i
  %121 = getelementptr inbounds i8, i8* %26, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %121, align 1, !noalias !267
  %122 = icmp sgt i8 %_23.i.i.i.i, -1
  br i1 %122, label %bb11.i.i.i.i, label %bb22.i.i

bb17.i.i.i.i:                                     ; preds = %bb18.i.i, %bb17.i.i.i.i
  %probe_seq.sroa.0.025.i.i.i.i = phi i64 [ %125, %bb17.i.i.i.i ], [ %_3.i.i.i.i.i, %bb18.i.i ]
  %probe_seq.sroa.7.024.i.i.i.i = phi i64 [ %123, %bb17.i.i.i.i ], [ 0, %bb18.i.i ]
  %123 = add i64 %probe_seq.sroa.7.024.i.i.i.i, 16
  %124 = add i64 %123, %probe_seq.sroa.0.025.i.i.i.i
  %125 = and i64 %124, %_42.i.i.i.i.i
  %126 = getelementptr inbounds i8, i8* %26, i64 %125
  %127 = bitcast i8* %126 to <16 x i8>*
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %127, align 1, !noalias !260
  %128 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %129 = bitcast <16 x i1> %128 to i16
  %.not.i.i.i.i = icmp eq i16 %129, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i8.i.i
  %130 = load <16 x i8>, <16 x i8>* %31, align 16, !noalias !268
  %131 = icmp slt <16 x i8> %130, zeroinitializer
  %132 = bitcast <16 x i1> %131 to i16
  %133 = tail call i16 @llvm.cttz.i16(i16 %132, i1 true) #24, !range !27
  %_2.i.i.i.i.i = zext i16 %133 to i64
  br label %bb22.i.i

bb22.i.i:                                         ; preds = %bb11.i.i.i.i, %bb7.i.i8.i.i
  %.0.i.i.i.i = phi i64 [ %_2.i.i.i.i.i, %bb11.i.i.i.i ], [ %result.i.i.i.i, %bb7.i.i8.i.i ]
  %top7.i.i.i.i.i = lshr i64 %115, 57
  %134 = trunc i64 %top7.i.i.i.i.i to i8
  %135 = add i64 %.0.i.i.i.i, -16
  %_5.i.i.i9.i.i = and i64 %135, %_42.i.i.i.i.i
  %index2.i.i.i.i.i = add i64 %_5.i.i.i9.i.i, 16
  %136 = getelementptr inbounds i8, i8* %26, i64 %.0.i.i.i.i
  store i8 %134, i8* %136, align 1, !noalias !273
  %137 = getelementptr inbounds i8, i8* %26, i64 %index2.i.i.i.i.i
  store i8 %134, i8* %137, align 1, !noalias !273
  %_12.neg.i.i.i = xor i64 %iter.sroa.0.0100.i.i, -1
  %_11.neg.i.i.i = shl i64 %_12.neg.i.i.i, 5
  %138 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %_11.neg.i.i.i
  %_12.neg.i10.i.i = xor i64 %.0.i.i.i.i, -1
  %_11.neg.i11.i.i = shl i64 %_12.neg.i10.i.i, 5
  %139 = getelementptr inbounds i8, i8* %26, i64 %_11.neg.i11.i.i
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 16 dereferenceable(32) %139, i8* noundef nonnull align 1 dereferenceable(32) %138, i64 32, i1 false) #24, !noalias !245
  br label %bb9.backedge.i.i

bb26.i.i:                                         ; preds = %bb9.backedge.i.i
  %140 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !278
  store i8* %26, i8** %29, align 8, !alias.scope !278
  store i64 %28, i64* %140, align 8, !alias.scope !278
  %141 = icmp eq i64 %_16.i, 0
  br i1 %141, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit", label %bb2.i.i.i14.i.i

bb2.i.i.i14.i.i:                                  ; preds = %bb26.i.i, %bb26.thread.i.i
  %142 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %_5.i.i, i64 32) #24
  %143 = extractvalue { i64, i1 } %142, 1
  %144 = xor i1 %143, true
  tail call void @llvm.assume(i1 %144) #24
  %145 = extractvalue { i64, i1 } %142, 0
  %_31.i.i.i.i.i.i.i = add i64 %_16.i, 17
  %146 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %145, i64 %_31.i.i.i.i.i.i.i) #24
  %147 = extractvalue { i64, i1 } %146, 1
  %148 = xor i1 %147, true
  tail call void @llvm.assume(i1 %148) #24
  %149 = extractvalue { i64, i1 } %146, 0
  %150 = icmp eq i64 %149, 0
  br i1 %150, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit", label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb2.i.i.i14.i.i
  %151 = icmp ne i8* %a.i.i.sroa.4.0.copyload.pre.i.i, null
  tail call void @llvm.assume(i1 %151)
  %152 = sub i64 0, %145
  %153 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %152
  tail call void @__rust_dealloc(i8* nonnull %153, i64 %149, i64 16) #24, !noalias !281
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb7.i:                                            ; preds = %bb4.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !288)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !291)
  %154 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %self.idx12.val.i.i.i = load i8*, i8** %154, align 8, !alias.scope !294
  %155 = bitcast i8* %self.idx12.val.i.i.i to { i64, %ObjectInfo }*
  br label %bb4.i.i.i

bb4.i.i.i:                                        ; preds = %bb6.i.i.i, %bb7.i
  %iter.sroa.0.0.i.i.i = phi i64 [ 0, %bb7.i ], [ %iter.sroa.0.165.i.i.i, %bb6.i.i.i ]
  %_2.not.i.i.i.i = phi i1 [ false, %bb7.i ], [ true, %bb6.i.i.i ]
  br i1 %_2.not.i.i.i.i, label %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i", label %bb1.i.i.i.i

bb1.i.i.i.i:                                      ; preds = %bb4.i.i.i
  %156 = icmp ult i64 %iter.sroa.0.0.i.i.i, %_5.i.i
  br i1 %156, label %bb6.i.i.i, label %bb8.i.i.i

"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i": ; preds = %bb4.i.i.i
  %157 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %iter.sroa.0.0.i.i.i, i64 15) #24
  %158 = extractvalue { i64, i1 } %157, 0
  %159 = extractvalue { i64, i1 } %157, 1
  %_5.1.not.i.i.i.i.i.i.i.i = xor i1 %159, true
  %160 = icmp ult i64 %158, %_5.i.i
  %or.cond.i.i.i.i.i.i = select i1 %_5.1.not.i.i.i.i.i.i.i.i, i1 %160, i1 false
  br i1 %or.cond.i.i.i.i.i.i, label %bb6.i.i.i, label %bb8.i.i.i

bb8.i.i.i:                                        ; preds = %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i", %bb1.i.i.i.i
  %_25.i.i.i = icmp ult i64 %_5.i.i, 16
  br i1 %_25.i.i.i, label %bb5.i2.i, label %bb5.thread.i.i

bb6.i.i.i:                                        ; preds = %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i", %bb1.i.i.i.i
  %_3.val.i.i.pn.i67.i.i.i = phi i64 [ %158, %"_ZN105_$LT$core..iter..adapters..step_by..StepBy$LT$I$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17hc5c813c954344339E.exit.i.i.i" ], [ %iter.sroa.0.0.i.i.i, %bb1.i.i.i.i ]
  %iter.sroa.0.165.i.i.i = add nuw i64 %_3.val.i.i.pn.i67.i.i.i, 1
  %161 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_3.val.i.i.pn.i67.i.i.i
  %162 = bitcast i8* %161 to <2 x i64>*
  %163 = bitcast i8* %161 to <16 x i8>*
  %164 = load <16 x i8>, <16 x i8>* %163, align 16, !noalias !295
  %.lobit.i.i.i.i = ashr <16 x i8> %164, <i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7>
  %165 = bitcast <16 x i8> %.lobit.i.i.i.i to <2 x i64>
  %166 = or <2 x i64> %165, <i64 -9187201950435737472, i64 -9187201950435737472>
  store <2 x i64> %166, <2 x i64>* %162, align 16, !noalias !300
  br label %bb4.i.i.i

bb5.thread.i.i:                                   ; preds = %bb8.i.i.i
  %167 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_5.i.i
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(16) %167, i8* noundef nonnull align 1 dereferenceable(16) %self.idx12.val.i.i.i, i64 16, i1 false) #24, !noalias !294
  br label %bb12.lr.ph.i.i

bb5.i2.i:                                         ; preds = %bb8.i.i.i
  %168 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 16
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* nonnull align 1 %168, i8* align 1 %self.idx12.val.i.i.i, i64 %_5.i.i, i1 false) #24, !noalias !294
  %.not.i1.i = icmp eq i64 %_5.i.i, 0
  br i1 %.not.i1.i, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i", label %bb12.lr.ph.i.i

bb12.lr.ph.i.i:                                   ; preds = %bb5.i2.i, %bb5.thread.i.i
  %169 = getelementptr i64, i64* %0, i64 1
  %_6.idx.val.i.i.i10.i = load i64, i64* %0, align 8
  %_6.idx1.val.i.i.i12.i = load i64, i64* %169, align 8
  %170 = xor i64 %_6.idx.val.i.i.i10.i, 8317987319222330741
  %171 = xor i64 %_6.idx1.val.i.i.i12.i, 7237128888997146477
  %172 = xor i64 %_6.idx.val.i.i.i10.i, 7816392313619706465
  %173 = add i64 %171, %170
  %174 = tail call i64 @llvm.fshl.i64(i64 %171, i64 %171, i64 13) #24
  %175 = xor i64 %173, %174
  %176 = tail call i64 @llvm.fshl.i64(i64 %173, i64 %173, i64 32) #24
  %177 = tail call i64 @llvm.fshl.i64(i64 %175, i64 %175, i64 17) #24
  %178 = bitcast i8* %self.idx12.val.i.i.i to <16 x i8>*
  %179 = bitcast i8* %self.idx12.val.i.i.i to { i64, %ObjectInfo }*
  br label %bb12.i.i

bb12.i.i:                                         ; preds = %bb40.i.i, %bb12.lr.ph.i.i
  %table.idx.val4.i41.i.i = phi { i64, %ObjectInfo }* [ %155, %bb12.lr.ph.i.i ], [ %table.idx.val4.i42.i.i, %bb40.i.i ]
  %iter.sroa.0.028.i.i = phi i64 [ 0, %bb12.lr.ph.i.i ], [ %180, %bb40.i.i ]
  %180 = add nuw i64 %iter.sroa.0.028.i.i, 1
  %181 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.028.i.i
  %_23.i.i = load i8, i8* %181, align 1, !noalias !303
  %_22.not.i.i = icmp eq i8 %_23.i.i, -128
  br i1 %_22.not.i.i, label %bb14.i.i, label %bb40.i.i

bb40.i.i:                                         ; preds = %bb34.i.i, %bb27.i.i, %bb12.i.i
  %table.idx.val4.i42.i.i = phi { i64, %ObjectInfo }* [ %155, %bb34.i.i ], [ %179, %bb27.i.i ], [ %table.idx.val4.i41.i.i, %bb12.i.i ]
  %exitcond.not.i3.i = icmp eq i64 %iter.sroa.0.028.i.i, %_16.i
  br i1 %exitcond.not.i3.i, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i", label %bb12.i.i

bb14.i.i:                                         ; preds = %bb12.i.i
  %_12.neg.i.i4.i = xor i64 %iter.sroa.0.028.i.i, -1
  %_11.neg.i.i5.i = shl i64 %_12.neg.i.i4.i, 5
  %182 = getelementptr i8, i8* %self.idx12.val.i.i.i, i64 %_11.neg.i.i5.i
  %183 = sub i64 0, %iter.sroa.0.028.i.i
  %184 = getelementptr inbounds i8, i8* %182, i64 16
  %185 = bitcast i8* %182 to <16 x i8>*
  %186 = bitcast i8* %182 to <16 x i8>*
  %187 = bitcast i8* %184 to <16 x i8>*
  %188 = bitcast i8* %184 to <16 x i8>*
  br label %bb19.i.i

bb19.i.i:                                         ; preds = %bb2.i.i10.i.i.preheader, %bb14.i.i
  %table.idx.val4.i.i.i = phi { i64, %ObjectInfo }* [ %table.idx.val4.i41.i.i, %bb14.i.i ], [ %155, %bb2.i.i10.i.i.preheader ]
  %189 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %table.idx.val4.i.i.i, i64 %183, i32 0
  %190 = getelementptr inbounds i64, i64* %189, i64 -4
  %_7.idx.val.i.i7.i = load i64, i64* %190, align 8, !alias.scope !304, !noalias !307
  %191 = xor i64 %_7.idx.val.i.i7.i, %_6.idx1.val.i.i.i12.i
  %192 = xor i64 %191, 8387220255154660723
  %193 = add i64 %192, %172
  %194 = tail call i64 @llvm.fshl.i64(i64 %192, i64 %192, i64 16) #24
  %195 = xor i64 %194, %193
  %196 = add i64 %195, %176
  %197 = tail call i64 @llvm.fshl.i64(i64 %195, i64 %195, i64 21) #24
  %198 = xor i64 %197, %196
  %199 = add i64 %175, %193
  %200 = xor i64 %199, %177
  %201 = tail call i64 @llvm.fshl.i64(i64 %199, i64 %199, i64 32) #24
  %202 = xor i64 %196, %_7.idx.val.i.i7.i
  %203 = xor i64 %198, 576460752303423488
  %204 = add i64 %202, %200
  %205 = tail call i64 @llvm.fshl.i64(i64 %200, i64 %200, i64 13) #24
  %206 = xor i64 %204, %205
  %207 = tail call i64 @llvm.fshl.i64(i64 %204, i64 %204, i64 32) #24
  %208 = add i64 %203, %201
  %209 = tail call i64 @llvm.fshl.i64(i64 %198, i64 %203, i64 16) #24
  %210 = xor i64 %209, %208
  %211 = add i64 %210, %207
  %212 = tail call i64 @llvm.fshl.i64(i64 %210, i64 %210, i64 21) #24
  %213 = xor i64 %212, %211
  %214 = add i64 %208, %206
  %215 = tail call i64 @llvm.fshl.i64(i64 %206, i64 %206, i64 17) #24
  %216 = xor i64 %214, %215
  %217 = tail call i64 @llvm.fshl.i64(i64 %214, i64 %214, i64 32) #24
  %218 = xor i64 %211, 576460752303423488
  %219 = xor i64 %217, 255
  %220 = add i64 %218, %216
  %221 = tail call i64 @llvm.fshl.i64(i64 %216, i64 %216, i64 13) #24
  %222 = xor i64 %220, %221
  %223 = tail call i64 @llvm.fshl.i64(i64 %220, i64 %220, i64 32) #24
  %224 = add i64 %213, %219
  %225 = tail call i64 @llvm.fshl.i64(i64 %213, i64 %213, i64 16) #24
  %226 = xor i64 %225, %224
  %227 = add i64 %226, %223
  %228 = tail call i64 @llvm.fshl.i64(i64 %226, i64 %226, i64 21) #24
  %229 = xor i64 %228, %227
  %230 = add i64 %222, %224
  %231 = tail call i64 @llvm.fshl.i64(i64 %222, i64 %222, i64 17) #24
  %232 = xor i64 %230, %231
  %233 = tail call i64 @llvm.fshl.i64(i64 %230, i64 %230, i64 32) #24
  %234 = add i64 %232, %227
  %235 = tail call i64 @llvm.fshl.i64(i64 %232, i64 %232, i64 13) #24
  %236 = xor i64 %235, %234
  %237 = tail call i64 @llvm.fshl.i64(i64 %234, i64 %234, i64 32) #24
  %238 = add i64 %229, %233
  %239 = tail call i64 @llvm.fshl.i64(i64 %229, i64 %229, i64 16) #24
  %240 = xor i64 %239, %238
  %241 = add i64 %240, %237
  %242 = tail call i64 @llvm.fshl.i64(i64 %240, i64 %240, i64 21) #24
  %243 = xor i64 %242, %241
  %244 = add i64 %236, %238
  %245 = tail call i64 @llvm.fshl.i64(i64 %236, i64 %236, i64 17) #24
  %246 = xor i64 %245, %244
  %247 = tail call i64 @llvm.fshl.i64(i64 %244, i64 %244, i64 32) #24
  %248 = add i64 %246, %241
  %249 = tail call i64 @llvm.fshl.i64(i64 %246, i64 %246, i64 13) #24
  %250 = xor i64 %249, %248
  %251 = add i64 %243, %247
  %252 = tail call i64 @llvm.fshl.i64(i64 %243, i64 %243, i64 16) #24
  %253 = xor i64 %252, %251
  %254 = tail call i64 @llvm.fshl.i64(i64 %253, i64 %253, i64 21) #24
  %255 = add i64 %250, %251
  %256 = tail call i64 @llvm.fshl.i64(i64 %250, i64 %250, i64 17) #24
  %257 = tail call i64 @llvm.fshl.i64(i64 %255, i64 %255, i64 32) #24
  %_17.i.i.i.i.i.i.i.i13.i = xor i64 %255, %254
  %258 = xor i64 %_17.i.i.i.i.i.i.i.i13.i, %256
  %259 = xor i64 %258, %257
  %_3.i.i3.i.i = and i64 %259, %_16.i
  %260 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_3.i.i3.i.i
  %261 = bitcast i8* %260 to <16 x i8>*
  %.0.copyload.i2122.i.i.i = load <16 x i8>, <16 x i8>* %261, align 1, !noalias !311
  %262 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i, zeroinitializer
  %263 = bitcast <16 x i1> %262 to i16
  %.not23.i.i.i = icmp eq i16 %263, 0
  br i1 %.not23.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb7.i.i.i:                                        ; preds = %bb17.i.i.i, %bb19.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i = phi i64 [ %_3.i.i3.i.i, %bb19.i.i ], [ %269, %bb17.i.i.i ]
  %.lcssa.i.i.i = phi i16 [ %263, %bb19.i.i ], [ %273, %bb17.i.i.i ]
  %264 = tail call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i14.i = zext i16 %264 to i64
  %_17.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i, %_2.i.i.i.i14.i
  %result.i.i.i = and i64 %_17.i.i.i, %_16.i
  %265 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %result.i.i.i
  %_23.i.i.i = load i8, i8* %265, align 1, !noalias !316
  %266 = icmp sgt i8 %_23.i.i.i, -1
  br i1 %266, label %bb11.i.i.i, label %bb24.i.i

bb17.i.i.i:                                       ; preds = %bb19.i.i, %bb17.i.i.i
  %probe_seq.sroa.0.025.i.i.i = phi i64 [ %269, %bb17.i.i.i ], [ %_3.i.i3.i.i, %bb19.i.i ]
  %probe_seq.sroa.7.024.i.i.i = phi i64 [ %267, %bb17.i.i.i ], [ 0, %bb19.i.i ]
  %267 = add i64 %probe_seq.sroa.7.024.i.i.i, 16
  %268 = add i64 %267, %probe_seq.sroa.0.025.i.i.i
  %269 = and i64 %268, %_16.i
  %270 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %269
  %271 = bitcast i8* %270 to <16 x i8>*
  %.0.copyload.i21.i.i.i = load <16 x i8>, <16 x i8>* %271, align 1, !noalias !311
  %272 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i, zeroinitializer
  %273 = bitcast <16 x i1> %272 to i16
  %.not.i.i.i = icmp eq i16 %273, 0
  br i1 %.not.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb11.i.i.i:                                       ; preds = %bb7.i.i.i
  %274 = load <16 x i8>, <16 x i8>* %178, align 16, !noalias !317
  %275 = icmp slt <16 x i8> %274, zeroinitializer
  %276 = bitcast <16 x i1> %275 to i16
  %277 = tail call i16 @llvm.cttz.i16(i16 %276, i1 true) #24, !range !27
  %_2.i.i4.i.i = zext i16 %277 to i64
  br label %bb24.i.i

bb24.i.i:                                         ; preds = %bb11.i.i.i, %bb7.i.i.i
  %.0.i.i.i = phi i64 [ %_2.i.i4.i.i, %bb11.i.i.i ], [ %result.i.i.i, %bb7.i.i.i ]
  %_12.neg.i5.i.i = xor i64 %.0.i.i.i, -1
  %_11.neg.i6.i.i = shl i64 %_12.neg.i5.i.i, 5
  %278 = getelementptr i8, i8* %self.idx12.val.i.i.i, i64 %_11.neg.i6.i.i
  %279 = sub i64 %iter.sroa.0.028.i.i, %_3.i.i3.i.i
  %280 = sub i64 %.0.i.i.i, %_3.i.i3.i.i
  %_3.i612.i.i.i = xor i64 %280, %279
  %.unshifted.i.i.i = and i64 %_3.i612.i.i.i, %_16.i
  %281 = icmp ult i64 %.unshifted.i.i.i, 16
  br i1 %281, label %bb27.i.i, label %bb31.i.i

bb27.i.i:                                         ; preds = %bb24.i.i
  %top7.i.i.i.i = lshr i64 %259, 57
  %282 = trunc i64 %top7.i.i.i.i to i8
  %283 = add i64 %iter.sroa.0.028.i.i, -16
  %_5.i.i.i.i = and i64 %283, %_16.i
  %index2.i.i.i.i = add i64 %_5.i.i.i.i, 16
  %284 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.028.i.i
  store i8 %282, i8* %284, align 1, !noalias !322
  %285 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i
  store i8 %282, i8* %285, align 1, !noalias !322
  br label %bb40.i.i

bb31.i.i:                                         ; preds = %bb24.i.i
  %286 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %.0.i.i.i
  %prev_ctrl.i.i.i = load i8, i8* %286, align 1, !noalias !327
  %top7.i.i.i.i15.i = lshr i64 %259, 57
  %287 = trunc i64 %top7.i.i.i.i15.i to i8
  %288 = add i64 %.0.i.i.i, -16
  %_5.i.i.i.i16.i = and i64 %288, %_16.i
  %index2.i.i.i.i17.i = add i64 %_5.i.i.i.i16.i, 16
  store i8 %287, i8* %286, align 1, !noalias !330
  %289 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i17.i
  store i8 %287, i8* %289, align 1, !noalias !330
  %_73.i.i = icmp eq i8 %prev_ctrl.i.i.i, -1
  br i1 %_73.i.i, label %bb34.i.i, label %bb2.i.i10.i.i.preheader

bb2.i.i10.i.i.preheader:                          ; preds = %bb31.i.i
  %290 = load <16 x i8>, <16 x i8>* %185, align 1, !alias.scope !335, !noalias !303
  %291 = bitcast i8* %278 to <16 x i8>*
  %292 = load <16 x i8>, <16 x i8>* %291, align 1, !alias.scope !353, !noalias !303
  store <16 x i8> %292, <16 x i8>* %186, align 1, !alias.scope !335, !noalias !303
  %293 = bitcast i8* %278 to <16 x i8>*
  store <16 x i8> %290, <16 x i8>* %293, align 1, !alias.scope !353, !noalias !303
  %294 = getelementptr inbounds i8, i8* %278, i64 16
  %295 = load <16 x i8>, <16 x i8>* %187, align 1, !alias.scope !370, !noalias !303
  %296 = bitcast i8* %294 to <16 x i8>*
  %297 = load <16 x i8>, <16 x i8>* %296, align 1, !alias.scope !387, !noalias !303
  store <16 x i8> %297, <16 x i8>* %188, align 1, !alias.scope !370, !noalias !303
  %298 = bitcast i8* %294 to <16 x i8>*
  store <16 x i8> %295, <16 x i8>* %298, align 1, !alias.scope !387, !noalias !303
  br label %bb19.i.i

bb34.i.i:                                         ; preds = %bb31.i.i
  %299 = add i64 %iter.sroa.0.028.i.i, -16
  %_5.i.i.i = and i64 %299, %_16.i
  %index2.i.i.i = add i64 %_5.i.i.i, 16
  %300 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.028.i.i
  store i8 -1, i8* %300, align 1, !noalias !404
  %301 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i
  store i8 -1, i8* %301, align 1, !noalias !404
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(32) %278, i8* noundef nonnull align 1 dereferenceable(32) %182, i64 32, i1 false) #24, !noalias !303
  br label %bb40.i.i

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i": ; preds = %bb40.i.i, %bb5.i2.i
  %302 = phi i64 [ 0, %bb5.i2.i ], [ %.0.i.i, %bb40.i.i ]
  %303 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  %304 = sub i64 %302, %_9.i
  store i64 %304, i64* %303, align 8, !alias.scope !303
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit": ; preds = %bb2.i, %bb5.i.i, %bb26.i.i, %bb2.i.i.i14.i.i, %bb2.i.i.i.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i"
  %.sroa.3.0.i = phi i64 [ -9223372036854775807, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i" ], [ %_13.1.i, %bb2.i ], [ %_7.sroa.13.0.i.i.i, %bb5.i.i ], [ -9223372036854775807, %bb26.i.i ], [ -9223372036854775807, %bb2.i.i.i14.i.i ], [ -9223372036854775807, %bb2.i.i.i.i.i.i.i ]
  %.sroa.0.0.i = phi i64 [ undef, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i" ], [ %_13.0.i, %bb2.i ], [ %_7.sroa.7.0.i.i.i, %bb5.i.i ], [ undef, %bb26.i.i ], [ undef, %bb2.i.i.i14.i.i ], [ undef, %bb2.i.i.i.i.i.i.i ]
  %305 = insertvalue { i64, i64 } undef, i64 %.sroa.0.0.i, 0
  %306 = insertvalue { i64, i64 } %305, i64 %.sroa.3.0.i, 1
  ret { i64, i64 } %306
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
define internal fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef readonly align 8 dereferenceable(8) %f) unnamed_addr #12 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
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
  store %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %slot, align 8
  %2 = bitcast %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14 to i8*
  call void @llvm.lifetime.start.p0i8(i64 24, i8* nonnull %2)
  %3 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 0
  store i64** %f1, i64*** %3, align 8
  %4 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 1
  store %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %slot, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %4, align 8
  %5 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_14, i64 0, i32 2
  store %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"* %res, %"core::result::Result<(), once_cell::sync::OnceCell<T>::get_or_init::Void>::Ok"** %5, align 8
; call once_cell::imp::initialize_or_wait
  call void @_ZN9once_cell3imp18initialize_or_wait17h9b3310b1603d0203E(%"core::sync::atomic::AtomicUsize"* noundef align 8 dereferenceable(8) bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"core::sync::atomic::AtomicUsize"*), i8* noundef nonnull align 1 %2, i8* bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.c to i8*))
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
  %2 = load i64, i64* %1, align 8, !alias.scope !407
  store i64* null, i64** %_15, align 8, !alias.scope !407
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<i64>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !414)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !417)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %_8.i.i, align 8, !alias.scope !420, !noalias !421, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !424, !noalias !427
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !424, !noalias !427
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !427
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<i64>"*)*
  call void %7(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %_5.sroa.0), !noalias !414
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
define internal noundef zeroext i1 @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* noalias nocapture noundef readonly align 8 dereferenceable(24) %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0 = alloca %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", align 8
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15 = load i64**, i64*** %0, align 8, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15 to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !428
  store i64* null, i64** %_15, align 8, !alias.scope !428
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #24
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !435)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !438)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_8.i.i, align 8, !alias.scope !441, !noalias !442, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !445, !noalias !448
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !445, !noalias !448
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #23, !noalias !448
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0), !noalias !435
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !nonnull !85, !align !86, !noundef !85
  %_17 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 0
  %_2.i16 = load i64, i64* %9, align 8, !range !120, !noundef !85
  %10 = icmp eq i64 %_2.i16, 0
  br i1 %10, label %bb9, label %bb2.i

bb2.i:                                            ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1
  %12 = bitcast [7 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %bb4.i.i unwind label %cleanup.i.i

cleanup.i.i:                                      ; preds = %bb2.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %14 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 1
  %15 = bitcast i64* %14 to %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*
; call core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #25
  %_20 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %_10.sroa.0.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx, align 8
  %_10.sroa.5.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20, i64 56, i1 false)
  resume { i8*, i32 } %13

bb4.i.i:                                          ; preds = %bb2.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 3
  tail call void @llvm.experimental.noalias.scope.decl(metadata !449) #24
  %_2.i.i.i.i.i.i.i.i = load i64, i64* %16, align 8, !alias.scope !452
  %17 = icmp eq i64 %_2.i.i.i.i.i.i.i.i, 0
  br i1 %17, label %bb9, label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb4.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !455) #24
  tail call void @llvm.experimental.noalias.scope.decl(metadata !458) #24
  %18 = add i64 %_2.i.i.i.i.i.i.i.i, 1
  %19 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %18, i64 32) #24
  %20 = extractvalue { i64, i1 } %19, 1
  %21 = xor i1 %20, true
  tail call void @llvm.assume(i1 %21) #24
  %22 = extractvalue { i64, i1 } %19, 0
  %_31.i.i.i.i.i.i.i.i.i.i = add i64 %_2.i.i.i.i.i.i.i.i, 17
  %23 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %22, i64 %_31.i.i.i.i.i.i.i.i.i.i) #24
  %24 = extractvalue { i64, i1 } %23, 1
  %25 = xor i1 %24, true
  tail call void @llvm.assume(i1 %25) #24
  %26 = extractvalue { i64, i1 } %23, 0
  %27 = icmp eq i64 %26, 0
  br i1 %27, label %bb9, label %bb2.i.i.i.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i.i.i.i:                          ; preds = %bb2.i.i.i.i.i.i.i
  %28 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 4
  %29 = bitcast i64* %28 to i8**
  %_17.i.i.i.i.i.i.i.i.i = load i8*, i8** %29, align 8, !alias.scope !461, !nonnull !85, !noundef !85
  %30 = sub i64 0, %22
  %31 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i.i.i.i.i, i64 %30
  tail call void @__rust_dealloc(i8* nonnull %31, i64 %26, i64 16) #24, !noalias !461
  br label %bb9

bb9:                                              ; preds = %bb2.i.i.i.i.i.i.i.i.i.i, %bb2.i.i.i.i.i.i.i, %bb4.i.i, %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit"
  %_22 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
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
  %e.i45 = alloca { i64*, i8 }, align 8
  %this.i.i31 = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %e.i25 = alloca { i64*, i8 }, align 8
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc71 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %2, align 8, !alias.scope !462, !noalias !465
  %3 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 0, i32 1
  store i64 1, i64* %3, align 8, !alias.scope !462, !noalias !465
  %4 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 1, i32 0
  store i64* null, i64** %4, align 8, !alias.scope !462, !noalias !465
  %5 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{}>* @alloc73 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %5, align 8, !alias.scope !462, !noalias !465
  %6 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_10, i64 0, i32 2, i32 1
  store i64 0, i64* %6, align 8, !alias.scope !462, !noalias !465
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_10)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1)
  %7 = bitcast { [0 x i8]*, i64 }* %name_c_str1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %7)
  %_18.sroa.4.0..sroa_idx74 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1
  %_18.sroa.4.0..sroa_cast = bitcast [2 x i64]* %_18.sroa.4.0..sroa_idx74 to [0 x i8]**
  %_18.sroa.4.0.copyload = load [0 x i8]*, [0 x i8]** %_18.sroa.4.0..sroa_cast, align 8
  %_18.sroa.6.0..sroa_idx76 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1, i64 1
  %_18.sroa.6.0.copyload = load i64, i64* %_18.sroa.6.0..sroa_idx76, align 8
  %8 = bitcast %"core::str::error::Utf8Error"* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %8), !noalias !468
  %_18.sroa.4.8..sroa_cast = bitcast %"core::str::error::Utf8Error"* %e.i to [0 x i8]**
  store [0 x i8]* %_18.sroa.4.0.copyload, [0 x i8]** %_18.sroa.4.8..sroa_cast, align 8
  %_18.sroa.6.8..sroa_idx78 = getelementptr inbounds %"core::str::error::Utf8Error", %"core::str::error::Utf8Error"* %e.i, i64 0, i32 1
  %_18.sroa.6.8..sroa_cast = bitcast { i8, i8 }* %_18.sroa.6.8..sroa_idx78 to i64*
  store i64 %_18.sroa.6.0.copyload, i64* %_18.sroa.6.8..sroa_cast, align 8
  %_6.0.i = bitcast %"core::str::error::Utf8Error"* %e.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc422 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.4 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc468 to %"core::panic::location::Location"*)) #23, !noalias !468
  unreachable

"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E.exit": ; preds = %start
  %9 = bitcast { [0 x i8]*, i64 }* %name_c_str1 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %9)
  %_18.sroa.4.0..sroa_idx7494 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1
  %_18.sroa.4.0..sroa_cast95 = bitcast [2 x i64]* %_18.sroa.4.0..sroa_idx7494 to [0 x i8]**
  %_18.sroa.4.0.copyload96 = load [0 x i8]*, [0 x i8]** %_18.sroa.4.0..sroa_cast95, align 8, !nonnull !85
  %_18.sroa.6.0..sroa_idx7697 = getelementptr inbounds %"core::result::Result<&str, core::str::error::Utf8Error>", %"core::result::Result<&str, core::str::error::Utf8Error>"* %name_c_str, i64 0, i32 1, i64 1
  %_18.sroa.6.0.copyload98 = load i64, i64* %_18.sroa.6.0..sroa_idx7697, align 8
  %.fca.0.gep = getelementptr inbounds { [0 x i8]*, i64 }, { [0 x i8]*, i64 }* %name_c_str1, i64 0, i32 0
  store [0 x i8]* %_18.sroa.4.0.copyload96, [0 x i8]** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { [0 x i8]*, i64 }, { [0 x i8]*, i64 }* %name_c_str1, i64 0, i32 1
  store i64 %_18.sroa.6.0.copyload98, i64* %.fca.1.gep, align 8
  %10 = bitcast { i64*, i8 }* %guard to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %10)
  %11 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %11)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i, align 8
  %12 = load atomic i64, i64* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to i64*) acquire, align 8, !noalias !471
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
  %15 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !476
  %16 = extractvalue { i32, i1 } %15, 1
  br i1 %16, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !476
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
  %17 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !479
  %_1.i.i.i.i.i.i = and i64 %17, 9223372036854775807
  %18 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %18, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %19 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !479
  %phi.bo.i.i.i.i.i = xor i1 %19, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %20 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !479
  %.not102 = icmp eq i8 %20, 0
  br i1 %.not102, label %bb18, label %bb1.i30

bb1.i30:                                          ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %21 = bitcast { i64*, i8 }* %e.i25 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %21), !noalias !482
  %22 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i25, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %22, align 8, !noalias !482
  %23 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i25, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %23, align 8, !noalias !482
  %_6.0.i29 = bitcast { i64*, i8 }* %e.i25 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc422 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i29, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc470 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !482

cleanup.i:                                        ; preds = %bb1.i30
  %24 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i25) #25
          to label %common.resume unwind label %abort.i, !noalias !482

unreachable.i:                                    ; preds = %bb1.i30
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %25 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !482
  unreachable

common.resume:                                    ; preds = %bb28, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %24, %cleanup.i ], [ %.pn, %bb28 ]
  resume { i8*, i32 } %common.resume.op

bb28:                                             ; preds = %cleanup.i51, %cleanup, %cleanup2
  %.pn = phi { i8*, i32 } [ %65, %cleanup2 ], [ %26, %cleanup ], [ %63, %cleanup.i51 ]
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %guard) #25
          to label %common.resume unwind label %abort

cleanup:                                          ; preds = %bb2.i.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb3.i.i.i.i.i.i40, %bb3.i.i.i35, %bb3.i.i.i.i32, %bb18
  %26 = landingpad { i8*, i32 }
          cleanup
  br label %bb28

bb18:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %.fca.0.gep4 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep4, align 8
  %.fca.1.gep6 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep6, align 8
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc76 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %42, align 8, !alias.scope !485, !noalias !488
  %43 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 0, i32 1
  store i64 4, i64* %43, align 8, !alias.scope !485, !noalias !488
  %44 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 1, i32 0
  store i64* bitcast (<{ [168 x i8] }>* @alloc96 to i64*), i64** %44, align 8, !alias.scope !485, !noalias !488
  %45 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 1, i32 1
  store i64 3, i64* %45, align 8, !alias.scope !485, !noalias !488
  %46 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 2, i32 0
  %47 = bitcast [0 x { i8*, i64* }]** %46 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_37, [3 x { i8*, i64* }]** %47, align 8, !alias.scope !485, !noalias !488
  %48 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_30, i64 0, i32 2, i32 1
  store i64 3, i64* %48, align 8, !alias.scope !485, !noalias !488
; invoke std::io::stdio::_print
  invoke void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_30)
          to label %bb19 unwind label %cleanup

bb19:                                             ; preds = %bb18
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %30)
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %32)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %31)
  %49 = bitcast { i64*, i8 }* %object_table to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %49)
  %50 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i31 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %50)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i31, align 8
  %51 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to i64*) acquire, align 8, !noalias !492
  %52 = icmp eq i64 %51, 2
  br i1 %52, label %bb20, label %bb3.i.i.i.i32

bb3.i.i.i.i32:                                    ; preds = %bb19
  %53 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i31 to i64*
; invoke once_cell::imp::OnceCell<T>::initialize
  invoke fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %53)
          to label %bb20 unwind label %cleanup

bb20:                                             ; preds = %bb19, %bb3.i.i.i.i32
  %_6.i.i.i.i.i.i.i33 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i34 = icmp ne i64 %_6.i.i.i.i.i.i.i33, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i34) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %50)
  %54 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !497
  %55 = extractvalue { i32, i1 } %54, 1
  br i1 %55, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i37, label %bb3.i.i.i35

bb3.i.i.i35:                                      ; preds = %bb20
; invoke std::sys::unix::locks::futex::Mutex::lock_contended
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i37 unwind label %cleanup

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i37: ; preds = %bb3.i.i.i35, %bb20
  %56 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !500
  %_1.i.i.i.i.i.i36 = and i64 %56, 9223372036854775807
  %57 = icmp eq i64 %_1.i.i.i.i.i.i36, 0
  br i1 %57, label %bb21, label %bb3.i.i.i.i.i.i40

bb3.i.i.i.i.i.i40:                                ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i37
; invoke std::panicking::panic_count::is_zero_slow_path
  %58 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc44 unwind label %cleanup

.noexc44:                                         ; preds = %bb3.i.i.i.i.i.i40
  %phi.bo.i.i.i.i.i38 = xor i1 %58, true
  %phi.cast.i.i.i39 = zext i1 %phi.bo.i.i.i.i.i38 to i8
  br label %bb21

bb21:                                             ; preds = %.noexc44, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i37
  %.0.i.i.i.i.i.i41 = phi i8 [ %phi.cast.i.i.i39, %.noexc44 ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i37 ]
  %59 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !500
  %.not103 = icmp eq i8 %59, 0
  br i1 %.not103, label %bb23, label %bb1.i50

bb1.i50:                                          ; preds = %bb21
  %60 = bitcast { i64*, i8 }* %e.i45 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %60), !noalias !503
  %61 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i45, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %61, align 8, !noalias !503
  %62 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i45, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i41, i8* %62, align 8, !noalias !503
  %_6.0.i49 = bitcast { i64*, i8 }* %e.i45 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc422 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i49, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc472 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i52 unwind label %cleanup.i51, !noalias !507

cleanup.i51:                                      ; preds = %bb1.i50
  %63 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i45) #25
          to label %bb28 unwind label %abort.i53, !noalias !507

unreachable.i52:                                  ; preds = %bb1.i50
  unreachable

abort.i53:                                        ; preds = %cleanup.i51
  %64 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !507
  unreachable

cleanup2:                                         ; preds = %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i"
  %65 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %object_table) #25
          to label %bb28 unwind label %abort

bb23:                                             ; preds = %bb21
  %.fca.0.gep8 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep8, align 8
  %.fca.1.gep10 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i41, i8* %.fca.1.gep10, align 8
  %_60 = load i64, i64* %objid, align 8
  call void @llvm.experimental.noalias.scope.decl(metadata !508)
  call void @llvm.experimental.noalias.scope.decl(metadata !511)
  %_6.idx.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !514, !noalias !515
  %_6.idx11.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !514, !noalias !515
  %66 = xor i64 %_6.idx.val.i.i, 8317987319222330741
  %67 = xor i64 %_6.idx11.val.i.i, 7237128888997146477
  %68 = xor i64 %_6.idx.val.i.i, 7816392313619706465
  %69 = xor i64 %_60, %_6.idx11.val.i.i
  %70 = xor i64 %69, 8387220255154660723
  %71 = add i64 %67, %66
  %72 = call i64 @llvm.fshl.i64(i64 %67, i64 %67, i64 13) #24
  %73 = xor i64 %71, %72
  %74 = call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 32) #24
  %75 = add i64 %70, %68
  %76 = call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 16) #24
  %77 = xor i64 %76, %75
  %78 = add i64 %77, %74
  %79 = call i64 @llvm.fshl.i64(i64 %77, i64 %77, i64 21) #24
  %80 = xor i64 %79, %78
  %81 = add i64 %73, %75
  %82 = call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 17) #24
  %83 = xor i64 %81, %82
  %84 = call i64 @llvm.fshl.i64(i64 %81, i64 %81, i64 32) #24
  %85 = xor i64 %78, %_60
  %86 = xor i64 %80, 576460752303423488
  %87 = add i64 %85, %83
  %88 = call i64 @llvm.fshl.i64(i64 %83, i64 %83, i64 13) #24
  %89 = xor i64 %87, %88
  %90 = call i64 @llvm.fshl.i64(i64 %87, i64 %87, i64 32) #24
  %91 = add i64 %86, %84
  %92 = call i64 @llvm.fshl.i64(i64 %80, i64 %86, i64 16) #24
  %93 = xor i64 %92, %91
  %94 = add i64 %93, %90
  %95 = call i64 @llvm.fshl.i64(i64 %93, i64 %93, i64 21) #24
  %96 = xor i64 %95, %94
  %97 = add i64 %91, %89
  %98 = call i64 @llvm.fshl.i64(i64 %89, i64 %89, i64 17) #24
  %99 = xor i64 %97, %98
  %100 = call i64 @llvm.fshl.i64(i64 %97, i64 %97, i64 32) #24
  %101 = xor i64 %94, 576460752303423488
  %102 = xor i64 %100, 255
  %103 = add i64 %101, %99
  %104 = call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 13) #24
  %105 = xor i64 %103, %104
  %106 = call i64 @llvm.fshl.i64(i64 %103, i64 %103, i64 32) #24
  %107 = add i64 %96, %102
  %108 = call i64 @llvm.fshl.i64(i64 %96, i64 %96, i64 16) #24
  %109 = xor i64 %108, %107
  %110 = add i64 %109, %106
  %111 = call i64 @llvm.fshl.i64(i64 %109, i64 %109, i64 21) #24
  %112 = xor i64 %111, %110
  %113 = add i64 %105, %107
  %114 = call i64 @llvm.fshl.i64(i64 %105, i64 %105, i64 17) #24
  %115 = xor i64 %113, %114
  %116 = call i64 @llvm.fshl.i64(i64 %113, i64 %113, i64 32) #24
  %117 = add i64 %115, %110
  %118 = call i64 @llvm.fshl.i64(i64 %115, i64 %115, i64 13) #24
  %119 = xor i64 %118, %117
  %120 = call i64 @llvm.fshl.i64(i64 %117, i64 %117, i64 32) #24
  %121 = add i64 %112, %116
  %122 = call i64 @llvm.fshl.i64(i64 %112, i64 %112, i64 16) #24
  %123 = xor i64 %122, %121
  %124 = add i64 %123, %120
  %125 = call i64 @llvm.fshl.i64(i64 %123, i64 %123, i64 21) #24
  %126 = xor i64 %125, %124
  %127 = add i64 %119, %121
  %128 = call i64 @llvm.fshl.i64(i64 %119, i64 %119, i64 17) #24
  %129 = xor i64 %128, %127
  %130 = call i64 @llvm.fshl.i64(i64 %127, i64 %127, i64 32) #24
  %131 = add i64 %129, %124
  %132 = call i64 @llvm.fshl.i64(i64 %129, i64 %129, i64 13) #24
  %133 = xor i64 %132, %131
  %134 = add i64 %126, %130
  %135 = call i64 @llvm.fshl.i64(i64 %126, i64 %126, i64 16) #24
  %136 = xor i64 %135, %134
  %137 = call i64 @llvm.fshl.i64(i64 %136, i64 %136, i64 21) #24
  %138 = add i64 %133, %134
  %139 = call i64 @llvm.fshl.i64(i64 %133, i64 %133, i64 17) #24
  %140 = call i64 @llvm.fshl.i64(i64 %138, i64 %138, i64 32) #24
  %_17.i.i.i.i.i.i.i = xor i64 %138, %137
  %141 = xor i64 %_17.i.i.i.i.i.i.i, %139
  %142 = xor i64 %141, %140
  call void @llvm.experimental.noalias.scope.decl(metadata !520)
  call void @llvm.experimental.noalias.scope.decl(metadata !523) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !526) #24
  %top7.i.i.i.i.i.i = lshr i64 %142, 57
  %143 = trunc i64 %top7.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !529, !noalias !532
  %_3.i.i.i.i.i.i = and i64 %142, %_6.i.i.i.i.i.i
  %self.idx.val.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !534, !noalias !532
  %.0.vec.insert.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %143, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb21.i.i.i.i.i, %bb23
  %probe_seq.sroa.7.0.i.i.i.i.i = phi i64 [ 0, %bb23 ], [ %156, %bb21.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb23 ], [ %158, %bb21.i.i.i.i.i ]
  %144 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i
  %145 = bitcast i8* %144 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i = load <16 x i8>, <16 x i8>* %145, align 1, !noalias !535
  %146 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i
  %147 = bitcast <16 x i1> %146 to i16
  br label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb10.i.i.i.i.i, %bb3.i.i.i.i.i
  %iter.0.i.i.i.i.i = phi i16 [ %147, %bb3.i.i.i.i.i ], [ %_2.i.i.i.i.i.i.i, %bb10.i.i.i.i.i ]
  %148 = icmp eq i16 %iter.0.i.i.i.i.i, 0
  br i1 %148, label %bb12.i.i.i.i.i, label %bb10.i.i.i.i.i

bb12.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %149 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %150 = bitcast <16 x i1> %149 to i16
  %.not.i.i.i.i.i = icmp eq i16 %150, 0
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %bb6.i.i

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %151 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %151 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %152 = sub i64 0, %index.i.i.i.i.i
  %153 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %152, i32 0
  %154 = getelementptr inbounds i64, i64* %153, i64 -4
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %154, align 8, !noalias !538
  %155 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %_60
  br i1 %155, label %bb24, label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %156 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %157 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %156
  %158 = and i64 %157, %_6.i.i.i.i.i.i
  br label %bb3.i.i.i.i.i

bb6.i.i:                                          ; preds = %bb12.i.i.i.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !541)
  %159 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_3.i.i.i.i.i.i
  %160 = bitcast i8* %159 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %160, align 1, !noalias !544
  %161 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %162 = bitcast <16 x i1> %161 to i16
  %.not23.i.i.i.i = icmp eq i16 %162, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb17.i.i.i.i, %bb6.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb6.i.i ], [ %168, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %162, %bb6.i.i ], [ %172, %bb17.i.i.i.i ]
  %163 = call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i = zext i16 %163 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_6.i.i.i.i.i.i
  %164 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %164, align 1, !noalias !551
  %165 = icmp sgt i8 %_23.i.i.i.i, -1
  br i1 %165, label %bb11.i.i.i.i, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"

bb17.i.i.i.i:                                     ; preds = %bb6.i.i, %bb17.i.i.i.i
  %probe_seq.sroa.0.025.i.i.i.i = phi i64 [ %168, %bb17.i.i.i.i ], [ %_3.i.i.i.i.i.i, %bb6.i.i ]
  %probe_seq.sroa.7.024.i.i.i.i = phi i64 [ %166, %bb17.i.i.i.i ], [ 0, %bb6.i.i ]
  %166 = add i64 %probe_seq.sroa.7.024.i.i.i.i, 16
  %167 = add i64 %166, %probe_seq.sroa.0.025.i.i.i.i
  %168 = and i64 %167, %_6.i.i.i.i.i.i
  %169 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %168
  %170 = bitcast i8* %169 to <16 x i8>*
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %170, align 1, !noalias !544
  %171 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %172 = bitcast <16 x i1> %171 to i16
  %.not.i.i.i.i = icmp eq i16 %172, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i.i.i
  %173 = bitcast i8* %self.idx.val.i.i.i.i.i to <16 x i8>*
  %174 = load <16 x i8>, <16 x i8>* %173, align 16, !noalias !552
  %175 = icmp slt <16 x i8> %174, zeroinitializer
  %176 = bitcast <16 x i1> %175 to i16
  %177 = call i16 @llvm.cttz.i16(i16 %176, i1 true) #24, !range !27
  %_2.i.i.i.i.i = zext i16 %177 to i64
  %.phi.trans.insert.i.i.i = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_2.i.i.i.i.i
  %old_ctrl.pre.i.i.i = load i8, i8* %.phi.trans.insert.i.i.i, align 1, !noalias !557
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i": ; preds = %bb11.i.i.i.i, %bb7.i.i.i.i
  %old_ctrl.i.i.i = phi i8 [ %old_ctrl.pre.i.i.i, %bb11.i.i.i.i ], [ %_23.i.i.i.i, %bb7.i.i.i.i ]
  %.0.i.i.i.i = phi i64 [ %_2.i.i.i.i.i, %bb11.i.i.i.i ], [ %result.i.i.i.i, %bb7.i.i.i.i ]
  %_14.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !558, !noalias !559
  %178 = icmp eq i64 %_14.i.i.i, 0
  %_2.i.i.i.i = and i8 %old_ctrl.i.i.i, 1
  %179 = icmp ne i8 %_2.i.i.i.i, 0
  %or.cond.i.i.i = select i1 %178, i1 %179, i1 false
  br i1 %or.cond.i.i.i, label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i", label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"

"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i": ; preds = %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"
; invoke hashbrown::raw::RawTable<T,A>::reserve_rehash
  %180 = invoke fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h320d5dd485a72968E"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias noundef nonnull align 8 dereferenceable(32) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"*), i64* noalias noundef nonnull readonly align 8 dereferenceable(16) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to i64*))
          to label %.noexc56 unwind label %cleanup2

.noexc56:                                         ; preds = %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i"
  %.fca.1.extract.i.i.i.i = extractvalue { i64, i64 } %180, 1
  %.not.i2.i.i.i = icmp eq i64 %.fca.1.extract.i.i.i.i, -9223372036854775807
  call void @llvm.assume(i1 %.not.i2.i.i.i)
  call void @llvm.experimental.noalias.scope.decl(metadata !560)
  %_6.i.i4.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !563, !noalias !559
  %_3.i.i5.i.i.i = and i64 %_6.i.i4.i.i.i, %142
  %self.idx11.val.i7.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !566, !noalias !559
  %181 = getelementptr inbounds i8, i8* %self.idx11.val.i7.i.i.i, i64 %_3.i.i5.i.i.i
  %182 = bitcast i8* %181 to <16 x i8>*
  %.0.copyload.i2122.i8.i.i.i = load <16 x i8>, <16 x i8>* %182, align 1, !noalias !567
  %183 = icmp slt <16 x i8> %.0.copyload.i2122.i8.i.i.i, zeroinitializer
  %184 = bitcast <16 x i1> %183 to i16
  %.not23.i9.i.i.i = icmp eq i16 %184, 0
  br i1 %.not23.i9.i.i.i, label %bb17.i21.i.i.i, label %bb7.i16.i.i.i

bb7.i16.i.i.i:                                    ; preds = %bb17.i21.i.i.i, %.noexc56
  %probe_seq.sroa.0.0.lcssa.i10.i.i.i = phi i64 [ %_3.i.i5.i.i.i, %.noexc56 ], [ %190, %bb17.i21.i.i.i ]
  %.lcssa.i11.i.i.i = phi i16 [ %184, %.noexc56 ], [ %194, %bb17.i21.i.i.i ]
  %185 = call i16 @llvm.cttz.i16(i16 %.lcssa.i11.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i12.i.i.i = zext i16 %185 to i64
  %_17.i13.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i10.i.i.i, %_2.i.i.i12.i.i.i
  %result.i14.i.i.i = and i64 %_17.i13.i.i.i, %_6.i.i4.i.i.i
  %186 = getelementptr inbounds i8, i8* %self.idx11.val.i7.i.i.i, i64 %result.i14.i.i.i
  %_23.i15.i.i.i = load i8, i8* %186, align 1, !noalias !570
  %187 = icmp sgt i8 %_23.i15.i.i.i, -1
  br i1 %187, label %bb11.i23.i.i.i, label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"

bb17.i21.i.i.i:                                   ; preds = %.noexc56, %bb17.i21.i.i.i
  %probe_seq.sroa.0.025.i17.i.i.i = phi i64 [ %190, %bb17.i21.i.i.i ], [ %_3.i.i5.i.i.i, %.noexc56 ]
  %probe_seq.sroa.7.024.i18.i.i.i = phi i64 [ %188, %bb17.i21.i.i.i ], [ 0, %.noexc56 ]
  %188 = add i64 %probe_seq.sroa.7.024.i18.i.i.i, 16
  %189 = add i64 %188, %probe_seq.sroa.0.025.i17.i.i.i
  %190 = and i64 %189, %_6.i.i4.i.i.i
  %191 = getelementptr inbounds i8, i8* %self.idx11.val.i7.i.i.i, i64 %190
  %192 = bitcast i8* %191 to <16 x i8>*
  %.0.copyload.i21.i19.i.i.i = load <16 x i8>, <16 x i8>* %192, align 1, !noalias !567
  %193 = icmp slt <16 x i8> %.0.copyload.i21.i19.i.i.i, zeroinitializer
  %194 = bitcast <16 x i1> %193 to i16
  %.not.i20.i.i.i = icmp eq i16 %194, 0
  br i1 %.not.i20.i.i.i, label %bb17.i21.i.i.i, label %bb7.i16.i.i.i

bb11.i23.i.i.i:                                   ; preds = %bb7.i16.i.i.i
  %195 = bitcast i8* %self.idx11.val.i7.i.i.i to <16 x i8>*
  %196 = load <16 x i8>, <16 x i8>* %195, align 16, !noalias !571
  %197 = icmp slt <16 x i8> %196, zeroinitializer
  %198 = bitcast <16 x i1> %197 to i16
  %199 = call i16 @llvm.cttz.i16(i16 %198, i1 true) #24, !range !27
  %_2.i.i22.i.i.i = zext i16 %199 to i64
  br label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"

"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i": ; preds = %bb11.i23.i.i.i, %bb7.i16.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"
  %self.idx1.val.i.i.i.i.i.i = phi i8* [ %self.idx.val.i.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i" ], [ %self.idx11.val.i7.i.i.i, %bb11.i23.i.i.i ], [ %self.idx11.val.i7.i.i.i, %bb7.i16.i.i.i ]
  %_8.i.i.i.i.i.i = phi i64 [ %_6.i.i.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i" ], [ %_6.i.i4.i.i.i, %bb11.i23.i.i.i ], [ %_6.i.i4.i.i.i, %bb7.i16.i.i.i ]
  %index.0.i.i.i = phi i64 [ %.0.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i" ], [ %_2.i.i22.i.i.i, %bb11.i23.i.i.i ], [ %result.i14.i.i.i, %bb7.i16.i.i.i ]
  %self.idx.val28.i.i.i = bitcast i8* %self.idx1.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  call void @llvm.experimental.noalias.scope.decl(metadata !576)
  %sext.i.i.i.i = sub nsw i8 0, %_2.i.i.i.i
  %_5.neg.i.i.i.i = sext i8 %sext.i.i.i.i to i64
  %200 = add i64 %index.0.i.i.i, -16
  %_5.i.i.i.i.i.i = and i64 %200, %_8.i.i.i.i.i.i
  %index2.i.i.i.i.i.i = add i64 %_5.i.i.i.i.i.i, 16
  %201 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index.0.i.i.i
  store i8 %143, i8* %201, align 1, !noalias !579
  %202 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i
  store i8 %143, i8* %202, align 1, !noalias !579
  %203 = load <2 x i64>, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !584, !noalias !559
  %204 = insertelement <2 x i64> <i64 poison, i64 1>, i64 %_5.neg.i.i.i.i, i64 0
  %205 = add <2 x i64> %203, %204
  store <2 x i64> %205, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !584, !noalias !559
  %206 = sub i64 0, %index.0.i.i.i
  %207 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %self.idx.val28.i.i.i, i64 %206, i32 0
  %_23.sroa.0.0..sroa_idx.i.i = getelementptr inbounds i64, i64* %207, i64 -4
  store i64 %_60, i64* %_23.sroa.0.0..sroa_idx.i.i, align 8, !noalias !585
  br label %bb24

bb24:                                             ; preds = %bb10.i.i.i.i.i, %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"
  %.pn104 = phi i64* [ %207, %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i" ], [ %153, %bb10.i.i.i.i.i ]
  %tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.in = getelementptr inbounds i64, i64* %.pn104, i64 -3
  store i64 %_60, i64* %tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.in, align 8, !noalias !586
  %_68.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx105 = getelementptr inbounds i64, i64* %.pn104, i64 -2
  store i64 %33, i64* %_68.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx105, align 8, !noalias !586
  %_68.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx106 = getelementptr inbounds i64, i64* %.pn104, i64 -1
  store i64 1, i64* %_68.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx106, align 8, !noalias !586
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i41, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb24
  %208 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !587
  %_1.i.i.i.i.i.i57 = and i64 %208, 9223372036854775807
  %209 = icmp eq i64 %_1.i.i.i.i.i.i57, 0
  br i1 %209, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; invoke std::panicking::panic_count::is_zero_slow_path
  %210 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc58 unwind label %cleanup

.noexc58:                                         ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  br i1 %210, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %.noexc58
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !587
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %.noexc58, %bb2.i.i.i, %bb24
  %211 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !587
  %212 = icmp eq i32 %211, 2
  br i1 %212, label %bb2.i.i.i.i, label %bb25

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; invoke std::sys::unix::locks::futex::Mutex::wake
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %bb25 unwind label %cleanup

abort:                                            ; preds = %bb28, %cleanup2
  %213 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26
  unreachable

bb25:                                             ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %49)
  call void @llvm.experimental.noalias.scope.decl(metadata !590)
  %_8.i.i60 = load %"std::sync::mutex::Mutex<i64>"*, %"std::sync::mutex::Mutex<i64>"** %27, align 8, !alias.scope !590, !nonnull !85, !align !86, !noundef !85
  %_5.val.i.i62 = load i8, i8* %.fca.1.gep6, align 8, !alias.scope !590
  %_5.not.i.i.i63 = icmp eq i8 %_5.val.i.i62, 0
  br i1 %_5.not.i.i.i63, label %bb2.i.i.i65, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70

bb2.i.i.i65:                                      ; preds = %bb25
  %214 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !590
  %_1.i.i.i.i.i.i64 = and i64 %214, 9223372036854775807
  %215 = icmp eq i64 %_1.i.i.i.i.i.i64, 0
  br i1 %215, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66: ; preds = %bb2.i.i.i65
; call std::panicking::panic_count::is_zero_slow_path
  %216 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !590
  br i1 %216, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70, label %bb5.i.i.i68

bb5.i.i.i68:                                      ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66
  %_6.i.i.i.i67 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i60, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i67 monotonic, align 4, !noalias !590
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70: ; preds = %bb5.i.i.i68, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i66, %bb2.i.i.i65, %bb25
  %_5.i.i.i.i.i69 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i60, i64 0, i32 0, i32 0, i32 0, i32 0
  %217 = atomicrmw xchg i32* %_5.i.i.i.i.i69, i32 0 release, align 4, !noalias !590
  %218 = icmp eq i32 %217, 2
  br i1 %218, label %bb2.i.i.i.i72, label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

bb2.i.i.i.i72:                                    ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70
  %_2.i.i.i71 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i60, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i71), !noalias !590
  br label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i70, %bb2.i.i.i.i72
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %10)
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %9)
  call void @llvm.lifetime.end.p0i8(i64 24, i8* nonnull %0)
  %219 = load i64, i64* %objid, align 8
  ret i64 %219
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc167 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %19, align 8, !alias.scope !593, !noalias !596
  %20 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 0, i32 1
  store i64 5, i64* %20, align 8, !alias.scope !593, !noalias !596
  %21 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 0
  store i64* bitcast (<{ [224 x i8] }>* @alloc253 to i64*), i64** %21, align 8, !alias.scope !593, !noalias !596
  %22 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 1
  store i64 4, i64* %22, align 8, !alias.scope !593, !noalias !596
  %23 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 0
  %24 = bitcast [0 x { i8*, i64* }]** %23 to [4 x { i8*, i64* }]**
  store [4 x { i8*, i64* }]* %_12, [4 x { i8*, i64* }]** %24, align 8, !alias.scope !593, !noalias !596
  %25 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 1
  store i64 4, i64* %25, align 8, !alias.scope !593, !noalias !596
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
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %28 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to i64*) acquire, align 8, !noalias !600
  %29 = icmp eq i64 %28, 2
  br i1 %29, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb11
  %30 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %30)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit": ; preds = %bb11, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %27)
  %31 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !605
  %32 = extractvalue { i32, i1 } %31, 1
  br i1 %32, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !605
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
  %33 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !608
  %_1.i.i.i.i.i.i = and i64 %33, 9223372036854775807
  %34 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %34, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %35 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !608
  %phi.bo.i.i.i.i.i = xor i1 %35, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %36 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !608
  %.not = icmp eq i8 %36, 0
  br i1 %.not, label %bb15, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %37 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %37), !noalias !611
  %38 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %38, align 8, !noalias !611
  %39 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %39, align 8, !noalias !611
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc422 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc476 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !615

cleanup.i:                                        ; preds = %bb1.i
  %40 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i) #25
          to label %common.resume unwind label %abort.i, !noalias !615

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %41 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !615
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc261 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_48.sroa.0.0..sroa_cast, align 8
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
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc255 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_47, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc474 to %"core::panic::location::Location"*)) #23
  unreachable

cleanup:                                          ; preds = %bb1.i21, %bb28, %bb19
  %47 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %object_table) #25
          to label %common.resume unwind label %abort

bb15:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !616
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_66 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h7c6dbde3483cee85E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc206 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %52, align 8, !alias.scope !619, !noalias !622
  %53 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 0, i32 1
  store i64 2, i64* %53, align 8, !alias.scope !619, !noalias !622
  %54 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 1, i32 0
  store i64* null, i64** %54, align 8, !alias.scope !619, !noalias !622
  %55 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 0
  %56 = bitcast [0 x { i8*, i64* }]** %55 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_79, [1 x { i8*, i64* }]** %56, align 8, !alias.scope !619, !noalias !622
  %57 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 1
  store i64 1, i64* %57, align 8, !alias.scope !619, !noalias !622
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_72, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc478 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb28, %bb19
  unreachable

bb21:                                             ; preds = %bb15
  %obj_id.val19 = load i64, i64* %obj_id, align 8, !alias.scope !616
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_84 = call fastcc noundef align 8 dereferenceable_or_null(24) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h806e044307d21e0aE"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val19)
  %58 = icmp eq i64* %_84, null
  br i1 %58, label %bb1.i21, label %bb23

bb1.i21:                                          ; preds = %bb21
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc410 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc480 to %"core::panic::location::Location"*)) #23
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
  %61 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !625
  %_1.i.i.i.i.i.i22 = and i64 %61, 9223372036854775807
  %62 = icmp eq i64 %_1.i.i.i.i.i.i22, 0
  br i1 %62, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %63 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !625
  br i1 %63, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !625
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb29
  %64 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !625
  %65 = icmp eq i32 %64, 2
  br i1 %65, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !625
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc213 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_106.sroa.0.0..sroa_cast, align 8
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
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %59, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_105, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc482 to %"core::panic::location::Location"*)) #23
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc227 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %19, align 8, !alias.scope !628, !noalias !631
  %20 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 0, i32 1
  store i64 5, i64* %20, align 8, !alias.scope !628, !noalias !631
  %21 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 0
  store i64* bitcast (<{ [224 x i8] }>* @alloc253 to i64*), i64** %21, align 8, !alias.scope !628, !noalias !631
  %22 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 1, i32 1
  store i64 4, i64* %22, align 8, !alias.scope !628, !noalias !631
  %23 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 0
  %24 = bitcast [0 x { i8*, i64* }]** %23 to [4 x { i8*, i64* }]**
  store [4 x { i8*, i64* }]* %_12, [4 x { i8*, i64* }]** %24, align 8, !alias.scope !628, !noalias !631
  %25 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_5, i64 0, i32 2, i32 1
  store i64 4, i64* %25, align 8, !alias.scope !628, !noalias !631
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
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %28 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to i64*) acquire, align 8, !noalias !635
  %29 = icmp eq i64 %28, 2
  br i1 %29, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb11
  %30 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %30)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit": ; preds = %bb11, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !120
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #24
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %27)
  %31 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !640
  %32 = extractvalue { i32, i1 } %31, 1
  br i1 %32, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !640
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
  %33 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !643
  %_1.i.i.i.i.i.i = and i64 %33, 9223372036854775807
  %34 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %34, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %35 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !643
  %phi.bo.i.i.i.i.i = xor i1 %35, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %36 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !643
  %.not = icmp eq i8 %36, 0
  br i1 %.not, label %bb15, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %37 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %37), !noalias !646
  %38 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %38, align 8, !noalias !646
  %39 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %39, align 8, !noalias !646
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc422 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc486 to %"core::panic::location::Location"*)) #23
          to label %unreachable.i unwind label %cleanup.i, !noalias !650

cleanup.i:                                        ; preds = %bb1.i
  %40 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i) #25
          to label %common.resume unwind label %abort.i, !noalias !650

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %41 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #26, !noalias !650
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc261 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_48.sroa.0.0..sroa_cast, align 8
  %_48.sroa.4.0..sroa_idx29 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 0
  store i64 2, i64* %_48.sroa.4.0..sroa_idx29, align 8
  %_48.sroa.5.0..sroa_idx31 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 1
  %_48.sroa.5.0..sroa_cast = bitcast i64* %_48.sroa.5.0..sroa_idx31 to i64**
  store i64* null, i64** %_48.sroa.5.0..sroa_cast, align 8
  %_48.sroa.635.0..sroa_idx36 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 3
  %46 = bitcast i64* %_48.sroa.635.0..sroa_idx36 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_55, [1 x { i8*, i64* }]** %46, align 8
  %_48.sroa.7.0..sroa_idx38 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_47, i64 0, i32 1, i64 4
  store i64 1, i64* %_48.sroa.7.0..sroa_idx38, align 8
; call core::panicking::assert_failed
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc255 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_47, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc484 to %"core::panic::location::Location"*)) #23
  unreachable

cleanup:                                          ; preds = %bb1.i22, %bb28, %bb19
  %47 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %object_info) #25
          to label %common.resume unwind label %abort

bb15:                                             ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !616
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_66 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h7c6dbde3483cee85E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc266 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %52, align 8, !alias.scope !651, !noalias !654
  %53 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 0, i32 1
  store i64 2, i64* %53, align 8, !alias.scope !651, !noalias !654
  %54 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 1, i32 0
  store i64* null, i64** %54, align 8, !alias.scope !651, !noalias !654
  %55 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 0
  %56 = bitcast [0 x { i8*, i64* }]** %55 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_79, [1 x { i8*, i64* }]** %56, align 8, !alias.scope !651, !noalias !654
  %57 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_72, i64 0, i32 2, i32 1
  store i64 1, i64* %57, align 8, !alias.scope !651, !noalias !654
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_72, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc488 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb28, %bb19
  unreachable

bb21:                                             ; preds = %bb15
  %obj_id.val19 = load i64, i64* %obj_id, align 8, !alias.scope !616
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_84 = call fastcc noundef align 8 dereferenceable_or_null(24) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h806e044307d21e0aE"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val19)
  %58 = icmp eq i64* %_84, null
  br i1 %58, label %bb1.i22, label %bb23

bb1.i22:                                          ; preds = %bb21
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc410 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc490 to %"core::panic::location::Location"*)) #23
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
  br i1 %61, label %bb32, label %bb34

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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc273 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_106.sroa.0.0..sroa_cast, align 8
  %_106.sroa.4.0..sroa_idx45 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 0
  store i64 3, i64* %_106.sroa.4.0..sroa_idx45, align 8
  %_106.sroa.5.0..sroa_idx47 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 1
  %_106.sroa.5.0..sroa_cast = bitcast i64* %_106.sroa.5.0..sroa_idx47 to i64**
  store i64* null, i64** %_106.sroa.5.0..sroa_cast, align 8
  %_106.sroa.651.0..sroa_idx52 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 3
  %72 = bitcast i64* %_106.sroa.651.0..sroa_idx52 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_113, [3 x { i8*, i64* }]** %72, align 8
  %_106.sroa.7.0..sroa_idx54 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_105, i64 0, i32 1, i64 4
  store i64 3, i64* %_106.sroa.7.0..sroa_idx54, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %59, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_105, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc492 to %"core::panic::location::Location"*)) #23
          to label %unreachable unwind label %cleanup

bb34:                                             ; preds = %bb12.i.i.i.i.i.i, %bb4.i.i, %bb29
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb34
  %73 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !657
  %_1.i.i.i.i.i.i25 = and i64 %73, 9223372036854775807
  %74 = icmp eq i64 %_1.i.i.i.i.i.i25, 0
  br i1 %74, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %75 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !657
  br i1 %75, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !657
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb34
  %76 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !657
  %77 = icmp eq i32 %76, 2
  br i1 %77, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !657
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %26)
  ret void

bb32:                                             ; preds = %bb29
  %obj_id.val20 = load i64, i64* %obj_id, align 8, !alias.scope !616
  call void @llvm.experimental.noalias.scope.decl(metadata !660)
  call void @llvm.experimental.noalias.scope.decl(metadata !663) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !666) #24
  %_5.idx.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !669, !noalias !670
  %_5.idx1.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !669, !noalias !670
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
  call void @llvm.experimental.noalias.scope.decl(metadata !674) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !677) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !680) #24
  %top7.i.i.i.i.i.i.i = lshr i64 %154, 57
  %155 = trunc i64 %top7.i.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i.i26 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !683, !noalias !686
  %self.idx.val.i.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !689, !noalias !686
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
  %.0.copyload.i9.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %157, align 1, !noalias !690
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
  br i1 %.not.i.i.i.i.i.i, label %bb21.i.i.i.i.i.i, label %bb34

bb10.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %163 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i.i, i1 true) #24, !range !27
  %_2.i.i.i.i.i.i.i.i.i = zext i16 %163 to i64
  %_4.i.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i.i
  %_25.i.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i.i = and i64 %_25.i.i.i.i.i.i, %_6.i.i.i.i.i.i.i26
  %164 = sub i64 0, %index.i.i.i.i.i.i
  %165 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i.i, i64 %164, i32 0
  %166 = getelementptr inbounds i64, i64* %165, i64 -4
  %_6.idx.val.i.i.i.i.i.i.i = load i64, i64* %166, align 8, !noalias !693
  %167 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i.i, %obj_id.val20
  br i1 %167, label %bb4.i.i.i.i, label %bb8.i.i.i.i.i.i

bb21.i.i.i.i.i.i:                                 ; preds = %bb12.i.i.i.i.i.i
  %168 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i.i, 16
  %169 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %168
  br label %bb3.i.i.i.i.i.i27

bb4.i.i.i.i:                                      ; preds = %bb10.i.i.i.i.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !696) #24
  call void @llvm.experimental.noalias.scope.decl(metadata !699) #24
  %170 = ptrtoint i8* %self.idx.val.i.i.i.i.i.i to i64
  %171 = ptrtoint i64* %165 to i64
  %172 = sub i64 %170, %171
  %173 = ashr exact i64 %172, 5
  call void @llvm.experimental.noalias.scope.decl(metadata !702) #24
  %174 = add nsw i64 %173, -16
  %index_before.i.i.i.i.i.i.i = and i64 %174, %_6.i.i.i.i.i.i.i26
  %175 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index_before.i.i.i.i.i.i.i
  %176 = bitcast i8* %175 to <16 x i8>*
  %.0.copyload.i17.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %176, align 1, !noalias !705
  %177 = icmp eq <16 x i8> %.0.copyload.i17.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %178 = bitcast <16 x i1> %177 to i16
  %179 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %173
  %180 = bitcast i8* %179 to <16 x i8>*
  %.0.copyload.i418.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %180, align 1, !noalias !709
  %181 = icmp eq <16 x i8> %.0.copyload.i418.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %182 = bitcast <16 x i1> %181 to i16
  %183 = call i16 @llvm.ctlz.i16(i16 %178, i1 false) #24, !range !27
  %184 = call i16 @llvm.cttz.i16(i16 %182, i1 false) #24, !range !27
  %narrow.i.i.i.i.i.i.i = add nuw nsw i16 %184, %183
  %_20.i.i.i.i.i.i.i = icmp ugt i16 %narrow.i.i.i.i.i.i.i, 15
  br i1 %_20.i.i.i.i.i.i.i, label %bb4.i.i, label %bb11.i.i.i.i.i.i.i

bb11.i.i.i.i.i.i.i:                               ; preds = %bb4.i.i.i.i
  %185 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !712, !noalias !713
  %186 = add i64 %185, 1
  store i64 %186, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !712, !noalias !713
  br label %bb4.i.i

bb4.i.i:                                          ; preds = %bb11.i.i.i.i.i.i.i, %bb4.i.i.i.i
  %.sink20.i.i.i.i.i.i.i = phi i8 [ -1, %bb11.i.i.i.i.i.i.i ], [ -128, %bb4.i.i.i.i ]
  %index2.i.i.i.i.i.i.i.i = add i64 %index_before.i.i.i.i.i.i.i, 16
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %179, align 1, !noalias !714
  %187 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i.i.i
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %187, align 1, !noalias !714
  %188 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !712, !noalias !713
  %189 = add i64 %188, -1
  store i64 %189, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !712, !noalias !713
  br label %bb34

abort:                                            ; preds = %cleanup
  %190 = landingpad { i8*, i32 }
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
!3 = distinct !{!3, !4, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h7f4c47b100c2fe02E: %self"}
!4 = distinct !{!4, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12contains_key17h7f4c47b100c2fe02E"}
!5 = !{!6}
!6 = distinct !{!6, !7, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$9get_inner17hd6c9dacb8bc31cf4E: %self"}
!7 = distinct !{!7, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$9get_inner17hd6c9dacb8bc31cf4E"}
!8 = !{!6, !3}
!9 = !{!10}
!10 = distinct !{!10, !11, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$3get17h49cf20e68cb4ce2bE: %self"}
!11 = distinct !{!11, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$3get17h49cf20e68cb4ce2bE"}
!12 = !{!13}
!13 = distinct !{!13, !14, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: %self"}
!14 = distinct !{!14, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E"}
!15 = !{!16}
!16 = distinct !{!16, !17, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!17 = distinct !{!17, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!18 = !{!19, !16, !13, !10, !6, !3}
!19 = distinct !{!19, !20, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!20 = distinct !{!20, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!21 = !{!22}
!22 = distinct !{!22, !14, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: argument 1"}
!23 = !{!13, !10, !6, !3}
!24 = !{!25, !16, !13, !22, !10, !6, !3}
!25 = distinct !{!25, !26, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!26 = distinct !{!26, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!27 = !{i16 0, i16 17}
!28 = !{!29, !16, !13, !22, !10, !6, !3}
!29 = distinct !{!29, !30, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E: %_1"}
!30 = distinct !{!30, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E"}
!31 = !{!32}
!32 = distinct !{!32, !33, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E: %self"}
!33 = distinct !{!33, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$7get_mut17hbd52a0731ab08309E"}
!34 = !{!35}
!35 = distinct !{!35, !36, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$13get_inner_mut17h76f4d04471e07fd8E: %self"}
!36 = distinct !{!36, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$13get_inner_mut17h76f4d04471e07fd8E"}
!37 = !{!35, !32}
!38 = !{!39}
!39 = distinct !{!39, !40, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h23367aad273c1206E: %self"}
!40 = distinct !{!40, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h23367aad273c1206E"}
!41 = !{!42}
!42 = distinct !{!42, !43, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: %self"}
!43 = distinct !{!43, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E"}
!44 = !{!45}
!45 = distinct !{!45, !46, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!46 = distinct !{!46, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!47 = !{!48, !45, !42, !39, !35, !32}
!48 = distinct !{!48, !49, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!49 = distinct !{!49, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!50 = !{!51}
!51 = distinct !{!51, !43, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: argument 1"}
!52 = !{!42, !39, !35, !32}
!53 = !{!54, !45, !42, !51, !39, !35, !32}
!54 = distinct !{!54, !55, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!55 = distinct !{!55, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!56 = !{!57, !45, !42, !51, !39, !35, !32}
!57 = distinct !{!57, !58, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E: %_1"}
!58 = distinct !{!58, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E"}
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
!122 = distinct !{!122, !123, !"_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E: %_1"}
!123 = distinct !{!123, !"_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E"}
!124 = !{!125}
!125 = distinct !{!125, !126, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E: %_1"}
!126 = distinct !{!126, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E"}
!127 = !{!125, !122}
!128 = !{!129, !131, !133}
!129 = distinct !{!129, !130, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E: %dest"}
!130 = distinct !{!130, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E"}
!131 = distinct !{!131, !132, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E: %self"}
!132 = distinct !{!132, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E"}
!133 = distinct !{!133, !134, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE: %val"}
!134 = distinct !{!134, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE"}
!135 = !{!136}
!136 = distinct !{!136, !137, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: %_1"}
!137 = distinct !{!137, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE"}
!138 = !{!139}
!139 = distinct !{!139, !140, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: %_1"}
!140 = distinct !{!140, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E"}
!141 = !{!139, !136}
!142 = !{!143, !144, !125, !122}
!143 = distinct !{!143, !140, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: argument 0"}
!144 = distinct !{!144, !137, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: argument 0"}
!145 = !{!146}
!146 = distinct !{!146, !147, !"_ZN4core3mem7replace17he877d779398bb476E: %dest"}
!147 = distinct !{!147, !"_ZN4core3mem7replace17he877d779398bb476E"}
!148 = !{!143, !139, !144, !136, !125, !122}
!149 = !{!136, !125, !122}
!150 = !{!151}
!151 = distinct !{!151, !152, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E: %self"}
!152 = distinct !{!152, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E"}
!153 = !{!154, !151}
!154 = distinct !{!154, !155, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!155 = distinct !{!155, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!156 = !{!157}
!157 = distinct !{!157, !158, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE: %self"}
!158 = distinct !{!158, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE"}
!159 = !{!160}
!160 = distinct !{!160, !161, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!161 = distinct !{!161, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!162 = !{!160, !157, !151}
!163 = !{!160, !157, !151, !125, !122}
!164 = !{!165}
!165 = distinct !{!165, !166, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE: argument 0"}
!166 = distinct !{!166, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE"}
!167 = !{!168}
!168 = distinct !{!168, !169, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E: argument 0"}
!169 = distinct !{!169, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E"}
!170 = !{!168, !165}
!171 = !{!172}
!172 = distinct !{!172, !173, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E: argument 0"}
!173 = distinct !{!173, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E"}
!174 = !{!175}
!175 = distinct !{!175, !176, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE: argument 0"}
!176 = distinct !{!176, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE"}
!177 = !{!178, !180, !182, !175, !172}
!178 = distinct !{!178, !179, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE: %init"}
!179 = distinct !{!179, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE"}
!180 = distinct !{!180, !181, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E: %init"}
!181 = distinct !{!181, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E"}
!182 = distinct !{!182, !183, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE: argument 0"}
!183 = distinct !{!183, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE"}
!184 = !{!182, !175, !172}
!185 = !{!175, !172}
!186 = !{!187}
!187 = distinct !{!187, !188, !"_ZN4core3mem7replace17h3116444c89fcbd6bE: %dest"}
!188 = distinct !{!188, !"_ZN4core3mem7replace17h3116444c89fcbd6bE"}
!189 = !{!190, !175}
!190 = distinct !{!190, !191, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17hb01b02706bcc63abE: argument 0"}
!191 = distinct !{!191, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17hb01b02706bcc63abE"}
!192 = !{!193}
!193 = distinct !{!193, !194, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hbc7cbddf8870e563E: argument 0"}
!194 = distinct !{!194, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hbc7cbddf8870e563E"}
!195 = !{!196}
!196 = distinct !{!196, !194, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hbc7cbddf8870e563E: %t"}
!197 = !{!193, !196, !172}
!198 = !{!193, !172}
!199 = !{!193, !196}
!200 = !{!201}
!201 = distinct !{!201, !202, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E: %self"}
!202 = distinct !{!202, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E"}
!203 = !{!204, !201}
!204 = distinct !{!204, !205, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!205 = distinct !{!205, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!206 = !{!207}
!207 = distinct !{!207, !208, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE: %self"}
!208 = distinct !{!208, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE"}
!209 = !{!210}
!210 = distinct !{!210, !211, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!211 = distinct !{!211, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!212 = !{!210, !207, !201}
!213 = !{!214}
!214 = distinct !{!214, !215, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!215 = distinct !{!215, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!216 = !{!217}
!217 = distinct !{!217, !218, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!218 = distinct !{!218, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!219 = !{!220, !222}
!220 = distinct !{!220, !221, !"_ZN4core3mem7replace17h788e58c37a635438E: %dest"}
!221 = distinct !{!221, !"_ZN4core3mem7replace17h788e58c37a635438E"}
!222 = distinct !{!222, !223, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE: %self"}
!223 = distinct !{!223, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE"}
!224 = !{!225}
!225 = distinct !{!225, !226, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE: %x.0"}
!226 = distinct !{!226, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE"}
!227 = !{!228}
!228 = distinct !{!228, !229, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E: %self"}
!229 = distinct !{!229, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E"}
!230 = !{!231}
!231 = distinct !{!231, !232, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E: %self"}
!232 = distinct !{!232, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E"}
!233 = !{i64 0, i64 65}
!234 = !{!235, !237, !239, !231, !228}
!235 = distinct !{!235, !236, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE: argument 0"}
!236 = distinct !{!236, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE"}
!237 = distinct !{!237, !238, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E: argument 0"}
!238 = distinct !{!238, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E"}
!239 = distinct !{!239, !240, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E: argument 0"}
!240 = distinct !{!240, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E"}
!241 = !{!242, !237, !239, !231, !228}
!242 = distinct !{!242, !243, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E: argument 0"}
!243 = distinct !{!243, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E"}
!244 = !{!237, !239, !231, !228}
!245 = !{!231, !228}
!246 = !{!247, !249, !250, !252, !231, !228}
!247 = distinct !{!247, !248, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %x"}
!248 = distinct !{!248, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E"}
!249 = distinct !{!249, !248, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y:thread"}
!250 = distinct !{!250, !251, !"_ZN4core3mem4swap17h8292e61c571debd1E: %x"}
!251 = distinct !{!251, !"_ZN4core3mem4swap17h8292e61c571debd1E"}
!252 = distinct !{!252, !251, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y:thread"}
!253 = !{!254}
!254 = distinct !{!254, !255, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!255 = distinct !{!255, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!256 = !{!257, !259, !231, !228}
!257 = distinct !{!257, !258, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %_1"}
!258 = distinct !{!258, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE"}
!259 = distinct !{!259, !258, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %table"}
!260 = !{!261, !263, !265, !231, !228}
!261 = distinct !{!261, !262, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!262 = distinct !{!262, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!263 = distinct !{!263, !264, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!264 = distinct !{!264, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!265 = distinct !{!265, !266, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E: %self"}
!266 = distinct !{!266, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E"}
!267 = !{!263, !265, !231, !228}
!268 = !{!269, !271, !263, !265, !231, !228}
!269 = distinct !{!269, !270, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!270 = distinct !{!270, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!271 = distinct !{!271, !272, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!272 = distinct !{!272, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!273 = !{!274, !276, !265, !231, !228}
!274 = distinct !{!274, !275, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!275 = distinct !{!275, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!276 = distinct !{!276, !277, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!277 = distinct !{!277, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!278 = !{!247, !279, !250, !280, !231, !228}
!279 = distinct !{!279, !248, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y"}
!280 = distinct !{!280, !251, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y"}
!281 = !{!282, !284, !286, !231, !228}
!282 = distinct !{!282, !283, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!283 = distinct !{!283, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!284 = distinct !{!284, !285, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E: %self_"}
!285 = distinct !{!285, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E"}
!286 = distinct !{!286, !287, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E: %self"}
!287 = distinct !{!287, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E"}
!288 = !{!289}
!289 = distinct !{!289, !290, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E: %self"}
!290 = distinct !{!290, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E"}
!291 = !{!292}
!292 = distinct !{!292, !293, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E: %self"}
!293 = distinct !{!293, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E"}
!294 = !{!292, !289, !228}
!295 = !{!296, !298, !292, !289, !228}
!296 = distinct !{!296, !297, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!297 = distinct !{!297, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!298 = distinct !{!298, !299, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!299 = distinct !{!299, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!300 = !{!301, !292, !289, !228}
!301 = distinct !{!301, !302, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE: %a"}
!302 = distinct !{!302, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE"}
!303 = !{!289, !228}
!304 = !{!305}
!305 = distinct !{!305, !306, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!306 = distinct !{!306, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!307 = !{!308, !310, !289, !228}
!308 = distinct !{!308, !309, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %_1"}
!309 = distinct !{!309, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE"}
!310 = distinct !{!310, !309, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %table"}
!311 = !{!312, !314, !289, !228}
!312 = distinct !{!312, !313, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!313 = distinct !{!313, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!314 = distinct !{!314, !315, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!315 = distinct !{!315, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!316 = !{!314, !289, !228}
!317 = !{!318, !320, !314, !289, !228}
!318 = distinct !{!318, !319, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!319 = distinct !{!319, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!320 = distinct !{!320, !321, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!321 = distinct !{!321, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!322 = !{!323, !325, !289, !228}
!323 = distinct !{!323, !324, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!324 = distinct !{!324, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!325 = distinct !{!325, !326, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!326 = distinct !{!326, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!327 = !{!328, !289, !228}
!328 = distinct !{!328, !329, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E: %self"}
!329 = distinct !{!329, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E"}
!330 = !{!331, !333, !328, !289, !228}
!331 = distinct !{!331, !332, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!332 = distinct !{!332, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!333 = distinct !{!333, !334, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!334 = distinct !{!334, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!335 = !{!336, !338, !339, !340, !341, !342, !343, !344, !345, !346, !347, !348, !349, !350, !351, !352}
!336 = distinct !{!336, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It15"}
!337 = distinct !{!337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE"}
!338 = distinct !{!338, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It14"}
!339 = distinct !{!339, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It13"}
!340 = distinct !{!340, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It12"}
!341 = distinct !{!341, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It11"}
!342 = distinct !{!342, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It10"}
!343 = distinct !{!343, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It9"}
!344 = distinct !{!344, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It8"}
!345 = distinct !{!345, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It7"}
!346 = distinct !{!346, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It6"}
!347 = distinct !{!347, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It5"}
!348 = distinct !{!348, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It4"}
!349 = distinct !{!349, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It3"}
!350 = distinct !{!350, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It2"}
!351 = distinct !{!351, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It1"}
!352 = distinct !{!352, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x"}
!353 = !{!354, !355, !356, !357, !358, !359, !360, !361, !362, !363, !364, !365, !366, !367, !368, !369}
!354 = distinct !{!354, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It15"}
!355 = distinct !{!355, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It14"}
!356 = distinct !{!356, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It13"}
!357 = distinct !{!357, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It12"}
!358 = distinct !{!358, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It11"}
!359 = distinct !{!359, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It10"}
!360 = distinct !{!360, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It9"}
!361 = distinct !{!361, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It8"}
!362 = distinct !{!362, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It7"}
!363 = distinct !{!363, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It6"}
!364 = distinct !{!364, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It5"}
!365 = distinct !{!365, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It4"}
!366 = distinct !{!366, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It3"}
!367 = distinct !{!367, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It2"}
!368 = distinct !{!368, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It1"}
!369 = distinct !{!369, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y"}
!370 = !{!371, !372, !373, !374, !375, !376, !377, !378, !379, !380, !381, !382, !383, !384, !385, !386}
!371 = distinct !{!371, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It31"}
!372 = distinct !{!372, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It30"}
!373 = distinct !{!373, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It29"}
!374 = distinct !{!374, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It28"}
!375 = distinct !{!375, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It27"}
!376 = distinct !{!376, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It26"}
!377 = distinct !{!377, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It25"}
!378 = distinct !{!378, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It24"}
!379 = distinct !{!379, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It23"}
!380 = distinct !{!380, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It22"}
!381 = distinct !{!381, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It21"}
!382 = distinct !{!382, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It20"}
!383 = distinct !{!383, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It19"}
!384 = distinct !{!384, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It18"}
!385 = distinct !{!385, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It17"}
!386 = distinct !{!386, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It16"}
!387 = !{!388, !389, !390, !391, !392, !393, !394, !395, !396, !397, !398, !399, !400, !401, !402, !403}
!388 = distinct !{!388, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It31"}
!389 = distinct !{!389, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It30"}
!390 = distinct !{!390, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It29"}
!391 = distinct !{!391, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It28"}
!392 = distinct !{!392, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It27"}
!393 = distinct !{!393, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It26"}
!394 = distinct !{!394, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It25"}
!395 = distinct !{!395, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It24"}
!396 = distinct !{!396, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It23"}
!397 = distinct !{!397, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It22"}
!398 = distinct !{!398, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It21"}
!399 = distinct !{!399, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It20"}
!400 = distinct !{!400, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It19"}
!401 = distinct !{!401, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It18"}
!402 = distinct !{!402, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It17"}
!403 = distinct !{!403, !337, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It16"}
!404 = !{!405, !289, !228}
!405 = distinct !{!405, !406, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!406 = distinct !{!406, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!407 = !{!408, !410, !412}
!408 = distinct !{!408, !409, !"_ZN4core3mem7replace17ha318695de15894dbE: %dest"}
!409 = distinct !{!409, !"_ZN4core3mem7replace17ha318695de15894dbE"}
!410 = distinct !{!410, !411, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E: %self"}
!411 = distinct !{!411, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E"}
!412 = distinct !{!412, !413, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E: %val"}
!413 = distinct !{!413, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E"}
!414 = !{!415}
!415 = distinct !{!415, !416, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: %_1"}
!416 = distinct !{!416, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE"}
!417 = !{!418}
!418 = distinct !{!418, !419, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: %_1"}
!419 = distinct !{!419, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE"}
!420 = !{!418, !415}
!421 = !{!422, !423}
!422 = distinct !{!422, !419, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: argument 0"}
!423 = distinct !{!423, !416, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: argument 0"}
!424 = !{!425}
!425 = distinct !{!425, !426, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E: %dest"}
!426 = distinct !{!426, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E"}
!427 = !{!422, !418, !423, !415}
!428 = !{!429, !431, !433}
!429 = distinct !{!429, !430, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E: %dest"}
!430 = distinct !{!430, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E"}
!431 = distinct !{!431, !432, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E: %self"}
!432 = distinct !{!432, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E"}
!433 = distinct !{!433, !434, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE: %val"}
!434 = distinct !{!434, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE"}
!435 = !{!436}
!436 = distinct !{!436, !437, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: %_1"}
!437 = distinct !{!437, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE"}
!438 = !{!439}
!439 = distinct !{!439, !440, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: %_1"}
!440 = distinct !{!440, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E"}
!441 = !{!439, !436}
!442 = !{!443, !444}
!443 = distinct !{!443, !440, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: argument 0"}
!444 = distinct !{!444, !437, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: argument 0"}
!445 = !{!446}
!446 = distinct !{!446, !447, !"_ZN4core3mem7replace17he877d779398bb476E: %dest"}
!447 = distinct !{!447, !"_ZN4core3mem7replace17he877d779398bb476E"}
!448 = !{!443, !439, !444, !436}
!449 = !{!450}
!450 = distinct !{!450, !451, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E: %self"}
!451 = distinct !{!451, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E"}
!452 = !{!453, !450}
!453 = distinct !{!453, !454, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!454 = distinct !{!454, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!455 = !{!456}
!456 = distinct !{!456, !457, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE: %self"}
!457 = distinct !{!457, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE"}
!458 = !{!459}
!459 = distinct !{!459, !460, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!460 = distinct !{!460, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!461 = !{!459, !456, !450}
!462 = !{!463}
!463 = distinct !{!463, !464, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!464 = distinct !{!464, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!465 = !{!466, !467}
!466 = distinct !{!466, !464, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!467 = distinct !{!467, !464, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!468 = !{!469}
!469 = distinct !{!469, !470, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E: %self"}
!470 = distinct !{!470, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h7beda6ed374dc037E"}
!471 = !{!472, !474}
!472 = distinct !{!472, !473, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E: %f"}
!473 = distinct !{!473, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E"}
!474 = distinct !{!474, !475, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E: %f"}
!475 = distinct !{!475, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E"}
!476 = !{!477}
!477 = distinct !{!477, !478, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE: argument 0"}
!478 = distinct !{!478, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE"}
!479 = !{!480, !477}
!480 = distinct !{!480, !481, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E: argument 0"}
!481 = distinct !{!481, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E"}
!482 = !{!483}
!483 = distinct !{!483, !484, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E: %self"}
!484 = distinct !{!484, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E"}
!485 = !{!486}
!486 = distinct !{!486, !487, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: argument 0"}
!487 = distinct !{!487, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E"}
!488 = !{!489, !490, !491}
!489 = distinct !{!489, !487, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %pieces.0"}
!490 = distinct !{!490, !487, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %args.0"}
!491 = distinct !{!491, !487, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %fmt.0"}
!492 = !{!493, !495}
!493 = distinct !{!493, !494, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE: %f"}
!494 = distinct !{!494, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE"}
!495 = distinct !{!495, !496, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E: %f"}
!496 = distinct !{!496, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E"}
!497 = !{!498}
!498 = distinct !{!498, !499, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E: argument 0"}
!499 = distinct !{!499, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E"}
!500 = !{!501, !498}
!501 = distinct !{!501, !502, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE: argument 0"}
!502 = distinct !{!502, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE"}
!503 = !{!504, !506}
!504 = distinct !{!504, !505, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: %self"}
!505 = distinct !{!505, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE"}
!506 = distinct !{!506, !505, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: argument 1"}
!507 = !{!504}
!508 = !{!509}
!509 = distinct !{!509, !510, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E: %self"}
!510 = distinct !{!510, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E"}
!511 = !{!512}
!512 = distinct !{!512, !513, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE: %self"}
!513 = distinct !{!513, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE"}
!514 = !{!512, !509}
!515 = !{!516, !517, !518, !519}
!516 = distinct !{!516, !513, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE: argument 0"}
!517 = distinct !{!517, !513, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE: %v"}
!518 = distinct !{!518, !510, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E: argument 0"}
!519 = distinct !{!519, !510, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E: %v"}
!520 = !{!521}
!521 = distinct !{!521, !522, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h23367aad273c1206E: %self"}
!522 = distinct !{!522, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h23367aad273c1206E"}
!523 = !{!524}
!524 = distinct !{!524, !525, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: %self"}
!525 = distinct !{!525, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E"}
!526 = !{!527}
!527 = distinct !{!527, !528, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!528 = distinct !{!528, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!529 = !{!530, !527, !524, !521, !512, !509}
!530 = distinct !{!530, !531, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!531 = distinct !{!531, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!532 = !{!533, !516, !517, !518, !519}
!533 = distinct !{!533, !525, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: argument 1"}
!534 = !{!524, !521, !512, !509}
!535 = !{!536, !527, !524, !533, !521, !516, !512, !517, !518, !509, !519}
!536 = distinct !{!536, !537, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!537 = distinct !{!537, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!538 = !{!539, !527, !524, !533, !521, !516, !512, !517, !518, !509, !519}
!539 = distinct !{!539, !540, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E: %_1"}
!540 = distinct !{!540, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E"}
!541 = !{!542}
!542 = distinct !{!542, !543, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE: %self"}
!543 = distinct !{!543, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE"}
!544 = !{!545, !547, !542, !549, !550, !516, !512, !517, !518, !509, !519}
!545 = distinct !{!545, !546, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!546 = distinct !{!546, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!547 = distinct !{!547, !548, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!548 = distinct !{!548, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!549 = distinct !{!549, !543, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE: %value"}
!550 = distinct !{!550, !543, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE: %hasher"}
!551 = !{!547, !542, !549, !550, !516, !512, !517, !518, !509, !519}
!552 = !{!553, !555, !547, !542, !549, !550, !516, !512, !517, !518, !509, !519}
!553 = distinct !{!553, !554, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!554 = distinct !{!554, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!555 = distinct !{!555, !556, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!556 = distinct !{!556, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!557 = !{!542, !549, !550, !516, !512, !517, !518, !509, !519}
!558 = !{!542, !512, !509}
!559 = !{!549, !550, !516, !517, !518, !519}
!560 = !{!561}
!561 = distinct !{!561, !562, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!562 = distinct !{!562, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!563 = !{!564, !561, !542, !512, !509}
!564 = distinct !{!564, !565, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!565 = distinct !{!565, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!566 = !{!561, !542, !512, !509}
!567 = !{!568, !561, !542, !549, !516, !517, !518, !519}
!568 = distinct !{!568, !569, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!569 = distinct !{!569, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!570 = !{!561, !542, !549, !516, !517, !518, !519}
!571 = !{!572, !574, !561, !542, !549, !516, !517, !518, !519}
!572 = distinct !{!572, !573, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!573 = distinct !{!573, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!574 = distinct !{!574, !575, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!575 = distinct !{!575, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!576 = !{!577}
!577 = distinct !{!577, !578, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E: %self"}
!578 = distinct !{!578, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E"}
!579 = !{!580, !582, !577, !542, !549, !516, !517, !518, !519}
!580 = distinct !{!580, !581, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!581 = distinct !{!581, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!582 = distinct !{!582, !583, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!583 = distinct !{!583, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!584 = !{!577, !542, !512, !509}
!585 = !{!542, !516, !517, !518, !519}
!586 = !{!518}
!587 = !{!588}
!588 = distinct !{!588, !589, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!589 = distinct !{!589, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!590 = !{!591}
!591 = distinct !{!591, !592, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE: %self"}
!592 = distinct !{!592, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE"}
!593 = !{!594}
!594 = distinct !{!594, !595, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: argument 0"}
!595 = distinct !{!595, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E"}
!596 = !{!597, !598, !599}
!597 = distinct !{!597, !595, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %pieces.0"}
!598 = distinct !{!598, !595, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %args.0"}
!599 = distinct !{!599, !595, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %fmt.0"}
!600 = !{!601, !603}
!601 = distinct !{!601, !602, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE: %f"}
!602 = distinct !{!602, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE"}
!603 = distinct !{!603, !604, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E: %f"}
!604 = distinct !{!604, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E"}
!605 = !{!606}
!606 = distinct !{!606, !607, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E: argument 0"}
!607 = distinct !{!607, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E"}
!608 = !{!609, !606}
!609 = distinct !{!609, !610, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE: argument 0"}
!610 = distinct !{!610, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE"}
!611 = !{!612, !614}
!612 = distinct !{!612, !613, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: %self"}
!613 = distinct !{!613, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE"}
!614 = distinct !{!614, !613, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: argument 1"}
!615 = !{!612}
!616 = !{!617}
!617 = distinct !{!617, !618, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!618 = distinct !{!618, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!619 = !{!620}
!620 = distinct !{!620, !621, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!621 = distinct !{!621, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!622 = !{!623, !624}
!623 = distinct !{!623, !621, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!624 = distinct !{!624, !621, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!625 = !{!626}
!626 = distinct !{!626, !627, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!627 = distinct !{!627, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!628 = !{!629}
!629 = distinct !{!629, !630, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: argument 0"}
!630 = distinct !{!630, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E"}
!631 = !{!632, !633, !634}
!632 = distinct !{!632, !630, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %pieces.0"}
!633 = distinct !{!633, !630, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %args.0"}
!634 = distinct !{!634, !630, !"_ZN4core3fmt9Arguments16new_v1_formatted17h305cdac3d1ef4645E: %fmt.0"}
!635 = !{!636, !638}
!636 = distinct !{!636, !637, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE: %f"}
!637 = distinct !{!637, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE"}
!638 = distinct !{!638, !639, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E: %f"}
!639 = distinct !{!639, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E"}
!640 = !{!641}
!641 = distinct !{!641, !642, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E: argument 0"}
!642 = distinct !{!642, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E"}
!643 = !{!644, !641}
!644 = distinct !{!644, !645, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE: argument 0"}
!645 = distinct !{!645, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE"}
!646 = !{!647, !649}
!647 = distinct !{!647, !648, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: %self"}
!648 = distinct !{!648, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE"}
!649 = distinct !{!649, !648, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: argument 1"}
!650 = !{!647}
!651 = !{!652}
!652 = distinct !{!652, !653, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!653 = distinct !{!653, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!654 = !{!655, !656}
!655 = distinct !{!655, !653, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!656 = distinct !{!656, !653, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!657 = !{!658}
!658 = distinct !{!658, !659, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!659 = distinct !{!659, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!660 = !{!661}
!661 = distinct !{!661, !662, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h07c9a48d0726e1afE: %self"}
!662 = distinct !{!662, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h07c9a48d0726e1afE"}
!663 = !{!664}
!664 = distinct !{!664, !665, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h467dae58b8e28e55E: %self"}
!665 = distinct !{!665, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h467dae58b8e28e55E"}
!666 = !{!667}
!667 = distinct !{!667, !668, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h8a63ae6c0f3b74a7E: %self"}
!668 = distinct !{!668, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h8a63ae6c0f3b74a7E"}
!669 = !{!667, !664, !661}
!670 = !{!671, !672, !673}
!671 = distinct !{!671, !668, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h8a63ae6c0f3b74a7E: argument 0"}
!672 = distinct !{!672, !665, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h467dae58b8e28e55E: argument 0"}
!673 = distinct !{!673, !662, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h07c9a48d0726e1afE: argument 0"}
!674 = !{!675}
!675 = distinct !{!675, !676, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17head61d0e4749a2cfE: %self"}
!676 = distinct !{!676, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17head61d0e4749a2cfE"}
!677 = !{!678}
!678 = distinct !{!678, !679, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: %self"}
!679 = distinct !{!679, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E"}
!680 = !{!681}
!681 = distinct !{!681, !682, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!682 = distinct !{!682, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!683 = !{!684, !681, !678, !675, !667, !664, !661}
!684 = distinct !{!684, !685, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!685 = distinct !{!685, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!686 = !{!687, !688, !671, !672, !673}
!687 = distinct !{!687, !679, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: argument 1"}
!688 = distinct !{!688, !676, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17head61d0e4749a2cfE: argument 0"}
!689 = !{!678, !675, !667, !664, !661}
!690 = !{!691, !681, !678, !687, !688, !675, !671, !667, !672, !664, !673, !661}
!691 = distinct !{!691, !692, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!692 = distinct !{!692, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!693 = !{!694, !681, !678, !687, !688, !675, !671, !667, !672, !664, !673, !661}
!694 = distinct !{!694, !695, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E: %_1"}
!695 = distinct !{!695, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E"}
!696 = !{!697}
!697 = distinct !{!697, !698, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17h12237f430f8cfaadE: %self"}
!698 = distinct !{!698, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17h12237f430f8cfaadE"}
!699 = !{!700}
!700 = distinct !{!700, !701, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h0cfad37b6833ba5fE: %self"}
!701 = distinct !{!701, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h0cfad37b6833ba5fE"}
!702 = !{!703}
!703 = distinct !{!703, !704, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E: %self"}
!704 = distinct !{!704, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E"}
!705 = !{!706, !703, !700, !708, !697, !688, !675, !671, !667, !672, !664, !673, !661}
!706 = distinct !{!706, !707, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!707 = distinct !{!707, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!708 = distinct !{!708, !698, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17h12237f430f8cfaadE: argument 0"}
!709 = !{!710, !703, !700, !708, !697, !688, !675, !671, !667, !672, !664, !673, !661}
!710 = distinct !{!710, !711, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!711 = distinct !{!711, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!712 = !{!703, !700, !697, !675, !667, !664, !661}
!713 = !{!708, !688, !671, !672, !673}
!714 = !{!703, !700, !708, !697, !688, !675, !671, !667, !672, !664, !673, !661}
