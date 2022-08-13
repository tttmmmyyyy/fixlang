; ModuleID = 'fixruntime.2895a438-cgu.0'
source_filename = "fixruntime.2895a438-cgu.0"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

%"std::fmt::Arguments" = type { { [0 x { [0 x i8]*, i64 }]*, i64 }, { i64*, i64 }, { [0 x { i8*, i64* }]*, i64 } }

@alloc2 = private unnamed_addr constant <{ [0 x i8] }> zeroinitializer, align 8
@alloc4 = private unnamed_addr constant <{ [15 x i8] }> <{ [15 x i8] c"Hello runtime!\0A" }>, align 1
@alloc5 = private unnamed_addr constant <{ i8*, [8 x i8] }> <{ i8* getelementptr inbounds (<{ [15 x i8] }>, <{ [15 x i8] }>* @alloc4, i32 0, i32 0, i32 0), [8 x i8] c"\0F\00\00\00\00\00\00\00" }>, align 8

; Function Attrs: nonlazybind uwtable
define void @hello_runtime() unnamed_addr #0 {
start:
  %_2 = alloca %"std::fmt::Arguments", align 8
  %0 = bitcast %"std::fmt::Arguments"* %_2 to i8*
  call void @llvm.lifetime.start.p0i8(i64 48, i8* nonnull %0)
  %1 = getelementptr inbounds %"std::fmt::Arguments", %"std::fmt::Arguments"* %_2, i64 0, i32 0, i32 0
  store [0 x { [0 x i8]*, i64 }]* bitcast (<{ i8*, [8 x i8] }>* @alloc5 to [0 x { [0 x i8]*, i64 }]*), [0 x { [0 x i8]*, i64 }]** %1, align 8, !alias.scope !2
  %2 = getelementptr inbounds %"std::fmt::Arguments", %"std::fmt::Arguments"* %_2, i64 0, i32 0, i32 1
  store i64 1, i64* %2, align 8, !alias.scope !2
  %3 = getelementptr inbounds %"std::fmt::Arguments", %"std::fmt::Arguments"* %_2, i64 0, i32 1, i32 0
  store i64* null, i64** %3, align 8, !alias.scope !2
  %4 = getelementptr inbounds %"std::fmt::Arguments", %"std::fmt::Arguments"* %_2, i64 0, i32 2, i32 0
  store [0 x { i8*, i64* }]* bitcast (<{ [0 x i8] }>* @alloc2 to [0 x { i8*, i64* }]*), [0 x { i8*, i64* }]** %4, align 8, !alias.scope !2
  %5 = getelementptr inbounds %"std::fmt::Arguments", %"std::fmt::Arguments"* %_2, i64 0, i32 2, i32 1
  store i64 0, i64* %5, align 8, !alias.scope !2
; call std::io::stdio::_print
  call void @_ZN3std2io5stdio6_print17ha2acac7a448afbe2E(%"std::fmt::Arguments"* noalias nocapture nonnull dereferenceable(48) %_2)
  call void @llvm.lifetime.end.p0i8(i64 48, i8* nonnull %0)
  ret void
}

; Function Attrs: argmemonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.lifetime.start.p0i8(i64 immarg, i8* nocapture) #1

; Function Attrs: argmemonly mustprogress nofree nosync nounwind willreturn
declare void @llvm.lifetime.end.p0i8(i64 immarg, i8* nocapture) #1

; std::io::stdio::_print
; Function Attrs: nonlazybind uwtable
declare void @_ZN3std2io5stdio6_print17ha2acac7a448afbe2E(%"std::fmt::Arguments"* noalias nocapture dereferenceable(48)) unnamed_addr #0

attributes #0 = { nonlazybind uwtable "probe-stack"="__rust_probestack" "target-cpu"="x86-64" }
attributes #1 = { argmemonly mustprogress nofree nosync nounwind willreturn }

!llvm.module.flags = !{!0, !1}

!0 = !{i32 7, !"PIC Level", i32 2}
!1 = !{i32 2, !"RtLibUseGOT", i32 1}
!2 = !{!3}
!3 = distinct !{!3, !4, !"_ZN4core3fmt9Arguments6new_v117hbdb2878a72b61ff4E: argument 0"}
!4 = distinct !{!4, !"_ZN4core3fmt9Arguments6new_v117hbdb2878a72b61ff4E"}
