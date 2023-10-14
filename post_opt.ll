; ModuleID = 'Main'
source_filename = "Main"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@"GlobalVar#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3" = local_unnamed_addr global { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3" = local_unnamed_addr global i8 0
@"GlobalVar#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c" = local_unnamed_addr global { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c" = local_unnamed_addr global i8 0
@"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590" = local_unnamed_addr global { { void ({ i8 }, i8*, i8*)*, i8* } } zeroinitializer
@"InitFlag#Main::main#18e42987d75046f070b34a440ad41590" = local_unnamed_addr global i8 0
@"GlobalVar#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d" = local_unnamed_addr global { void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d" = local_unnamed_addr global i8 0
@"GlobalVar#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd" = local_unnamed_addr global { void ({ i8 }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd" = local_unnamed_addr global i8 0
@"GlobalVar#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e" = local_unnamed_addr global { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e" = local_unnamed_addr global i8 0
@"GlobalVar#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31" = local_unnamed_addr global { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31" = local_unnamed_addr global i8 0
@"GlobalVar#Std::String::@_data#0ab050cccd1520fd77505875edc888e7" = local_unnamed_addr global { i8* ({ i8* }, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::String::@_data#0ab050cccd1520fd77505875edc888e7" = local_unnamed_addr global i8 0
@"GlobalVar#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a" = local_unnamed_addr global { void ({ i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a" = local_unnamed_addr global i8 0
@"GlobalVar#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289" = local_unnamed_addr global { void (i8*, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289" = local_unnamed_addr global i8 0
@"GlobalVar#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a" = local_unnamed_addr global { void ({ i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a" = local_unnamed_addr global i8 0
@0 = private unnamed_addr constant [13 x i8] c"Hello World!\00", align 1
@1 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1

define void @retain_obj(i8* %0) local_unnamed_addr {
  %2 = bitcast i8* %0 to { i64 }*
  %3 = getelementptr inbounds { i64 }, { i64 }* %2, i32 0, i32 0
  %4 = load i64, i64* %3, align 8
  %5 = add nsw i64 %4, 1
  store i64 %5, i64* %3, align 8
  ret void
}

define void @release_obj(i8* %0, void (i8*)* %1) local_unnamed_addr {
  %3 = bitcast i8* %0 to { i64 }*
  %4 = getelementptr inbounds { i64 }, { i64 }* %3, i32 0, i32 0
  %5 = load i64, i64* %4, align 8
  %6 = sub nsw i64 %5, 1
  store i64 %6, i64* %4, align 8
  %7 = icmp eq i64 %5, 1
  br i1 %7, label %8, label %11

8:                                                ; preds = %2
  fence acquire
  %9 = ptrtoint void (i8*)* %1 to i64
  %10 = icmp eq i64 %9, 0
  br i1 %10, label %12, label %13

11:                                               ; preds = %12, %2
  ret void

12:                                               ; preds = %13, %8
  tail call void @free(i8* %0)
  br label %11

13:                                               ; preds = %8
  tail call void %1(i8* %0)
  br label %12
}

declare void @free(i8*) local_unnamed_addr

define internal void @2({ i8 } %0, i8* %1, i8* %2) {
  %4 = bitcast i8* %1 to { { i64 }, void (i8*)*, { i8 } }*
  %5 = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %4, i32 0, i32 2
  %6 = load { i8 }, { i8 }* %5, align 1
  %7 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %8 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %7, i32 0, i32 1
  %9 = load void (i8*)*, void (i8*)** %8, align 8
  %10 = bitcast i8* %1 to { i64 }*
  %11 = getelementptr inbounds { i64 }, { i64 }* %10, i32 0, i32 0
  %12 = load i64, i64* %11, align 8
  %13 = sub nsw i64 %12, 1
  store i64 %13, i64* %11, align 8
  %14 = icmp eq i64 %12, 1
  br i1 %14, label %15, label %20

15:                                               ; preds = %3
  fence acquire
  %16 = ptrtoint void (i8*)* %9 to i64
  %17 = icmp eq i64 %16, 0
  br i1 %17, label %18, label %19

18:                                               ; preds = %19, %15
  tail call void @free(i8* %1)
  br label %20

19:                                               ; preds = %15
  tail call void %9(i8* %1)
  br label %18

20:                                               ; preds = %3, %18
  %21 = bitcast i8* %2 to { i8 }*
  store { i8 } %6, { i8 }* %21, align 1
  ret void
}

declare noalias i8* @malloc(i32) local_unnamed_addr

define internal void @3(i8* %0) {
  ret void
}

; Function Attrs: argmemonly nofree nosync nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #0

define internal void @4({ i8 } %0, i8* %1, i8* %2) {
  %4 = tail call i8* @malloc(i32 mul (i32 trunc (i64 add (i64 ptrtoint ({ i8 }* getelementptr inbounds ({ { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* null, i32 0, i32 3) to i64), i64 mul (i64 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i64), i64 2)) to i32), i32 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)))
  %5 = bitcast i8* %4 to { { i64 }, i64, i64, { i8 } }*
  %6 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %5, i32 0, i32 0
  %7 = getelementptr inbounds { i64 }, { i64 }* %6, i32 0, i32 0
  store i64 1, i64* %7, align 8
  %8 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %5, i32 0, i32 2
  store i64 2, i64* %8, align 8
  %9 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %5, i32 0, i32 1
  store i64 2, i64* %9, align 8
  %10 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %5, i32 0, i32 3
  %11 = bitcast { i8 }* %10 to i8*
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 1 %11, i8* align 1 getelementptr inbounds ([2 x i8], [2 x i8]* @1, i32 0, i32 0), i64 2, i1 false)
  %12 = insertvalue { i8* } undef, i8* %4, 0
  %13 = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8* } }* getelementptr ({ { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* null, i32 1) to i32))
  %14 = bitcast i8* %13 to { { i64 }, void (i8*)*, { i8* } }*
  %15 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %14, i32 0, i32 0
  %16 = getelementptr inbounds { i64 }, { i64 }* %15, i32 0, i32 0
  store i64 1, i64* %16, align 8
  %17 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %14, i32 0, i32 1
  store void (i8*)* @9, void (i8*)** %17, align 8
  %18 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %14, i32 0, i32 2
  store { i8* } %12, { i8* }* %18, align 8
  %19 = insertvalue { { void ({ i8 }, i8*, i8*)*, i8* } } { { void ({ i8 }, i8*, i8*)*, i8* } { void ({ i8 }, i8*, i8*)* @8, i8* undef } }, i8* %13, 0, 1
  %20 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %21 = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %20, i32 0, i32 0
  %22 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %21, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @7, void ({ i8 }, i8*, i8*)** %22, align 8
  %23 = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* null, i32 1) to i32))
  %24 = bitcast i8* %23 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %25 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %24, i32 0, i32 0
  %26 = getelementptr inbounds { i64 }, { i64 }* %25, i32 0, i32 0
  store i64 1, i64* %26, align 8
  %27 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %24, i32 0, i32 1
  store void (i8*)* @6, void (i8*)** %27, align 8
  %28 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %24, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } { void ({ i8 }, i8*, i8*)* @5, i8* null }, { void ({ i8 }, i8*, i8*)*, i8* }* %28, align 8
  %29 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %24, i32 0, i32 3
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %19, { { void ({ i8 }, i8*, i8*)*, i8* } }* %29, align 8
  %30 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %21, i32 0, i32 1
  store i8* %23, i8** %30, align 8
  ret void
}

define internal void @5({ i8 } %0, i8* %1, i8* %2) {
  %4 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %5 = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %4, i32 0, i32 0
  %6 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %5, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @2, void ({ i8 }, i8*, i8*)** %6, align 8
  %7 = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8 } }* getelementptr ({ { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* null, i32 1) to i32))
  %8 = bitcast i8* %7 to { { i64 }, void (i8*)*, { i8 } }*
  %9 = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %8, i32 0, i32 0
  %10 = getelementptr inbounds { i64 }, { i64 }* %9, i32 0, i32 0
  store i64 1, i64* %10, align 8
  %11 = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %8, i32 0, i32 1
  store void (i8*)* @3, void (i8*)** %11, align 8
  %12 = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %8, i32 0, i32 2
  store { i8 } undef, { i8 }* %12, align 1
  %13 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %5, i32 0, i32 1
  store i8* %7, i8** %13, align 8
  ret void
}

define internal void @6(i8* %0) {
  %2 = bitcast i8* %0 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %2, i32 0, i32 2
  %4 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %3, i32 0, i32 1
  %5 = load i8*, i8** %4, align 8
  %6 = icmp eq i8* %5, null
  br i1 %6, label %21, label %7

7:                                                ; preds = %1
  %8 = bitcast i8* %5 to { { i64 }, void (i8*)* }*
  %9 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %8, i32 0, i32 1
  %10 = load void (i8*)*, void (i8*)** %9, align 8
  %11 = bitcast i8* %5 to { i64 }*
  %12 = getelementptr inbounds { i64 }, { i64 }* %11, i32 0, i32 0
  %13 = load i64, i64* %12, align 8
  %14 = sub nsw i64 %13, 1
  store i64 %14, i64* %12, align 8
  %15 = icmp eq i64 %13, 1
  br i1 %15, label %16, label %21

16:                                               ; preds = %7
  fence acquire
  %17 = ptrtoint void (i8*)* %10 to i64
  %18 = icmp eq i64 %17, 0
  br i1 %18, label %19, label %20

19:                                               ; preds = %20, %16
  tail call void @free(i8* %5)
  br label %21

20:                                               ; preds = %16
  tail call void %10(i8* %5)
  br label %19

21:                                               ; preds = %1, %7, %19
  %22 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %2, i32 0, i32 3
  %23 = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %22, i32 0, i32 0
  %24 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %23, i32 0, i32 1
  %25 = load i8*, i8** %24, align 8
  %26 = icmp eq i8* %25, null
  br i1 %26, label %41, label %27

27:                                               ; preds = %21
  %28 = bitcast i8* %25 to { { i64 }, void (i8*)* }*
  %29 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %28, i32 0, i32 1
  %30 = load void (i8*)*, void (i8*)** %29, align 8
  %31 = bitcast i8* %25 to { i64 }*
  %32 = getelementptr inbounds { i64 }, { i64 }* %31, i32 0, i32 0
  %33 = load i64, i64* %32, align 8
  %34 = sub nsw i64 %33, 1
  store i64 %34, i64* %32, align 8
  %35 = icmp eq i64 %33, 1
  br i1 %35, label %36, label %41

36:                                               ; preds = %27
  fence acquire
  %37 = ptrtoint void (i8*)* %30 to i64
  %38 = icmp eq i64 %37, 0
  br i1 %38, label %39, label %40

39:                                               ; preds = %40, %36
  tail call void @free(i8* %25)
  br label %41

40:                                               ; preds = %36
  tail call void %30(i8* %25)
  br label %39

41:                                               ; preds = %21, %27, %39
  ret void
}

define internal void @7({ i8 } %0, i8* %1, i8* %2) {
  %4 = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %5 = alloca { i8 }, align 8
  %6 = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %7 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %6, i32 0, i32 2
  %8 = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %7, align 8
  %9 = extractvalue { void ({ i8 }, i8*, i8*)*, i8* } %8, 0
  %10 = extractvalue { void ({ i8 }, i8*, i8*)*, i8* } %8, 1
  %11 = icmp eq i8* %10, null
  br i1 %11, label %17, label %12

12:                                               ; preds = %3
  %13 = bitcast i8* %10 to { i64 }*
  %14 = getelementptr inbounds { i64 }, { i64 }* %13, i32 0, i32 0
  %15 = load i64, i64* %14, align 8
  %16 = add nsw i64 %15, 1
  store i64 %16, i64* %14, align 8
  br label %17

17:                                               ; preds = %12, %3
  %18 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %6, i32 0, i32 3
  %19 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %18, align 8
  %20 = extractvalue { { void ({ i8 }, i8*, i8*)*, i8* } } %19, 0, 0
  %21 = extractvalue { { void ({ i8 }, i8*, i8*)*, i8* } } %19, 0, 1
  %22 = icmp eq i8* %21, null
  br i1 %22, label %28, label %23

23:                                               ; preds = %17
  %24 = bitcast i8* %21 to { i64 }*
  %25 = getelementptr inbounds { i64 }, { i64 }* %24, i32 0, i32 0
  %26 = load i64, i64* %25, align 8
  %27 = add nsw i64 %26, 1
  store i64 %27, i64* %25, align 8
  br label %28

28:                                               ; preds = %23, %17
  %29 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %30 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %29, i32 0, i32 1
  %31 = load void (i8*)*, void (i8*)** %30, align 8
  %32 = bitcast i8* %1 to { i64 }*
  %33 = getelementptr inbounds { i64 }, { i64 }* %32, i32 0, i32 0
  %34 = load i64, i64* %33, align 8
  %35 = sub nsw i64 %34, 1
  store i64 %35, i64* %33, align 8
  %36 = icmp eq i64 %34, 1
  br i1 %36, label %37, label %42

37:                                               ; preds = %28
  fence acquire
  %38 = ptrtoint void (i8*)* %31 to i64
  %39 = icmp eq i64 %38, 0
  br i1 %39, label %40, label %41

40:                                               ; preds = %41, %37
  tail call void @free(i8* %1)
  br label %42

41:                                               ; preds = %37
  tail call void %31(i8* %1)
  br label %40

42:                                               ; preds = %28, %40
  %43 = bitcast { i8 }* %5 to i8*
  tail call void %20({ i8 } undef, i8* %21, i8* %43)
  %44 = getelementptr inbounds { i8 }, { i8 }* %5, i32 0, i32 0
  %45 = load i8, i8* %44, align 1
  %46 = insertvalue { i8 } undef, i8 %45, 0
  %47 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %4 to i8*
  tail call void %9({ i8 } %46, i8* %10, i8* %47)
  %48 = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %4, i32 0, i32 0, i32 0
  %49 = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %48, align 8
  %50 = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %4, i32 0, i32 0, i32 1
  %51 = load i8*, i8** %50, align 8
  tail call void %49({ i8 } undef, i8* %51, i8* %2)
  ret void
}

define internal void @8({ i8 } %0, i8* %1, i8* %2) {
  %4 = bitcast i8* %1 to { { i64 }, void (i8*)*, { i8* } }*
  %5 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %4, i32 0, i32 2
  %6 = load { i8* }, { i8* }* %5, align 8
  %7 = extractvalue { i8* } %6, 0
  %8 = bitcast i8* %7 to { i64 }*
  %9 = getelementptr inbounds { i64 }, { i64 }* %8, i32 0, i32 0
  %10 = load i64, i64* %9, align 8
  %11 = add nsw i64 %10, 1
  store i64 %11, i64* %9, align 8
  %12 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %13 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %12, i32 0, i32 1
  %14 = load void (i8*)*, void (i8*)** %13, align 8
  %15 = bitcast i8* %1 to { i64 }*
  %16 = getelementptr inbounds { i64 }, { i64 }* %15, i32 0, i32 0
  %17 = load i64, i64* %16, align 8
  %18 = sub nsw i64 %17, 1
  store i64 %18, i64* %16, align 8
  %19 = icmp eq i64 %17, 1
  br i1 %19, label %20, label %25

20:                                               ; preds = %3
  fence acquire
  %21 = ptrtoint void (i8*)* %14 to i64
  %22 = icmp eq i64 %21, 0
  br i1 %22, label %23, label %24

23:                                               ; preds = %24, %20
  tail call void @free(i8* %1)
  br label %25

24:                                               ; preds = %20
  tail call void %14(i8* %1)
  br label %23

25:                                               ; preds = %3, %23
  %26 = load i64, i64* %9, align 8
  %27 = add nsw i64 %26, 1
  %28 = bitcast i8* %7 to { { i64 }, i64, i64, { i8 } }*
  %29 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %28, i32 0, i32 3
  %30 = bitcast { i8 }* %29 to i8*
  %31 = icmp eq i64 %27, 1
  br i1 %31, label %32, label %33

32:                                               ; preds = %25
  fence acquire
  tail call void @free(i8* %7)
  br label %33

33:                                               ; preds = %32, %25
  %34 = tail call i32 (i8*, ...) @printf(i8* %30)
  %35 = load i64, i64* %9, align 8
  %36 = sub nsw i64 %35, 1
  store i64 %36, i64* %9, align 8
  %37 = icmp eq i64 %35, 1
  br i1 %37, label %38, label %39

38:                                               ; preds = %33
  fence acquire
  tail call void @free(i8* %7)
  br label %39

39:                                               ; preds = %33, %38
  ret void
}

declare i32 @printf(i8*, ...) local_unnamed_addr

define internal void @9(i8* %0) {
  %2 = bitcast i8* %0 to { { i64 }, void (i8*)*, { i8* } }*
  %3 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %2, i32 0, i32 2
  %4 = getelementptr inbounds { i8* }, { i8* }* %3, i32 0, i32 0
  %5 = load i8*, i8** %4, align 8
  %6 = bitcast i8* %5 to { i64 }*
  %7 = getelementptr inbounds { i64 }, { i64 }* %6, i32 0, i32 0
  %8 = load i64, i64* %7, align 8
  %9 = sub nsw i64 %8, 1
  store i64 %9, i64* %7, align 8
  %10 = icmp eq i64 %8, 1
  br i1 %10, label %11, label %12

11:                                               ; preds = %1
  fence acquire
  tail call void @free(i8* %5)
  br label %12

12:                                               ; preds = %1, %11
  ret void
}

define i32 @main() local_unnamed_addr {
  %1 = alloca { i8 }, align 8
  %2 = load i8, i8* @"InitFlag#Main::main#18e42987d75046f070b34a440ad41590", align 1
  %3 = icmp eq i8 %2, 0
  br i1 %3, label %4, label %28

4:                                                ; preds = %0
  %5 = tail call i8* @malloc(i32 mul (i32 trunc (i64 add (i64 ptrtoint ({ i8 }* getelementptr inbounds ({ { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* null, i32 0, i32 3) to i64), i64 mul (i64 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i64), i64 13)) to i32), i32 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)))
  %6 = bitcast i8* %5 to { { i64 }, i64, i64, { i8 } }*
  %7 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %6, i32 0, i32 0
  %8 = getelementptr inbounds { i64 }, { i64 }* %7, i32 0, i32 0
  store i64 1, i64* %8, align 8
  %9 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %6, i32 0, i32 2
  store i64 13, i64* %9, align 8
  %10 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %6, i32 0, i32 1
  store i64 13, i64* %10, align 8
  %11 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %6, i32 0, i32 3
  %12 = bitcast { i8 }* %11 to i8*
  tail call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 1 %12, i8* align 1 getelementptr inbounds ([13 x i8], [13 x i8]* @0, i32 0, i32 0), i64 13, i1 false)
  %13 = insertvalue { i8* } undef, i8* %5, 0
  %14 = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8* } }* getelementptr ({ { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* null, i32 1) to i32))
  %15 = bitcast i8* %14 to { { i64 }, void (i8*)*, { i8* } }*
  %16 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %15, i32 0, i32 0
  %17 = getelementptr inbounds { i64 }, { i64 }* %16, i32 0, i32 0
  store i64 1, i64* %17, align 8
  %18 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %15, i32 0, i32 1
  store void (i8*)* @9, void (i8*)** %18, align 8
  %19 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %15, i32 0, i32 2
  store { i8* } %13, { i8* }* %19, align 8
  %20 = insertvalue { { void ({ i8 }, i8*, i8*)*, i8* } } { { void ({ i8 }, i8*, i8*)*, i8* } { void ({ i8 }, i8*, i8*)* @8, i8* undef } }, i8* %14, 0, 1
  store void ({ i8 }, i8*, i8*)* @7, void ({ i8 }, i8*, i8*)** getelementptr inbounds ({ { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* @"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590", i64 0, i32 0, i32 0), align 8
  %21 = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* null, i32 1) to i32))
  %22 = bitcast i8* %21 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %23 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %22, i32 0, i32 0
  %24 = getelementptr inbounds { i64 }, { i64 }* %23, i32 0, i32 0
  store i64 1, i64* %24, align 8
  %25 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %22, i32 0, i32 1
  store void (i8*)* @6, void (i8*)** %25, align 8
  %26 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %22, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } { void ({ i8 }, i8*, i8*)* @4, i8* null }, { void ({ i8 }, i8*, i8*)*, i8* }* %26, align 8
  %27 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %22, i32 0, i32 3
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %20, { { void ({ i8 }, i8*, i8*)*, i8* } }* %27, align 8
  store i8* %21, i8** getelementptr inbounds ({ { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* @"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590", i64 0, i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Main::main#18e42987d75046f070b34a440ad41590", align 1
  br label %31

28:                                               ; preds = %0
  %29 = load i8*, i8** getelementptr inbounds ({ { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* @"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590", i64 0, i32 0, i32 1), align 8
  %30 = icmp eq i8* %29, null
  br i1 %30, label %37, label %31

31:                                               ; preds = %4, %28
  %32 = phi i8* [ %21, %4 ], [ %29, %28 ]
  %33 = bitcast i8* %32 to { i64 }*
  %34 = getelementptr inbounds { i64 }, { i64 }* %33, i32 0, i32 0
  %35 = load i64, i64* %34, align 8
  %36 = add nsw i64 %35, 1
  store i64 %36, i64* %34, align 8
  br label %37

37:                                               ; preds = %31, %28
  %38 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* @"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590", align 8
  %39 = extractvalue { { void ({ i8 }, i8*, i8*)*, i8* } } %38, 0, 0
  %40 = extractvalue { { void ({ i8 }, i8*, i8*)*, i8* } } %38, 0, 1
  %41 = bitcast { i8 }* %1 to i8*
  tail call void %39({ i8 } undef, i8* %40, i8* %41)
  ret i32 0
}

attributes #0 = { argmemonly nofree nosync nounwind willreturn }
