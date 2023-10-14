; ModuleID = 'Main'
source_filename = "Main"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-unknown-linux-gnu"

@"GlobalVar#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3" = global { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3" = global i8 0
@"GlobalVar#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c" = global { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c" = global i8 0
@"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590" = global { { void ({ i8 }, i8*, i8*)*, i8* } } zeroinitializer
@"InitFlag#Main::main#18e42987d75046f070b34a440ad41590" = global i8 0
@"GlobalVar#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d" = global { void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d" = global i8 0
@"GlobalVar#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd" = global { void ({ i8 }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd" = global i8 0
@"GlobalVar#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e" = global { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e" = global i8 0
@"GlobalVar#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31" = global { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31" = global i8 0
@"GlobalVar#Std::String::@_data#0ab050cccd1520fd77505875edc888e7" = global { i8* ({ i8* }, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::String::@_data#0ab050cccd1520fd77505875edc888e7" = global i8 0
@"GlobalVar#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a" = global { void ({ i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a" = global i8 0
@"GlobalVar#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289" = global { void (i8*, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289" = global i8 0
@"GlobalVar#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a" = global { void ({ i8* }, i8*, i8*)*, i8* } zeroinitializer
@"InitFlag#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a" = global i8 0
@string_literal = private unnamed_addr constant [13 x i8] c"Hello World!\00", align 1
@string_literal.4 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1
@string_literal.13 = private unnamed_addr constant [2 x i8] c"\0A\00", align 1

declare void @abort()

declare void @fixruntime_eprint(i8*, ...)

declare i32 @sprintf(i8*, i8*, ...)

define void @retain_obj(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { i64 }*
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %pointer_cast, i32 0, i32 0
  %1 = load i64, i64* %ptr_to_refcnt, align 8
  %2 = add nsw i64 %1, 1
  store i64 %2, i64* %ptr_to_refcnt, align 8
  ret void
}

define void @release_obj(i8* %0, void (i8*)* %1) {
entry:
  %pointer_cast = bitcast i8* %0 to { i64 }*
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %pointer_cast, i32 0, i32 0
  %2 = load i64, i64* %ptr_to_refcnt, align 8
  %3 = sub nsw i64 %2, 1
  store i64 %3, i64* %ptr_to_refcnt, align 8
  %is_refcnt_zero = icmp eq i64 %2, 1
  br i1 %is_refcnt_zero, label %refcnt_zero_after_release, label %end

refcnt_zero_after_release:                        ; preds = %entry
  fence acquire
  %ptr_to_dtor = ptrtoint void (i8*)* %1 to i64
  %is_dtor_null = icmp eq i64 %ptr_to_dtor, 0
  br i1 %is_dtor_null, label %free, label %call_dtor

end:                                              ; preds = %free, %entry
  ret void

free:                                             ; preds = %call_dtor, %refcnt_zero_after_release
  tail call void @free(i8* %0)
  br label %end

call_dtor:                                        ; preds = %refcnt_zero_after_release
  call void %1(i8* %0)
  br label %free
}

declare void @free(i8*)

define internal void @"closure[Std::#FunPtr1 () (Std::IO ())]"({ i8 } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()]", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8 } }* getelementptr ({ { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_77d94fe6fddb2d6b08c6d888998c5504, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { i8 }, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %pointer_cast2, i32 0, i32 2
  store { i8 } %load_unbox, { i8 }* %ptr_to_field3, align 1
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field5 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field5, align 8
  ret void
}

define internal void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) Std::String Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %0, { i8* } %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { i8* }, align 8
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %0, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  store { i8* } %1, { i8* }* %alloca_for_unboxed_obj1, align 8
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj1, align 8
  %call_lambda = tail call i8* @"closure[Std::#FunPtr1 Std::String (Std::Array Std::U8)]"({ i8* } %load_unbox)
  %load_unbox2 = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %2 to { i32 }*
  %pointer_cast3 = bitcast { i32 }* %pointer_cast to i8*
  tail call void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) (Std::Array Std::U8) Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %load_unbox2, i8* %call_lambda, i8* %pointer_cast3)
  ret void
}

define internal i8* @"closure[Std::#FunPtr1 Std::String (Std::Array Std::U8)]"({ i8* } %0) {
entry:
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value = load i8*, i8** %ptr_to_field, align 8
  ret i8* %field_value
}

define internal void @"closure[Std::#FunPtr1 (Std::Ptr -> Std::I32) (Std::Array Std::U8 -> Std::I32)]"({ void ({ i8* }, i8*, i8*)*, i8* } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %0, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { void (i8*, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void (i8*, i8*, i8*)*, i8* }, { void (i8*, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 0
  store void (i8*, i8*, i8*)* @"closure[Std::Array Std::U8 -> Std::I32]", void (i8*, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_fe3166cacc608f483c86d91be4ff2782, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast2, i32 0, i32 2
  store { void ({ i8* }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %pointer_cast5 = bitcast i8* %1 to { void (i8*, i8*, i8*)*, i8* }*
  %ptr_to_field6 = getelementptr inbounds { void (i8*, i8*, i8*)*, i8* }, { void (i8*, i8*, i8*)*, i8* }* %pointer_cast5, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field6, align 8
  ret void
}

define internal i8* @"Get#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)* @"closure[(Std::Ptr -> Std::I32) -> Std::Array Std::U8 -> Std::I32]", void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)** getelementptr inbounds ({ void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }, { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }, { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Array::borrow_ptr#649bc93c65886245836449c1ab99fab3" to i8*)
}

define internal i8* @"Get#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)* @"closure[(Std::Ptr -> Std::I32) -> Std::String -> Std::I32]", void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)** getelementptr inbounds ({ void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }, { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }, { void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ void ({ i8* }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::String::borrow_c_str#190c09380328de6fd1c381e4c8e9850c" to i8*)
}

define internal i8* @"Get#Main::main#18e42987d75046f070b34a440ad41590"() {
entry:
  %"alloca@allocate_obj" = alloca { i8* }, align 8
  %load_init_flag = load i8, i8* @"InitFlag#Main::main#18e42987d75046f070b34a440ad41590", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  %0 = trunc i64 add (i64 ptrtoint ({ i8 }* getelementptr inbounds ({ { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* null, i32 0, i32 3) to i64), i64 mul (i64 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i64), i64 13)) to i32
  %mallocsize = mul i32 %0, ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)
  %"malloc_array@allocate_obj" = tail call i8* @malloc(i32 %mallocsize)
  %pointer_cast = bitcast i8* %"malloc_array@allocate_obj" to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_size_field = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 2
  store i64 13, i64* %ptr_to_size_field, align 8
  %ptr_to_field = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 1
  store i64 13, i64* %ptr_to_field, align 8
  %ptr_to_field_nocap = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 3
  %1 = bitcast { i8 }* %ptr_to_field_nocap to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 1 %1, i8* align 1 getelementptr inbounds ([13 x i8], [13 x i8]* @string_literal, i32 0, i32 0), i64 13, i1 false)
  %pointer_cast2 = bitcast { { i64 }, i64, i64, { i8 } }* %pointer_cast to i8*
  %ptr_to_field3 = getelementptr inbounds { i8* }, { i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store i8* %pointer_cast2, i8** %ptr_to_field3, align 8
  %load_unbox = load { i8* }, { i8* }* %"alloca@allocate_obj", align 8
  tail call void @"closure[Std::#FunPtr1 Std::String (Std::IO ())]"({ i8* } %load_unbox, i8* bitcast ({ { void ({ i8 }, i8*, i8*)*, i8* } }* @"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590" to i8*))
  store i8 1, i8* @"InitFlag#Main::main#18e42987d75046f070b34a440ad41590", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ { void ({ i8 }, i8*, i8*)*, i8* } }* @"GlobalVar#Main::main#18e42987d75046f070b34a440ad41590" to i8*)
}

define internal void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) (Std::Array Std::U8) Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj1" = alloca { i32 }, align 8
  %"alloca@allocate_obj" = alloca { i8* }, align 8
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %0, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  call void @retain_obj(i8* %1)
  %pointer_cast = bitcast { i8* }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::Array Std::U8) Std::Ptr]"(i8* %1, i8* %pointer_cast)
  %load_unbox = load { i8* }, { i8* }* %"alloca@allocate_obj", align 8
  %ptr_to_field = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value = load void ({ i8* }, i8*, i8*)*, void ({ i8* }, i8*, i8*)** %ptr_to_field, align 8
  %pointer_cast2 = bitcast { i32 }* %"alloca@allocate_obj1" to i8*
  %ptr_to_field3 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 1
  %field_value4 = load i8*, i8** %ptr_to_field3, align 8
  tail call void %field_value({ i8* } %load_unbox, i8* %field_value4, i8* %pointer_cast2)
  call void @release_obj(i8* %1, void (i8*)* @dtor_82a1dcae1a6595589047bba1a0f55e25)
  %load_unbox5 = load { i32 }, { i32 }* %"alloca@allocate_obj1", align 4
  %pointer_cast6 = bitcast i8* %2 to { i32 }*
  store { i32 } %load_unbox5, { i32 }* %pointer_cast6, align 4
  ret void
}

define internal void @"closure[Std::#FunPtr1 Std::String (Std::IO ())]"({ i8* } %0, i8* %1) {
entry:
  %"alloca@allocate_obj2" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> Std::IO ()]", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  store i8* null, i8** %ptr_to_field1, align 8
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj2" to i8*
  tail call void @"closure[Std::#FunPtr1 Std::String (Std::IO ())].1"({ i8* } %load_unbox, i8* %pointer_cast)
  %load_unbox3 = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", align 8
  %load_unbox4 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj2", align 8
  %pointer_cast5 = bitcast i8* %1 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %pointer_cast6 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast5 to i8*
  tail call void @"closure[Std::#FunPtr2 (() -> Std::IO ()) (Std::IO ()) (Std::IO ())]"({ void ({ i8 }, i8*, i8*)*, i8* } %load_unbox3, { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox4, i8* %pointer_cast6)
  ret void
}

define internal void @"closure[Std::#FunPtr1 (() -> Std::IO ()) (Std::IO () -> Std::IO ())]"({ void ({ i8 }, i8*, i8*)*, i8* } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %0, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 0
  store void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)* @"closure[Std::IO () -> Std::IO ()]", void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_626b039d13292bddadb8d093deabe646, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast2, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %pointer_cast5 = bitcast i8* %1 to { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }*
  %ptr_to_field6 = getelementptr inbounds { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* %pointer_cast5, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field6, align 8
  ret void
}

define internal void @"closure[Std::#FunPtr2 (() -> Std::IO ()) (Std::IO ()) (Std::IO ())]"({ void ({ i8 }, i8*, i8*)*, i8* } %0, { { void ({ i8 }, i8*, i8*)*, i8* } } %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %alloca_for_unboxed_obj = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %0, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %1, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()].6", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_6d53b2aad890c6416389a152b34a0482, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast2 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast3 = bitcast i8* %pointer_cast2 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field4 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast3, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field4, align 8
  %load_unbox5 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast6 = bitcast i8* %pointer_cast2 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field7 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast6, i32 0, i32 3
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox5, { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_field7, align 8
  %pointer_cast8 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field9 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast8, i8** %ptr_to_field9, align 8
  ret void
}

define internal i8* @"Get#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)* @"closure[(() -> Std::IO ()) -> Std::IO () -> Std::IO ()]", void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)** getelementptr inbounds ({ void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }, { void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }, { void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ void ({ i8 }, i8*, i8*)*, i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Monad::bind#4f6db82402c69f9b6c5b0b897559280d" to i8*)
}

define internal void @"closure[Std::#FunPtr1 (Std::Array Std::U8) Std::Ptr]"(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_field_nocap = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 3
  %pointer_cast1 = bitcast { i8 }* %ptr_to_field_nocap to i8*
  call void @release_obj(i8* %0, void (i8*)* @dtor_82a1dcae1a6595589047bba1a0f55e25)
  %pointer_cast2 = bitcast i8* %1 to { i8* }*
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %pointer_cast2, i32 0, i32 0
  store i8* %pointer_cast1, i8** %ptr_to_field, align 8
  ret void
}

define internal i8* @"Get#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ i8 }, i8*, i8*)* @"closure[() -> Std::IO ()].9", void ({ i8 }, i8*, i8*)** getelementptr inbounds ({ void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ i8 }, i8*, i8*)*, i8* }* @"GlobalVar#Std::Monad::pure#ed7a95ea8dceca97841a665139e044cd" to i8*)
}

define internal i8* @"Get#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)* @"closure[Std::IO () -> ()]", void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)** getelementptr inbounds ({ void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::_unsafe_perform#2ef258871dd26dd2dff92025ee09b43e" to i8*)
}

define internal i8* @"Get#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)* @"closure[Std::IO () -> () -> ()]", void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)** getelementptr inbounds ({ void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::@_data#f3d2e280b39296daeb4639ac10d3eb31" to i8*)
}

define internal void @"closure[Std::#FunPtr1 (Std::IO ()) (() -> ())]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %0, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, i32 0, i32 0
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, align 8
  %pointer_cast = bitcast i8* %1 to { void ({ i8 }, i8*, i8*)*, i8* }*
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %pointer_cast, align 8
  ret void
}

define internal i8* @"Get#Std::String::@_data#0ab050cccd1520fd77505875edc888e7"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::String::@_data#0ab050cccd1520fd77505875edc888e7", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store i8* ({ i8* }, i8*)* @"closure[Std::String -> Std::Array Std::U8]", i8* ({ i8* }, i8*)** getelementptr inbounds ({ i8* ({ i8* }, i8*)*, i8* }, { i8* ({ i8* }, i8*)*, i8* }* @"GlobalVar#Std::String::@_data#0ab050cccd1520fd77505875edc888e7", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ i8* ({ i8* }, i8*)*, i8* }, { i8* ({ i8* }, i8*)*, i8* }* @"GlobalVar#Std::String::@_data#0ab050cccd1520fd77505875edc888e7", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::String::@_data#0ab050cccd1520fd77505875edc888e7", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ i8* ({ i8* }, i8*)*, i8* }* @"GlobalVar#Std::String::@_data#0ab050cccd1520fd77505875edc888e7" to i8*)
}

define internal void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %0, i8* %1) {
entry:
  %"alloca@allocate_obj1" = alloca { i8 }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %0, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %load_unbox = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) (() -> ())]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox, i8* %pointer_cast)
  %load_unbox2 = load { i8 }, { i8 }* %"alloca@allocate_obj1", align 1
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  %field_value = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %pointer_cast3 = bitcast i8* %1 to { i8 }*
  %pointer_cast4 = bitcast { i8 }* %pointer_cast3 to i8*
  %ptr_to_field5 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  tail call void %field_value({ i8 } %load_unbox2, i8* %field_value6, i8* %pointer_cast4)
  ret void
}

define internal i8* @"Get#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ i8* }, i8*, i8*)* @"closure[Std::String -> Std::IO ()]", void ({ i8* }, i8*, i8*)** getelementptr inbounds ({ void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::println#77a87979eb17715dc4bfee49e8ae989a" to i8*)
}

define internal i8* @"Get#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void (i8*, i8*, i8*)* @"closure[Std::Array Std::U8 -> Std::Ptr]", void (i8*, i8*, i8*)** getelementptr inbounds ({ void (i8*, i8*, i8*)*, i8* }, { void (i8*, i8*, i8*)*, i8* }* @"GlobalVar#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void (i8*, i8*, i8*)*, i8* }, { void (i8*, i8*, i8*)*, i8* }* @"GlobalVar#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void (i8*, i8*, i8*)*, i8* }* @"GlobalVar#Std::Array::_get_ptr#734a0b3d1469c1df62e1a38a4d79e289" to i8*)
}

define internal void @"closure[Std::#FunPtr1 (Std::Ptr -> Std::I32) (Std::String -> Std::I32)]"({ void ({ i8* }, i8*, i8*)*, i8* } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %0, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { void ({ i8* }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 0
  store void ({ i8* }, i8*, i8*)* @"closure[Std::String -> Std::I32].14", void ({ i8* }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_fe3166cacc608f483c86d91be4ff2782, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast2, i32 0, i32 2
  store { void ({ i8* }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %pointer_cast5 = bitcast i8* %1 to { void ({ i8* }, i8*, i8*)*, i8* }*
  %ptr_to_field6 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %pointer_cast5, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field6, align 8
  ret void
}

define internal void @"closure[Std::#FunPtr1 Std::String (Std::IO ())].1"({ i8* } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()].15", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8* } }* getelementptr ({ { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_5a1aaed28630590aeb6e3317f6304e19, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %pointer_cast2, i32 0, i32 2
  store { i8* } %load_unbox, { i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field5 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field5, align 8
  ret void
}

define internal i8* @"Get#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a"() {
entry:
  %load_init_flag = load i8, i8* @"InitFlag#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a", align 1
  %flag_is_zero = icmp eq i8 %load_init_flag, 0
  br i1 %flag_is_zero, label %flag_is_zero1, label %flag_is_nonzero

flag_is_zero1:                                    ; preds = %entry
  store void ({ i8* }, i8*, i8*)* @"closure[Std::String -> Std::IO ()].16", void ({ i8* }, i8*, i8*)** getelementptr inbounds ({ void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a", i32 0, i32 0), align 8
  store i8* null, i8** getelementptr inbounds ({ void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a", i32 0, i32 1), align 8
  store i8 1, i8* @"InitFlag#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a", align 1
  br label %flag_is_nonzero

flag_is_nonzero:                                  ; preds = %flag_is_zero1, %entry
  ret i8* bitcast ({ void ({ i8* }, i8*, i8*)*, i8* }* @"GlobalVar#Std::IO::print#77a87979eb17715dc4bfee49e8ae989a" to i8*)
}

define internal void @"closure[() -> ()]"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { i8 }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %pointer_cast, i32 0, i32 2
  %field_value = load { i8 }, { i8 }* %ptr_to_field, align 1
  store { i8 } %field_value, { i8 }* %alloca_for_unboxed_obj1, align 1
  %pointer_cast2 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast2, i32 0, i32 1
  %field_value4 = load void (i8*)*, void (i8*)** %ptr_to_field3, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value4)
  %pointer_cast5 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast5)
  %load_unbox = load { i8 }, { i8 }* %alloca_for_unboxed_obj1, align 1
  %pointer_cast6 = bitcast i8* %2 to { i8 }*
  store { i8 } %load_unbox, { i8 }* %pointer_cast6, align 1
  ret void
}

define internal void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %0) {
entry:
  ret void
}

declare noalias i8* @malloc(i32)

define internal void @dtor_77d94fe6fddb2d6b08c6d888998c5504(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_2th_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %pointer_cast, i32 0, i32 2
  %pointer_cast1 = bitcast { i8 }* %ptr_to_2th_field to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast1)
  ret void
}

define internal void @"closure[Std::Array Std::U8 -> Std::I32]"(i8* %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj9" = alloca { i32 }, align 8
  %"alloca@allocate_obj" = alloca { i8* }, align 8
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %field_value, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field1 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 1
  %field_value2 = load i8*, i8** %ptr_to_field1, align 8
  %is_null = icmp eq i8* %field_value2, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value2)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast3 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field4 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast3, i32 0, i32 1
  %field_value5 = load void (i8*)*, void (i8*)** %ptr_to_field4, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value5)
  call void @retain_obj(i8* %0)
  %pointer_cast6 = bitcast { i8* }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::Array Std::U8) Std::Ptr]"(i8* %0, i8* %pointer_cast6)
  %load_unbox = load { i8* }, { i8* }* %"alloca@allocate_obj", align 8
  %ptr_to_field7 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value8 = load void ({ i8* }, i8*, i8*)*, void ({ i8* }, i8*, i8*)** %ptr_to_field7, align 8
  %pointer_cast10 = bitcast { i32 }* %"alloca@allocate_obj9" to i8*
  %ptr_to_field11 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 1
  %field_value12 = load i8*, i8** %ptr_to_field11, align 8
  tail call void %field_value8({ i8* } %load_unbox, i8* %field_value12, i8* %pointer_cast10)
  call void @release_obj(i8* %0, void (i8*)* @dtor_82a1dcae1a6595589047bba1a0f55e25)
  %load_unbox13 = load { i32 }, { i32 }* %"alloca@allocate_obj9", align 4
  %pointer_cast14 = bitcast i8* %2 to { i32 }*
  store { i32 } %load_unbox13, { i32 }* %pointer_cast14, align 4
  ret void
}

define internal void @dtor_82a1dcae1a6595589047bba1a0f55e25(i8* %0) {
entry:
  %release_loop_counter = alloca i64, align 8
  %pointer_cast = bitcast i8* %0 to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 1
  %field_value = load i64, i64* %ptr_to_field, align 8
  %pointer_cast1 = bitcast i8* %0 to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_3th_field = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast1, i32 0, i32 3
  store i64 0, i64* %release_loop_counter, align 8
  br label %loop_release_array_elements

loop_release_array_elements:                      ; preds = %loop_body, %entry
  %counter_val = load i64, i64* %release_loop_counter, align 8
  %is_end = icmp eq i64 %counter_val, %field_value
  br i1 %is_end, label %after_loop, label %loop_body

loop_body:                                        ; preds = %loop_release_array_elements
  %pointer_cast2 = bitcast i64* %release_loop_counter to { i64 }*
  %ptr_to_field3 = getelementptr inbounds { i64 }, { i64 }* %pointer_cast2, i32 0, i32 0
  %field_value4 = load i64, i64* %ptr_to_field3, align 8
  %ptr_to_elem_of_array = getelementptr { i8 }, { i8 }* %ptr_to_3th_field, i64 %field_value4
  %pointer_cast5 = bitcast { i8 }* %ptr_to_elem_of_array to i8*
  call void @dtor_77dad52a4782825eeeede360ed15e924(i8* %pointer_cast5)
  %incremented_counter_val = add i64 %counter_val, 1
  store i64 %incremented_counter_val, i64* %release_loop_counter, align 8
  br label %loop_release_array_elements

after_loop:                                       ; preds = %loop_release_array_elements
  ret void
}

define internal void @dtor_77dad52a4782825eeeede360ed15e924(i8* %0) {
entry:
  ret void
}

define internal void @dtor_fe3166cacc608f483c86d91be4ff2782(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_2th_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %pointer_cast1 = bitcast { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_2th_field to i8*
  call void @dtor_32d99c0fa90d841608eaaef22e1cc6f5(i8* %pointer_cast1)
  ret void
}

define internal void @dtor_32d99c0fa90d841608eaaef22e1cc6f5(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { void ({ i8* }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 1
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %is_null = icmp eq i8* %field_value, null
  br i1 %is_null, label %cont_in_release_dynamic, label %nonnull_in_release_dynamic

nonnull_in_release_dynamic:                       ; preds = %entry
  %pointer_cast1 = bitcast i8* %field_value to { { i64 }, void (i8*)* }*
  %ptr_to_field2 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load void (i8*)*, void (i8*)** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value, void (i8*)* %field_value3)
  br label %cont_in_release_dynamic

cont_in_release_dynamic:                          ; preds = %nonnull_in_release_dynamic, %entry
  ret void
}

define internal void @"closure[(Std::Ptr -> Std::I32) -> Std::Array Std::U8 -> Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %0, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %2 to { void (i8*, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void (i8*, i8*, i8*)*, i8* }, { void (i8*, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 0
  store void (i8*, i8*, i8*)* @"closure[Std::Array Std::U8 -> Std::I32].2", void (i8*, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_fe3166cacc608f483c86d91be4ff2782, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast2, i32 0, i32 2
  store { void ({ i8* }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %pointer_cast5 = bitcast i8* %2 to { void (i8*, i8*, i8*)*, i8* }*
  %ptr_to_field6 = getelementptr inbounds { void (i8*, i8*, i8*)*, i8* }, { void (i8*, i8*, i8*)*, i8* }* %pointer_cast5, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field6, align 8
  ret void
}

define internal void @"closure[Std::Array Std::U8 -> Std::I32].2"(i8* %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj9" = alloca { i32 }, align 8
  %"alloca@allocate_obj" = alloca { i8* }, align 8
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %field_value, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field1 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 1
  %field_value2 = load i8*, i8** %ptr_to_field1, align 8
  %is_null = icmp eq i8* %field_value2, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value2)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast3 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field4 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast3, i32 0, i32 1
  %field_value5 = load void (i8*)*, void (i8*)** %ptr_to_field4, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value5)
  call void @retain_obj(i8* %0)
  %pointer_cast6 = bitcast { i8* }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::Array Std::U8) Std::Ptr]"(i8* %0, i8* %pointer_cast6)
  %load_unbox = load { i8* }, { i8* }* %"alloca@allocate_obj", align 8
  %ptr_to_field7 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value8 = load void ({ i8* }, i8*, i8*)*, void ({ i8* }, i8*, i8*)** %ptr_to_field7, align 8
  %pointer_cast10 = bitcast { i32 }* %"alloca@allocate_obj9" to i8*
  %ptr_to_field11 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, i32 0, i32 1
  %field_value12 = load i8*, i8** %ptr_to_field11, align 8
  tail call void %field_value8({ i8* } %load_unbox, i8* %field_value12, i8* %pointer_cast10)
  call void @release_obj(i8* %0, void (i8*)* @dtor_82a1dcae1a6595589047bba1a0f55e25)
  %load_unbox13 = load { i32 }, { i32 }* %"alloca@allocate_obj9", align 4
  %pointer_cast14 = bitcast i8* %2 to { i32 }*
  store { i32 } %load_unbox13, { i32 }* %pointer_cast14, align 4
  ret void
}

define internal void @"closure[(Std::Ptr -> Std::I32) -> Std::String -> Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %0, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %2 to { void ({ i8* }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 0
  store void ({ i8* }, i8*, i8*)* @"closure[Std::String -> Std::I32]", void ({ i8* }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_fe3166cacc608f483c86d91be4ff2782, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast2, i32 0, i32 2
  store { void ({ i8* }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %pointer_cast5 = bitcast i8* %2 to { void ({ i8* }, i8*, i8*)*, i8* }*
  %ptr_to_field6 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %pointer_cast5, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field6, align 8
  ret void
}

define internal void @"closure[Std::String -> Std::I32]"({ i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %field_value, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load void (i8*)*, void (i8*)** %ptr_to_field5, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value6)
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj, align 8
  %call_lambda = tail call i8* @"closure[Std::#FunPtr1 Std::String (Std::Array Std::U8)]"({ i8* } %load_unbox)
  %load_unbox7 = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast8 = bitcast i8* %2 to { i32 }*
  %pointer_cast9 = bitcast { i32 }* %pointer_cast8 to i8*
  tail call void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) (Std::Array Std::U8) Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %load_unbox7, i8* %call_lambda, i8* %pointer_cast9)
  ret void
}

; Function Attrs: argmemonly nofree nosync nounwind willreturn
declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg) #0

define internal void @"closure[() -> Std::IO ()]"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj7" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj4" = alloca { i8* }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast)
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> Std::IO ()].3", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  store i8* null, i8** %ptr_to_field1, align 8
  %3 = trunc i64 add (i64 ptrtoint ({ i8 }* getelementptr inbounds ({ { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* null, i32 0, i32 3) to i64), i64 mul (i64 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i64), i64 2)) to i32
  %mallocsize = mul i32 %3, ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)
  %"malloc_array@allocate_obj" = tail call i8* @malloc(i32 %mallocsize)
  %pointer_cast2 = bitcast i8* %"malloc_array@allocate_obj" to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_size_field = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 2
  store i64 2, i64* %ptr_to_size_field, align 8
  %ptr_to_field3 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 1
  store i64 2, i64* %ptr_to_field3, align 8
  %ptr_to_field_nocap = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 3
  %4 = bitcast { i8 }* %ptr_to_field_nocap to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 1 %4, i8* align 1 getelementptr inbounds ([2 x i8], [2 x i8]* @string_literal.4, i32 0, i32 0), i64 2, i1 false)
  %pointer_cast5 = bitcast { { i64 }, i64, i64, { i8 } }* %pointer_cast2 to i8*
  %ptr_to_field6 = getelementptr inbounds { i8* }, { i8* }* %"alloca@allocate_obj4", i32 0, i32 0
  store i8* %pointer_cast5, i8** %ptr_to_field6, align 8
  %load_unbox = load { i8* }, { i8* }* %"alloca@allocate_obj4", align 8
  %pointer_cast8 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj7" to i8*
  tail call void @"closure[Std::#FunPtr1 Std::String (Std::IO ())].1"({ i8* } %load_unbox, i8* %pointer_cast8)
  %load_unbox9 = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", align 8
  %load_unbox10 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj7", align 8
  %pointer_cast11 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %pointer_cast12 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast11 to i8*
  tail call void @"closure[Std::#FunPtr2 (() -> Std::IO ()) (Std::IO ()) (Std::IO ())]"({ void ({ i8 }, i8*, i8*)*, i8* } %load_unbox9, { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox10, i8* %pointer_cast12)
  ret void
}

define internal void @"closure[() -> Std::IO ()].3"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj" = alloca { i8 }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast)
  %load_unbox = load { i8 }, { i8 }* %"alloca@allocate_obj", align 1
  %pointer_cast1 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %pointer_cast2 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast1 to i8*
  tail call void @"closure[Std::#FunPtr1 () (Std::IO ())]"({ i8 } %load_unbox, i8* %pointer_cast2)
  ret void
}

define internal void @"closure[Std::IO () -> Std::IO ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %0, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %field_value, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load void (i8*)*, void (i8*)** %ptr_to_field5, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value6)
  %pointer_cast7 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast7, i32 0, i32 0
  %ptr_to_field8 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()].5", void ({ i8 }, i8*, i8*)** %ptr_to_field8, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_6d53b2aad890c6416389a152b34a0482, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast9 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast10 = bitcast i8* %pointer_cast9 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field11 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast10, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field11, align 8
  %load_unbox12 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %pointer_cast13 = bitcast i8* %pointer_cast9 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field14 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast13, i32 0, i32 3
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox12, { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_field14, align 8
  %pointer_cast15 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field16 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast15, i8** %ptr_to_field16, align 8
  ret void
}

define internal void @"closure[() -> ()].5"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj21" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj" = alloca { i8 }, align 8
  %alloca_for_unboxed_obj7 = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %alloca_for_unboxed_obj1 = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %field_value, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast4, i32 0, i32 3
  %field_value6 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_field5, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %field_value6, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, align 8
  %ptr_to_0th_field = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, i32 0, i32 0
  %ptr_to_field8 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_0th_field, i32 0, i32 1
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  %is_null12 = icmp eq i8* %field_value9, null
  br i1 %is_null12, label %cont_in_retain_dynamic11, label %nonnull_in_retain_dynamic10

nonnull_in_retain_dynamic10:                      ; preds = %cont_in_retain_dynamic
  call void @retain_obj(i8* %field_value9)
  br label %cont_in_retain_dynamic11

cont_in_retain_dynamic11:                         ; preds = %nonnull_in_retain_dynamic10, %cont_in_retain_dynamic
  %pointer_cast13 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field14 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast13, i32 0, i32 1
  %field_value15 = load void (i8*)*, void (i8*)** %ptr_to_field14, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value15)
  %pointer_cast16 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast16)
  %load_unbox = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, align 8
  %pointer_cast17 = bitcast { i8 }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox, i8* %pointer_cast17)
  %load_unbox18 = load { i8 }, { i8 }* %"alloca@allocate_obj", align 1
  %ptr_to_field19 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 0
  %field_value20 = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %ptr_to_field19, align 8
  %pointer_cast22 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj21" to i8*
  %ptr_to_field23 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value24 = load i8*, i8** %ptr_to_field23, align 8
  tail call void %field_value20({ i8 } %load_unbox18, i8* %field_value24, i8* %pointer_cast22)
  %load_unbox25 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj21", align 8
  %pointer_cast26 = bitcast i8* %2 to { i8 }*
  %pointer_cast27 = bitcast { i8 }* %pointer_cast26 to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox25, i8* %pointer_cast27)
  ret void
}

define internal void @dtor_6d53b2aad890c6416389a152b34a0482(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_2th_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast, i32 0, i32 2
  %pointer_cast1 = bitcast { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_2th_field to i8*
  call void @dtor_ed7a95ea8dceca97841a665139e044cd(i8* %pointer_cast1)
  %pointer_cast2 = bitcast i8* %0 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_3th_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast2, i32 0, i32 3
  %pointer_cast3 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_3th_field to i8*
  call void @dtor_18e42987d75046f070b34a440ad41590(i8* %pointer_cast3)
  ret void
}

define internal void @dtor_ed7a95ea8dceca97841a665139e044cd(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { void ({ i8 }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 1
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %is_null = icmp eq i8* %field_value, null
  br i1 %is_null, label %cont_in_release_dynamic, label %nonnull_in_release_dynamic

nonnull_in_release_dynamic:                       ; preds = %entry
  %pointer_cast1 = bitcast i8* %field_value to { { i64 }, void (i8*)* }*
  %ptr_to_field2 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load void (i8*)*, void (i8*)** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value, void (i8*)* %field_value3)
  br label %cont_in_release_dynamic

cont_in_release_dynamic:                          ; preds = %nonnull_in_release_dynamic, %entry
  ret void
}

define internal void @dtor_18e42987d75046f070b34a440ad41590(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_0th_field = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %pointer_cast1 = bitcast { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_0th_field to i8*
  call void @dtor_ec9d27f5f144de01a2b4af4f6b840719(i8* %pointer_cast1)
  ret void
}

define internal void @dtor_ec9d27f5f144de01a2b4af4f6b840719(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { void ({ i8 }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 1
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %is_null = icmp eq i8* %field_value, null
  br i1 %is_null, label %cont_in_release_dynamic, label %nonnull_in_release_dynamic

nonnull_in_release_dynamic:                       ; preds = %entry
  %pointer_cast1 = bitcast i8* %field_value to { { i64 }, void (i8*)* }*
  %ptr_to_field2 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load void (i8*)*, void (i8*)** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value, void (i8*)* %field_value3)
  br label %cont_in_release_dynamic

cont_in_release_dynamic:                          ; preds = %nonnull_in_release_dynamic, %entry
  ret void
}

define internal void @dtor_626b039d13292bddadb8d093deabe646(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_2th_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %pointer_cast1 = bitcast { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_2th_field to i8*
  call void @dtor_ed7a95ea8dceca97841a665139e044cd(i8* %pointer_cast1)
  ret void
}

define internal void @"closure[() -> ()].6"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj21" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj" = alloca { i8 }, align 8
  %alloca_for_unboxed_obj7 = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %alloca_for_unboxed_obj1 = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %field_value, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast4, i32 0, i32 3
  %field_value6 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_field5, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %field_value6, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, align 8
  %ptr_to_0th_field = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, i32 0, i32 0
  %ptr_to_field8 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_0th_field, i32 0, i32 1
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  %is_null12 = icmp eq i8* %field_value9, null
  br i1 %is_null12, label %cont_in_retain_dynamic11, label %nonnull_in_retain_dynamic10

nonnull_in_retain_dynamic10:                      ; preds = %cont_in_retain_dynamic
  call void @retain_obj(i8* %field_value9)
  br label %cont_in_retain_dynamic11

cont_in_retain_dynamic11:                         ; preds = %nonnull_in_retain_dynamic10, %cont_in_retain_dynamic
  %pointer_cast13 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field14 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast13, i32 0, i32 1
  %field_value15 = load void (i8*)*, void (i8*)** %ptr_to_field14, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value15)
  %pointer_cast16 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast16)
  %load_unbox = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, align 8
  %pointer_cast17 = bitcast { i8 }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox, i8* %pointer_cast17)
  %load_unbox18 = load { i8 }, { i8 }* %"alloca@allocate_obj", align 1
  %ptr_to_field19 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 0
  %field_value20 = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %ptr_to_field19, align 8
  %pointer_cast22 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj21" to i8*
  %ptr_to_field23 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value24 = load i8*, i8** %ptr_to_field23, align 8
  tail call void %field_value20({ i8 } %load_unbox18, i8* %field_value24, i8* %pointer_cast22)
  %load_unbox25 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj21", align 8
  %pointer_cast26 = bitcast i8* %2 to { i8 }*
  %pointer_cast27 = bitcast { i8 }* %pointer_cast26 to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox25, i8* %pointer_cast27)
  ret void
}

define internal void @"closure[(() -> Std::IO ()) -> Std::IO () -> Std::IO ()]"({ void ({ i8 }, i8*, i8*)*, i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %0, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %2 to { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 0
  store void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)* @"closure[Std::IO () -> Std::IO ()].7", void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_626b039d13292bddadb8d093deabe646, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast2, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %"malloc@allocate_obj" to i8*
  %pointer_cast5 = bitcast i8* %2 to { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }*
  %ptr_to_field6 = getelementptr inbounds { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }, { void ({ { void ({ i8 }, i8*, i8*)*, i8* } }, i8*, i8*)*, i8* }* %pointer_cast5, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field6, align 8
  ret void
}

define internal void @"closure[Std::IO () -> Std::IO ()].7"({ { void ({ i8 }, i8*, i8*)*, i8* } } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %0, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %field_value, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load void (i8*)*, void (i8*)** %ptr_to_field5, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value6)
  %pointer_cast7 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast7, i32 0, i32 0
  %ptr_to_field8 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()].8", void ({ i8 }, i8*, i8*)** %ptr_to_field8, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* getelementptr ({ { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_6d53b2aad890c6416389a152b34a0482, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast9 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast10 = bitcast i8* %pointer_cast9 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field11 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast10, i32 0, i32 2
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field11, align 8
  %load_unbox12 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %pointer_cast13 = bitcast i8* %pointer_cast9 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field14 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast13, i32 0, i32 3
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox12, { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_field14, align 8
  %pointer_cast15 = bitcast { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field16 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast15, i8** %ptr_to_field16, align 8
  ret void
}

define internal void @"closure[() -> ()].8"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj21" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj" = alloca { i8 }, align 8
  %alloca_for_unboxed_obj7 = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %alloca_for_unboxed_obj1 = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %field_value, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }, { { i64 }, void (i8*)*, { void ({ i8 }, i8*, i8*)*, i8* }, { { void ({ i8 }, i8*, i8*)*, i8* } } }* %pointer_cast4, i32 0, i32 3
  %field_value6 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %ptr_to_field5, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %field_value6, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, align 8
  %ptr_to_0th_field = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, i32 0, i32 0
  %ptr_to_field8 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_0th_field, i32 0, i32 1
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  %is_null12 = icmp eq i8* %field_value9, null
  br i1 %is_null12, label %cont_in_retain_dynamic11, label %nonnull_in_retain_dynamic10

nonnull_in_retain_dynamic10:                      ; preds = %cont_in_retain_dynamic
  call void @retain_obj(i8* %field_value9)
  br label %cont_in_retain_dynamic11

cont_in_retain_dynamic11:                         ; preds = %nonnull_in_retain_dynamic10, %cont_in_retain_dynamic
  %pointer_cast13 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field14 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast13, i32 0, i32 1
  %field_value15 = load void (i8*)*, void (i8*)** %ptr_to_field14, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value15)
  %pointer_cast16 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast16)
  %load_unbox = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj7, align 8
  %pointer_cast17 = bitcast { i8 }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox, i8* %pointer_cast17)
  %load_unbox18 = load { i8 }, { i8 }* %"alloca@allocate_obj", align 1
  %ptr_to_field19 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 0
  %field_value20 = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %ptr_to_field19, align 8
  %pointer_cast22 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj21" to i8*
  %ptr_to_field23 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value24 = load i8*, i8** %ptr_to_field23, align 8
  tail call void %field_value20({ i8 } %load_unbox18, i8* %field_value24, i8* %pointer_cast22)
  %load_unbox25 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj21", align 8
  %pointer_cast26 = bitcast i8* %2 to { i8 }*
  %pointer_cast27 = bitcast { i8 }* %pointer_cast26 to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox25, i8* %pointer_cast27)
  ret void
}

define internal void @"closure[() -> Std::IO ()].9"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()].10", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8 } }* getelementptr ({ { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_77d94fe6fddb2d6b08c6d888998c5504, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { i8 }, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %pointer_cast2, i32 0, i32 2
  store { i8 } %load_unbox, { i8 }* %ptr_to_field3, align 1
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { i8 } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field5 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field5, align 8
  ret void
}

define internal void @"closure[() -> ()].10"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { i8 }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { i8 } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8 } }, { { i64 }, void (i8*)*, { i8 } }* %pointer_cast, i32 0, i32 2
  %field_value = load { i8 }, { i8 }* %ptr_to_field, align 1
  store { i8 } %field_value, { i8 }* %alloca_for_unboxed_obj1, align 1
  %pointer_cast2 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast2, i32 0, i32 1
  %field_value4 = load void (i8*)*, void (i8*)** %ptr_to_field3, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value4)
  %pointer_cast5 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast5)
  %load_unbox = load { i8 }, { i8 }* %alloca_for_unboxed_obj1, align 1
  %pointer_cast6 = bitcast i8* %2 to { i8 }*
  store { i8 } %load_unbox, { i8 }* %pointer_cast6, align 1
  ret void
}

define internal void @"closure[Std::IO () -> ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj1" = alloca { i8 }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %0, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %load_unbox = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj" to i8*
  tail call void @"closure[Std::#FunPtr1 (Std::IO ()) (() -> ())]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox, i8* %pointer_cast)
  %load_unbox2 = load { i8 }, { i8 }* %"alloca@allocate_obj1", align 1
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  %field_value = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %pointer_cast3 = bitcast i8* %2 to { i8 }*
  %pointer_cast4 = bitcast { i8 }* %pointer_cast3 to i8*
  %ptr_to_field5 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  tail call void %field_value({ i8 } %load_unbox2, i8* %field_value6, i8* %pointer_cast4)
  ret void
}

define internal void @"closure[Std::IO () -> () -> ()]"({ { void ({ i8 }, i8*, i8*)*, i8* } } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %0, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, i32 0, i32 0
  %load_unbox = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, align 8
  %pointer_cast = bitcast i8* %2 to { void ({ i8 }, i8*, i8*)*, i8* }*
  store { void ({ i8 }, i8*, i8*)*, i8* } %load_unbox, { void ({ i8 }, i8*, i8*)*, i8* }* %pointer_cast, align 8
  ret void
}

define internal i8* @"closure[Std::String -> Std::Array Std::U8]"({ i8* } %0, i8* %1) {
entry:
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value = load i8*, i8** %ptr_to_field, align 8
  ret i8* %field_value
}

define internal void @"closure[Std::String -> Std::IO ()]"({ i8* } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj2" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> Std::IO ()].11", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  store i8* null, i8** %ptr_to_field1, align 8
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj2" to i8*
  tail call void @"closure[Std::#FunPtr1 Std::String (Std::IO ())].1"({ i8* } %load_unbox, i8* %pointer_cast)
  %load_unbox3 = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", align 8
  %load_unbox4 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj2", align 8
  %pointer_cast5 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %pointer_cast6 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast5 to i8*
  tail call void @"closure[Std::#FunPtr2 (() -> Std::IO ()) (Std::IO ()) (Std::IO ())]"({ void ({ i8 }, i8*, i8*)*, i8* } %load_unbox3, { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox4, i8* %pointer_cast6)
  ret void
}

define internal void @"closure[() -> Std::IO ()].11"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj7" = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %"alloca@allocate_obj4" = alloca { i8* }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast)
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> Std::IO ()].12", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  store i8* null, i8** %ptr_to_field1, align 8
  %3 = trunc i64 add (i64 ptrtoint ({ i8 }* getelementptr inbounds ({ { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* null, i32 0, i32 3) to i64), i64 mul (i64 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i64), i64 2)) to i32
  %mallocsize = mul i32 %3, ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i32)
  %"malloc_array@allocate_obj" = tail call i8* @malloc(i32 %mallocsize)
  %pointer_cast2 = bitcast i8* %"malloc_array@allocate_obj" to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_size_field = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 2
  store i64 2, i64* %ptr_to_size_field, align 8
  %ptr_to_field3 = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 1
  store i64 2, i64* %ptr_to_field3, align 8
  %ptr_to_field_nocap = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast2, i32 0, i32 3
  %4 = bitcast { i8 }* %ptr_to_field_nocap to i8*
  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 1 %4, i8* align 1 getelementptr inbounds ([2 x i8], [2 x i8]* @string_literal.13, i32 0, i32 0), i64 2, i1 false)
  %pointer_cast5 = bitcast { { i64 }, i64, i64, { i8 } }* %pointer_cast2 to i8*
  %ptr_to_field6 = getelementptr inbounds { i8* }, { i8* }* %"alloca@allocate_obj4", i32 0, i32 0
  store i8* %pointer_cast5, i8** %ptr_to_field6, align 8
  %load_unbox = load { i8* }, { i8* }* %"alloca@allocate_obj4", align 8
  %pointer_cast8 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj7" to i8*
  tail call void @"closure[Std::#FunPtr1 Std::String (Std::IO ())].1"({ i8* } %load_unbox, i8* %pointer_cast8)
  %load_unbox9 = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", align 8
  %load_unbox10 = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %"alloca@allocate_obj7", align 8
  %pointer_cast11 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %pointer_cast12 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast11 to i8*
  tail call void @"closure[Std::#FunPtr2 (() -> Std::IO ()) (Std::IO ()) (Std::IO ())]"({ void ({ i8 }, i8*, i8*)*, i8* } %load_unbox9, { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox10, i8* %pointer_cast12)
  ret void
}

define internal void @"closure[() -> Std::IO ()].12"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj" = alloca { i8 }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast)
  %load_unbox = load { i8 }, { i8 }* %"alloca@allocate_obj", align 1
  %pointer_cast1 = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %pointer_cast2 = bitcast { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast1 to i8*
  tail call void @"closure[Std::#FunPtr1 () (Std::IO ())]"({ i8 } %load_unbox, i8* %pointer_cast2)
  ret void
}

define internal void @"closure[Std::Array Std::U8 -> Std::Ptr]"(i8* %0, i8* %1, i8* %2) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, i64, i64, { i8 } }*
  %ptr_to_field_nocap = getelementptr inbounds { { i64 }, i64, i64, { i8 } }, { { i64 }, i64, i64, { i8 } }* %pointer_cast, i32 0, i32 3
  %pointer_cast1 = bitcast { i8 }* %ptr_to_field_nocap to i8*
  call void @release_obj(i8* %0, void (i8*)* @dtor_82a1dcae1a6595589047bba1a0f55e25)
  %pointer_cast2 = bitcast i8* %2 to { i8* }*
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %pointer_cast2, i32 0, i32 0
  store i8* %pointer_cast1, i8** %ptr_to_field, align 8
  ret void
}

define internal void @"closure[Std::String -> Std::I32].14"({ i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj1 = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }, { { i64 }, void (i8*)*, { void ({ i8* }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %ptr_to_field, align 8
  store { void ({ i8* }, i8*, i8*)*, i8* } %field_value, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, i32 0, i32 1
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %is_null = icmp eq i8* %field_value3, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load void (i8*)*, void (i8*)** %ptr_to_field5, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value6)
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj, align 8
  %call_lambda = tail call i8* @"closure[Std::#FunPtr1 Std::String (Std::Array Std::U8)]"({ i8* } %load_unbox)
  %load_unbox7 = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast8 = bitcast i8* %2 to { i32 }*
  %pointer_cast9 = bitcast { i32 }* %pointer_cast8 to i8*
  tail call void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) (Std::Array Std::U8) Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %load_unbox7, i8* %call_lambda, i8* %pointer_cast9)
  ret void
}

define internal void @"closure[() -> ()].15"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj11" = alloca { i32 }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj1 = alloca { i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { i8* }, { i8* }* %ptr_to_field, align 8
  store { i8* } %field_value, { i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { i8* }, { i8* }* %alloca_for_unboxed_obj1, i32 0, i32 0
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @retain_obj(i8* %field_value3)
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load void (i8*)*, void (i8*)** %ptr_to_field5, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value6)
  %pointer_cast7 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast7)
  %ptr_to_field8 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store void ({ i8* }, i8*, i8*)* @"closure[Std::Ptr -> Std::I32]", void ({ i8* }, i8*, i8*)** %ptr_to_field8, align 8
  %ptr_to_field9 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  store i8* null, i8** %ptr_to_field9, align 8
  %load_unbox = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", align 8
  %load_unbox10 = load { i8* }, { i8* }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast12 = bitcast { i32 }* %"alloca@allocate_obj11" to i8*
  tail call void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) Std::String Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %load_unbox, { i8* } %load_unbox10, i8* %pointer_cast12)
  %pointer_cast13 = bitcast { i32 }* %"alloca@allocate_obj11" to i8*
  call void @dtor_005ac899be43b3a48eda563bd4f649f3(i8* %pointer_cast13)
  ret void
}

define internal void @"closure[Std::Ptr -> Std::I32]"({ i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %"CALL_C(printf)" = call i32 (i8*, ...) @printf(i8* %field_value)
  %pointer_cast = bitcast i8* %2 to { i32 }*
  %ptr_to_field1 = getelementptr inbounds { i32 }, { i32 }* %pointer_cast, i32 0, i32 0
  store i32 %"CALL_C(printf)", i32* %ptr_to_field1, align 4
  ret void
}

declare i32 @printf(i8*, ...)

define internal void @dtor_005ac899be43b3a48eda563bd4f649f3(i8* %0) {
entry:
  ret void
}

define internal void @dtor_5a1aaed28630590aeb6e3317f6304e19(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_2th_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %pointer_cast, i32 0, i32 2
  %pointer_cast1 = bitcast { i8* }* %ptr_to_2th_field to i8*
  call void @dtor_6d7b6a0a12005b29aac15be081401018(i8* %pointer_cast1)
  ret void
}

define internal void @dtor_6d7b6a0a12005b29aac15be081401018(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { i8* }*
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %pointer_cast, i32 0, i32 0
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value, void (i8*)* @dtor_82a1dcae1a6595589047bba1a0f55e25)
  ret void
}

define internal void @"closure[Std::String -> Std::IO ()].16"({ i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast = bitcast i8* %2 to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_field_nocap = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 0
  store void ({ i8 }, i8*, i8*)* @"closure[() -> ()].17", void ({ i8 }, i8*, i8*)** %ptr_to_field, align 8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64 }, void (i8*)*, { i8* } }* getelementptr ({ { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* null, i32 1) to i32))
  %"malloc@allocate_obj" = bitcast i8* %malloccall to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_control_block = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj", i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64 }, { i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 8
  %ptr_to_dtor_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj", i32 0, i32 1
  store void (i8*)* @dtor_5a1aaed28630590aeb6e3317f6304e19, void (i8*)** %ptr_to_dtor_field, align 8
  %pointer_cast1 = bitcast { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj" to i8*
  %load_unbox = load { i8* }, { i8* }* %alloca_for_unboxed_obj, align 8
  %pointer_cast2 = bitcast i8* %pointer_cast1 to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_field3 = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %pointer_cast2, i32 0, i32 2
  store { i8* } %load_unbox, { i8* }* %ptr_to_field3, align 8
  %pointer_cast4 = bitcast { { i64 }, void (i8*)*, { i8* } }* %"malloc@allocate_obj" to i8*
  %ptr_to_field5 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field_nocap, i32 0, i32 1
  store i8* %pointer_cast4, i8** %ptr_to_field5, align 8
  ret void
}

define internal void @"closure[() -> ()].17"({ i8 } %0, i8* %1, i8* %2) {
entry:
  %"alloca@allocate_obj11" = alloca { i32 }, align 8
  %"alloca@allocate_obj" = alloca { void ({ i8* }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj1 = alloca { i8* }, align 8
  %alloca_for_unboxed_obj = alloca { i8 }, align 8
  store { i8 } %0, { i8 }* %alloca_for_unboxed_obj, align 1
  %pointer_cast = bitcast i8* %1 to { { i64 }, void (i8*)*, { i8* } }*
  %ptr_to_field = getelementptr inbounds { { i64 }, void (i8*)*, { i8* } }, { { i64 }, void (i8*)*, { i8* } }* %pointer_cast, i32 0, i32 2
  %field_value = load { i8* }, { i8* }* %ptr_to_field, align 8
  store { i8* } %field_value, { i8* }* %alloca_for_unboxed_obj1, align 8
  %ptr_to_field2 = getelementptr inbounds { i8* }, { i8* }* %alloca_for_unboxed_obj1, i32 0, i32 0
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @retain_obj(i8* %field_value3)
  %pointer_cast4 = bitcast i8* %1 to { { i64 }, void (i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64 }, void (i8*)* }, { { i64 }, void (i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load void (i8*)*, void (i8*)** %ptr_to_field5, align 8
  call void @release_obj(i8* %1, void (i8*)* %field_value6)
  %pointer_cast7 = bitcast { i8 }* %alloca_for_unboxed_obj to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast7)
  %ptr_to_field8 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 0
  store void ({ i8* }, i8*, i8*)* @"closure[Std::Ptr -> Std::I32].18", void ({ i8* }, i8*, i8*)** %ptr_to_field8, align 8
  %ptr_to_field9 = getelementptr inbounds { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", i32 0, i32 1
  store i8* null, i8** %ptr_to_field9, align 8
  %load_unbox = load { void ({ i8* }, i8*, i8*)*, i8* }, { void ({ i8* }, i8*, i8*)*, i8* }* %"alloca@allocate_obj", align 8
  %load_unbox10 = load { i8* }, { i8* }* %alloca_for_unboxed_obj1, align 8
  %pointer_cast12 = bitcast { i32 }* %"alloca@allocate_obj11" to i8*
  tail call void @"closure[Std::#FunPtr2 (Std::Ptr -> Std::I32) Std::String Std::I32]"({ void ({ i8* }, i8*, i8*)*, i8* } %load_unbox, { i8* } %load_unbox10, i8* %pointer_cast12)
  %pointer_cast13 = bitcast { i32 }* %"alloca@allocate_obj11" to i8*
  call void @dtor_005ac899be43b3a48eda563bd4f649f3(i8* %pointer_cast13)
  ret void
}

define internal void @"closure[Std::Ptr -> Std::I32].18"({ i8* } %0, i8* %1, i8* %2) {
entry:
  %alloca_for_unboxed_obj = alloca { i8* }, align 8
  store { i8* } %0, { i8* }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field = getelementptr inbounds { i8* }, { i8* }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %"CALL_C(printf)" = call i32 (i8*, ...) @printf(i8* %field_value)
  %pointer_cast = bitcast i8* %2 to { i32 }*
  %ptr_to_field1 = getelementptr inbounds { i32 }, { i32 }* %pointer_cast, i32 0, i32 0
  store i32 %"CALL_C(printf)", i32* %ptr_to_field1, align 4
  ret void
}

define i32 @main() {
entry:
  %"alloca@allocate_obj8" = alloca { i8 }, align 8
  %"alloca@allocate_obj" = alloca { i8 }, align 8
  %alloca_for_unboxed_obj4 = alloca { void ({ i8 }, i8*, i8*)*, i8* }, align 8
  %alloca_for_unboxed_obj = alloca { { void ({ i8 }, i8*, i8*)*, i8* } }, align 8
  %get_ptr = call i8* @"Get#Main::main#18e42987d75046f070b34a440ad41590"()
  %pointer_cast = bitcast i8* %get_ptr to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %ptr_to_0th_field = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast, i32 0, i32 0
  %ptr_to_field = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_0th_field, i32 0, i32 1
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %is_null = icmp eq i8* %field_value, null
  br i1 %is_null, label %cont_in_retain_dynamic, label %nonnull_in_retain_dynamic

nonnull_in_retain_dynamic:                        ; preds = %entry
  call void @retain_obj(i8* %field_value)
  br label %cont_in_retain_dynamic

cont_in_retain_dynamic:                           ; preds = %nonnull_in_retain_dynamic, %entry
  %pointer_cast1 = bitcast i8* %get_ptr to { { void ({ i8 }, i8*, i8*)*, i8* } }*
  %load_unbox = load { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %pointer_cast1, align 8
  store { { void ({ i8 }, i8*, i8*)*, i8* } } %load_unbox, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, align 8
  %ptr_to_field2 = getelementptr inbounds { { void ({ i8 }, i8*, i8*)*, i8* } }, { { void ({ i8 }, i8*, i8*)*, i8* } }* %alloca_for_unboxed_obj, i32 0, i32 0
  %field_value3 = load { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %ptr_to_field2, align 8
  store { void ({ i8 }, i8*, i8*)*, i8* } %field_value3, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj4, align 8
  %load_unbox5 = load { i8 }, { i8 }* %"alloca@allocate_obj", align 1
  %ptr_to_field6 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj4, i32 0, i32 0
  %field_value7 = load void ({ i8 }, i8*, i8*)*, void ({ i8 }, i8*, i8*)** %ptr_to_field6, align 8
  %pointer_cast9 = bitcast { i8 }* %"alloca@allocate_obj8" to i8*
  %ptr_to_field10 = getelementptr inbounds { void ({ i8 }, i8*, i8*)*, i8* }, { void ({ i8 }, i8*, i8*)*, i8* }* %alloca_for_unboxed_obj4, i32 0, i32 1
  %field_value11 = load i8*, i8** %ptr_to_field10, align 8
  tail call void %field_value7({ i8 } %load_unbox5, i8* %field_value11, i8* %pointer_cast9)
  %pointer_cast12 = bitcast { i8 }* %"alloca@allocate_obj8" to i8*
  call void @dtor_bcd8b0c2eb1fce714eab6cef0d771acc(i8* %pointer_cast12)
  ret i32 0
}

attributes #0 = { argmemonly nofree nosync nounwind willreturn }
