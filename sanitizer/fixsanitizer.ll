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
%"core::fmt::Arguments" = type { { [0 x { [0 x i8]*, i64 }]*, i64 }, { i64*, i64 }, { [0 x { i8*, i64* }]*, i64 } }
%"unwind::libunwind::_Unwind_Exception" = type { i64, void (i32, %"unwind::libunwind::_Unwind_Exception"*)*, [6 x i64] }
%"unwind::libunwind::_Unwind_Context" = type { [0 x i8] }

@_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE = external thread_local global %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"
@alloc360 = private unnamed_addr constant <{ [70 x i8] }> <{ [70 x i8] c"cannot access a Thread Local Storage value during or after destruction" }>, align 1
@alloc363 = private unnamed_addr constant <{ [79 x i8] }> <{ [79 x i8] c"/rustc/a8314ef7d0ec7b75c336af2c9857bfaf43002bfc/library/std/src/thread/local.rs" }>, align 1
@alloc362 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [79 x i8] }>, <{ [79 x i8] }>* @alloc363, i32 0, i32 0, i32 0), [16 x i8] c"O\00\00\00\00\00\00\00\A5\01\00\00\1A\00\00\00" }>, align 8
@vtable.0 = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17h92e001d5e4efd74cE" to i8*), i8* bitcast ({ {}*, [3 x i64]* } ({ i8*, i64 }*)* @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17hc9f8af2660d4514aE" to i8*) }>, align 8
@_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E = external local_unnamed_addr global %"core::sync::atomic::AtomicUsize"
@alloc399 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Option::unwrap()` on a `None` value" }>, align 1
@vtable.3 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\00\00\00\00\00\00\00\00\01\00\00\00\00\00\00\00", i8* bitcast (i1 (%"std::thread::local::AccessError"*, %"core::fmt::Formatter"*)* @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h514ef917cd5ecc1bE" to i8*) }>, align 8
@alloc407 = private unnamed_addr constant <{ [43 x i8] }> <{ [43 x i8] c"called `Result::unwrap()` on an `Err` value" }>, align 1
@vtable.5 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void ({ i64*, i8 }*)* @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 ({ i64*, i8 }*, %"core::fmt::Formatter"*)* @"_ZN76_$LT$std..sync..poison..PoisonError$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h0cd32de15374fa48E" to i8*) }>, align 8
@vtable.6 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\08\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (i64**, %"core::fmt::Formatter"*)* @"_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hc715f6c95a655b17E" to i8*) }>, align 8
@alloc417 = private unnamed_addr constant <{ [11 x i8] }> <{ [11 x i8] c"PoisonError" }>, align 1
@vtable.7 = private unnamed_addr constant <{ i8*, [16 x i8], i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\10\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i64 ({ [0 x i8]*, i64 }*)* @"_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17ha7daf7c2b2ea8d27E" to i8*) }>, align 8
@alloc67 = private unnamed_addr constant <{ [16 x i8] }> <{ [16 x i8] c"\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF\FF" }>, align 16
@vtable.a = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h42a39cd9ab169dceE" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<i64>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<i64>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17h69c8db5230288c49E" to i8*) }>, align 8
@vtable.b = private unnamed_addr constant <{ i8*, [16 x i8], i8*, i8* }> <{ i8* bitcast (void (i64**)* @"_ZN4core3ptr28drop_in_place$LT$$RF$i64$GT$17h4de5395864ed3692E" to i8*), [16 x i8] c"\18\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00", i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hcf0b305cdf28ac00E" to i8*), i8* bitcast (i1 (%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"*)* @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E" to i8*) }>, align 8
@alloc449 = private unnamed_addr constant <{ [42 x i8] }> <{ [42 x i8] c"Lazy instance has previously been poisoned" }>, align 1
@alloc450 = private unnamed_addr constant <{ [90 x i8] }> <{ [90 x i8] c"/home/maruyama/.cargo/registry/src/github.com-1ecc6299db9ec823/once_cell-1.13.0/src/lib.rs" }>, align 1
@alloc448 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [90 x i8] }>, <{ [90 x i8] }>* @alloc450, i32 0, i32 0, i32 0), [16 x i8] c"Z\00\00\00\00\00\00\00\CF\04\00\00\19\00\00\00" }>, align 8
@_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE = internal global <{ [16 x i8], [16 x i8], i8* }> <{ [16 x i8] zeroinitializer, [16 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<i64>"*)* @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE to i8*) }>, align 8
@_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE = internal global <{ [16 x i8], [56 x i8], i8* }> <{ [16 x i8] zeroinitializer, [56 x i8] undef, i8* bitcast (void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)* @_ZN4core3ops8function6FnOnce9call_once17hd20ed85d13df1445E to i8*) }>, align 8
@alloc474 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"src/lib.rs" }>, align 1
@alloc453 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00\1D\00\00\00)\00\00\00" }>, align 8
@alloc455 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00&\00\00\003\00\00\00" }>, align 8
@alloc244 = private unnamed_addr constant <{ [8 x i8] }> zeroinitializer, align 8
@alloc249 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"Object id=" }>, align 1
@alloc251 = private unnamed_addr constant <{ [31 x i8] }> <{ [31 x i8] c" whose refcnt zero is retained!" }>, align 1
@alloc250 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc249, i32 0, i32 0, i32 0), [8 x i8] c"\0A\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [31 x i8] }>, <{ [31 x i8] }>* @alloc251, i32 0, i32 0, i32 0), [8 x i8] c"\1F\00\00\00\00\00\00\00" }>, align 8
@alloc457 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00;\00\00\00\05\00\00\00" }>, align 8
@alloc459 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00@\00\00\003\00\00\00" }>, align 8
@alloc194 = private unnamed_addr constant <{ [20 x i8] }> <{ [20 x i8] c"Retain of object id=" }>, align 1
@alloc256 = private unnamed_addr constant <{ [50 x i8] }> <{ [50 x i8] c" is reported but it isn't registered to sanitizer." }>, align 1
@alloc195 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [20 x i8] }>, <{ [20 x i8] }>* @alloc194, i32 0, i32 0, i32 0), [8 x i8] c"\14\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc256, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc461 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00A\00\00\00\05\00\00\00" }>, align 8
@alloc463 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00F\00\00\00.\00\00\00" }>, align 8
@alloc261 = private unnamed_addr constant <{ [24 x i8] }> <{ [24 x i8] c"The refcnt of object id=" }>, align 1
@alloc263 = private unnamed_addr constant <{ [20 x i8] }> <{ [20 x i8] c" mismatch! reported=" }>, align 1
@alloc264 = private unnamed_addr constant <{ [12 x i8] }> <{ [12 x i8] c", sanitizer=" }>, align 1
@alloc262 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [24 x i8] }>, <{ [24 x i8] }>* @alloc261, i32 0, i32 0, i32 0), [8 x i8] c"\18\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [20 x i8] }>, <{ [20 x i8] }>* @alloc263, i32 0, i32 0, i32 0), [8 x i8] c"\14\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [12 x i8] }>, <{ [12 x i8] }>* @alloc264, i32 0, i32 0, i32 0), [8 x i8] c"\0C\00\00\00\00\00\00\00" }>, align 8
@alloc465 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00G\00\00\00\05\00\00\00" }>, align 8
@alloc467 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00Z\00\00\00\05\00\00\00" }>, align 8
@alloc469 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00_\00\00\002\00\00\00" }>, align 8
@alloc254 = private unnamed_addr constant <{ [21 x i8] }> <{ [21 x i8] c"Release of object id=" }>, align 1
@alloc255 = private unnamed_addr constant <{ i8*, [8 x i8], i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [21 x i8] }>, <{ [21 x i8] }>* @alloc254, i32 0, i32 0, i32 0), [8 x i8] c"\15\00\00\00\00\00\00\00", i8* getelementptr inbounds (<{ [50 x i8] }>, <{ [50 x i8] }>* @alloc256, i32 0, i32 0, i32 0), [8 x i8] c"2\00\00\00\00\00\00\00" }>, align 8
@alloc471 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00`\00\00\00\05\00\00\00" }>, align 8
@alloc473 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00e\00\00\00-\00\00\00" }>, align 8
@alloc475 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc474, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00f\00\00\00\05\00\00\00" }>, align 8

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
  tail call fastcc void @"_ZN3std9panicking11begin_panic28_$u7b$$u7b$closure$u7d$$u7d$17h56b3894ae78ba8e2E"([0 x i8]* %_2.sroa.0.0.copyload, i64 %_2.sroa.3.0.copyload, %"core::panic::location::Location"* %_2.sroa.4.0.copyload) #22
  unreachable
}

; std::collections::hash::map::HashMap<K,V,S>::contains_key
; Function Attrs: inlinehint nofree nosync nounwind nonlazybind uwtable
define internal fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h7c6dbde3483cee85E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias nocapture noundef readonly align 8 dereferenceable(48) %self, i64 %k.val1) unnamed_addr #2 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !2)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !5) #23
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
  %7 = tail call i64 @llvm.fshl.i64(i64 %2, i64 %2, i64 13) #23
  %8 = xor i64 %6, %7
  %9 = tail call i64 @llvm.fshl.i64(i64 %6, i64 %6, i64 32) #23
  %10 = add i64 %5, %3
  %11 = tail call i64 @llvm.fshl.i64(i64 %5, i64 %5, i64 16) #23
  %12 = xor i64 %11, %10
  %13 = add i64 %12, %9
  %14 = tail call i64 @llvm.fshl.i64(i64 %12, i64 %12, i64 21) #23
  %15 = xor i64 %14, %13
  %16 = add i64 %8, %10
  %17 = tail call i64 @llvm.fshl.i64(i64 %8, i64 %8, i64 17) #23
  %18 = xor i64 %16, %17
  %19 = tail call i64 @llvm.fshl.i64(i64 %16, i64 %16, i64 32) #23
  %20 = xor i64 %13, %k.val1
  %21 = xor i64 %15, 576460752303423488
  %22 = add i64 %20, %18
  %23 = tail call i64 @llvm.fshl.i64(i64 %18, i64 %18, i64 13) #23
  %24 = xor i64 %22, %23
  %25 = tail call i64 @llvm.fshl.i64(i64 %22, i64 %22, i64 32) #23
  %26 = add i64 %21, %19
  %27 = tail call i64 @llvm.fshl.i64(i64 %15, i64 %21, i64 16) #23
  %28 = xor i64 %27, %26
  %29 = add i64 %28, %25
  %30 = tail call i64 @llvm.fshl.i64(i64 %28, i64 %28, i64 21) #23
  %31 = xor i64 %30, %29
  %32 = add i64 %26, %24
  %33 = tail call i64 @llvm.fshl.i64(i64 %24, i64 %24, i64 17) #23
  %34 = xor i64 %32, %33
  %35 = tail call i64 @llvm.fshl.i64(i64 %32, i64 %32, i64 32) #23
  %36 = xor i64 %29, 576460752303423488
  %37 = xor i64 %35, 255
  %38 = add i64 %36, %34
  %39 = tail call i64 @llvm.fshl.i64(i64 %34, i64 %34, i64 13) #23
  %40 = xor i64 %38, %39
  %41 = tail call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 32) #23
  %42 = add i64 %31, %37
  %43 = tail call i64 @llvm.fshl.i64(i64 %31, i64 %31, i64 16) #23
  %44 = xor i64 %43, %42
  %45 = add i64 %44, %41
  %46 = tail call i64 @llvm.fshl.i64(i64 %44, i64 %44, i64 21) #23
  %47 = xor i64 %46, %45
  %48 = add i64 %40, %42
  %49 = tail call i64 @llvm.fshl.i64(i64 %40, i64 %40, i64 17) #23
  %50 = xor i64 %48, %49
  %51 = tail call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 32) #23
  %52 = add i64 %50, %45
  %53 = tail call i64 @llvm.fshl.i64(i64 %50, i64 %50, i64 13) #23
  %54 = xor i64 %53, %52
  %55 = tail call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 32) #23
  %56 = add i64 %47, %51
  %57 = tail call i64 @llvm.fshl.i64(i64 %47, i64 %47, i64 16) #23
  %58 = xor i64 %57, %56
  %59 = add i64 %58, %55
  %60 = tail call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 21) #23
  %61 = xor i64 %60, %59
  %62 = add i64 %54, %56
  %63 = tail call i64 @llvm.fshl.i64(i64 %54, i64 %54, i64 17) #23
  %64 = xor i64 %63, %62
  %65 = tail call i64 @llvm.fshl.i64(i64 %62, i64 %62, i64 32) #23
  %66 = add i64 %64, %59
  %67 = tail call i64 @llvm.fshl.i64(i64 %64, i64 %64, i64 13) #23
  %68 = xor i64 %67, %66
  %69 = add i64 %61, %65
  %70 = tail call i64 @llvm.fshl.i64(i64 %61, i64 %61, i64 16) #23
  %71 = xor i64 %70, %69
  %72 = tail call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 21) #23
  %73 = add i64 %68, %69
  %74 = tail call i64 @llvm.fshl.i64(i64 %68, i64 %68, i64 17) #23
  %75 = tail call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 32) #23
  %_17.i.i.i.i.i.i.i = xor i64 %73, %72
  %76 = xor i64 %_17.i.i.i.i.i.i.i, %74
  %77 = xor i64 %76, %75
  tail call void @llvm.experimental.noalias.scope.decl(metadata !9) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !12) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !15) #23
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
  %88 = tail call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #23, !range !27
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !34) #23
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
  %7 = tail call i64 @llvm.fshl.i64(i64 %2, i64 %2, i64 13) #23
  %8 = xor i64 %6, %7
  %9 = tail call i64 @llvm.fshl.i64(i64 %6, i64 %6, i64 32) #23
  %10 = add i64 %5, %3
  %11 = tail call i64 @llvm.fshl.i64(i64 %5, i64 %5, i64 16) #23
  %12 = xor i64 %11, %10
  %13 = add i64 %12, %9
  %14 = tail call i64 @llvm.fshl.i64(i64 %12, i64 %12, i64 21) #23
  %15 = xor i64 %14, %13
  %16 = add i64 %8, %10
  %17 = tail call i64 @llvm.fshl.i64(i64 %8, i64 %8, i64 17) #23
  %18 = xor i64 %16, %17
  %19 = tail call i64 @llvm.fshl.i64(i64 %16, i64 %16, i64 32) #23
  %20 = xor i64 %13, %k.val1
  %21 = xor i64 %15, 576460752303423488
  %22 = add i64 %20, %18
  %23 = tail call i64 @llvm.fshl.i64(i64 %18, i64 %18, i64 13) #23
  %24 = xor i64 %22, %23
  %25 = tail call i64 @llvm.fshl.i64(i64 %22, i64 %22, i64 32) #23
  %26 = add i64 %21, %19
  %27 = tail call i64 @llvm.fshl.i64(i64 %15, i64 %21, i64 16) #23
  %28 = xor i64 %27, %26
  %29 = add i64 %28, %25
  %30 = tail call i64 @llvm.fshl.i64(i64 %28, i64 %28, i64 21) #23
  %31 = xor i64 %30, %29
  %32 = add i64 %26, %24
  %33 = tail call i64 @llvm.fshl.i64(i64 %24, i64 %24, i64 17) #23
  %34 = xor i64 %32, %33
  %35 = tail call i64 @llvm.fshl.i64(i64 %32, i64 %32, i64 32) #23
  %36 = xor i64 %29, 576460752303423488
  %37 = xor i64 %35, 255
  %38 = add i64 %36, %34
  %39 = tail call i64 @llvm.fshl.i64(i64 %34, i64 %34, i64 13) #23
  %40 = xor i64 %38, %39
  %41 = tail call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 32) #23
  %42 = add i64 %31, %37
  %43 = tail call i64 @llvm.fshl.i64(i64 %31, i64 %31, i64 16) #23
  %44 = xor i64 %43, %42
  %45 = add i64 %44, %41
  %46 = tail call i64 @llvm.fshl.i64(i64 %44, i64 %44, i64 21) #23
  %47 = xor i64 %46, %45
  %48 = add i64 %40, %42
  %49 = tail call i64 @llvm.fshl.i64(i64 %40, i64 %40, i64 17) #23
  %50 = xor i64 %48, %49
  %51 = tail call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 32) #23
  %52 = add i64 %50, %45
  %53 = tail call i64 @llvm.fshl.i64(i64 %50, i64 %50, i64 13) #23
  %54 = xor i64 %53, %52
  %55 = tail call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 32) #23
  %56 = add i64 %47, %51
  %57 = tail call i64 @llvm.fshl.i64(i64 %47, i64 %47, i64 16) #23
  %58 = xor i64 %57, %56
  %59 = add i64 %58, %55
  %60 = tail call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 21) #23
  %61 = xor i64 %60, %59
  %62 = add i64 %54, %56
  %63 = tail call i64 @llvm.fshl.i64(i64 %54, i64 %54, i64 17) #23
  %64 = xor i64 %63, %62
  %65 = tail call i64 @llvm.fshl.i64(i64 %62, i64 %62, i64 32) #23
  %66 = add i64 %64, %59
  %67 = tail call i64 @llvm.fshl.i64(i64 %64, i64 %64, i64 13) #23
  %68 = xor i64 %67, %66
  %69 = add i64 %61, %65
  %70 = tail call i64 @llvm.fshl.i64(i64 %61, i64 %61, i64 16) #23
  %71 = xor i64 %70, %69
  %72 = tail call i64 @llvm.fshl.i64(i64 %71, i64 %71, i64 21) #23
  %73 = add i64 %68, %69
  %74 = tail call i64 @llvm.fshl.i64(i64 %68, i64 %68, i64 17) #23
  %75 = tail call i64 @llvm.fshl.i64(i64 %73, i64 %73, i64 32) #23
  %_17.i.i.i.i.i.i.i = xor i64 %73, %72
  %76 = xor i64 %_17.i.i.i.i.i.i.i, %74
  %77 = xor i64 %76, %75
  tail call void @llvm.experimental.noalias.scope.decl(metadata !38) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !41) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !44) #23
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
  %88 = tail call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #23, !range !27
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !68) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !71) #23
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
  store [0 x i8]* bitcast (<{ [42 x i8] }>* @alloc449 to [0 x i8]*), [0 x i8]** %1, align 8
  %2 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 0, i32 1
  store i64 42, i64* %2, align 8
  %3 = getelementptr inbounds %"[closure@std::panicking::begin_panic<&str>::{closure#0}]", %"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* %_3, i64 0, i32 1
  store %"core::panic::location::Location"* bitcast (<{ i8*, [16 x i8] }>* @alloc448 to %"core::panic::location::Location"*), %"core::panic::location::Location"** %3, align 8
; call std::sys_common::backtrace::__rust_end_short_backtrace
  call fastcc void @_ZN3std10sys_common9backtrace26__rust_end_short_backtrace17hea36b766ad666feaE(%"[closure@std::panicking::begin_panic<&str>::{closure#0}]"* noalias nocapture noundef nonnull dereferenceable(24) %_3) #22
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
  call void @_ZN3std9panicking20rust_panic_with_hook17hc82286af2030e925E({}* noundef nonnull align 1 %_2.0, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.0 to [3 x i64]*), i64* noalias noundef readonly align 8 dereferenceable_or_null(48) null, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) %_1.0.1.val, i1 noundef zeroext true) #22
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
  tail call void @llvm.assume(i1 %3) #23
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
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #22, !noalias !117
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
define internal noundef zeroext i1 @"_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17hcf0b305cdf28ac00E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* nocapture readonly %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
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
  tail call void @llvm.assume(i1 %3) #23
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
  br i1 %6, label %bb2.i.i.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i"

bb2.i.i.i.i:                                      ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #22, !noalias !147
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0.i.i), !noalias !148
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !alias.scope !126, !nonnull !85, !align !86, !noundef !85
  %_17.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !126
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 0
  %_2.i16.i.i = load i64, i64* %9, align 8, !range !119, !noalias !126, !noundef !85
  %10 = icmp eq i64 %_2.i16.i.i, 0
  br i1 %10, label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit, label %bb2.i.i.i

bb2.i.i.i:                                        ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i"
  %11 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1
  %12 = bitcast [7 x i64]* %11 to %"std::sys_common::mutex::MovableMutex"*
; invoke <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
  invoke void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef nonnull align 4 dereferenceable(4) %12)
          to label %bb4.i.i.i.i unwind label %cleanup.i.i.i.i, !noalias !126

cleanup.i.i.i.i:                                  ; preds = %bb2.i.i.i
  %13 = landingpad { i8*, i32 }
          cleanup
  %14 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 1
  %15 = bitcast i64* %14 to %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*
; call core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #24, !noalias !126
  %_20.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !126
  %_10.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx.i.i, align 8, !noalias !126
  %_10.sroa.5.0..sroa_idx.i.i = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20.i.i, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast.i.i = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast.i.i, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20.i.i, i64 56, i1 false), !noalias !126
  resume { i8*, i32 } %13

bb4.i.i.i.i:                                      ; preds = %bb2.i.i.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 3
  tail call void @llvm.experimental.noalias.scope.decl(metadata !149) #23
  %_2.i.i.i.i.i.i.i.i.i.i = load i64, i64* %16, align 8, !alias.scope !152, !noalias !126
  %17 = icmp eq i64 %_2.i.i.i.i.i.i.i.i.i.i, 0
  br i1 %17, label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit, label %bb2.i.i.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i.i.i:                            ; preds = %bb4.i.i.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !155) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !158) #23
  %18 = add i64 %_2.i.i.i.i.i.i.i.i.i.i, 1
  %19 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %18, i64 32) #23
  %20 = extractvalue { i64, i1 } %19, 1
  %21 = xor i1 %20, true
  tail call void @llvm.assume(i1 %21) #23
  %22 = extractvalue { i64, i1 } %19, 0
  %_31.i.i.i.i.i.i.i.i.i.i.i.i = add i64 %_2.i.i.i.i.i.i.i.i.i.i, 17
  %23 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %22, i64 %_31.i.i.i.i.i.i.i.i.i.i.i.i) #23
  %24 = extractvalue { i64, i1 } %23, 1
  %25 = xor i1 %24, true
  tail call void @llvm.assume(i1 %25) #23
  %26 = extractvalue { i64, i1 } %23, 0
  %27 = icmp eq i64 %26, 0
  br i1 %27, label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit, label %bb2.i.i.i.i.i.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i.i.i.i.i.i:                      ; preds = %bb2.i.i.i.i.i.i.i.i.i
  %28 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17.i.i, i64 0, i32 1, i64 4
  %29 = bitcast i64* %28 to i8**
  %_17.i.i.i.i.i.i.i.i.i.i.i = load i8*, i8** %29, align 8, !alias.scope !161, !noalias !126, !nonnull !85, !noundef !85
  %30 = sub i64 0, %22
  %31 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i.i.i.i.i.i.i, i64 %30
  tail call void @__rust_dealloc(i8* nonnull %31, i64 %26, i64 16) #23, !noalias !162
  br label %_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit

_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E.exit: ; preds = %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit.i.i", %bb4.i.i.i.i, %bb2.i.i.i.i.i.i.i.i.i, %bb2.i.i.i.i.i.i.i.i.i.i.i.i
  %_22.i.i = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16.i.i, align 8, !noalias !126
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
define internal void @_ZN4core3ops8function6FnOnce9call_once17hb6a066d613893e2fE(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  tail call void @llvm.experimental.noalias.scope.decl(metadata !163)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !166)
  %1 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %1), !noalias !169
; call std::sys_common::mutex::MovableMutex::new
  %2 = tail call i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E(), !noalias !169
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %.0..sroa_idx.i.i, align 4, !noalias !169
; invoke std::sync::poison::Flag::new
  %3 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit" unwind label %cleanup1.i.i, !noalias !169

cleanup1.i.i:                                     ; preds = %start
  %4 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #24
          to label %bb5.i.i unwind label %abort.i.i, !noalias !169

abort.i.i:                                        ; preds = %cleanup1.i.i
  %5 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25, !noalias !169
  unreachable

bb5.i.i:                                          ; preds = %cleanup1.i.i
  resume { i8*, i32 } %4

"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE.exit": ; preds = %start
  %6 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %2, i32* %6, align 8, !alias.scope !169
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %3, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !169
  %7 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %0, i64 0, i32 3
  store i64 0, i64* %7, align 8, !alias.scope !169
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %1), !noalias !169
  ret void
}

; core::ops::function::FnOnce::call_once
; Function Attrs: inlinehint nonlazybind uwtable
define internal void @_ZN4core3ops8function6FnOnce9call_once17hd20ed85d13df1445E(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef writeonly sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %0) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i.i.i.i.i.i = alloca %"std::thread::local::AccessError", align 1
  %_2.i.i = alloca %"std::sys_common::mutex::MovableMutex", align 4
  %_2.i = alloca %"std::collections::hash::map::HashMap<i64, ObjectInfo>", align 16
  tail call void @llvm.experimental.noalias.scope.decl(metadata !170)
  %1 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1), !noalias !170
  tail call void @llvm.experimental.noalias.scope.decl(metadata !173)
  %_2.i.i.i.i.i.i.i.i.i.i = load i64, i64* getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 0), align 8, !range !119, !noalias !176, !noundef !85
  %trunc.not.i.i.i.i.i.i.i.i.i.i = icmp eq i64 %_2.i.i.i.i.i.i.i.i.i.i, 0
  br i1 %trunc.not.i.i.i.i.i.i.i.i.i.i, label %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"

_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i: ; preds = %start
; call std::thread::local::fast::Key<T>::try_initialize
  %2 = tail call fastcc noundef align 8 dereferenceable_or_null(16) i64* @"_ZN3std6thread5local4fast12Key$LT$T$GT$14try_initialize17hd4e535fd74b46a6dE"(i64* noalias noundef align 8 dereferenceable_or_null(24) null), !noalias !183
  %3 = icmp eq i64* %2, null
  br i1 %3, label %bb1.i.i.i.i.i.i, label %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"

bb1.i.i.i.i.i.i:                                  ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i
  %4 = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 0, i8* nonnull %4), !noalias !184
  %_6.0.i.i.i.i.i.i = bitcast %"std::thread::local::AccessError"* %e.i.i.i.i.i.i to {}*
; call core::result::unwrap_failed
  call void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [70 x i8] }>* @alloc360 to [0 x i8]*), i64 70, {}* noundef nonnull align 1 %_6.0.i.i.i.i.i.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.3 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc362 to %"core::panic::location::Location"*)) #22, !noalias !184
  unreachable

"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i": ; preds = %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i, %start
  %.0.i.i2.i.i.i.i.i.i = phi i64* [ %2, %_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E.exit.i.i.i.i.i.i ], [ getelementptr inbounds (%"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>", %"std::thread::local::fast::Key<core::cell::Cell<(u64, u64)>>"* @_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit5__KEY17h22b218cd95a9775eE, i64 0, i32 0, i32 0, i32 0, i32 1, i64 0), %start ]
  %5 = bitcast i64* %.0.i.i2.i.i.i.i.i.i to <2 x i64>*
  %6 = load <2 x i64>, <2 x i64>* %5, align 8, !noalias !183
  %7 = extractelement <2 x i64> %6, i64 0
  %8 = add i64 %7, 1
  store i64 %8, i64* %.0.i.i2.i.i.i.i.i.i, align 8, !alias.scope !185, !noalias !183
  %_2.sroa.7.0..sroa_idx.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 3
  %_2.sroa.7.0..sroa_idx1516.i.i.i = bitcast i64* %_2.sroa.7.0..sroa_idx.i.i.i to i8*
  call void @llvm.memset.p0i8.i64(i8* noundef nonnull align 16 dereferenceable(16) %_2.sroa.7.0..sroa_idx1516.i.i.i, i8 0, i64 16, i1 false) #23, !alias.scope !188, !noalias !170
  %9 = bitcast %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i to <2 x i64>*
  store <2 x i64> %6, <2 x i64>* %9, align 16, !alias.scope !188, !noalias !170
  %_2.sroa.5.0..sroa_idx4.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1
  %_2.sroa.5.0..sroa_cast.i.i.i = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %_2.sroa.5.0..sroa_idx4.i.i.i to i64*
  store i64 0, i64* %_2.sroa.5.0..sroa_cast.i.i.i, align 16, !alias.scope !188, !noalias !170
  %_2.sroa.6.0..sroa_idx6.i.i.i = getelementptr inbounds %"std::collections::hash::map::HashMap<i64, ObjectInfo>", %"std::collections::hash::map::HashMap<i64, ObjectInfo>"* %_2.i, i64 0, i32 0, i32 1, i32 1, i32 2
  store i8* getelementptr inbounds (<{ [16 x i8] }>, <{ [16 x i8] }>* @alloc67, i64 0, i32 0, i64 0), i8** %_2.sroa.6.0..sroa_idx6.i.i.i, align 8, !alias.scope !188, !noalias !170
  tail call void @llvm.experimental.noalias.scope.decl(metadata !191)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !194)
  %10 = bitcast %"std::sys_common::mutex::MovableMutex"* %_2.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 4, i8* nonnull %10), !noalias !196
; invoke std::sys_common::mutex::MovableMutex::new
  %11 = invoke i32 @_ZN3std10sys_common5mutex12MovableMutex3new17h415ac39822de5dd2E()
          to label %bb1.i.i unwind label %cleanup.i.i, !noalias !196

cleanup.i.i:                                      ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"
  %12 = landingpad { i8*, i32 }
          cleanup
  br label %bb6.i.i

bb1.i.i:                                          ; preds = %"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE.exit.i"
  %.0..sroa_idx.i.i = getelementptr inbounds %"std::sys_common::mutex::MovableMutex", %"std::sys_common::mutex::MovableMutex"* %_2.i.i, i64 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %.0..sroa_idx.i.i, align 4, !noalias !196
; invoke std::sync::poison::Flag::new
  %13 = invoke i8 @_ZN3std4sync6poison4Flag3new17ha1e695e9415c2058E()
          to label %"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E.exit" unwind label %cleanup1.i.i, !noalias !196

cleanup1.i.i:                                     ; preds = %bb1.i.i
  %14 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sys_common::mutex::MovableMutex>
  invoke fastcc void @"_ZN4core3ptr57drop_in_place$LT$std..sys_common..mutex..MovableMutex$GT$17h9f8779a79873e5ebE"(%"std::sys_common::mutex::MovableMutex"* nonnull %_2.i.i) #24
          to label %bb6.i.i unwind label %abort.i.i, !noalias !196

abort.i.i:                                        ; preds = %cleanup1.i.i
  %15 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25, !noalias !196
  unreachable

bb6.i.i:                                          ; preds = %cleanup1.i.i, %cleanup.i.i
  %.pn.i.i = phi { i8*, i32 } [ %14, %cleanup1.i.i ], [ %12, %cleanup.i.i ]
  call fastcc void bitcast (void (%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)* @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE" to void (%"std::collections::hash::map::HashMap<i64, ObjectInfo>"*)*)(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* nonnull %_2.i) #24, !noalias !197
  resume { i8*, i32 } %.pn.i.i

"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E.exit": ; preds = %bb1.i.i
  %_4.sroa.0.0..sroa_idx26.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 3, i32 0, i32 0
  %_4.sroa.0.0..sroa_idx2627.i.i = bitcast %"hashbrown::map::HashMap<i64, ObjectInfo, std::collections::hash::map::RandomState>"* %_4.sroa.0.0..sroa_idx26.i.i to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(48) %_4.sroa.0.0..sroa_idx2627.i.i, i8* noundef nonnull align 16 dereferenceable(48) %1, i64 48, i1 false), !alias.scope !198
  %16 = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 0, i32 0, i32 0, i32 0
  store i32 %11, i32* %16, align 8, !alias.scope !197, !noalias !194
  %_3.sroa.0.0..sroa_idx.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %0, i64 0, i32 1, i32 0, i32 0
  store i8 %13, i8* %_3.sroa.0.0..sroa_idx.i.i, align 4, !alias.scope !197, !noalias !194
  call void @llvm.lifetime.end.p0i8(i64 4, i8* nonnull %10), !noalias !196
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %1), !noalias !170
  ret void
}

; core::ptr::drop_in_place<core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
; Function Attrs: nounwind nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nocapture readonly %_1) unnamed_addr #8 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %0 = getelementptr inbounds %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_1, i64 0, i32 0, i32 0, i32 1
  tail call void @llvm.experimental.noalias.scope.decl(metadata !199) #23
  %1 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %0 to i64*
  %_2.i.i.i.i.i = load i64, i64* %1, align 8, !alias.scope !202
  %2 = icmp eq i64 %_2.i.i.i.i.i, 0
  br i1 %2, label %"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit", label %bb2.i.i.i.i

bb2.i.i.i.i:                                      ; preds = %start
  tail call void @llvm.experimental.noalias.scope.decl(metadata !205) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !208) #23
  %3 = add i64 %_2.i.i.i.i.i, 1
  %4 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %3, i64 32) #23
  %5 = extractvalue { i64, i1 } %4, 1
  %6 = xor i1 %5, true
  tail call void @llvm.assume(i1 %6) #23
  %7 = extractvalue { i64, i1 } %4, 0
  %_31.i.i.i.i.i.i.i = add i64 %_2.i.i.i.i.i, 17
  %8 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %7, i64 %_31.i.i.i.i.i.i.i) #23
  %9 = extractvalue { i64, i1 } %8, 1
  %10 = xor i1 %9, true
  tail call void @llvm.assume(i1 %10) #23
  %11 = extractvalue { i64, i1 } %8, 0
  %12 = icmp eq i64 %11, 0
  br i1 %12, label %"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit", label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb2.i.i.i.i
  %13 = getelementptr inbounds %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_1, i64 0, i32 0, i32 0, i32 1, i32 1, i32 2
  %_17.i.i.i.i.i.i = load i8*, i8** %13, align 8, !alias.scope !211, !nonnull !85, !noundef !85
  %14 = sub i64 0, %7
  %15 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i.i, i64 %14
  tail call void @__rust_dealloc(i8* nonnull %15, i64 %11, i64 16) #23, !noalias !211
  br label %"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit"

"_ZN4core3ptr95drop_in_place$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$17h248fc0657b0de242E.exit": ; preds = %start, %bb2.i.i.i.i, %bb2.i.i.i.i.i.i.i
  ret void
}

; core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
; Function Attrs: nonlazybind uwtable
define internal fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !212)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !212, !nonnull !85, !align !86, !noundef !85
  %_5.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i = load i8, i8* %_5.i, align 8, !alias.scope !212
  %_5.not.i.i = icmp eq i8 %_5.val.i, 0
  br i1 %_5.not.i.i, label %bb2.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

bb2.i.i:                                          ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !212
  %_1.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i: ; preds = %bb2.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !212
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, label %bb5.i.i

bb5.i.i:                                          ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i
  %_6.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i monotonic, align 4, !noalias !212
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i: ; preds = %bb5.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i, %bb2.i.i, %start
  %_5.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i, i32 0 release, align 4, !noalias !212
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i, label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE.exit"

bb2.i.i.i:                                        ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i
  %_2.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i), !noalias !212
  br label %"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE.exit"

"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i, %bb2.i.i.i
  ret void
}

; core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
; Function Attrs: nonlazybind uwtable
define internal void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nocapture readonly %_1) unnamed_addr #6 {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !215)
  %0 = bitcast { i64*, i8 }* %_1 to %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"**
  %_8.i.i = load %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*, %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"** %0, align 8, !alias.scope !215, !nonnull !85, !align !86, !noundef !85
  %_5.i.i = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %_1, i64 0, i32 1
  %_5.val.i.i = load i8, i8* %_5.i.i, align 8, !alias.scope !215
  %_5.not.i.i.i = icmp eq i8 %_5.val.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %start
  %1 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !215
  %_1.i.i.i.i.i.i = and i64 %1, 9223372036854775807
  %2 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %2, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %3 = tail call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !215
  br i1 %3, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  %_6.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i monotonic, align 4, !noalias !215
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %start
  %_5.i.i.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0, i32 0, i32 0
  %4 = atomicrmw xchg i32* %_5.i.i.i.i.i, i32 0 release, align 4, !noalias !215
  %5 = icmp eq i32 %4, 2
  br i1 %5, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
  %_2.i.i.i = getelementptr inbounds %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_8.i.i, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  tail call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i), !noalias !215
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
  call void @_ZN4core9panicking19assert_failed_inner17h36469c68b6fc10f1E(i8 noundef %kind, {}* noundef nonnull align 1 %_6.0, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), {}* noundef nonnull align 1 %_9.0, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.6 to [3 x i64]*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_12, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) %2) #22
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
  call void @_ZN4core3fmt9Formatter12debug_struct17h65c357ef1edbbc54E(%"core::fmt::builders::DebugStruct"* noalias nocapture noundef nonnull sret(%"core::fmt::builders::DebugStruct") dereferenceable(16) %_4, %"core::fmt::Formatter"* noalias noundef nonnull align 8 dereferenceable(64) %f, [0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [11 x i8] }>* @alloc417 to [0 x i8]*), i64 11)
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
  tail call void @_ZN3std7process5abort17h9abe461bf20ade28E() #22
  unreachable

bb3:                                              ; preds = %start
  %_5.0 = bitcast { i8*, i64 }* %self to {}*
  %3 = insertvalue { {}*, [3 x i64]* } undef, {}* %_5.0, 0
  %4 = insertvalue { {}*, [3 x i64]* } %3, [3 x i64]* bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.7 to [3 x i64]*), 1
  ret { {}*, [3 x i64]* } %4
}

; <std::panicking::begin_panic::PanicPayload<A> as core::panic::BoxMeUp>::take_box
; Function Attrs: nonlazybind uwtable
define internal { {}*, [3 x i64]* } @"_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17h92e001d5e4efd74cE"({ i8*, i64 }* noalias nocapture noundef align 8 dereferenceable(16) %self) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %tmp.sroa.0.0..sroa_idx.i.i.i = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %self, i64 0, i32 0
  %tmp.sroa.0.0.copyload.i.i.i = load i8*, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !218
  %tmp.sroa.4.0..sroa_idx3.i.i.i = getelementptr inbounds { i8*, i64 }, { i8*, i64 }* %self, i64 0, i32 1
  %tmp.sroa.4.0.copyload.i.i.i = load i64, i64* %tmp.sroa.4.0..sroa_idx3.i.i.i, align 8, !alias.scope !218
  store i8* null, i8** %tmp.sroa.0.0..sroa_idx.i.i.i, align 8, !alias.scope !218
  %0 = icmp eq i8* %tmp.sroa.0.0.copyload.i.i.i, null
  br i1 %0, label %bb2, label %bb4

bb2:                                              ; preds = %start
; call std::process::abort
  tail call void @_ZN3std7process5abort17h9abe461bf20ade28E() #22
  unreachable

bb4:                                              ; preds = %start
  %1 = tail call align 8 dereferenceable_or_null(16) i8* @__rust_alloc(i64 16, i64 8) #23, !noalias !223
  %2 = icmp eq i8* %1, null
  br i1 %2, label %bb3.i.i, label %"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit"

bb3.i.i:                                          ; preds = %bb4
; call alloc::alloc::handle_alloc_error
  tail call void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64 16, i64 noundef 8) #22, !noalias !223
  unreachable

"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE.exit": ; preds = %bb4
  %3 = bitcast i8* %1 to i8**
  store i8* %tmp.sroa.0.0.copyload.i.i.i, i8** %3, align 8, !noalias !223
  %4 = getelementptr inbounds i8, i8* %1, i64 8
  %5 = bitcast i8* %4 to i64*
  store i64 %tmp.sroa.4.0.copyload.i.i.i, i64* %5, align 8, !noalias !223
  %_13.0.cast = bitcast i8* %1 to {}*
  %6 = insertvalue { {}*, [3 x i64]* } undef, {}* %_13.0.cast, 0
  %7 = insertvalue { {}*, [3 x i64]* } %6, [3 x i64]* bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.7 to [3 x i64]*), 1
  ret { {}*, [3 x i64]* } %7
}

; hashbrown::raw::RawTable<T,A>::reserve_rehash
; Function Attrs: cold noinline nonlazybind uwtable
define internal fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h320d5dd485a72968E"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias nocapture noundef align 8 dereferenceable(32) %self, i64* noalias noundef readonly align 8 dereferenceable(16) %0) unnamed_addr #11 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  tail call void @llvm.experimental.noalias.scope.decl(metadata !226)
  %1 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 4
  %_9.i = load i64, i64* %1, align 8, !alias.scope !226
  %2 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %_9.i, i64 1) #23
  %3 = extractvalue { i64, i1 } %2, 0
  %4 = extractvalue { i64, i1 } %2, 1
  br i1 %4, label %bb2.i, label %bb4.i

bb2.i:                                            ; preds = %start
; call hashbrown::raw::Fallibility::capacity_overflow
  %5 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !226
  %_13.0.i = extractvalue { i64, i64 } %5, 0
  %_13.1.i = extractvalue { i64, i64 } %5, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb4.i:                                            ; preds = %start
  %6 = bitcast %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self to i64*
  %_16.i = load i64, i64* %6, align 8, !alias.scope !226
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
  tail call void @llvm.experimental.noalias.scope.decl(metadata !229)
  %_5.i.i.i.i.i = icmp ult i64 %.0.sroa.speculated.i.i.i, 8
  br i1 %_5.i.i.i.i.i, label %bb1.i.i.i.i.i, label %bb5.i.i.i.i.i

bb5.i.i.i.i.i:                                    ; preds = %bb9.i
  %9 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %.0.sroa.speculated.i.i.i, i64 8) #23
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
  %12 = tail call i64 @llvm.ctlz.i64(i64 %p.i.i.i.i.i.i.i, i1 true) #23, !range !232
  %13 = lshr i64 -1, %12
  %phi.bo.i.i.i.i.i.i = add i64 %13, 1
  br label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb8.i.i.i.i.i, %bb1.i.i.i.i.i
  %.sroa.4.0.i.ph.i.i.i.i = phi i64 [ %phi.bo.i.i.i.i.i.i, %bb8.i.i.i.i.i ], [ %..i.i.i.i.i, %bb1.i.i.i.i.i ]
  %14 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %.sroa.4.0.i.ph.i.i.i.i, i64 32) #23
  %15 = extractvalue { i64, i1 } %14, 1
  br i1 %15, label %bb2.i.i.i.i.i, label %bb9.i.i.i.i.i.i

bb9.i.i.i.i.i.i:                                  ; preds = %bb7.i.i.i.i
  %16 = extractvalue { i64, i1 } %14, 0
  %_31.i.i.i.i.i.i = add nuw nsw i64 %.sroa.4.0.i.ph.i.i.i.i, 16
  %17 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %16, i64 %_31.i.i.i.i.i.i) #23
  %18 = extractvalue { i64, i1 } %17, 1
  br i1 %18, label %bb2.i.i.i.i.i, label %bb4.i.i.i.i.i

bb2.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i, %bb7.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %19 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !233
  br label %bb5.i.i

bb4.i.i.i.i.i:                                    ; preds = %bb9.i.i.i.i.i.i
  %20 = extractvalue { i64, i1 } %17, 0
  %21 = icmp eq i64 %20, 0
  br i1 %21, label %bb13.i.i.i.i, label %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i

_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i: ; preds = %bb4.i.i.i.i.i
  %22 = tail call align 16 i8* @__rust_alloc(i64 %20, i64 16) #23, !noalias !233
  %23 = icmp eq i8* %22, null
  br i1 %23, label %bb15.i.i.i.i.i, label %bb13.i.i.i.i

bb15.i.i.i.i.i:                                   ; preds = %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
; call hashbrown::raw::Fallibility::alloc_err
  %24 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility9alloc_err17h3f1a17e1376e6326E(i1 noundef zeroext true, i64 %20, i64 noundef 16), !noalias !233
  br label %bb5.i.i

bb9.i.i.i.i:                                      ; preds = %bb5.i.i.i.i.i
; call hashbrown::raw::Fallibility::capacity_overflow
  %25 = tail call { i64, i64 } @_ZN9hashbrown3raw11Fallibility17capacity_overflow17ha7db677ca228cb68E(i1 noundef zeroext true), !noalias !240
  br label %bb5.i.i

bb13.i.i.i.i:                                     ; preds = %bb4.i.i.i.i.i, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i
  %.sroa.0.0.i.i.i.i.i.i.i.i3 = phi i8* [ %22, %_ZN9hashbrown3raw5alloc5inner8do_alloc17h9180c3d940289751E.exit.i.i.i.i.i ], [ inttoptr (i64 16 to i8*), %bb4.i.i.i.i.i ]
  %26 = getelementptr inbounds i8, i8* %.sroa.0.0.i.i.i.i.i.i.i.i3, i64 %16
  %_42.i.i.i.i.i = add nsw i64 %.sroa.4.0.i.ph.i.i.i.i, -1
  %_2.i.i10.i.i.i.i = icmp ult i64 %_42.i.i.i.i.i, 8
  %_4.i.i.i.i.i.i = lshr i64 %.sroa.4.0.i.ph.i.i.i.i, 3
  %27 = mul nuw nsw i64 %_4.i.i.i.i.i.i, 7
  %.0.i.i.i.i.i.i = select i1 %_2.i.i10.i.i.i.i, i64 %_42.i.i.i.i.i, i64 %27
  tail call void @llvm.memset.p0i8.i64(i8* nonnull align 16 %26, i8 -1, i64 %_31.i.i.i.i.i.i, i1 false) #23, !noalias !243
  %28 = sub i64 %.0.i.i.i.i.i.i, %_9.i
  %.not.i.i = icmp eq i64 %_5.i.i, 0
  %29 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %a.i.i.sroa.4.0.copyload.pre.i.i = load i8*, i8** %29, align 8, !alias.scope !244
  br i1 %.not.i.i, label %bb26.thread.i.i, label %bb15.lr.ph.i.i

bb5.i.i:                                          ; preds = %bb9.i.i.i.i, %bb15.i.i.i.i.i, %bb2.i.i.i.i.i
  %.pn.i.pn.i.i.i = phi { i64, i64 } [ %25, %bb9.i.i.i.i ], [ %24, %bb15.i.i.i.i.i ], [ %19, %bb2.i.i.i.i.i ]
  %_7.sroa.7.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 0
  %_7.sroa.13.0.i.i.i = extractvalue { i64, i64 } %.pn.i.pn.i.i.i, 1
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb26.thread.i.i:                                  ; preds = %bb13.i.i.i.i
  %30 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !245
  store i8* %26, i8** %29, align 8, !alias.scope !245
  store i64 %28, i64* %30, align 8, !alias.scope !245
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
  %37 = tail call i64 @llvm.fshl.i64(i64 %34, i64 %34, i64 13) #23
  %38 = xor i64 %36, %37
  %39 = tail call i64 @llvm.fshl.i64(i64 %36, i64 %36, i64 32) #23
  %40 = tail call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 17) #23
  br label %bb15.i.i

bb15.i.i:                                         ; preds = %bb9.backedge.i.i, %bb15.lr.ph.i.i
  %iter.sroa.0.0100.i.i = phi i64 [ 0, %bb15.lr.ph.i.i ], [ %41, %bb9.backedge.i.i ]
  %41 = add nuw i64 %iter.sroa.0.0100.i.i, 1
  %42 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %iter.sroa.0.0100.i.i
  %_29.i.i = load i8, i8* %42, align 1, !noalias !244
  %43 = icmp sgt i8 %_29.i.i, -1
  br i1 %43, label %bb18.i.i, label %bb9.backedge.i.i

bb9.backedge.i.i:                                 ; preds = %bb22.i.i, %bb15.i.i
  %exitcond.not.i.i = icmp eq i64 %iter.sroa.0.0100.i.i, %_16.i
  br i1 %exitcond.not.i.i, label %bb26.i.i, label %bb15.i.i

bb18.i.i:                                         ; preds = %bb15.i.i
  %44 = sub i64 0, %iter.sroa.0.0100.i.i
  %45 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %table.idx.val4.i.cast.i.i, i64 %44, i32 0
  %46 = getelementptr inbounds i64, i64* %45, i64 -4
  %_7.idx.val.i.i.i = load i64, i64* %46, align 8, !alias.scope !252, !noalias !255
  %47 = xor i64 %_7.idx.val.i.i.i, %_6.idx1.val.i.i.i.i
  %48 = xor i64 %47, 8387220255154660723
  %49 = add i64 %48, %35
  %50 = tail call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 16) #23
  %51 = xor i64 %50, %49
  %52 = add i64 %51, %39
  %53 = tail call i64 @llvm.fshl.i64(i64 %51, i64 %51, i64 21) #23
  %54 = xor i64 %53, %52
  %55 = add i64 %38, %49
  %56 = xor i64 %55, %40
  %57 = tail call i64 @llvm.fshl.i64(i64 %55, i64 %55, i64 32) #23
  %58 = xor i64 %52, %_7.idx.val.i.i.i
  %59 = xor i64 %54, 576460752303423488
  %60 = add i64 %58, %56
  %61 = tail call i64 @llvm.fshl.i64(i64 %56, i64 %56, i64 13) #23
  %62 = xor i64 %60, %61
  %63 = tail call i64 @llvm.fshl.i64(i64 %60, i64 %60, i64 32) #23
  %64 = add i64 %59, %57
  %65 = tail call i64 @llvm.fshl.i64(i64 %54, i64 %59, i64 16) #23
  %66 = xor i64 %65, %64
  %67 = add i64 %66, %63
  %68 = tail call i64 @llvm.fshl.i64(i64 %66, i64 %66, i64 21) #23
  %69 = xor i64 %68, %67
  %70 = add i64 %64, %62
  %71 = tail call i64 @llvm.fshl.i64(i64 %62, i64 %62, i64 17) #23
  %72 = xor i64 %70, %71
  %73 = tail call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 32) #23
  %74 = xor i64 %67, 576460752303423488
  %75 = xor i64 %73, 255
  %76 = add i64 %74, %72
  %77 = tail call i64 @llvm.fshl.i64(i64 %72, i64 %72, i64 13) #23
  %78 = xor i64 %76, %77
  %79 = tail call i64 @llvm.fshl.i64(i64 %76, i64 %76, i64 32) #23
  %80 = add i64 %69, %75
  %81 = tail call i64 @llvm.fshl.i64(i64 %69, i64 %69, i64 16) #23
  %82 = xor i64 %81, %80
  %83 = add i64 %82, %79
  %84 = tail call i64 @llvm.fshl.i64(i64 %82, i64 %82, i64 21) #23
  %85 = xor i64 %84, %83
  %86 = add i64 %78, %80
  %87 = tail call i64 @llvm.fshl.i64(i64 %78, i64 %78, i64 17) #23
  %88 = xor i64 %86, %87
  %89 = tail call i64 @llvm.fshl.i64(i64 %86, i64 %86, i64 32) #23
  %90 = add i64 %88, %83
  %91 = tail call i64 @llvm.fshl.i64(i64 %88, i64 %88, i64 13) #23
  %92 = xor i64 %91, %90
  %93 = tail call i64 @llvm.fshl.i64(i64 %90, i64 %90, i64 32) #23
  %94 = add i64 %85, %89
  %95 = tail call i64 @llvm.fshl.i64(i64 %85, i64 %85, i64 16) #23
  %96 = xor i64 %95, %94
  %97 = add i64 %96, %93
  %98 = tail call i64 @llvm.fshl.i64(i64 %96, i64 %96, i64 21) #23
  %99 = xor i64 %98, %97
  %100 = add i64 %92, %94
  %101 = tail call i64 @llvm.fshl.i64(i64 %92, i64 %92, i64 17) #23
  %102 = xor i64 %101, %100
  %103 = tail call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 32) #23
  %104 = add i64 %102, %97
  %105 = tail call i64 @llvm.fshl.i64(i64 %102, i64 %102, i64 13) #23
  %106 = xor i64 %105, %104
  %107 = add i64 %99, %103
  %108 = tail call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 16) #23
  %109 = xor i64 %108, %107
  %110 = tail call i64 @llvm.fshl.i64(i64 %109, i64 %109, i64 21) #23
  %111 = add i64 %106, %107
  %112 = tail call i64 @llvm.fshl.i64(i64 %106, i64 %106, i64 17) #23
  %113 = tail call i64 @llvm.fshl.i64(i64 %111, i64 %111, i64 32) #23
  %_17.i.i.i.i.i.i.i.i.i = xor i64 %111, %110
  %114 = xor i64 %_17.i.i.i.i.i.i.i.i.i, %112
  %115 = xor i64 %114, %113
  %_3.i.i.i.i.i = and i64 %115, %_42.i.i.i.i.i
  %116 = getelementptr inbounds i8, i8* %26, i64 %_3.i.i.i.i.i
  %117 = bitcast i8* %116 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %117, align 1, !noalias !259
  %118 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %119 = bitcast <16 x i1> %118 to i16
  %.not23.i.i.i.i = icmp eq i16 %119, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb7.i.i8.i.i:                                     ; preds = %bb17.i.i.i.i, %bb18.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i, %bb18.i.i ], [ %125, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %119, %bb18.i.i ], [ %129, %bb17.i.i.i.i ]
  %120 = tail call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #23, !range !27
  %_2.i.i.i.i.i.i = zext i16 %120 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_42.i.i.i.i.i
  %121 = getelementptr inbounds i8, i8* %26, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %121, align 1, !noalias !266
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
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %127, align 1, !noalias !259
  %128 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %129 = bitcast <16 x i1> %128 to i16
  %.not.i.i.i.i = icmp eq i16 %129, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i8.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i8.i.i
  %130 = load <16 x i8>, <16 x i8>* %31, align 16, !noalias !267
  %131 = icmp slt <16 x i8> %130, zeroinitializer
  %132 = bitcast <16 x i1> %131 to i16
  %133 = tail call i16 @llvm.cttz.i16(i16 %132, i1 true) #23, !range !27
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
  store i8 %134, i8* %136, align 1, !noalias !272
  %137 = getelementptr inbounds i8, i8* %26, i64 %index2.i.i.i.i.i
  store i8 %134, i8* %137, align 1, !noalias !272
  %_12.neg.i.i.i = xor i64 %iter.sroa.0.0100.i.i, -1
  %_11.neg.i.i.i = shl i64 %_12.neg.i.i.i, 5
  %138 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %_11.neg.i.i.i
  %_12.neg.i10.i.i = xor i64 %.0.i.i.i.i, -1
  %_11.neg.i11.i.i = shl i64 %_12.neg.i10.i.i, 5
  %139 = getelementptr inbounds i8, i8* %26, i64 %_11.neg.i11.i.i
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 16 dereferenceable(32) %139, i8* noundef nonnull align 1 dereferenceable(32) %138, i64 32, i1 false) #23, !noalias !244
  br label %bb9.backedge.i.i

bb26.i.i:                                         ; preds = %bb9.backedge.i.i
  %140 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  store i64 %_42.i.i.i.i.i, i64* %6, align 8, !alias.scope !277
  store i8* %26, i8** %29, align 8, !alias.scope !277
  store i64 %28, i64* %140, align 8, !alias.scope !277
  %141 = icmp eq i64 %_16.i, 0
  br i1 %141, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit", label %bb2.i.i.i14.i.i

bb2.i.i.i14.i.i:                                  ; preds = %bb26.i.i, %bb26.thread.i.i
  %142 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %_5.i.i, i64 32) #23
  %143 = extractvalue { i64, i1 } %142, 1
  %144 = xor i1 %143, true
  tail call void @llvm.assume(i1 %144) #23
  %145 = extractvalue { i64, i1 } %142, 0
  %_31.i.i.i.i.i.i.i = add i64 %_16.i, 17
  %146 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %145, i64 %_31.i.i.i.i.i.i.i) #23
  %147 = extractvalue { i64, i1 } %146, 1
  %148 = xor i1 %147, true
  tail call void @llvm.assume(i1 %148) #23
  %149 = extractvalue { i64, i1 } %146, 0
  %150 = icmp eq i64 %149, 0
  br i1 %150, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit", label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb2.i.i.i14.i.i
  %151 = icmp ne i8* %a.i.i.sroa.4.0.copyload.pre.i.i, null
  tail call void @llvm.assume(i1 %151)
  %152 = sub i64 0, %145
  %153 = getelementptr inbounds i8, i8* %a.i.i.sroa.4.0.copyload.pre.i.i, i64 %152
  tail call void @__rust_dealloc(i8* nonnull %153, i64 %149, i64 16) #23, !noalias !280
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E.exit"

bb7.i:                                            ; preds = %bb4.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !287)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !290)
  %154 = getelementptr %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 2
  %self.idx12.val.i.i.i = load i8*, i8** %154, align 8, !alias.scope !293
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
  %157 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %iter.sroa.0.0.i.i.i, i64 15) #23
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
  %164 = load <16 x i8>, <16 x i8>* %163, align 16, !noalias !294
  %.lobit.i.i.i.i = ashr <16 x i8> %164, <i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7, i8 7>
  %165 = bitcast <16 x i8> %.lobit.i.i.i.i to <2 x i64>
  %166 = or <2 x i64> %165, <i64 -9187201950435737472, i64 -9187201950435737472>
  store <2 x i64> %166, <2 x i64>* %162, align 16, !noalias !299
  br label %bb4.i.i.i

bb5.thread.i.i:                                   ; preds = %bb8.i.i.i
  %167 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_5.i.i
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(16) %167, i8* noundef nonnull align 1 dereferenceable(16) %self.idx12.val.i.i.i, i64 16, i1 false) #23, !noalias !293
  br label %bb12.lr.ph.i.i

bb5.i2.i:                                         ; preds = %bb8.i.i.i
  %168 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 16
  tail call void @llvm.memmove.p0i8.p0i8.i64(i8* nonnull align 1 %168, i8* align 1 %self.idx12.val.i.i.i, i64 %_5.i.i, i1 false) #23, !noalias !293
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
  %174 = tail call i64 @llvm.fshl.i64(i64 %171, i64 %171, i64 13) #23
  %175 = xor i64 %173, %174
  %176 = tail call i64 @llvm.fshl.i64(i64 %173, i64 %173, i64 32) #23
  %177 = tail call i64 @llvm.fshl.i64(i64 %175, i64 %175, i64 17) #23
  %178 = bitcast i8* %self.idx12.val.i.i.i to <16 x i8>*
  %179 = bitcast i8* %self.idx12.val.i.i.i to { i64, %ObjectInfo }*
  br label %bb12.i.i

bb12.i.i:                                         ; preds = %bb40.i.i, %bb12.lr.ph.i.i
  %table.idx.val4.i41.i.i = phi { i64, %ObjectInfo }* [ %155, %bb12.lr.ph.i.i ], [ %table.idx.val4.i42.i.i, %bb40.i.i ]
  %iter.sroa.0.028.i.i = phi i64 [ 0, %bb12.lr.ph.i.i ], [ %180, %bb40.i.i ]
  %180 = add nuw i64 %iter.sroa.0.028.i.i, 1
  %181 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.028.i.i
  %_23.i.i = load i8, i8* %181, align 1, !noalias !302
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
  %_7.idx.val.i.i7.i = load i64, i64* %190, align 8, !alias.scope !303, !noalias !306
  %191 = xor i64 %_7.idx.val.i.i7.i, %_6.idx1.val.i.i.i12.i
  %192 = xor i64 %191, 8387220255154660723
  %193 = add i64 %192, %172
  %194 = tail call i64 @llvm.fshl.i64(i64 %192, i64 %192, i64 16) #23
  %195 = xor i64 %194, %193
  %196 = add i64 %195, %176
  %197 = tail call i64 @llvm.fshl.i64(i64 %195, i64 %195, i64 21) #23
  %198 = xor i64 %197, %196
  %199 = add i64 %175, %193
  %200 = xor i64 %199, %177
  %201 = tail call i64 @llvm.fshl.i64(i64 %199, i64 %199, i64 32) #23
  %202 = xor i64 %196, %_7.idx.val.i.i7.i
  %203 = xor i64 %198, 576460752303423488
  %204 = add i64 %202, %200
  %205 = tail call i64 @llvm.fshl.i64(i64 %200, i64 %200, i64 13) #23
  %206 = xor i64 %204, %205
  %207 = tail call i64 @llvm.fshl.i64(i64 %204, i64 %204, i64 32) #23
  %208 = add i64 %203, %201
  %209 = tail call i64 @llvm.fshl.i64(i64 %198, i64 %203, i64 16) #23
  %210 = xor i64 %209, %208
  %211 = add i64 %210, %207
  %212 = tail call i64 @llvm.fshl.i64(i64 %210, i64 %210, i64 21) #23
  %213 = xor i64 %212, %211
  %214 = add i64 %208, %206
  %215 = tail call i64 @llvm.fshl.i64(i64 %206, i64 %206, i64 17) #23
  %216 = xor i64 %214, %215
  %217 = tail call i64 @llvm.fshl.i64(i64 %214, i64 %214, i64 32) #23
  %218 = xor i64 %211, 576460752303423488
  %219 = xor i64 %217, 255
  %220 = add i64 %218, %216
  %221 = tail call i64 @llvm.fshl.i64(i64 %216, i64 %216, i64 13) #23
  %222 = xor i64 %220, %221
  %223 = tail call i64 @llvm.fshl.i64(i64 %220, i64 %220, i64 32) #23
  %224 = add i64 %213, %219
  %225 = tail call i64 @llvm.fshl.i64(i64 %213, i64 %213, i64 16) #23
  %226 = xor i64 %225, %224
  %227 = add i64 %226, %223
  %228 = tail call i64 @llvm.fshl.i64(i64 %226, i64 %226, i64 21) #23
  %229 = xor i64 %228, %227
  %230 = add i64 %222, %224
  %231 = tail call i64 @llvm.fshl.i64(i64 %222, i64 %222, i64 17) #23
  %232 = xor i64 %230, %231
  %233 = tail call i64 @llvm.fshl.i64(i64 %230, i64 %230, i64 32) #23
  %234 = add i64 %232, %227
  %235 = tail call i64 @llvm.fshl.i64(i64 %232, i64 %232, i64 13) #23
  %236 = xor i64 %235, %234
  %237 = tail call i64 @llvm.fshl.i64(i64 %234, i64 %234, i64 32) #23
  %238 = add i64 %229, %233
  %239 = tail call i64 @llvm.fshl.i64(i64 %229, i64 %229, i64 16) #23
  %240 = xor i64 %239, %238
  %241 = add i64 %240, %237
  %242 = tail call i64 @llvm.fshl.i64(i64 %240, i64 %240, i64 21) #23
  %243 = xor i64 %242, %241
  %244 = add i64 %236, %238
  %245 = tail call i64 @llvm.fshl.i64(i64 %236, i64 %236, i64 17) #23
  %246 = xor i64 %245, %244
  %247 = tail call i64 @llvm.fshl.i64(i64 %244, i64 %244, i64 32) #23
  %248 = add i64 %246, %241
  %249 = tail call i64 @llvm.fshl.i64(i64 %246, i64 %246, i64 13) #23
  %250 = xor i64 %249, %248
  %251 = add i64 %243, %247
  %252 = tail call i64 @llvm.fshl.i64(i64 %243, i64 %243, i64 16) #23
  %253 = xor i64 %252, %251
  %254 = tail call i64 @llvm.fshl.i64(i64 %253, i64 %253, i64 21) #23
  %255 = add i64 %250, %251
  %256 = tail call i64 @llvm.fshl.i64(i64 %250, i64 %250, i64 17) #23
  %257 = tail call i64 @llvm.fshl.i64(i64 %255, i64 %255, i64 32) #23
  %_17.i.i.i.i.i.i.i.i13.i = xor i64 %255, %254
  %258 = xor i64 %_17.i.i.i.i.i.i.i.i13.i, %256
  %259 = xor i64 %258, %257
  %_3.i.i3.i.i = and i64 %259, %_16.i
  %260 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %_3.i.i3.i.i
  %261 = bitcast i8* %260 to <16 x i8>*
  %.0.copyload.i2122.i.i.i = load <16 x i8>, <16 x i8>* %261, align 1, !noalias !310
  %262 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i, zeroinitializer
  %263 = bitcast <16 x i1> %262 to i16
  %.not23.i.i.i = icmp eq i16 %263, 0
  br i1 %.not23.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb7.i.i.i:                                        ; preds = %bb17.i.i.i, %bb19.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i = phi i64 [ %_3.i.i3.i.i, %bb19.i.i ], [ %269, %bb17.i.i.i ]
  %.lcssa.i.i.i = phi i16 [ %263, %bb19.i.i ], [ %273, %bb17.i.i.i ]
  %264 = tail call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i, i1 true) #23, !range !27
  %_2.i.i.i.i14.i = zext i16 %264 to i64
  %_17.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i, %_2.i.i.i.i14.i
  %result.i.i.i = and i64 %_17.i.i.i, %_16.i
  %265 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %result.i.i.i
  %_23.i.i.i = load i8, i8* %265, align 1, !noalias !315
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
  %.0.copyload.i21.i.i.i = load <16 x i8>, <16 x i8>* %271, align 1, !noalias !310
  %272 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i, zeroinitializer
  %273 = bitcast <16 x i1> %272 to i16
  %.not.i.i.i = icmp eq i16 %273, 0
  br i1 %.not.i.i.i, label %bb17.i.i.i, label %bb7.i.i.i

bb11.i.i.i:                                       ; preds = %bb7.i.i.i
  %274 = load <16 x i8>, <16 x i8>* %178, align 16, !noalias !316
  %275 = icmp slt <16 x i8> %274, zeroinitializer
  %276 = bitcast <16 x i1> %275 to i16
  %277 = tail call i16 @llvm.cttz.i16(i16 %276, i1 true) #23, !range !27
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
  store i8 %282, i8* %284, align 1, !noalias !321
  %285 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i
  store i8 %282, i8* %285, align 1, !noalias !321
  br label %bb40.i.i

bb31.i.i:                                         ; preds = %bb24.i.i
  %286 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %.0.i.i.i
  %prev_ctrl.i.i.i = load i8, i8* %286, align 1, !noalias !326
  %top7.i.i.i.i15.i = lshr i64 %259, 57
  %287 = trunc i64 %top7.i.i.i.i15.i to i8
  %288 = add i64 %.0.i.i.i, -16
  %_5.i.i.i.i16.i = and i64 %288, %_16.i
  %index2.i.i.i.i17.i = add i64 %_5.i.i.i.i16.i, 16
  store i8 %287, i8* %286, align 1, !noalias !329
  %289 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i.i17.i
  store i8 %287, i8* %289, align 1, !noalias !329
  %_73.i.i = icmp eq i8 %prev_ctrl.i.i.i, -1
  br i1 %_73.i.i, label %bb34.i.i, label %bb2.i.i10.i.i.preheader

bb2.i.i10.i.i.preheader:                          ; preds = %bb31.i.i
  %290 = load <16 x i8>, <16 x i8>* %185, align 1, !alias.scope !334, !noalias !302
  %291 = bitcast i8* %278 to <16 x i8>*
  %292 = load <16 x i8>, <16 x i8>* %291, align 1, !alias.scope !352, !noalias !302
  store <16 x i8> %292, <16 x i8>* %186, align 1, !alias.scope !334, !noalias !302
  %293 = bitcast i8* %278 to <16 x i8>*
  store <16 x i8> %290, <16 x i8>* %293, align 1, !alias.scope !352, !noalias !302
  %294 = getelementptr inbounds i8, i8* %278, i64 16
  %295 = load <16 x i8>, <16 x i8>* %187, align 1, !alias.scope !369, !noalias !302
  %296 = bitcast i8* %294 to <16 x i8>*
  %297 = load <16 x i8>, <16 x i8>* %296, align 1, !alias.scope !386, !noalias !302
  store <16 x i8> %297, <16 x i8>* %188, align 1, !alias.scope !369, !noalias !302
  %298 = bitcast i8* %294 to <16 x i8>*
  store <16 x i8> %295, <16 x i8>* %298, align 1, !alias.scope !386, !noalias !302
  br label %bb19.i.i

bb34.i.i:                                         ; preds = %bb31.i.i
  %299 = add i64 %iter.sroa.0.028.i.i, -16
  %_5.i.i.i = and i64 %299, %_16.i
  %index2.i.i.i = add i64 %_5.i.i.i, 16
  %300 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %iter.sroa.0.028.i.i
  store i8 -1, i8* %300, align 1, !noalias !403
  %301 = getelementptr inbounds i8, i8* %self.idx12.val.i.i.i, i64 %index2.i.i.i
  store i8 -1, i8* %301, align 1, !noalias !403
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 1 dereferenceable(32) %278, i8* noundef nonnull align 1 dereferenceable(32) %182, i64 32, i1 false) #23, !noalias !302
  br label %bb40.i.i

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E.exit.thread.i": ; preds = %bb40.i.i, %bb5.i2.i
  %302 = phi i64 [ 0, %bb5.i2.i ], [ %.0.i.i, %bb40.i.i ]
  %303 = getelementptr inbounds %"hashbrown::raw::RawTable<(i64, ObjectInfo)>", %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* %self, i64 0, i32 1, i32 3
  %304 = sub i64 %302, %_9.i
  store i64 %304, i64* %303, align 8, !alias.scope !302
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
  call void @_ZN9once_cell3imp18initialize_or_wait17h9b3310b1603d0203E(%"core::sync::atomic::AtomicUsize"* noundef align 8 dereferenceable(8) bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to %"core::sync::atomic::AtomicUsize"*), i8* noundef nonnull align 1 %2, i8* bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.a to i8*))
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
  call void @_ZN9once_cell3imp18initialize_or_wait17h9b3310b1603d0203E(%"core::sync::atomic::AtomicUsize"* noundef align 8 dereferenceable(8) bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"core::sync::atomic::AtomicUsize"*), i8* noundef nonnull align 1 %2, i8* bitcast (<{ i8*, [16 x i8], i8*, i8* }>* @vtable.b to i8*))
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
  %2 = load i64, i64* %1, align 8, !alias.scope !406
  store i64* null, i64** %_15, align 8, !alias.scope !406
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #23
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<i64>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !413)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !416)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %_8.i.i, align 8, !alias.scope !419, !noalias !420, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !423, !noalias !426
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !423, !noalias !426
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #22, !noalias !426
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<i64>"*)*
  call void %7(%"std::sync::mutex::Mutex<i64>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<i64>") dereferenceable(16) %_5.sroa.0), !noalias !413
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
define internal noundef zeroext i1 @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E"(%"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* noalias nocapture noundef readonly align 8 dereferenceable(24) %_1) unnamed_addr #7 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %_5.sroa.0 = alloca %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>", align 8
  %0 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 0
  %_15 = load i64**, i64*** %0, align 8, !nonnull !85, !align !86, !noundef !85
  %1 = bitcast i64** %_15 to i64*
  %2 = load i64, i64* %1, align 8, !alias.scope !427
  store i64* null, i64** %_15, align 8, !alias.scope !427
  %3 = icmp ne i64 %2, 0
  tail call void @llvm.assume(i1 %3) #23
  %_5.sroa.0.0.sroa_cast20 = bitcast %"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* %_5.sroa.0 to i8*
  call void @llvm.lifetime.start.p0i8(i64 56, i8* nonnull %_5.sroa.0.0.sroa_cast20)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !434)
  tail call void @llvm.experimental.noalias.scope.decl(metadata !437)
  %_8.i.i = inttoptr i64 %2 to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**
  %_9.i.i = load %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_8.i.i, align 8, !alias.scope !440, !noalias !441, !nonnull !85, !align !86, !noundef !85
  %_3.i.i = getelementptr inbounds %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_9.i.i, i64 0, i32 1
  %4 = bitcast i64** %_3.i.i to i64*
  %5 = load i64, i64* %4, align 8, !alias.scope !444, !noalias !447
  store i64* null, i64** %_3.i.i, align 8, !alias.scope !444, !noalias !447
  %6 = icmp eq i64 %5, 0
  br i1 %6, label %bb2.i.i, label %"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit"

bb2.i.i:                                          ; preds = %start
; call std::panicking::begin_panic
  tail call fastcc void @_ZN3std9panicking11begin_panic17h012aeb35123007d8E() #22, !noalias !447
  unreachable

"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE.exit": ; preds = %start
  %7 = inttoptr i64 %5 to void (%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"*)*
  call void %7(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* noalias nocapture noundef nonnull sret(%"std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>") dereferenceable(56) %_5.sroa.0), !noalias !434
  %8 = getelementptr inbounds %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]", %"[closure@once_cell::imp::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::initialize<[closure@once_cell::sync::OnceCell<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::get_or_init<[closure@once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>::force::{closure#0}]>::{closure#0}], once_cell::sync::OnceCell<T>::get_or_init::Void>::{closure#0}]"* %_1, i64 0, i32 1
  %_16 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"**, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*** %8, align 8, !nonnull !85, !align !86, !noundef !85
  %_17 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %9 = getelementptr %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 0
  %_2.i16 = load i64, i64* %9, align 8, !range !119, !noundef !85
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
  tail call fastcc void @"_ZN4core3ptr125drop_in_place$LT$core..cell..UnsafeCell$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3e39694f7b50816dE"(%"core::cell::UnsafeCell<std::collections::hash::map::HashMap<i64, ObjectInfo>>"* nonnull %15) #24
  %_20 = load %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %_16, align 8
  %_10.sroa.0.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20, i64 0, i32 0
  store i64 1, i64* %_10.sroa.0.0..sroa_idx, align 8
  %_10.sroa.5.0..sroa_idx = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_20, i64 0, i32 1
  %_10.sroa.5.0..sroa_cast = bitcast [7 x i64]* %_10.sroa.5.0..sroa_idx to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* noundef nonnull align 8 dereferenceable(56) %_10.sroa.5.0..sroa_cast, i8* noundef nonnull align 8 dereferenceable(56) %_5.sroa.0.0.sroa_cast20, i64 56, i1 false)
  resume { i8*, i32 } %13

bb4.i.i:                                          ; preds = %bb2.i
  %16 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 3
  tail call void @llvm.experimental.noalias.scope.decl(metadata !448) #23
  %_2.i.i.i.i.i.i.i.i = load i64, i64* %16, align 8, !alias.scope !451
  %17 = icmp eq i64 %_2.i.i.i.i.i.i.i.i, 0
  br i1 %17, label %bb9, label %bb2.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i:                                ; preds = %bb4.i.i
  tail call void @llvm.experimental.noalias.scope.decl(metadata !454) #23
  tail call void @llvm.experimental.noalias.scope.decl(metadata !457) #23
  %18 = add i64 %_2.i.i.i.i.i.i.i.i, 1
  %19 = tail call { i64, i1 } @llvm.umul.with.overflow.i64(i64 %18, i64 32) #23
  %20 = extractvalue { i64, i1 } %19, 1
  %21 = xor i1 %20, true
  tail call void @llvm.assume(i1 %21) #23
  %22 = extractvalue { i64, i1 } %19, 0
  %_31.i.i.i.i.i.i.i.i.i.i = add i64 %_2.i.i.i.i.i.i.i.i, 17
  %23 = tail call { i64, i1 } @llvm.uadd.with.overflow.i64(i64 %22, i64 %_31.i.i.i.i.i.i.i.i.i.i) #23
  %24 = extractvalue { i64, i1 } %23, 1
  %25 = xor i1 %24, true
  tail call void @llvm.assume(i1 %25) #23
  %26 = extractvalue { i64, i1 } %23, 0
  %27 = icmp eq i64 %26, 0
  br i1 %27, label %bb9, label %bb2.i.i.i.i.i.i.i.i.i.i

bb2.i.i.i.i.i.i.i.i.i.i:                          ; preds = %bb2.i.i.i.i.i.i.i
  %28 = getelementptr inbounds %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>", %"core::option::Option<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* %_17, i64 0, i32 1, i64 4
  %29 = bitcast i64* %28 to i8**
  %_17.i.i.i.i.i.i.i.i.i = load i8*, i8** %29, align 8, !alias.scope !460, !nonnull !85, !noundef !85
  %30 = sub i64 0, %22
  %31 = getelementptr inbounds i8, i8* %_17.i.i.i.i.i.i.i.i.i, i64 %30
  tail call void @__rust_dealloc(i8* nonnull %31, i64 %26, i64 16) #23, !noalias !460
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
define i64 @report_malloc(i8* %address) unnamed_addr #6 personality i32 (i32, i32, i64, %"unwind::libunwind::_Unwind_Exception"*, %"unwind::libunwind::_Unwind_Context"*)* @rust_eh_personality {
start:
  %e.i32 = alloca { i64*, i8 }, align 8
  %this.i.i18 = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*, align 8
  %e.i = alloca { i64*, i8 }, align 8
  %this.i.i = alloca %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*, align 8
  %object_table = alloca { i64*, i8 }, align 8
  %guard = alloca { i64*, i8 }, align 8
  %0 = bitcast { i64*, i8 }* %guard to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %0)
  %1 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %1)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i, align 8
  %2 = load atomic i64, i64* bitcast (<{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE to i64*) acquire, align 8, !noalias !461
  %3 = icmp eq i64 %2, 2
  br i1 %3, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %start
  %4 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<i64>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17h1ed77e854a4795c8E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %4)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit": ; preds = %start, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #23
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %1)
  %5 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !466
  %6 = extractvalue { i32, i1 } %5, 1
  br i1 %6, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !466
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h28aba0b295b609edE.exit"
  %7 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !469
  %_1.i.i.i.i.i.i = and i64 %7, 9223372036854775807
  %8 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %8, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %9 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !469
  %phi.bo.i.i.i.i.i = xor i1 %9, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %10 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !469
  %.not = icmp eq i8 %10, 0
  br i1 %.not, label %bb5, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %11 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %11), !noalias !472
  %12 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %12, align 8, !noalias !472
  %13 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %13, align 8, !noalias !472
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc407 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.5 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc453 to %"core::panic::location::Location"*)) #22
          to label %unreachable.i unwind label %cleanup.i, !noalias !472

cleanup.i:                                        ; preds = %bb1.i
  %14 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i) #24
          to label %common.resume unwind label %abort.i, !noalias !472

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %15 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25, !noalias !472
  unreachable

common.resume:                                    ; preds = %bb14, %cleanup.i
  %common.resume.op = phi { i8*, i32 } [ %14, %cleanup.i ], [ %.pn, %bb14 ]
  resume { i8*, i32 } %common.resume.op

bb14:                                             ; preds = %cleanup.i38, %cleanup, %cleanup1
  %.pn = phi { i8*, i32 } [ %36, %cleanup1 ], [ %16, %cleanup ], [ %34, %cleanup.i38 ]
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %guard) #24
          to label %common.resume unwind label %abort

cleanup:                                          ; preds = %bb2.i.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb3.i.i.i.i.i.i27, %bb3.i.i.i22, %bb3.i.i.i.i19
  %16 = landingpad { i8*, i32 }
          cleanup
  br label %bb14

bb5:                                              ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %guard, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %17 = bitcast { i64*, i8 }* %guard to %"std::sync::mutex::Mutex<i64>"**
  %18 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 8) to i64*), align 8
  %19 = add i64 %18, 1
  store i64 %19, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [16 x i8], i8* }>, <{ [16 x i8], [16 x i8], i8* }>* @_ZN12fixsanitizer9OBJECT_ID17h6edf6e3689c5261eE, i64 0, i32 1, i64 8) to i64*), align 8
  %20 = bitcast { i64*, i8 }* %object_table to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %20)
  %21 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i18 to i8*
  call void @llvm.lifetime.start.p0i8(i64 8, i8* nonnull %21)
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i18, align 8
  %22 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to i64*) acquire, align 8, !noalias !475
  %23 = icmp eq i64 %22, 2
  br i1 %23, label %bb6, label %bb3.i.i.i.i19

bb3.i.i.i.i19:                                    ; preds = %bb5
  %24 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i18 to i64*
; invoke once_cell::imp::OnceCell<T>::initialize
  invoke fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %24)
          to label %bb6 unwind label %cleanup

bb6:                                              ; preds = %bb5, %bb3.i.i.i.i19
  %_6.i.i.i.i.i.i.i20 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i21 = icmp ne i64 %_6.i.i.i.i.i.i.i20, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i21) #23
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %21)
  %25 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !480
  %26 = extractvalue { i32, i1 } %25, 1
  br i1 %26, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i24, label %bb3.i.i.i22

bb3.i.i.i22:                                      ; preds = %bb6
; invoke std::sys::unix::locks::futex::Mutex::lock_contended
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i24 unwind label %cleanup

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i24: ; preds = %bb3.i.i.i22, %bb6
  %27 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !483
  %_1.i.i.i.i.i.i23 = and i64 %27, 9223372036854775807
  %28 = icmp eq i64 %_1.i.i.i.i.i.i23, 0
  br i1 %28, label %bb7, label %bb3.i.i.i.i.i.i27

bb3.i.i.i.i.i.i27:                                ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i24
; invoke std::panicking::panic_count::is_zero_slow_path
  %29 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc31 unwind label %cleanup

.noexc31:                                         ; preds = %bb3.i.i.i.i.i.i27
  %phi.bo.i.i.i.i.i25 = xor i1 %29, true
  %phi.cast.i.i.i26 = zext i1 %phi.bo.i.i.i.i.i25 to i8
  br label %bb7

bb7:                                              ; preds = %.noexc31, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i24
  %.0.i.i.i.i.i.i28 = phi i8 [ %phi.cast.i.i.i26, %.noexc31 ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i24 ]
  %30 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !483
  %.not75 = icmp eq i8 %30, 0
  br i1 %.not75, label %bb9, label %bb1.i37

bb1.i37:                                          ; preds = %bb7
  %31 = bitcast { i64*, i8 }* %e.i32 to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %31), !noalias !486
  %32 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i32, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %32, align 8, !noalias !486
  %33 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i32, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i28, i8* %33, align 8, !noalias !486
  %_6.0.i36 = bitcast { i64*, i8 }* %e.i32 to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc407 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i36, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.5 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc455 to %"core::panic::location::Location"*)) #22
          to label %unreachable.i39 unwind label %cleanup.i38, !noalias !490

cleanup.i38:                                      ; preds = %bb1.i37
  %34 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i32) #24
          to label %bb14 unwind label %abort.i40, !noalias !490

unreachable.i39:                                  ; preds = %bb1.i37
  unreachable

abort.i40:                                        ; preds = %cleanup.i38
  %35 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25, !noalias !490
  unreachable

cleanup1:                                         ; preds = %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i"
  %36 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %object_table) #24
          to label %bb14 unwind label %abort

bb9:                                              ; preds = %bb7
  %.fca.0.gep3 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep3, align 8
  %.fca.1.gep5 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i28, i8* %.fca.1.gep5, align 8
  %_20 = ptrtoint i8* %address to i64
  call void @llvm.experimental.noalias.scope.decl(metadata !491)
  call void @llvm.experimental.noalias.scope.decl(metadata !494)
  %_6.idx.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !497, !noalias !498
  %_6.idx11.val.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !497, !noalias !498
  %37 = xor i64 %_6.idx.val.i.i, 8317987319222330741
  %38 = xor i64 %_6.idx11.val.i.i, 7237128888997146477
  %39 = xor i64 %_6.idx.val.i.i, 7816392313619706465
  %40 = xor i64 %19, %_6.idx11.val.i.i
  %41 = xor i64 %40, 8387220255154660723
  %42 = add i64 %38, %37
  %43 = call i64 @llvm.fshl.i64(i64 %38, i64 %38, i64 13) #23
  %44 = xor i64 %42, %43
  %45 = call i64 @llvm.fshl.i64(i64 %42, i64 %42, i64 32) #23
  %46 = add i64 %41, %39
  %47 = call i64 @llvm.fshl.i64(i64 %41, i64 %41, i64 16) #23
  %48 = xor i64 %47, %46
  %49 = add i64 %48, %45
  %50 = call i64 @llvm.fshl.i64(i64 %48, i64 %48, i64 21) #23
  %51 = xor i64 %50, %49
  %52 = add i64 %44, %46
  %53 = call i64 @llvm.fshl.i64(i64 %44, i64 %44, i64 17) #23
  %54 = xor i64 %52, %53
  %55 = call i64 @llvm.fshl.i64(i64 %52, i64 %52, i64 32) #23
  %56 = xor i64 %49, %19
  %57 = xor i64 %51, 576460752303423488
  %58 = add i64 %56, %54
  %59 = call i64 @llvm.fshl.i64(i64 %54, i64 %54, i64 13) #23
  %60 = xor i64 %58, %59
  %61 = call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 32) #23
  %62 = add i64 %57, %55
  %63 = call i64 @llvm.fshl.i64(i64 %51, i64 %57, i64 16) #23
  %64 = xor i64 %63, %62
  %65 = add i64 %64, %61
  %66 = call i64 @llvm.fshl.i64(i64 %64, i64 %64, i64 21) #23
  %67 = xor i64 %66, %65
  %68 = add i64 %62, %60
  %69 = call i64 @llvm.fshl.i64(i64 %60, i64 %60, i64 17) #23
  %70 = xor i64 %68, %69
  %71 = call i64 @llvm.fshl.i64(i64 %68, i64 %68, i64 32) #23
  %72 = xor i64 %65, 576460752303423488
  %73 = xor i64 %71, 255
  %74 = add i64 %72, %70
  %75 = call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 13) #23
  %76 = xor i64 %74, %75
  %77 = call i64 @llvm.fshl.i64(i64 %74, i64 %74, i64 32) #23
  %78 = add i64 %67, %73
  %79 = call i64 @llvm.fshl.i64(i64 %67, i64 %67, i64 16) #23
  %80 = xor i64 %79, %78
  %81 = add i64 %80, %77
  %82 = call i64 @llvm.fshl.i64(i64 %80, i64 %80, i64 21) #23
  %83 = xor i64 %82, %81
  %84 = add i64 %76, %78
  %85 = call i64 @llvm.fshl.i64(i64 %76, i64 %76, i64 17) #23
  %86 = xor i64 %84, %85
  %87 = call i64 @llvm.fshl.i64(i64 %84, i64 %84, i64 32) #23
  %88 = add i64 %86, %81
  %89 = call i64 @llvm.fshl.i64(i64 %86, i64 %86, i64 13) #23
  %90 = xor i64 %89, %88
  %91 = call i64 @llvm.fshl.i64(i64 %88, i64 %88, i64 32) #23
  %92 = add i64 %83, %87
  %93 = call i64 @llvm.fshl.i64(i64 %83, i64 %83, i64 16) #23
  %94 = xor i64 %93, %92
  %95 = add i64 %94, %91
  %96 = call i64 @llvm.fshl.i64(i64 %94, i64 %94, i64 21) #23
  %97 = xor i64 %96, %95
  %98 = add i64 %90, %92
  %99 = call i64 @llvm.fshl.i64(i64 %90, i64 %90, i64 17) #23
  %100 = xor i64 %99, %98
  %101 = call i64 @llvm.fshl.i64(i64 %98, i64 %98, i64 32) #23
  %102 = add i64 %100, %95
  %103 = call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 13) #23
  %104 = xor i64 %103, %102
  %105 = add i64 %97, %101
  %106 = call i64 @llvm.fshl.i64(i64 %97, i64 %97, i64 16) #23
  %107 = xor i64 %106, %105
  %108 = call i64 @llvm.fshl.i64(i64 %107, i64 %107, i64 21) #23
  %109 = add i64 %104, %105
  %110 = call i64 @llvm.fshl.i64(i64 %104, i64 %104, i64 17) #23
  %111 = call i64 @llvm.fshl.i64(i64 %109, i64 %109, i64 32) #23
  %_17.i.i.i.i.i.i.i = xor i64 %109, %108
  %112 = xor i64 %_17.i.i.i.i.i.i.i, %110
  %113 = xor i64 %112, %111
  call void @llvm.experimental.noalias.scope.decl(metadata !503)
  call void @llvm.experimental.noalias.scope.decl(metadata !506) #23
  call void @llvm.experimental.noalias.scope.decl(metadata !509) #23
  %top7.i.i.i.i.i.i = lshr i64 %113, 57
  %114 = trunc i64 %top7.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !512, !noalias !515
  %_3.i.i.i.i.i.i = and i64 %113, %_6.i.i.i.i.i.i
  %self.idx.val.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !517, !noalias !515
  %.0.vec.insert.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %114, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i

bb3.i.i.i.i.i:                                    ; preds = %bb21.i.i.i.i.i, %bb9
  %probe_seq.sroa.7.0.i.i.i.i.i = phi i64 [ 0, %bb9 ], [ %127, %bb21.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb9 ], [ %129, %bb21.i.i.i.i.i ]
  %115 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i
  %116 = bitcast i8* %115 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i = load <16 x i8>, <16 x i8>* %116, align 1, !noalias !518
  %117 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i
  %118 = bitcast <16 x i1> %117 to i16
  br label %bb8.i.i.i.i.i

bb8.i.i.i.i.i:                                    ; preds = %bb10.i.i.i.i.i, %bb3.i.i.i.i.i
  %iter.0.i.i.i.i.i = phi i16 [ %118, %bb3.i.i.i.i.i ], [ %_2.i.i.i.i.i.i.i, %bb10.i.i.i.i.i ]
  %119 = icmp eq i16 %iter.0.i.i.i.i.i, 0
  br i1 %119, label %bb12.i.i.i.i.i, label %bb10.i.i.i.i.i

bb12.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %120 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %121 = bitcast <16 x i1> %120 to i16
  %.not.i.i.i.i.i = icmp eq i16 %121, 0
  br i1 %.not.i.i.i.i.i, label %bb21.i.i.i.i.i, label %bb6.i.i

bb10.i.i.i.i.i:                                   ; preds = %bb8.i.i.i.i.i
  %122 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i, i1 true) #23, !range !27
  %_2.i.i.i.i.i.i.i.i = zext i16 %122 to i64
  %_4.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i
  %_25.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i = and i64 %_25.i.i.i.i.i, %_6.i.i.i.i.i.i
  %123 = sub i64 0, %index.i.i.i.i.i
  %124 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i, i64 %123, i32 0
  %125 = getelementptr inbounds i64, i64* %124, i64 -4
  %_6.idx.val.i.i.i.i.i.i = load i64, i64* %125, align 8, !noalias !521
  %126 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i, %19
  br i1 %126, label %bb10, label %bb8.i.i.i.i.i

bb21.i.i.i.i.i:                                   ; preds = %bb12.i.i.i.i.i
  %127 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i, 16
  %128 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i, %127
  %129 = and i64 %128, %_6.i.i.i.i.i.i
  br label %bb3.i.i.i.i.i

bb6.i.i:                                          ; preds = %bb12.i.i.i.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !524)
  %130 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_3.i.i.i.i.i.i
  %131 = bitcast i8* %130 to <16 x i8>*
  %.0.copyload.i2122.i.i.i.i = load <16 x i8>, <16 x i8>* %131, align 1, !noalias !527
  %132 = icmp slt <16 x i8> %.0.copyload.i2122.i.i.i.i, zeroinitializer
  %133 = bitcast <16 x i1> %132 to i16
  %.not23.i.i.i.i = icmp eq i16 %133, 0
  br i1 %.not23.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb7.i.i.i.i:                                      ; preds = %bb17.i.i.i.i, %bb6.i.i
  %probe_seq.sroa.0.0.lcssa.i.i.i.i = phi i64 [ %_3.i.i.i.i.i.i, %bb6.i.i ], [ %139, %bb17.i.i.i.i ]
  %.lcssa.i.i.i.i = phi i16 [ %133, %bb6.i.i ], [ %143, %bb17.i.i.i.i ]
  %134 = call i16 @llvm.cttz.i16(i16 %.lcssa.i.i.i.i, i1 true) #23, !range !27
  %_2.i.i.i.i.i.i = zext i16 %134 to i64
  %_17.i.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i.i.i.i, %_2.i.i.i.i.i.i
  %result.i.i.i.i = and i64 %_17.i.i.i.i, %_6.i.i.i.i.i.i
  %135 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %result.i.i.i.i
  %_23.i.i.i.i = load i8, i8* %135, align 1, !noalias !534
  %136 = icmp sgt i8 %_23.i.i.i.i, -1
  br i1 %136, label %bb11.i.i.i.i, label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"

bb17.i.i.i.i:                                     ; preds = %bb6.i.i, %bb17.i.i.i.i
  %probe_seq.sroa.0.025.i.i.i.i = phi i64 [ %139, %bb17.i.i.i.i ], [ %_3.i.i.i.i.i.i, %bb6.i.i ]
  %probe_seq.sroa.7.024.i.i.i.i = phi i64 [ %137, %bb17.i.i.i.i ], [ 0, %bb6.i.i ]
  %137 = add i64 %probe_seq.sroa.7.024.i.i.i.i, 16
  %138 = add i64 %137, %probe_seq.sroa.0.025.i.i.i.i
  %139 = and i64 %138, %_6.i.i.i.i.i.i
  %140 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %139
  %141 = bitcast i8* %140 to <16 x i8>*
  %.0.copyload.i21.i.i.i.i = load <16 x i8>, <16 x i8>* %141, align 1, !noalias !527
  %142 = icmp slt <16 x i8> %.0.copyload.i21.i.i.i.i, zeroinitializer
  %143 = bitcast <16 x i1> %142 to i16
  %.not.i.i.i.i = icmp eq i16 %143, 0
  br i1 %.not.i.i.i.i, label %bb17.i.i.i.i, label %bb7.i.i.i.i

bb11.i.i.i.i:                                     ; preds = %bb7.i.i.i.i
  %144 = bitcast i8* %self.idx.val.i.i.i.i.i to <16 x i8>*
  %145 = load <16 x i8>, <16 x i8>* %144, align 16, !noalias !535
  %146 = icmp slt <16 x i8> %145, zeroinitializer
  %147 = bitcast <16 x i1> %146 to i16
  %148 = call i16 @llvm.cttz.i16(i16 %147, i1 true) #23, !range !27
  %_2.i.i.i.i.i = zext i16 %148 to i64
  %.phi.trans.insert.i.i.i = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i, i64 %_2.i.i.i.i.i
  %old_ctrl.pre.i.i.i = load i8, i8* %.phi.trans.insert.i.i.i, align 1, !noalias !540
  br label %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"

"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i": ; preds = %bb11.i.i.i.i, %bb7.i.i.i.i
  %old_ctrl.i.i.i = phi i8 [ %old_ctrl.pre.i.i.i, %bb11.i.i.i.i ], [ %_23.i.i.i.i, %bb7.i.i.i.i ]
  %.0.i.i.i.i = phi i64 [ %_2.i.i.i.i.i, %bb11.i.i.i.i ], [ %result.i.i.i.i, %bb7.i.i.i.i ]
  %_14.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !541, !noalias !542
  %149 = icmp eq i64 %_14.i.i.i, 0
  %_2.i.i.i.i = and i8 %old_ctrl.i.i.i, 1
  %150 = icmp ne i8 %_2.i.i.i.i, 0
  %or.cond.i.i.i = select i1 %149, i1 %150, i1 false
  br i1 %or.cond.i.i.i, label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i", label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"

"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i": ; preds = %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"
; invoke hashbrown::raw::RawTable<T,A>::reserve_rehash
  %151 = invoke fastcc { i64, i64 } @"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash17h320d5dd485a72968E"(%"hashbrown::raw::RawTable<(i64, ObjectInfo)>"* noalias noundef nonnull align 8 dereferenceable(32) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to %"hashbrown::raw::RawTable<(i64, ObjectInfo)>"*), i64* noalias noundef nonnull readonly align 8 dereferenceable(16) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to i64*))
          to label %.noexc43 unwind label %cleanup1

.noexc43:                                         ; preds = %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7reserve17h74f2cd2d3469dba4E.exit.i.i.i"
  %.fca.1.extract.i.i.i.i = extractvalue { i64, i64 } %151, 1
  %.not.i2.i.i.i = icmp eq i64 %.fca.1.extract.i.i.i.i, -9223372036854775807
  call void @llvm.assume(i1 %.not.i2.i.i.i)
  call void @llvm.experimental.noalias.scope.decl(metadata !543)
  %_6.i.i4.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !546, !noalias !542
  %_3.i.i5.i.i.i = and i64 %_6.i.i4.i.i.i, %113
  %self.idx11.val.i7.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !549, !noalias !542
  %152 = getelementptr inbounds i8, i8* %self.idx11.val.i7.i.i.i, i64 %_3.i.i5.i.i.i
  %153 = bitcast i8* %152 to <16 x i8>*
  %.0.copyload.i2122.i8.i.i.i = load <16 x i8>, <16 x i8>* %153, align 1, !noalias !550
  %154 = icmp slt <16 x i8> %.0.copyload.i2122.i8.i.i.i, zeroinitializer
  %155 = bitcast <16 x i1> %154 to i16
  %.not23.i9.i.i.i = icmp eq i16 %155, 0
  br i1 %.not23.i9.i.i.i, label %bb17.i21.i.i.i, label %bb7.i16.i.i.i

bb7.i16.i.i.i:                                    ; preds = %bb17.i21.i.i.i, %.noexc43
  %probe_seq.sroa.0.0.lcssa.i10.i.i.i = phi i64 [ %_3.i.i5.i.i.i, %.noexc43 ], [ %161, %bb17.i21.i.i.i ]
  %.lcssa.i11.i.i.i = phi i16 [ %155, %.noexc43 ], [ %165, %bb17.i21.i.i.i ]
  %156 = call i16 @llvm.cttz.i16(i16 %.lcssa.i11.i.i.i, i1 true) #23, !range !27
  %_2.i.i.i12.i.i.i = zext i16 %156 to i64
  %_17.i13.i.i.i = add i64 %probe_seq.sroa.0.0.lcssa.i10.i.i.i, %_2.i.i.i12.i.i.i
  %result.i14.i.i.i = and i64 %_17.i13.i.i.i, %_6.i.i4.i.i.i
  %157 = getelementptr inbounds i8, i8* %self.idx11.val.i7.i.i.i, i64 %result.i14.i.i.i
  %_23.i15.i.i.i = load i8, i8* %157, align 1, !noalias !553
  %158 = icmp sgt i8 %_23.i15.i.i.i, -1
  br i1 %158, label %bb11.i23.i.i.i, label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"

bb17.i21.i.i.i:                                   ; preds = %.noexc43, %bb17.i21.i.i.i
  %probe_seq.sroa.0.025.i17.i.i.i = phi i64 [ %161, %bb17.i21.i.i.i ], [ %_3.i.i5.i.i.i, %.noexc43 ]
  %probe_seq.sroa.7.024.i18.i.i.i = phi i64 [ %159, %bb17.i21.i.i.i ], [ 0, %.noexc43 ]
  %159 = add i64 %probe_seq.sroa.7.024.i18.i.i.i, 16
  %160 = add i64 %159, %probe_seq.sroa.0.025.i17.i.i.i
  %161 = and i64 %160, %_6.i.i4.i.i.i
  %162 = getelementptr inbounds i8, i8* %self.idx11.val.i7.i.i.i, i64 %161
  %163 = bitcast i8* %162 to <16 x i8>*
  %.0.copyload.i21.i19.i.i.i = load <16 x i8>, <16 x i8>* %163, align 1, !noalias !550
  %164 = icmp slt <16 x i8> %.0.copyload.i21.i19.i.i.i, zeroinitializer
  %165 = bitcast <16 x i1> %164 to i16
  %.not.i20.i.i.i = icmp eq i16 %165, 0
  br i1 %.not.i20.i.i.i, label %bb17.i21.i.i.i, label %bb7.i16.i.i.i

bb11.i23.i.i.i:                                   ; preds = %bb7.i16.i.i.i
  %166 = bitcast i8* %self.idx11.val.i7.i.i.i to <16 x i8>*
  %167 = load <16 x i8>, <16 x i8>* %166, align 16, !noalias !554
  %168 = icmp slt <16 x i8> %167, zeroinitializer
  %169 = bitcast <16 x i1> %168 to i16
  %170 = call i16 @llvm.cttz.i16(i16 %169, i1 true) #23, !range !27
  %_2.i.i22.i.i.i = zext i16 %170 to i64
  br label %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"

"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i": ; preds = %bb11.i23.i.i.i, %bb7.i16.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i"
  %self.idx1.val.i.i.i.i.i.i = phi i8* [ %self.idx.val.i.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i" ], [ %self.idx11.val.i7.i.i.i, %bb11.i23.i.i.i ], [ %self.idx11.val.i7.i.i.i, %bb7.i16.i.i.i ]
  %_8.i.i.i.i.i.i = phi i64 [ %_6.i.i.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i" ], [ %_6.i.i4.i.i.i, %bb11.i23.i.i.i ], [ %_6.i.i4.i.i.i, %bb7.i16.i.i.i ]
  %index.0.i.i.i = phi i64 [ %.0.i.i.i.i, %"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E.exit.i.i.i" ], [ %_2.i.i22.i.i.i, %bb11.i23.i.i.i ], [ %result.i14.i.i.i, %bb7.i16.i.i.i ]
  %self.idx.val28.i.i.i = bitcast i8* %self.idx1.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  call void @llvm.experimental.noalias.scope.decl(metadata !559)
  %sext.i.i.i.i = sub nsw i8 0, %_2.i.i.i.i
  %_5.neg.i.i.i.i = sext i8 %sext.i.i.i.i to i64
  %171 = add i64 %index.0.i.i.i, -16
  %_5.i.i.i.i.i.i = and i64 %171, %_8.i.i.i.i.i.i
  %index2.i.i.i.i.i.i = add i64 %_5.i.i.i.i.i.i, 16
  %172 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index.0.i.i.i
  store i8 %114, i8* %172, align 1, !noalias !562
  %173 = getelementptr inbounds i8, i8* %self.idx1.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i
  store i8 %114, i8* %173, align 1, !noalias !562
  %174 = load <2 x i64>, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !567, !noalias !542
  %175 = insertelement <2 x i64> <i64 poison, i64 1>, i64 %_5.neg.i.i.i.i, i64 0
  %176 = add <2 x i64> %174, %175
  store <2 x i64> %176, <2 x i64>* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to <2 x i64>*), align 8, !alias.scope !567, !noalias !542
  %177 = sub i64 0, %index.0.i.i.i
  %178 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %self.idx.val28.i.i.i, i64 %177, i32 0
  %_23.sroa.0.0..sroa_idx.i.i = getelementptr inbounds i64, i64* %178, i64 -4
  store i64 %19, i64* %_23.sroa.0.0..sroa_idx.i.i, align 8, !noalias !568
  br label %bb10

bb10:                                             ; preds = %bb10.i.i.i.i.i, %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i"
  %.pn76 = phi i64* [ %178, %"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE.exit.i.i" ], [ %124, %bb10.i.i.i.i.i ]
  %tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.in = getelementptr inbounds i64, i64* %.pn76, i64 -3
  store i64 %19, i64* %tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.in, align 8, !noalias !569
  %_27.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx77 = getelementptr inbounds i64, i64* %.pn76, i64 -2
  store i64 %_20, i64* %_27.sroa.4.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx77, align 8, !noalias !569
  %_27.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx78 = getelementptr inbounds i64, i64* %.pn76, i64 -1
  store i64 1, i64* %_27.sroa.5.0.tmp.sroa.0.0..sroa_cast3.i.i.i.sink.i.sroa_idx78, align 8, !noalias !569
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i28, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb10
  %179 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !570
  %_1.i.i.i.i.i.i44 = and i64 %179, 9223372036854775807
  %180 = icmp eq i64 %_1.i.i.i.i.i.i44, 0
  br i1 %180, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; invoke std::panicking::panic_count::is_zero_slow_path
  %181 = invoke noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E()
          to label %.noexc45 unwind label %cleanup

.noexc45:                                         ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  br i1 %181, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %.noexc45
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !570
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %.noexc45, %bb2.i.i.i, %bb10
  %182 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !570
  %183 = icmp eq i32 %182, 2
  br i1 %183, label %bb2.i.i.i.i, label %bb11

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; invoke std::sys::unix::locks::futex::Mutex::wake
  invoke void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*))
          to label %bb11 unwind label %cleanup

abort:                                            ; preds = %bb14, %cleanup1
  %184 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25
  unreachable

bb11:                                             ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %20)
  call void @llvm.experimental.noalias.scope.decl(metadata !573)
  %_8.i.i47 = load %"std::sync::mutex::Mutex<i64>"*, %"std::sync::mutex::Mutex<i64>"** %17, align 8, !alias.scope !573, !nonnull !85, !align !86, !noundef !85
  %_5.val.i.i49 = load i8, i8* %.fca.1.gep, align 8, !alias.scope !573
  %_5.not.i.i.i50 = icmp eq i8 %_5.val.i.i49, 0
  br i1 %_5.not.i.i.i50, label %bb2.i.i.i52, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57

bb2.i.i.i52:                                      ; preds = %bb11
  %185 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !573
  %_1.i.i.i.i.i.i51 = and i64 %185, 9223372036854775807
  %186 = icmp eq i64 %_1.i.i.i.i.i.i51, 0
  br i1 %186, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i53

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i53: ; preds = %bb2.i.i.i52
; call std::panicking::panic_count::is_zero_slow_path
  %187 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !573
  br i1 %187, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57, label %bb5.i.i.i55

bb5.i.i.i55:                                      ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i53
  %_6.i.i.i.i54 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i47, i64 0, i32 1, i32 0, i32 0
  store atomic i8 1, i8* %_6.i.i.i.i54 monotonic, align 4, !noalias !573
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57: ; preds = %bb5.i.i.i55, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i53, %bb2.i.i.i52, %bb11
  %_5.i.i.i.i.i56 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i47, i64 0, i32 0, i32 0, i32 0, i32 0
  %188 = atomicrmw xchg i32* %_5.i.i.i.i.i56, i32 0 release, align 4, !noalias !573
  %189 = icmp eq i32 %188, 2
  br i1 %189, label %bb2.i.i.i.i59, label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

bb2.i.i.i.i59:                                    ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57
  %_2.i.i.i58 = getelementptr inbounds %"std::sync::mutex::Mutex<i64>", %"std::sync::mutex::Mutex<i64>"* %_8.i.i47, i64 0, i32 0, i32 0
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) %_2.i.i.i58), !noalias !573
  br label %"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit"

"_ZN4core3ptr60drop_in_place$LT$std..sync..mutex..MutexGuard$LT$i64$GT$$GT$17h23b57bf2d88cfd4eE.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i57, %bb2.i.i.i.i59
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %0)
  ret i64 %19
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
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %4 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to i64*) acquire, align 8, !noalias !576
  %5 = icmp eq i64 %4, 2
  br i1 %5, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb4
  %6 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %6)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit": ; preds = %bb4, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #23
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3)
  %7 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !581
  %8 = extractvalue { i32, i1 } %7, 1
  br i1 %8, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !581
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
  %9 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !584
  %_1.i.i.i.i.i.i = and i64 %9, 9223372036854775807
  %10 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %10, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %11 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !584
  %phi.bo.i.i.i.i.i = xor i1 %11, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %12 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !584
  %.not = icmp eq i8 %12, 0
  br i1 %.not, label %bb8, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %13 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %13), !noalias !587
  %14 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %14, align 8, !noalias !587
  %15 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %15, align 8, !noalias !587
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc407 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.5 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc459 to %"core::panic::location::Location"*)) #22
          to label %unreachable.i unwind label %cleanup.i, !noalias !591

cleanup.i:                                        ; preds = %bb1.i
  %16 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i) #24
          to label %common.resume unwind label %abort.i, !noalias !591

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %17 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25, !noalias !591
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc250 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_19.sroa.0.0..sroa_cast, align 8
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
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc244 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_18, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc457 to %"core::panic::location::Location"*)) #22
  unreachable

cleanup:                                          ; preds = %bb1.i17, %bb21, %bb12
  %23 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %object_table) #24
          to label %common.resume unwind label %abort

bb8:                                              ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_table, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !592
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_37 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h7c6dbde3483cee85E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc195 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %28, align 8, !alias.scope !595, !noalias !598
  %29 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 0, i32 1
  store i64 2, i64* %29, align 8, !alias.scope !595, !noalias !598
  %30 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 1, i32 0
  store i64* null, i64** %30, align 8, !alias.scope !595, !noalias !598
  %31 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 0
  %32 = bitcast [0 x { i8*, i64* }]** %31 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_50, [1 x { i8*, i64* }]** %32, align 8, !alias.scope !595, !noalias !598
  %33 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 1
  store i64 1, i64* %33, align 8, !alias.scope !595, !noalias !598
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_43, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc461 to %"core::panic::location::Location"*)) #22
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb21, %bb12
  unreachable

bb14:                                             ; preds = %bb8
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_55 = call fastcc noundef align 8 dereferenceable_or_null(24) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h806e044307d21e0aE"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  %34 = icmp eq i64* %_55, null
  br i1 %34, label %bb1.i17, label %bb16

bb1.i17:                                          ; preds = %bb14
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc399 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc463 to %"core::panic::location::Location"*)) #22
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
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb22
  %36 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !601
  %_1.i.i.i.i.i.i18 = and i64 %36, 9223372036854775807
  %37 = icmp eq i64 %_1.i.i.i.i.i.i18, 0
  br i1 %37, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %38 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !601
  br i1 %38, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !601
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb22
  %39 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !601
  %40 = icmp eq i32 %39, 2
  br i1 %40, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !601
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %2)
  ret void

bb21:                                             ; preds = %bb16
  %41 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %41)
  %42 = bitcast [3 x { i8*, i64* }]* %_84 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %42)
  %43 = bitcast [3 x { i8*, i64* }]* %_84 to i64**
  store i64* %obj_id, i64** %43, align 8
  %44 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %44, align 8
  %45 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 0
  %46 = bitcast i8** %45 to i64**
  store i64* %refcnt, i64** %46, align 8
  %47 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %47, align 8
  %48 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 0
  %49 = bitcast i8** %48 to i64**
  store i64* %35, i64** %49, align 8
  %50 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %50, align 8
  %_77.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc262 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_77.sroa.0.0..sroa_cast, align 8
  %_77.sroa.4.0..sroa_idx36 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 0
  store i64 3, i64* %_77.sroa.4.0..sroa_idx36, align 8
  %_77.sroa.5.0..sroa_idx38 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 1
  %_77.sroa.5.0..sroa_cast = bitcast i64* %_77.sroa.5.0..sroa_idx38 to i64**
  store i64* null, i64** %_77.sroa.5.0..sroa_cast, align 8
  %_77.sroa.642.0..sroa_idx43 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 3
  %51 = bitcast i64* %_77.sroa.642.0..sroa_idx43 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_84, [3 x { i8*, i64* }]** %51, align 8
  %_77.sroa.7.0..sroa_idx45 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 4
  store i64 3, i64* %_77.sroa.7.0..sroa_idx45, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %35, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_76, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc465 to %"core::panic::location::Location"*)) #22
          to label %unreachable unwind label %cleanup

abort:                                            ; preds = %cleanup
  %52 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25
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
  store %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"*), %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i, align 8
  %4 = load atomic i64, i64* bitcast (<{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE to i64*) acquire, align 8, !noalias !604
  %5 = icmp eq i64 %4, 2
  br i1 %5, label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit", label %bb3.i.i.i.i

bb3.i.i.i.i:                                      ; preds = %bb4
  %6 = bitcast %"once_cell::sync::Lazy<std::sync::mutex::Mutex<std::collections::hash::map::HashMap<i64, ObjectInfo>>>"** %this.i.i to i64*
; call once_cell::imp::OnceCell<T>::initialize
  call fastcc void @"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize17hd666801a3ecc6089E"(i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %6)
  br label %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"

"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit": ; preds = %bb4, %bb3.i.i.i.i
  %_6.i.i.i.i.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 0, i64 8) to i64*), align 8, !range !119
  %trunc.not.i.i.i.i.i.i.i = icmp ne i64 %_6.i.i.i.i.i.i.i, 0
  call void @llvm.assume(i1 %trunc.not.i.i.i.i.i.i.i) #23
  call void @llvm.lifetime.end.p0i8(i64 8, i8* nonnull %3)
  %7 = cmpxchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0, i32 1 acquire monotonic, align 4, !noalias !609
  %8 = extractvalue { i32, i1 } %7, 1
  br i1 %8, label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, label %bb3.i.i.i

bb3.i.i.i:                                        ; preds = %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
; call std::sys::unix::locks::futex::Mutex::lock_contended
  call void @_ZN3std3sys4unix5locks5futex5Mutex14lock_contended17h30317766f0f7458eE(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !609
  br label %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i

_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i: ; preds = %bb3.i.i.i, %"_ZN78_$LT$once_cell..sync..Lazy$LT$T$C$F$GT$$u20$as$u20$core..ops..deref..Deref$GT$5deref17h262e0536173a0d5fE.exit"
  %9 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !612
  %_1.i.i.i.i.i.i = and i64 %9, 9223372036854775807
  %10 = icmp eq i64 %_1.i.i.i.i.i.i, 0
  br i1 %10, label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit", label %bb3.i.i.i.i.i.i

bb3.i.i.i.i.i.i:                                  ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i
; call std::panicking::panic_count::is_zero_slow_path
  %11 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !612
  %phi.bo.i.i.i.i.i = xor i1 %11, true
  %phi.cast.i.i.i = zext i1 %phi.bo.i.i.i.i.i to i8
  br label %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"

"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit": ; preds = %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i, %bb3.i.i.i.i.i.i
  %.0.i.i.i.i.i.i = phi i8 [ %phi.cast.i.i.i, %bb3.i.i.i.i.i.i ], [ 0, %_ZN3std10sys_common5mutex12MovableMutex8raw_lock17hda2c0c7c086e1d9eE.exit.i ]
  %12 = load atomic i8, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !612
  %.not = icmp eq i8 %12, 0
  br i1 %.not, label %bb8, label %bb1.i

bb1.i:                                            ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %13 = bitcast { i64*, i8 }* %e.i to i8*
  call void @llvm.lifetime.start.p0i8(i64 16, i8* nonnull %13), !noalias !615
  %14 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %14, align 8, !noalias !615
  %15 = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %e.i, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %15, align 8, !noalias !615
  %_6.0.i = bitcast { i64*, i8 }* %e.i to {}*
; invoke core::result::unwrap_failed
  invoke void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc407 to [0 x i8]*), i64 43, {}* noundef nonnull align 1 %_6.0.i, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8], i8* }>* @vtable.5 to [3 x i64]*), %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc469 to %"core::panic::location::Location"*)) #22
          to label %unreachable.i unwind label %cleanup.i, !noalias !619

cleanup.i:                                        ; preds = %bb1.i
  %16 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::poison::PoisonError<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>>
  invoke void @"_ZN4core3ptr169drop_in_place$LT$std..sync..poison..PoisonError$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$$GT$17h7cf3bc78b1d23f11E"({ i64*, i8 }* nonnull %e.i) #24
          to label %common.resume unwind label %abort.i, !noalias !619

unreachable.i:                                    ; preds = %bb1.i
  unreachable

abort.i:                                          ; preds = %cleanup.i
  %17 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25, !noalias !619
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc250 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_19.sroa.0.0..sroa_cast, align 8
  %_19.sroa.4.0..sroa_idx25 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 0
  store i64 2, i64* %_19.sroa.4.0..sroa_idx25, align 8
  %_19.sroa.5.0..sroa_idx27 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 1
  %_19.sroa.5.0..sroa_cast = bitcast i64* %_19.sroa.5.0..sroa_idx27 to i64**
  store i64* null, i64** %_19.sroa.5.0..sroa_cast, align 8
  %_19.sroa.631.0..sroa_idx32 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 3
  %22 = bitcast i64* %_19.sroa.631.0..sroa_idx32 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_26, [1 x { i8*, i64* }]** %22, align 8
  %_19.sroa.7.0..sroa_idx34 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_18, i64 0, i32 1, i64 4
  store i64 1, i64* %_19.sroa.7.0..sroa_idx34, align 8
; call core::panicking::assert_failed
  call fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 1, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, i64* noalias noundef readonly align 8 dereferenceable(8) bitcast (<{ [8 x i8] }>* @alloc244 to i64*), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_18, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc467 to %"core::panic::location::Location"*)) #22
  unreachable

cleanup:                                          ; preds = %bb1.i18, %bb21, %bb12
  %23 = landingpad { i8*, i32 }
          cleanup
; invoke core::ptr::drop_in_place<std::sync::mutex::MutexGuard<std::collections::hash::map::HashMap<i64,fixsanitizer::ObjectInfo>>>
  invoke fastcc void @"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E"({ i64*, i8 }* nonnull %object_info) #24
          to label %common.resume unwind label %abort

bb8:                                              ; preds = %"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E.exit"
  %.fca.0.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 0
  store i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i64*), i64** %.fca.0.gep, align 8
  %.fca.1.gep = getelementptr inbounds { i64*, i8 }, { i64*, i8 }* %object_info, i64 0, i32 1
  store i8 %.0.i.i.i.i.i.i, i8* %.fca.1.gep, align 8
  %obj_id.val = load i64, i64* %obj_id, align 8, !alias.scope !592
; call std::collections::hash::map::HashMap<K,V,S>::contains_key
  %_37 = call fastcc noundef zeroext i1 @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$12contains_key17h7c6dbde3483cee85E"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull readonly align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
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
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8] }>* @alloc255 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %28, align 8, !alias.scope !620, !noalias !623
  %29 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 0, i32 1
  store i64 2, i64* %29, align 8, !alias.scope !620, !noalias !623
  %30 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 1, i32 0
  store i64* null, i64** %30, align 8, !alias.scope !620, !noalias !623
  %31 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 0
  %32 = bitcast [0 x { i8*, i64* }]** %31 to [1 x { i8*, i64* }]**
  store [1 x { i8*, i64* }]* %_50, [1 x { i8*, i64* }]** %32, align 8, !alias.scope !620, !noalias !623
  %33 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_43, i64 0, i32 2, i32 1
  store i64 1, i64* %33, align 8, !alias.scope !620, !noalias !623
; invoke core::panicking::panic_fmt
  invoke void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_43, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc471 to %"core::panic::location::Location"*)) #22
          to label %unreachable unwind label %cleanup

unreachable:                                      ; preds = %bb21, %bb12
  unreachable

bb14:                                             ; preds = %bb8
; call std::collections::hash::map::HashMap<K,V,S>::get_mut
  %_55 = call fastcc noundef align 8 dereferenceable_or_null(24) i64* @"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$7get_mut17h806e044307d21e0aE"(%"std::collections::hash::map::HashMap<i64, ObjectInfo>"* noalias noundef nonnull align 8 dereferenceable(48) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to %"std::collections::hash::map::HashMap<i64, ObjectInfo>"*), i64 %obj_id.val)
  %34 = icmp eq i64* %_55, null
  br i1 %34, label %bb1.i18, label %bb16

bb1.i18:                                          ; preds = %bb14
; invoke core::panicking::panic
  invoke void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1 bitcast (<{ [43 x i8] }>* @alloc399 to [0 x i8]*), i64 43, %"core::panic::location::Location"* noalias noundef nonnull readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc473 to %"core::panic::location::Location"*)) #22
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
  %36 = icmp eq i64 %_68, 0
  br i1 %36, label %bb25, label %bb27

bb21:                                             ; preds = %bb16
  %37 = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %37)
  %38 = bitcast [3 x { i8*, i64* }]* %_84 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %38)
  %39 = bitcast [3 x { i8*, i64* }]* %_84 to i64**
  store i64* %obj_id, i64** %39, align 8
  %40 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 0, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %40, align 8
  %41 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 0
  %42 = bitcast i8** %41 to i64**
  store i64* %refcnt, i64** %42, align 8
  %43 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 1, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %43, align 8
  %44 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 0
  %45 = bitcast i8** %44 to i64**
  store i64* %35, i64** %45, align 8
  %46 = getelementptr inbounds [3 x { i8*, i64* }], [3 x { i8*, i64* }]* %_84, i64 0, i64 2, i32 1
  store i64* bitcast (i1 (i64*, %"core::fmt::Formatter"*)* @"_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$i64$GT$3fmt17h59bda7149986ffa5E" to i64*), i64** %46, align 8
  %_77.sroa.0.0..sroa_cast = bitcast %"core::option::Option<core::fmt::Arguments>"* %_76 to [0 x { [0 x i8]*, i64 }]**
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8], i8*, [8 x i8], i8*, [8 x i8] }>* @alloc262 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %_77.sroa.0.0..sroa_cast, align 8
  %_77.sroa.4.0..sroa_idx41 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 0
  store i64 3, i64* %_77.sroa.4.0..sroa_idx41, align 8
  %_77.sroa.5.0..sroa_idx43 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 1
  %_77.sroa.5.0..sroa_cast = bitcast i64* %_77.sroa.5.0..sroa_idx43 to i64**
  store i64* null, i64** %_77.sroa.5.0..sroa_cast, align 8
  %_77.sroa.647.0..sroa_idx48 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 3
  %47 = bitcast i64* %_77.sroa.647.0..sroa_idx48 to [3 x { i8*, i64* }]**
  store [3 x { i8*, i64* }]* %_84, [3 x { i8*, i64* }]** %47, align 8
  %_77.sroa.7.0..sroa_idx50 = getelementptr inbounds %"core::option::Option<core::fmt::Arguments>", %"core::option::Option<core::fmt::Arguments>"* %_76, i64 0, i32 1, i64 4
  store i64 3, i64* %_77.sroa.7.0..sroa_idx50, align 8
; invoke core::panicking::assert_failed
  invoke fastcc void @_ZN4core9panicking13assert_failed17he718f771b6582cb2E(i8 noundef 0, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %35, i64* noalias noundef nonnull readonly align 8 dereferenceable(8) %refcnt, %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef nonnull dereferenceable(48) %_76, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc475 to %"core::panic::location::Location"*)) #22
          to label %unreachable unwind label %cleanup

bb27:                                             ; preds = %bb12.i.i.i.i.i.i, %bb4.i.i, %bb22
  %_5.not.i.i.i = icmp eq i8 %.0.i.i.i.i.i.i, 0
  br i1 %_5.not.i.i.i, label %bb2.i.i.i, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

bb2.i.i.i:                                        ; preds = %bb27
  %48 = load atomic i64, i64* getelementptr inbounds (%"core::sync::atomic::AtomicUsize", %"core::sync::atomic::AtomicUsize"* @_ZN3std9panicking11panic_count18GLOBAL_PANIC_COUNT17hf9f9ac73a64ff9c9E, i64 0, i32 0) monotonic, align 8, !noalias !626
  %_1.i.i.i.i.i.i21 = and i64 %48, 9223372036854775807
  %49 = icmp eq i64 %_1.i.i.i.i.i.i21, 0
  br i1 %49, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i

_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i: ; preds = %bb2.i.i.i
; call std::panicking::panic_count::is_zero_slow_path
  %50 = call noundef zeroext i1 @_ZN3std9panicking11panic_count17is_zero_slow_path17hc18bae4b1910c9f6E(), !noalias !626
  br i1 %50, label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, label %bb5.i.i.i

bb5.i.i.i:                                        ; preds = %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i
  store atomic i8 1, i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 4) monotonic, align 4, !noalias !626
  br label %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i

_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i: ; preds = %bb5.i.i.i, %_ZN3std6thread9panicking17hde6eaa60063263e2E.exit.i.i.i, %bb2.i.i.i, %bb27
  %51 = atomicrmw xchg i32* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to i32*), i32 0 release, align 4, !noalias !626
  %52 = icmp eq i32 %51, 2
  br i1 %52, label %bb2.i.i.i.i, label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

bb2.i.i.i.i:                                      ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i
; call std::sys::unix::locks::futex::Mutex::wake
  call void @_ZN3std3sys4unix5locks5futex5Mutex4wake17hcf5ba1fdaffa4cb3E(%"std::sys::unix::locks::futex::Mutex"* noundef nonnull align 4 dereferenceable(4) bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 0) to %"std::sys::unix::locks::futex::Mutex"*)), !noalias !626
  br label %"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit"

"_ZN4core3ptr131drop_in_place$LT$std..sync..mutex..MutexGuard$LT$std..collections..hash..map..HashMap$LT$i64$C$fixsanitizer..ObjectInfo$GT$$GT$$GT$17h3feacdb19f5b1a02E.exit": ; preds = %_ZN3std4sync6poison4Flag4done17he650c88cb33d2a4cE.exit.i.i, %bb2.i.i.i.i
  call void @llvm.lifetime.end.p0i8(i64 16, i8* nonnull %2)
  ret void

bb25:                                             ; preds = %bb22
  call void @llvm.experimental.noalias.scope.decl(metadata !629)
  call void @llvm.experimental.noalias.scope.decl(metadata !632) #23
  call void @llvm.experimental.noalias.scope.decl(metadata !635) #23
  %_5.idx.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 8) to i64*), align 8, !alias.scope !638, !noalias !639
  %_5.idx1.val.i.i.i = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 16) to i64*), align 8, !alias.scope !638, !noalias !639
  %53 = xor i64 %_5.idx.val.i.i.i, 8317987319222330741
  %54 = xor i64 %_5.idx1.val.i.i.i, 7237128888997146477
  %55 = xor i64 %_5.idx.val.i.i.i, 7816392313619706465
  %56 = xor i64 %obj_id.val, %_5.idx1.val.i.i.i
  %57 = xor i64 %56, 8387220255154660723
  %58 = add i64 %54, %53
  %59 = call i64 @llvm.fshl.i64(i64 %54, i64 %54, i64 13) #23
  %60 = xor i64 %58, %59
  %61 = call i64 @llvm.fshl.i64(i64 %58, i64 %58, i64 32) #23
  %62 = add i64 %57, %55
  %63 = call i64 @llvm.fshl.i64(i64 %57, i64 %57, i64 16) #23
  %64 = xor i64 %63, %62
  %65 = add i64 %64, %61
  %66 = call i64 @llvm.fshl.i64(i64 %64, i64 %64, i64 21) #23
  %67 = xor i64 %66, %65
  %68 = add i64 %60, %62
  %69 = call i64 @llvm.fshl.i64(i64 %60, i64 %60, i64 17) #23
  %70 = xor i64 %68, %69
  %71 = call i64 @llvm.fshl.i64(i64 %68, i64 %68, i64 32) #23
  %72 = xor i64 %65, %obj_id.val
  %73 = xor i64 %67, 576460752303423488
  %74 = add i64 %72, %70
  %75 = call i64 @llvm.fshl.i64(i64 %70, i64 %70, i64 13) #23
  %76 = xor i64 %74, %75
  %77 = call i64 @llvm.fshl.i64(i64 %74, i64 %74, i64 32) #23
  %78 = add i64 %73, %71
  %79 = call i64 @llvm.fshl.i64(i64 %67, i64 %73, i64 16) #23
  %80 = xor i64 %79, %78
  %81 = add i64 %80, %77
  %82 = call i64 @llvm.fshl.i64(i64 %80, i64 %80, i64 21) #23
  %83 = xor i64 %82, %81
  %84 = add i64 %78, %76
  %85 = call i64 @llvm.fshl.i64(i64 %76, i64 %76, i64 17) #23
  %86 = xor i64 %84, %85
  %87 = call i64 @llvm.fshl.i64(i64 %84, i64 %84, i64 32) #23
  %88 = xor i64 %81, 576460752303423488
  %89 = xor i64 %87, 255
  %90 = add i64 %88, %86
  %91 = call i64 @llvm.fshl.i64(i64 %86, i64 %86, i64 13) #23
  %92 = xor i64 %90, %91
  %93 = call i64 @llvm.fshl.i64(i64 %90, i64 %90, i64 32) #23
  %94 = add i64 %83, %89
  %95 = call i64 @llvm.fshl.i64(i64 %83, i64 %83, i64 16) #23
  %96 = xor i64 %95, %94
  %97 = add i64 %96, %93
  %98 = call i64 @llvm.fshl.i64(i64 %96, i64 %96, i64 21) #23
  %99 = xor i64 %98, %97
  %100 = add i64 %92, %94
  %101 = call i64 @llvm.fshl.i64(i64 %92, i64 %92, i64 17) #23
  %102 = xor i64 %100, %101
  %103 = call i64 @llvm.fshl.i64(i64 %100, i64 %100, i64 32) #23
  %104 = add i64 %102, %97
  %105 = call i64 @llvm.fshl.i64(i64 %102, i64 %102, i64 13) #23
  %106 = xor i64 %105, %104
  %107 = call i64 @llvm.fshl.i64(i64 %104, i64 %104, i64 32) #23
  %108 = add i64 %99, %103
  %109 = call i64 @llvm.fshl.i64(i64 %99, i64 %99, i64 16) #23
  %110 = xor i64 %109, %108
  %111 = add i64 %110, %107
  %112 = call i64 @llvm.fshl.i64(i64 %110, i64 %110, i64 21) #23
  %113 = xor i64 %112, %111
  %114 = add i64 %106, %108
  %115 = call i64 @llvm.fshl.i64(i64 %106, i64 %106, i64 17) #23
  %116 = xor i64 %115, %114
  %117 = call i64 @llvm.fshl.i64(i64 %114, i64 %114, i64 32) #23
  %118 = add i64 %116, %111
  %119 = call i64 @llvm.fshl.i64(i64 %116, i64 %116, i64 13) #23
  %120 = xor i64 %119, %118
  %121 = add i64 %113, %117
  %122 = call i64 @llvm.fshl.i64(i64 %113, i64 %113, i64 16) #23
  %123 = xor i64 %122, %121
  %124 = call i64 @llvm.fshl.i64(i64 %123, i64 %123, i64 21) #23
  %125 = add i64 %120, %121
  %126 = call i64 @llvm.fshl.i64(i64 %120, i64 %120, i64 17) #23
  %127 = call i64 @llvm.fshl.i64(i64 %125, i64 %125, i64 32) #23
  %_17.i.i.i.i.i.i.i.i = xor i64 %125, %124
  %128 = xor i64 %_17.i.i.i.i.i.i.i.i, %126
  %129 = xor i64 %128, %127
  call void @llvm.experimental.noalias.scope.decl(metadata !643) #23
  call void @llvm.experimental.noalias.scope.decl(metadata !646) #23
  call void @llvm.experimental.noalias.scope.decl(metadata !649) #23
  %top7.i.i.i.i.i.i.i = lshr i64 %129, 57
  %130 = trunc i64 %top7.i.i.i.i.i.i.i to i8
  %_6.i.i.i.i.i.i.i22 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 24) to i64*), align 8, !alias.scope !652, !noalias !655
  %self.idx.val.i.i.i.i.i.i = load i8*, i8** bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 32) to i8**), align 8, !alias.scope !658, !noalias !655
  %.0.vec.insert.i.i.i.i.i.i.i.i.i = insertelement <16 x i8> undef, i8 %130, i64 0
  %.15.vec.insert.i.i.i.i.i.i.i.i.i = shufflevector <16 x i8> %.0.vec.insert.i.i.i.i.i.i.i.i.i, <16 x i8> poison, <16 x i32> zeroinitializer
  %_12.idx.val3.i.i.cast.i.i.i.i.i = bitcast i8* %self.idx.val.i.i.i.i.i.i to { i64, %ObjectInfo }*
  br label %bb3.i.i.i.i.i.i23

bb3.i.i.i.i.i.i23:                                ; preds = %bb21.i.i.i.i.i.i, %bb25
  %probe_seq.sroa.7.0.i.i.i.i.i.i = phi i64 [ 0, %bb25 ], [ %143, %bb21.i.i.i.i.i.i ]
  %.pn.i.i.i = phi i64 [ %129, %bb25 ], [ %144, %bb21.i.i.i.i.i.i ]
  %probe_seq.sroa.0.0.i.i.i.i.i.i = and i64 %.pn.i.i.i, %_6.i.i.i.i.i.i.i22
  %131 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %probe_seq.sroa.0.0.i.i.i.i.i.i
  %132 = bitcast i8* %131 to <16 x i8>*
  %.0.copyload.i9.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %132, align 1, !noalias !659
  %133 = icmp eq <16 x i8> %.15.vec.insert.i.i.i.i.i.i.i.i.i, %.0.copyload.i9.i.i.i.i.i.i
  %134 = bitcast <16 x i1> %133 to i16
  br label %bb8.i.i.i.i.i.i

bb8.i.i.i.i.i.i:                                  ; preds = %bb10.i.i.i.i.i.i, %bb3.i.i.i.i.i.i23
  %iter.0.i.i.i.i.i.i = phi i16 [ %134, %bb3.i.i.i.i.i.i23 ], [ %_2.i.i.i.i.i.i.i.i, %bb10.i.i.i.i.i.i ]
  %135 = icmp eq i16 %iter.0.i.i.i.i.i.i, 0
  br i1 %135, label %bb12.i.i.i.i.i.i, label %bb10.i.i.i.i.i.i

bb12.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %136 = icmp eq <16 x i8> %.0.copyload.i9.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %137 = bitcast <16 x i1> %136 to i16
  %.not.i.i.i.i.i.i = icmp eq i16 %137, 0
  br i1 %.not.i.i.i.i.i.i, label %bb21.i.i.i.i.i.i, label %bb27

bb10.i.i.i.i.i.i:                                 ; preds = %bb8.i.i.i.i.i.i
  %138 = call i16 @llvm.cttz.i16(i16 %iter.0.i.i.i.i.i.i, i1 true) #23, !range !27
  %_2.i.i.i.i.i.i.i.i.i = zext i16 %138 to i64
  %_4.i.i.i.i.i.i.i.i = add i16 %iter.0.i.i.i.i.i.i, -1
  %_2.i.i.i.i.i.i.i.i = and i16 %_4.i.i.i.i.i.i.i.i, %iter.0.i.i.i.i.i.i
  %_25.i.i.i.i.i.i = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %_2.i.i.i.i.i.i.i.i.i
  %index.i.i.i.i.i.i = and i64 %_25.i.i.i.i.i.i, %_6.i.i.i.i.i.i.i22
  %139 = sub i64 0, %index.i.i.i.i.i.i
  %140 = getelementptr inbounds { i64, %ObjectInfo }, { i64, %ObjectInfo }* %_12.idx.val3.i.i.cast.i.i.i.i.i, i64 %139, i32 0
  %141 = getelementptr inbounds i64, i64* %140, i64 -4
  %_6.idx.val.i.i.i.i.i.i.i = load i64, i64* %141, align 8, !noalias !662
  %142 = icmp eq i64 %_6.idx.val.i.i.i.i.i.i.i, %obj_id.val
  br i1 %142, label %bb4.i.i.i.i, label %bb8.i.i.i.i.i.i

bb21.i.i.i.i.i.i:                                 ; preds = %bb12.i.i.i.i.i.i
  %143 = add i64 %probe_seq.sroa.7.0.i.i.i.i.i.i, 16
  %144 = add i64 %probe_seq.sroa.0.0.i.i.i.i.i.i, %143
  br label %bb3.i.i.i.i.i.i23

bb4.i.i.i.i:                                      ; preds = %bb10.i.i.i.i.i.i
  call void @llvm.experimental.noalias.scope.decl(metadata !665) #23
  call void @llvm.experimental.noalias.scope.decl(metadata !668) #23
  %145 = ptrtoint i8* %self.idx.val.i.i.i.i.i.i to i64
  %146 = ptrtoint i64* %140 to i64
  %147 = sub i64 %145, %146
  %148 = ashr exact i64 %147, 5
  call void @llvm.experimental.noalias.scope.decl(metadata !671) #23
  %149 = add nsw i64 %148, -16
  %index_before.i.i.i.i.i.i.i = and i64 %149, %_6.i.i.i.i.i.i.i22
  %150 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index_before.i.i.i.i.i.i.i
  %151 = bitcast i8* %150 to <16 x i8>*
  %.0.copyload.i17.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %151, align 1, !noalias !674
  %152 = icmp eq <16 x i8> %.0.copyload.i17.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %153 = bitcast <16 x i1> %152 to i16
  %154 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %148
  %155 = bitcast i8* %154 to <16 x i8>*
  %.0.copyload.i418.i.i.i.i.i.i.i = load <16 x i8>, <16 x i8>* %155, align 1, !noalias !678
  %156 = icmp eq <16 x i8> %.0.copyload.i418.i.i.i.i.i.i.i, <i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1, i8 -1>
  %157 = bitcast <16 x i1> %156 to i16
  %158 = call i16 @llvm.ctlz.i16(i16 %153, i1 false) #23, !range !27
  %159 = call i16 @llvm.cttz.i16(i16 %157, i1 false) #23, !range !27
  %narrow.i.i.i.i.i.i.i = add nuw nsw i16 %159, %158
  %_20.i.i.i.i.i.i.i = icmp ugt i16 %narrow.i.i.i.i.i.i.i, 15
  br i1 %_20.i.i.i.i.i.i.i, label %bb4.i.i, label %bb11.i.i.i.i.i.i.i

bb11.i.i.i.i.i.i.i:                               ; preds = %bb4.i.i.i.i
  %160 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !681, !noalias !682
  %161 = add i64 %160, 1
  store i64 %161, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 40) to i64*), align 8, !alias.scope !681, !noalias !682
  br label %bb4.i.i

bb4.i.i:                                          ; preds = %bb11.i.i.i.i.i.i.i, %bb4.i.i.i.i
  %.sink20.i.i.i.i.i.i.i = phi i8 [ -1, %bb11.i.i.i.i.i.i.i ], [ -128, %bb4.i.i.i.i ]
  %index2.i.i.i.i.i.i.i.i = add i64 %index_before.i.i.i.i.i.i.i, 16
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %154, align 1, !noalias !683
  %162 = getelementptr inbounds i8, i8* %self.idx.val.i.i.i.i.i.i, i64 %index2.i.i.i.i.i.i.i.i
  store i8 %.sink20.i.i.i.i.i.i.i, i8* %162, align 1, !noalias !683
  %163 = load i64, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !681, !noalias !682
  %164 = add i64 %163, -1
  store i64 %164, i64* bitcast (i8* getelementptr inbounds (<{ [16 x i8], [56 x i8], i8* }>, <{ [16 x i8], [56 x i8], i8* }>* @_ZN12fixsanitizer12OBJECT_TABLE17h1cadd8a5b35fe57eE, i64 0, i32 1, i64 48) to i64*), align 8, !alias.scope !681, !noalias !682
  br label %bb27

abort:                                            ; preds = %cleanup
  %165 = landingpad { i8*, i32 }
          cleanup
; call core::panicking::panic_no_unwind
  call void @_ZN4core9panicking15panic_no_unwind17h911e42a789e66c4eE() #25
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
declare void @llvm.assume(i1 noundef) #18

; core::panicking::panic
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core9panicking5panic17hab046c3856b52f65E([0 x i8]* noalias noundef nonnull readonly align 1, i64, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #4

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i16 @llvm.ctlz.i16(i16, i1 immarg) #19

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i16 @llvm.cttz.i16(i16, i1 immarg) #19

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.fshl.i64(i64, i64, i64) #19

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i64, i1 } @llvm.uadd.with.overflow.i64(i64, i64) #19

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare { i64, i1 } @llvm.umul.with.overflow.i64(i64, i64) #19

; Function Attrs: mustprogress nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.ctlz.i64(i64, i1 immarg) #19

; <std::sys_common::mutex::MovableMutex as core::ops::drop::Drop>::drop
; Function Attrs: nonlazybind uwtable
declare void @"_ZN78_$LT$std..sys_common..mutex..MovableMutex$u20$as$u20$core..ops..drop..Drop$GT$4drop17h6fa46602d8642d5cE"(%"std::sys_common::mutex::MovableMutex"* noalias noundef align 4 dereferenceable(4)) unnamed_addr #6

; <std::thread::local::AccessError as core::fmt::Debug>::fmt
; Function Attrs: nonlazybind uwtable
declare noundef zeroext i1 @"_ZN68_$LT$std..thread..local..AccessError$u20$as$u20$core..fmt..Debug$GT$3fmt17h514ef917cd5ecc1bE"(%"std::thread::local::AccessError"* noalias noundef nonnull readonly align 1, %"core::fmt::Formatter"* noalias noundef align 8 dereferenceable(64)) unnamed_addr #6

; core::result::unwrap_failed
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core6result13unwrap_failed17h995262f85f9c4e2cE([0 x i8]* noalias noundef nonnull readonly align 1, i64, {}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #4

; core::panicking::assert_failed_inner
; Function Attrs: noreturn nonlazybind uwtable
declare void @_ZN4core9panicking19assert_failed_inner17h36469c68b6fc10f1E(i8 noundef, {}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), {}* noundef nonnull align 1, [3 x i64]* noalias noundef readonly align 8 dereferenceable(24), %"core::option::Option<core::fmt::Arguments>"* noalias nocapture noundef dereferenceable(48), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #16

; alloc::alloc::handle_alloc_error
; Function Attrs: cold noreturn nonlazybind uwtable
declare void @_ZN5alloc5alloc18handle_alloc_error17h4913beb2b71b29d1E(i64, i64 noundef) unnamed_addr #10

; Function Attrs: nofree nounwind nonlazybind uwtable
declare noalias i8* @__rust_alloc(i64, i64) unnamed_addr #20

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

; Function Attrs: inaccessiblememonly nofree nosync nounwind willreturn
declare void @llvm.experimental.noalias.scope.decl(metadata) #21

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
attributes #18 = { inaccessiblememonly mustprogress nofree nosync nounwind willreturn }
attributes #19 = { mustprogress nofree nosync nounwind readnone speculatable willreturn }
attributes #20 = { nofree nounwind nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #21 = { inaccessiblememonly nofree nosync nounwind willreturn }
attributes #22 = { noreturn }
attributes #23 = { nounwind }
attributes #24 = { noinline }
attributes #25 = { noinline noreturn nounwind }

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
!121 = distinct !{!121, !122, !"_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E: %_1"}
!122 = distinct !{!122, !"_ZN4core3ops8function6FnOnce9call_once17h1ea5565e2dea7545E"}
!123 = !{!124}
!124 = distinct !{!124, !125, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E: %_1"}
!125 = distinct !{!125, !"_ZN9once_cell3imp17OnceCell$LT$T$GT$10initialize28_$u7b$$u7b$closure$u7d$$u7d$17hc865372cb27826c8E"}
!126 = !{!124, !121}
!127 = !{!128, !130, !132}
!128 = distinct !{!128, !129, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E: %dest"}
!129 = distinct !{!129, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E"}
!130 = distinct !{!130, !131, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E: %self"}
!131 = distinct !{!131, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E"}
!132 = distinct !{!132, !133, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE: %val"}
!133 = distinct !{!133, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE"}
!134 = !{!135}
!135 = distinct !{!135, !136, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: %_1"}
!136 = distinct !{!136, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE"}
!137 = !{!138}
!138 = distinct !{!138, !139, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: %_1"}
!139 = distinct !{!139, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E"}
!140 = !{!138, !135}
!141 = !{!142, !143, !124, !121}
!142 = distinct !{!142, !139, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: argument 0"}
!143 = distinct !{!143, !136, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: argument 0"}
!144 = !{!145}
!145 = distinct !{!145, !146, !"_ZN4core3mem7replace17he877d779398bb476E: %dest"}
!146 = distinct !{!146, !"_ZN4core3mem7replace17he877d779398bb476E"}
!147 = !{!142, !138, !143, !135, !124, !121}
!148 = !{!135, !124, !121}
!149 = !{!150}
!150 = distinct !{!150, !151, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E: %self"}
!151 = distinct !{!151, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E"}
!152 = !{!153, !150}
!153 = distinct !{!153, !154, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!154 = distinct !{!154, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!155 = !{!156}
!156 = distinct !{!156, !157, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE: %self"}
!157 = distinct !{!157, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE"}
!158 = !{!159}
!159 = distinct !{!159, !160, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!160 = distinct !{!160, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!161 = !{!159, !156, !150}
!162 = !{!159, !156, !150, !124, !121}
!163 = !{!164}
!164 = distinct !{!164, !165, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE: argument 0"}
!165 = distinct !{!165, !"_ZN12fixsanitizer9OBJECT_ID28_$u7b$$u7b$closure$u7d$$u7d$17h1e07f88a35090f7aE"}
!166 = !{!167}
!167 = distinct !{!167, !168, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E: argument 0"}
!168 = distinct !{!168, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hf1817c1ce1a82502E"}
!169 = !{!167, !164}
!170 = !{!171}
!171 = distinct !{!171, !172, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E: argument 0"}
!172 = distinct !{!172, !"_ZN12fixsanitizer12OBJECT_TABLE28_$u7b$$u7b$closure$u7d$$u7d$17h19014ce56e4f8c81E"}
!173 = !{!174}
!174 = distinct !{!174, !175, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE: argument 0"}
!175 = distinct !{!175, !"_ZN96_$LT$std..collections..hash..map..HashMap$LT$K$C$V$C$S$GT$$u20$as$u20$core..default..Default$GT$7default17h2145ccba0138e17fE"}
!176 = !{!177, !179, !181, !174, !171}
!177 = distinct !{!177, !178, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE: %init"}
!178 = distinct !{!178, !"_ZN3std6thread5local4fast12Key$LT$T$GT$3get17h616dedf3656d81adE"}
!179 = distinct !{!179, !180, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E: %init"}
!180 = distinct !{!180, !"_ZN3std11collections4hash3map11RandomState3new4KEYS7__getit17hc9e8b35f5d8d2cb9E"}
!181 = distinct !{!181, !182, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE: argument 0"}
!182 = distinct !{!182, !"_ZN3std6thread5local17LocalKey$LT$T$GT$8try_with17h6283c6b6f8ba717aE"}
!183 = !{!181, !174, !171}
!184 = !{!174, !171}
!185 = !{!186}
!186 = distinct !{!186, !187, !"_ZN4core3mem7replace17h3116444c89fcbd6bE: %dest"}
!187 = distinct !{!187, !"_ZN4core3mem7replace17h3116444c89fcbd6bE"}
!188 = !{!189, !174}
!189 = distinct !{!189, !190, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17hb01b02706bcc63abE: argument 0"}
!190 = distinct !{!190, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$11with_hasher17hb01b02706bcc63abE"}
!191 = !{!192}
!192 = distinct !{!192, !193, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hbc7cbddf8870e563E: argument 0"}
!193 = distinct !{!193, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hbc7cbddf8870e563E"}
!194 = !{!195}
!195 = distinct !{!195, !193, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$3new17hbc7cbddf8870e563E: %t"}
!196 = !{!192, !195, !171}
!197 = !{!192, !171}
!198 = !{!192, !195}
!199 = !{!200}
!200 = distinct !{!200, !201, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E: %self"}
!201 = distinct !{!201, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E"}
!202 = !{!203, !200}
!203 = distinct !{!203, !204, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!204 = distinct !{!204, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!205 = !{!206}
!206 = distinct !{!206, !207, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE: %self"}
!207 = distinct !{!207, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE"}
!208 = !{!209}
!209 = distinct !{!209, !210, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!210 = distinct !{!210, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!211 = !{!209, !206, !200}
!212 = !{!213}
!213 = distinct !{!213, !214, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!214 = distinct !{!214, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!215 = !{!216}
!216 = distinct !{!216, !217, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!217 = distinct !{!217, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!218 = !{!219, !221}
!219 = distinct !{!219, !220, !"_ZN4core3mem7replace17h788e58c37a635438E: %dest"}
!220 = distinct !{!220, !"_ZN4core3mem7replace17h788e58c37a635438E"}
!221 = distinct !{!221, !222, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE: %self"}
!222 = distinct !{!222, !"_ZN4core6option15Option$LT$T$GT$4take17h43e6886a5efc7f1cE"}
!223 = !{!224}
!224 = distinct !{!224, !225, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE: %x.0"}
!225 = distinct !{!225, !"_ZN5alloc5boxed12Box$LT$T$GT$3new17h40997283247b445bE"}
!226 = !{!227}
!227 = distinct !{!227, !228, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E: %self"}
!228 = distinct !{!228, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$20reserve_rehash_inner17h1cfb6975afad2257E"}
!229 = !{!230}
!230 = distinct !{!230, !231, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E: %self"}
!231 = distinct !{!231, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12resize_inner17heaaf9a1b84a1f1e4E"}
!232 = !{i64 0, i64 65}
!233 = !{!234, !236, !238, !230, !227}
!234 = distinct !{!234, !235, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE: argument 0"}
!235 = distinct !{!235, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$17new_uninitialized17h198cc3e39c258a1fE"}
!236 = distinct !{!236, !237, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E: argument 0"}
!237 = distinct !{!237, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$22fallible_with_capacity17h9cdf1e7c36b04ea6E"}
!238 = distinct !{!238, !239, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E: argument 0"}
!239 = distinct !{!239, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize17h5f33b94da90ae327E"}
!240 = !{!241, !236, !238, !230, !227}
!241 = distinct !{!241, !242, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E: argument 0"}
!242 = distinct !{!242, !"_ZN4core6option15Option$LT$T$GT$10ok_or_else17h95eb470a54480279E"}
!243 = !{!236, !238, !230, !227}
!244 = !{!230, !227}
!245 = !{!246, !248, !249, !251, !230, !227}
!246 = distinct !{!246, !247, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %x"}
!247 = distinct !{!247, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E"}
!248 = distinct !{!248, !247, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y:thread"}
!249 = distinct !{!249, !250, !"_ZN4core3mem4swap17h8292e61c571debd1E: %x"}
!250 = distinct !{!250, !"_ZN4core3mem4swap17h8292e61c571debd1E"}
!251 = distinct !{!251, !250, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y:thread"}
!252 = !{!253}
!253 = distinct !{!253, !254, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!254 = distinct !{!254, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!255 = !{!256, !258, !230, !227}
!256 = distinct !{!256, !257, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %_1"}
!257 = distinct !{!257, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE"}
!258 = distinct !{!258, !257, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %table"}
!259 = !{!260, !262, !264, !230, !227}
!260 = distinct !{!260, !261, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!261 = distinct !{!261, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!262 = distinct !{!262, !263, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!263 = distinct !{!263, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!264 = distinct !{!264, !265, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E: %self"}
!265 = distinct !{!265, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$19prepare_insert_slot17h76f309793d276c59E"}
!266 = !{!262, !264, !230, !227}
!267 = !{!268, !270, !262, !264, !230, !227}
!268 = distinct !{!268, !269, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!269 = distinct !{!269, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!270 = distinct !{!270, !271, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!271 = distinct !{!271, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!272 = !{!273, !275, !264, !230, !227}
!273 = distinct !{!273, !274, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!274 = distinct !{!274, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!275 = distinct !{!275, !276, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!276 = distinct !{!276, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!277 = !{!246, !278, !249, !279, !230, !227}
!278 = distinct !{!278, !247, !"_ZN4core3mem11swap_simple17h83890a786a04c2d2E: %y"}
!279 = distinct !{!279, !250, !"_ZN4core3mem4swap17h8292e61c571debd1E: %y"}
!280 = !{!281, !283, !285, !230, !227}
!281 = distinct !{!281, !282, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!282 = distinct !{!282, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!283 = distinct !{!283, !284, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E: %self_"}
!284 = distinct !{!284, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$14prepare_resize28_$u7b$$u7b$closure$u7d$$u7d$17hd2260e223080a513E"}
!285 = distinct !{!285, !286, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E: %self"}
!286 = distinct !{!286, !"_ZN88_$LT$hashbrown..scopeguard..ScopeGuard$LT$T$C$F$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h02f03726d4c0ba48E"}
!287 = !{!288}
!288 = distinct !{!288, !289, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E: %self"}
!289 = distinct !{!289, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15rehash_in_place17h8020e735b3b7b2b0E"}
!290 = !{!291}
!291 = distinct !{!291, !292, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E: %self"}
!292 = distinct !{!292, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$23prepare_rehash_in_place17h1fe4f61c5ace1438E"}
!293 = !{!291, !288, !227}
!294 = !{!295, !297, !291, !288, !227}
!295 = distinct !{!295, !296, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!296 = distinct !{!296, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!297 = distinct !{!297, !298, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!298 = distinct !{!298, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!299 = !{!300, !291, !288, !227}
!300 = distinct !{!300, !301, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE: %a"}
!301 = distinct !{!301, !"_ZN4core9core_arch3x864sse215_mm_store_si12817h9c6e7b64ac890fbbE"}
!302 = !{!288, !227}
!303 = !{!304}
!304 = distinct !{!304, !305, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!305 = distinct !{!305, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!306 = !{!307, !309, !288, !227}
!307 = distinct !{!307, !308, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %_1"}
!308 = distinct !{!308, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE"}
!309 = distinct !{!309, !308, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$14reserve_rehash28_$u7b$$u7b$closure$u7d$$u7d$17hd39cc8ba54ce6afeE: %table"}
!310 = !{!311, !313, !288, !227}
!311 = distinct !{!311, !312, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!312 = distinct !{!312, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!313 = distinct !{!313, !314, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!314 = distinct !{!314, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!315 = !{!313, !288, !227}
!316 = !{!317, !319, !313, !288, !227}
!317 = distinct !{!317, !318, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!318 = distinct !{!318, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!319 = distinct !{!319, !320, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!320 = distinct !{!320, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!321 = !{!322, !324, !288, !227}
!322 = distinct !{!322, !323, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!323 = distinct !{!323, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!324 = distinct !{!324, !325, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!325 = distinct !{!325, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!326 = !{!327, !288, !227}
!327 = distinct !{!327, !328, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E: %self"}
!328 = distinct !{!328, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$15replace_ctrl_h217h28f2613ce7dd2cb0E"}
!329 = !{!330, !332, !327, !288, !227}
!330 = distinct !{!330, !331, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!331 = distinct !{!331, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!332 = distinct !{!332, !333, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!333 = distinct !{!333, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!334 = !{!335, !337, !338, !339, !340, !341, !342, !343, !344, !345, !346, !347, !348, !349, !350, !351}
!335 = distinct !{!335, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It15"}
!336 = distinct !{!336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE"}
!337 = distinct !{!337, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It14"}
!338 = distinct !{!338, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It13"}
!339 = distinct !{!339, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It12"}
!340 = distinct !{!340, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It11"}
!341 = distinct !{!341, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It10"}
!342 = distinct !{!342, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It9"}
!343 = distinct !{!343, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It8"}
!344 = distinct !{!344, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It7"}
!345 = distinct !{!345, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It6"}
!346 = distinct !{!346, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It5"}
!347 = distinct !{!347, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It4"}
!348 = distinct !{!348, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It3"}
!349 = distinct !{!349, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It2"}
!350 = distinct !{!350, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It1"}
!351 = distinct !{!351, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x"}
!352 = !{!353, !354, !355, !356, !357, !358, !359, !360, !361, !362, !363, !364, !365, !366, !367, !368}
!353 = distinct !{!353, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It15"}
!354 = distinct !{!354, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It14"}
!355 = distinct !{!355, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It13"}
!356 = distinct !{!356, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It12"}
!357 = distinct !{!357, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It11"}
!358 = distinct !{!358, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It10"}
!359 = distinct !{!359, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It9"}
!360 = distinct !{!360, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It8"}
!361 = distinct !{!361, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It7"}
!362 = distinct !{!362, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It6"}
!363 = distinct !{!363, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It5"}
!364 = distinct !{!364, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It4"}
!365 = distinct !{!365, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It3"}
!366 = distinct !{!366, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It2"}
!367 = distinct !{!367, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It1"}
!368 = distinct !{!368, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y"}
!369 = !{!370, !371, !372, !373, !374, !375, !376, !377, !378, !379, !380, !381, !382, !383, !384, !385}
!370 = distinct !{!370, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It31"}
!371 = distinct !{!371, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It30"}
!372 = distinct !{!372, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It29"}
!373 = distinct !{!373, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It28"}
!374 = distinct !{!374, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It27"}
!375 = distinct !{!375, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It26"}
!376 = distinct !{!376, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It25"}
!377 = distinct !{!377, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It24"}
!378 = distinct !{!378, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It23"}
!379 = distinct !{!379, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It22"}
!380 = distinct !{!380, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It21"}
!381 = distinct !{!381, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It20"}
!382 = distinct !{!382, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It19"}
!383 = distinct !{!383, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It18"}
!384 = distinct !{!384, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It17"}
!385 = distinct !{!385, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %x:It16"}
!386 = !{!387, !388, !389, !390, !391, !392, !393, !394, !395, !396, !397, !398, !399, !400, !401, !402}
!387 = distinct !{!387, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It31"}
!388 = distinct !{!388, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It30"}
!389 = distinct !{!389, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It29"}
!390 = distinct !{!390, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It28"}
!391 = distinct !{!391, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It27"}
!392 = distinct !{!392, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It26"}
!393 = distinct !{!393, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It25"}
!394 = distinct !{!394, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It24"}
!395 = distinct !{!395, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It23"}
!396 = distinct !{!396, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It22"}
!397 = distinct !{!397, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It21"}
!398 = distinct !{!398, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It20"}
!399 = distinct !{!399, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It19"}
!400 = distinct !{!400, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It18"}
!401 = distinct !{!401, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It17"}
!402 = distinct !{!402, !336, !"_ZN4core3mem11swap_simple17h83bb422d1d703b9bE: %y:It16"}
!403 = !{!404, !288, !227}
!404 = distinct !{!404, !405, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!405 = distinct !{!405, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!406 = !{!407, !409, !411}
!407 = distinct !{!407, !408, !"_ZN4core3mem7replace17ha318695de15894dbE: %dest"}
!408 = distinct !{!408, !"_ZN4core3mem7replace17ha318695de15894dbE"}
!409 = distinct !{!409, !410, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E: %self"}
!410 = distinct !{!410, !"_ZN4core6option15Option$LT$T$GT$4take17h51e4eb8f5630ab19E"}
!411 = distinct !{!411, !412, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E: %val"}
!412 = distinct !{!412, !"_ZN9once_cell14take_unchecked17h8d99e23a054003c4E"}
!413 = !{!414}
!414 = distinct !{!414, !415, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: %_1"}
!415 = distinct !{!415, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE"}
!416 = !{!417}
!417 = distinct !{!417, !418, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: %_1"}
!418 = distinct !{!418, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE"}
!419 = !{!417, !414}
!420 = !{!421, !422}
!421 = distinct !{!421, !418, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17h0c9bedad0f38d45cE: argument 0"}
!422 = distinct !{!422, !415, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hcbd903f8de56affdE: argument 0"}
!423 = !{!424}
!424 = distinct !{!424, !425, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E: %dest"}
!425 = distinct !{!425, !"_ZN4core3mem7replace17hbfcf19dcc153ef97E"}
!426 = !{!421, !417, !422, !414}
!427 = !{!428, !430, !432}
!428 = distinct !{!428, !429, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E: %dest"}
!429 = distinct !{!429, !"_ZN4core3mem7replace17h534dbd68f5b0bbb9E"}
!430 = distinct !{!430, !431, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E: %self"}
!431 = distinct !{!431, !"_ZN4core6option15Option$LT$T$GT$4take17h63506b1f0eb101b6E"}
!432 = distinct !{!432, !433, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE: %val"}
!433 = distinct !{!433, !"_ZN9once_cell14take_unchecked17h767ec4f418178d0bE"}
!434 = !{!435}
!435 = distinct !{!435, !436, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: %_1"}
!436 = distinct !{!436, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE"}
!437 = !{!438}
!438 = distinct !{!438, !439, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: %_1"}
!439 = distinct !{!439, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E"}
!440 = !{!438, !435}
!441 = !{!442, !443}
!442 = distinct !{!442, !439, !"_ZN9once_cell4sync17Lazy$LT$T$C$F$GT$5force28_$u7b$$u7b$closure$u7d$$u7d$17haf3068eb45d993b1E: argument 0"}
!443 = distinct !{!443, !436, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init28_$u7b$$u7b$closure$u7d$$u7d$17hf02ba4f5fe573b3fE: argument 0"}
!444 = !{!445}
!445 = distinct !{!445, !446, !"_ZN4core3mem7replace17he877d779398bb476E: %dest"}
!446 = distinct !{!446, !"_ZN4core3mem7replace17he877d779398bb476E"}
!447 = !{!442, !438, !443, !435}
!448 = !{!449}
!449 = distinct !{!449, !450, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E: %self"}
!450 = distinct !{!450, !"_ZN79_$LT$hashbrown..raw..RawTable$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17h4b8171a598f676b7E"}
!451 = !{!452, !449}
!452 = distinct !{!452, !453, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE: %self"}
!453 = distinct !{!453, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$18is_empty_singleton17ha2d24a3b65a3ed0dE"}
!454 = !{!455}
!455 = distinct !{!455, !456, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE: %self"}
!456 = distinct !{!456, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12free_buckets17he862c26eb1aca55dE"}
!457 = !{!458}
!458 = distinct !{!458, !459, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E: %self"}
!459 = distinct !{!459, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$12free_buckets17h2b2eaf192e49cd01E"}
!460 = !{!458, !455, !449}
!461 = !{!462, !464}
!462 = distinct !{!462, !463, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E: %f"}
!463 = distinct !{!463, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hff3518b8f483c941E"}
!464 = distinct !{!464, !465, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E: %f"}
!465 = distinct !{!465, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17habbaba1fa2aa69a3E"}
!466 = !{!467}
!467 = distinct !{!467, !468, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE: argument 0"}
!468 = distinct !{!468, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hcb70c30dc68d33ffE"}
!469 = !{!470, !467}
!470 = distinct !{!470, !471, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E: argument 0"}
!471 = distinct !{!471, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17hff628d5b9f077f42E"}
!472 = !{!473}
!473 = distinct !{!473, !474, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E: %self"}
!474 = distinct !{!474, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17h9d7f59184fcf6511E"}
!475 = !{!476, !478}
!476 = distinct !{!476, !477, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE: %f"}
!477 = distinct !{!477, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE"}
!478 = distinct !{!478, !479, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E: %f"}
!479 = distinct !{!479, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E"}
!480 = !{!481}
!481 = distinct !{!481, !482, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E: argument 0"}
!482 = distinct !{!482, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E"}
!483 = !{!484, !481}
!484 = distinct !{!484, !485, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE: argument 0"}
!485 = distinct !{!485, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE"}
!486 = !{!487, !489}
!487 = distinct !{!487, !488, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: %self"}
!488 = distinct !{!488, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE"}
!489 = distinct !{!489, !488, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: argument 1"}
!490 = !{!487}
!491 = !{!492}
!492 = distinct !{!492, !493, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E: %self"}
!493 = distinct !{!493, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E"}
!494 = !{!495}
!495 = distinct !{!495, !496, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE: %self"}
!496 = distinct !{!496, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE"}
!497 = !{!495, !492}
!498 = !{!499, !500, !501, !502}
!499 = distinct !{!499, !496, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE: argument 0"}
!500 = distinct !{!500, !496, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6insert17hb9e918798952addfE: %v"}
!501 = distinct !{!501, !493, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E: argument 0"}
!502 = distinct !{!502, !493, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6insert17hd4aaf4002631dde7E: %v"}
!503 = !{!504}
!504 = distinct !{!504, !505, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h23367aad273c1206E: %self"}
!505 = distinct !{!505, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$7get_mut17h23367aad273c1206E"}
!506 = !{!507}
!507 = distinct !{!507, !508, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: %self"}
!508 = distinct !{!508, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E"}
!509 = !{!510}
!510 = distinct !{!510, !511, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!511 = distinct !{!511, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!512 = !{!513, !510, !507, !504, !495, !492}
!513 = distinct !{!513, !514, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!514 = distinct !{!514, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!515 = !{!516, !499, !500, !501, !502}
!516 = distinct !{!516, !508, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: argument 1"}
!517 = !{!507, !504, !495, !492}
!518 = !{!519, !510, !507, !516, !504, !499, !495, !500, !501, !492, !502}
!519 = distinct !{!519, !520, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!520 = distinct !{!520, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!521 = !{!522, !510, !507, !516, !504, !499, !495, !500, !501, !492, !502}
!522 = distinct !{!522, !523, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E: %_1"}
!523 = distinct !{!523, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E"}
!524 = !{!525}
!525 = distinct !{!525, !526, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE: %self"}
!526 = distinct !{!526, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE"}
!527 = !{!528, !530, !525, !532, !533, !499, !495, !500, !501, !492, !502}
!528 = distinct !{!528, !529, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!529 = distinct !{!529, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!530 = distinct !{!530, !531, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!531 = distinct !{!531, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!532 = distinct !{!532, !526, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE: %value"}
!533 = distinct !{!533, !526, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6insert17hfca125ca8cac617cE: %hasher"}
!534 = !{!530, !525, !532, !533, !499, !495, !500, !501, !492, !502}
!535 = !{!536, !538, !530, !525, !532, !533, !499, !495, !500, !501, !492, !502}
!536 = distinct !{!536, !537, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!537 = distinct !{!537, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!538 = distinct !{!538, !539, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!539 = distinct !{!539, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!540 = !{!525, !532, !533, !499, !495, !500, !501, !492, !502}
!541 = !{!525, !495, !492}
!542 = !{!532, !533, !499, !500, !501, !502}
!543 = !{!544}
!544 = distinct !{!544, !545, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E: %self"}
!545 = distinct !{!545, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$16find_insert_slot17h685eb579c1301109E"}
!546 = !{!547, !544, !525, !495, !492}
!547 = distinct !{!547, !548, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!548 = distinct !{!548, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!549 = !{!544, !525, !495, !492}
!550 = !{!551, !544, !525, !532, !499, !500, !501, !502}
!551 = distinct !{!551, !552, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!552 = distinct !{!552, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!553 = !{!544, !525, !532, !499, !500, !501, !502}
!554 = !{!555, !557, !544, !525, !532, !499, !500, !501, !502}
!555 = distinct !{!555, !556, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E: argument 0"}
!556 = distinct !{!556, !"_ZN4core9core_arch3x864sse214_mm_load_si12817h1fad3d8e6c601785E"}
!557 = distinct !{!557, !558, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E: argument 0"}
!558 = distinct !{!558, !"_ZN9hashbrown3raw4sse25Group12load_aligned17h73f057345d31e000E"}
!559 = !{!560}
!560 = distinct !{!560, !561, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E: %self"}
!561 = distinct !{!561, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$21record_item_insert_at17h5ffb8d3929fef937E"}
!562 = !{!563, !565, !560, !525, !532, !499, !500, !501, !502}
!563 = distinct !{!563, !564, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E: %self"}
!564 = distinct !{!564, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$8set_ctrl17h9bbfd698d932a711E"}
!565 = distinct !{!565, !566, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE: %self"}
!566 = distinct !{!566, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$11set_ctrl_h217he44f55e71eec496bE"}
!567 = !{!560, !525, !495, !492}
!568 = !{!525, !499, !500, !501, !502}
!569 = !{!501}
!570 = !{!571}
!571 = distinct !{!571, !572, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!572 = distinct !{!572, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!573 = !{!574}
!574 = distinct !{!574, !575, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE: %self"}
!575 = distinct !{!575, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hd6cb165fa4c0658dE"}
!576 = !{!577, !579}
!577 = distinct !{!577, !578, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE: %f"}
!578 = distinct !{!578, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE"}
!579 = distinct !{!579, !580, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E: %f"}
!580 = distinct !{!580, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E"}
!581 = !{!582}
!582 = distinct !{!582, !583, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E: argument 0"}
!583 = distinct !{!583, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E"}
!584 = !{!585, !582}
!585 = distinct !{!585, !586, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE: argument 0"}
!586 = distinct !{!586, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE"}
!587 = !{!588, !590}
!588 = distinct !{!588, !589, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: %self"}
!589 = distinct !{!589, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE"}
!590 = distinct !{!590, !589, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: argument 1"}
!591 = !{!588}
!592 = !{!593}
!593 = distinct !{!593, !594, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE: argument 0"}
!594 = distinct !{!594, !"_ZN4core4hash11BuildHasher8hash_one17h3950263e7bd14e9aE"}
!595 = !{!596}
!596 = distinct !{!596, !597, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!597 = distinct !{!597, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!598 = !{!599, !600}
!599 = distinct !{!599, !597, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!600 = distinct !{!600, !597, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!601 = !{!602}
!602 = distinct !{!602, !603, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!603 = distinct !{!603, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!604 = !{!605, !607}
!605 = distinct !{!605, !606, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE: %f"}
!606 = distinct !{!606, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$15get_or_try_init17hf18fae118442207cE"}
!607 = distinct !{!607, !608, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E: %f"}
!608 = distinct !{!608, !"_ZN9once_cell4sync17OnceCell$LT$T$GT$11get_or_init17h7827e20255db77a7E"}
!609 = !{!610}
!610 = distinct !{!610, !611, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E: argument 0"}
!611 = distinct !{!611, !"_ZN3std4sync5mutex14Mutex$LT$T$GT$4lock17hf52d91529eb7c375E"}
!612 = !{!613, !610}
!613 = distinct !{!613, !614, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE: argument 0"}
!614 = distinct !{!614, !"_ZN3std4sync5mutex19MutexGuard$LT$T$GT$3new17h7cf125ba114cc85aE"}
!615 = !{!616, !618}
!616 = distinct !{!616, !617, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: %self"}
!617 = distinct !{!617, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE"}
!618 = distinct !{!618, !617, !"_ZN4core6result19Result$LT$T$C$E$GT$6unwrap17hb18fe679fa61ed1cE: argument 1"}
!619 = !{!616}
!620 = !{!621}
!621 = distinct !{!621, !622, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!622 = distinct !{!622, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!623 = !{!624, !625}
!624 = distinct !{!624, !622, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!625 = distinct !{!625, !622, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %args.0"}
!626 = !{!627}
!627 = distinct !{!627, !628, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE: %self"}
!628 = distinct !{!628, !"_ZN79_$LT$std..sync..mutex..MutexGuard$LT$T$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17hf0c0eb7ff03f6f6cE"}
!629 = !{!630}
!630 = distinct !{!630, !631, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h07c9a48d0726e1afE: %self"}
!631 = distinct !{!631, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h07c9a48d0726e1afE"}
!632 = !{!633}
!633 = distinct !{!633, !634, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h467dae58b8e28e55E: %self"}
!634 = distinct !{!634, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h467dae58b8e28e55E"}
!635 = !{!636}
!636 = distinct !{!636, !637, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h8a63ae6c0f3b74a7E: %self"}
!637 = distinct !{!637, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h8a63ae6c0f3b74a7E"}
!638 = !{!636, !633, !630}
!639 = !{!640, !641, !642}
!640 = distinct !{!640, !637, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$12remove_entry17h8a63ae6c0f3b74a7E: argument 0"}
!641 = distinct !{!641, !634, !"_ZN9hashbrown3map28HashMap$LT$K$C$V$C$S$C$A$GT$6remove17h467dae58b8e28e55E: argument 0"}
!642 = distinct !{!642, !631, !"_ZN3std11collections4hash3map24HashMap$LT$K$C$V$C$S$GT$6remove17h07c9a48d0726e1afE: argument 0"}
!643 = !{!644}
!644 = distinct !{!644, !645, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17head61d0e4749a2cfE: %self"}
!645 = distinct !{!645, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17head61d0e4749a2cfE"}
!646 = !{!647}
!647 = distinct !{!647, !648, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: %self"}
!648 = distinct !{!648, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E"}
!649 = !{!650}
!650 = distinct !{!650, !651, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE: %self"}
!651 = distinct !{!651, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$10find_inner17h0f0af99a8220acaeE"}
!652 = !{!653, !650, !647, !644, !636, !633, !630}
!653 = distinct !{!653, !654, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE: %self"}
!654 = distinct !{!654, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$9probe_seq17hac1ccc2c90e1713bE"}
!655 = !{!656, !657, !640, !641, !642}
!656 = distinct !{!656, !648, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find17h57e4127dbf3b8522E: argument 1"}
!657 = distinct !{!657, !645, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$12remove_entry17head61d0e4749a2cfE: argument 0"}
!658 = !{!647, !644, !636, !633, !630}
!659 = !{!660, !650, !647, !656, !657, !644, !640, !636, !641, !633, !642, !630}
!660 = distinct !{!660, !661, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!661 = distinct !{!661, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!662 = !{!663, !650, !647, !656, !657, !644, !640, !636, !641, !633, !642, !630}
!663 = distinct !{!663, !664, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E: %_1"}
!664 = distinct !{!664, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$4find28_$u7b$$u7b$closure$u7d$$u7d$17ha4e45fc553af7f14E"}
!665 = !{!666}
!666 = distinct !{!666, !667, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17h12237f430f8cfaadE: %self"}
!667 = distinct !{!667, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17h12237f430f8cfaadE"}
!668 = !{!669}
!669 = distinct !{!669, !670, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h0cfad37b6833ba5fE: %self"}
!670 = distinct !{!670, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$13erase_no_drop17h0cfad37b6833ba5fE"}
!671 = !{!672}
!672 = distinct !{!672, !673, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E: %self"}
!673 = distinct !{!673, !"_ZN9hashbrown3raw22RawTableInner$LT$A$GT$5erase17h16e5e0ae5ca7e891E"}
!674 = !{!675, !672, !669, !677, !666, !657, !644, !640, !636, !641, !633, !642, !630}
!675 = distinct !{!675, !676, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!676 = distinct !{!676, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!677 = distinct !{!677, !667, !"_ZN9hashbrown3raw21RawTable$LT$T$C$A$GT$6remove17h12237f430f8cfaadE: argument 0"}
!678 = !{!679, !672, !669, !677, !666, !657, !644, !640, !636, !641, !633, !642, !630}
!679 = distinct !{!679, !680, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E: argument 0"}
!680 = distinct !{!680, !"_ZN9hashbrown3raw4sse25Group4load17h09d27dce32d3e709E"}
!681 = !{!672, !669, !666, !644, !636, !633, !630}
!682 = !{!677, !657, !640, !641, !642}
!683 = !{!672, !669, !677, !666, !657, !644, !640, !636, !641, !633, !642, !630}
