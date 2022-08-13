; ModuleID = 'fixsanitizer.0bcff7bf-cgu.0'
source_filename = "fixsanitizer.0bcff7bf-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%"core::fmt::Arguments" = type { { [0 x { [0 x i8]*, i64 }]*, i64 }, { i64*, i64 }, { [0 x { i8*, i64* }]*, i64 } }
%"core::panic::location::Location" = type { { [0 x i8]*, i64 }, i32, i32 }

@alloc78 = private unnamed_addr constant <{}> zeroinitializer, align 8
@alloc3 = private unnamed_addr constant <{ [15 x i8] }> <{ [15 x i8] c"Hello runtime!\0A" }>, align 1
@alloc4 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [15 x i8] }>, <{ [15 x i8] }>* @alloc3, i32 0, i32 0, i32 0), [8 x i8] c"\0F\00\00\00\00\00\00\00" }>, align 8
@alloc47 = private unnamed_addr constant <{ [36 x i8] }> <{ [36 x i8] c"Object with refcnt zero is retained!" }>, align 1
@alloc48 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [36 x i8] }>, <{ [36 x i8] }>* @alloc47, i32 0, i32 0, i32 0), [8 x i8] c"$\00\00\00\00\00\00\00" }>, align 8
@alloc83 = private unnamed_addr constant <{ [10 x i8] }> <{ [10 x i8] c"src/lib.rs" }>, align 1
@alloc82 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc83, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\001\00\00\00\09\00\00\00" }>, align 8
@alloc75 = private unnamed_addr constant <{ [36 x i8] }> <{ [36 x i8] c"Object with refcnt zero is released!" }>, align 1
@alloc76 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [36 x i8] }>, <{ [36 x i8] }>* @alloc75, i32 0, i32 0, i32 0), [8 x i8] c"$\00\00\00\00\00\00\00" }>, align 8
@alloc84 = private unnamed_addr constant <{ i8*, [16 x i8] }> <{ i8* getelementptr inbounds (<{ [10 x i8] }>, <{ [10 x i8] }>* @alloc83, i32 0, i32 0, i32 0), [16 x i8] c"\0A\00\00\00\00\00\00\00@\00\00\00\09\00\00\00" }>, align 8

; Function Attrs: nonlazybind uwtable
define void @hello_runtime() unnamed_addr #0 {
start:
  %_2 = alloca %"core::fmt::Arguments", align 8
  %0 = bitcast %"core::fmt::Arguments"* %_2 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %0)
  %1 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_2, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc4 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %1, align 8, !alias.scope !2, !noalias !5
  %2 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_2, i64 0, i32 0, i32 1
  store i64 1, i64* %2, align 8, !alias.scope !2, !noalias !5
  %3 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_2, i64 0, i32 1, i32 0
  store i64* null, i64** %3, align 8, !alias.scope !2, !noalias !5
  %4 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_2, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{}>* @alloc78 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %4, align 8, !alias.scope !2, !noalias !5
  %5 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_2, i64 0, i32 2, i32 1
  store i64 0, i64* %5, align 8, !alias.scope !2, !noalias !5
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_2)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %0)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind nonlazybind readnone uwtable willreturn
define void @report_malloc(i8* nocapture readnone %address) unnamed_addr #1 {
start:
  ret void
}

; Function Attrs: nonlazybind uwtable
define void @report_retain(i8* nocapture readnone %address, i64 %refcnt) unnamed_addr #0 {
start:
  %_4 = alloca %"core::fmt::Arguments", align 8
  %0 = icmp eq i64 %refcnt, 0
  br i1 %0, label %bb1, label %bb3

bb1:                                              ; preds = %start
  %1 = bitcast %"core::fmt::Arguments"* %_4 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1)
  %2 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc48 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %2, align 8, !alias.scope !7, !noalias !10
  %3 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 0, i32 1
  store i64 1, i64* %3, align 8, !alias.scope !7, !noalias !10
  %4 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 1, i32 0
  store i64* null, i64** %4, align 8, !alias.scope !7, !noalias !10
  %5 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{}>* @alloc78 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %5, align 8, !alias.scope !7, !noalias !10
  %6 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 2, i32 1
  store i64 0, i64* %6, align 8, !alias.scope !7, !noalias !10
; call core::panicking::panic_fmt
  call void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_4, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc82 to %"core::panic::location::Location"*)) #4
  unreachable

bb3:                                              ; preds = %start
  ret void
}

; Function Attrs: nonlazybind uwtable
define void @report_release(i8* nocapture readnone %address, i64 %refcnt) unnamed_addr #0 {
start:
  %_4 = alloca %"core::fmt::Arguments", align 8
  %0 = icmp eq i64 %refcnt, 0
  br i1 %0, label %bb1, label %bb3

bb1:                                              ; preds = %start
  %1 = bitcast %"core::fmt::Arguments"* %_4 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %1)
  %2 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc76 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %2, align 8, !alias.scope !12, !noalias !15
  %3 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 0, i32 1
  store i64 1, i64* %3, align 8, !alias.scope !12, !noalias !15
  %4 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 1, i32 0
  store i64* null, i64** %4, align 8, !alias.scope !12, !noalias !15
  %5 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{}>* @alloc78 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %5, align 8, !alias.scope !12, !noalias !15
  %6 = getelementptr inbounds %"core::fmt::Arguments", %"core::fmt::Arguments"* %_4, i64 0, i32 2, i32 1
  store i64 0, i64* %6, align 8, !alias.scope !12, !noalias !15
; call core::panicking::panic_fmt
  call void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef nonnull dereferenceable(48) %_4, %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24) bitcast (<{ i8*, [16 x i8] }>* @alloc84 to %"core::panic::location::Location"*)) #4
  unreachable

bb3:                                              ; preds = %start
  ret void
}

; Function Attrs: argmemonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.lifetime.start.p0i8(i64 immarg, i8* nocapture) #2

; core::panicking::panic_fmt
; Function Attrs: cold noinline noreturn nonlazybind uwtable
declare void @_ZN4core9panicking9panic_fmt17h741cfbfc95bc6112E(%"core::fmt::Arguments"* noalias nocapture noundef dereferenceable(48), %"core::panic::location::Location"* noalias noundef readonly align 8 dereferenceable(24)) unnamed_addr #3

; Function Attrs: argmemonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.lifetime.end.p0i8(i64 immarg, i8* nocapture) #2

; std::io::stdio::_print
; Function Attrs: nonlazybind uwtable
declare void @_ZN3std2io5stdio6_print17hf80401c345fb19f3E(%"core::fmt::Arguments"* noalias nocapture noundef dereferenceable(48)) unnamed_addr #0

attributes #0 = { nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #1 = { mustprogress nofree norecurse nosync nounwind nonlazybind readnone uwtable willreturn "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #2 = { argmemonly mustprogress nofree nosync nounwind willreturn }
attributes #3 = { cold noinline noreturn nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #4 = { noreturn }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 7, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{!3}
!3 = distinct !{!3, !4, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!4 = distinct !{!4, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!5 = !{!6}
!6 = distinct !{!6, !4, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!7 = !{!8}
!8 = distinct !{!8, !9, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!9 = distinct !{!9, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!10 = !{!11}
!11 = distinct !{!11, !9, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
!12 = !{!13}
!13 = distinct !{!13, !14, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: argument 0"}
!14 = distinct !{!14, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E"}
!15 = !{!16}
!16 = distinct !{!16, !14, !"_ZN4core3fmt9Arguments6new_v117hc426d01f280ffe99E: %pieces.0"}
