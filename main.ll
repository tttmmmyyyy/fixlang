; ModuleID = 'main'
source_filename = "main"

@name_of_obj = private unnamed_addr constant [20 x i8] c"newArray size value\00", align 1
@name_of_obj.2 = private unnamed_addr constant [30 x i8] c"\\value->(newArray size value)\00", align 1
@name_of_obj.4 = private unnamed_addr constant [39 x i8] c"\\size->(\\value->(newArray size value))\00", align 1
@name_of_obj.8 = private unnamed_addr constant [14 x i8] c"\\x->(fix f x)\00", align 1
@name_of_obj.9 = private unnamed_addr constant [20 x i8] c"\\f->(\\x->(fix f x))\00", align 1
@name_of_obj.12 = private unnamed_addr constant [11 x i8] c"eq lhs rhs\00", align 1
@name_of_obj.14 = private unnamed_addr constant [19 x i8] c"\\rhs->(eq lhs rhs)\00", align 1
@name_of_obj.15 = private unnamed_addr constant [27 x i8] c"\\lhs->(\\rhs->(eq lhs rhs))\00", align 1
@name_of_obj.18 = private unnamed_addr constant [12 x i8] c"add lhs rhs\00", align 1
@name_of_obj.20 = private unnamed_addr constant [20 x i8] c"\\rhs->(add lhs rhs)\00", align 1
@name_of_obj.21 = private unnamed_addr constant [28 x i8] c"\\lhs->(\\rhs->(add lhs rhs))\00", align 1
@name_of_obj.22 = private unnamed_addr constant [2 x i8] c"3\00", align 1
@name_of_obj.23 = private unnamed_addr constant [2 x i8] c"5\00", align 1

declare void @abort()

declare i32 @printf(i8*, ...)

declare i64 @report_malloc(i8*, i8*)

declare void @report_retain(i8*, i64, i64)

declare void @report_release(i8*, i64, i64)

declare void @check_leak()

define void @retain_obj(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { i64, void (i8*)*, i64 }*
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %pointer_cast, i32 0, i32 0
  %refcnt = load i64, i64* %ptr_to_refcnt, align 4
  %pointer_cast1 = bitcast i8* %0 to { i64, void (i8*)*, i64 }*
  %ptr_to_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %pointer_cast1, i32 0, i32 2
  %field_value = load i64, i64* %ptr_to_field, align 4
  call void @report_retain(i8* %0, i64 %field_value, i64 %refcnt)
  %refcnt2 = add i64 %refcnt, 1
  store i64 %refcnt2, i64* %ptr_to_refcnt, align 4
  ret void
}

define void @release_obj(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { i64, void (i8*)*, i64 }*
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %pointer_cast, i32 0, i32 0
  %refcnt = load i64, i64* %ptr_to_refcnt, align 4
  %pointer_cast1 = bitcast i8* %0 to { i64, void (i8*)*, i64 }*
  %ptr_to_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %pointer_cast1, i32 0, i32 2
  %field_value = load i64, i64* %ptr_to_field, align 4
  call void @report_release(i8* %0, i64 %field_value, i64 %refcnt)
  %refcnt2 = sub i64 %refcnt, 1
  store i64 %refcnt2, i64* %ptr_to_refcnt, align 4
  %is_refcnt_zero = icmp eq i64 %refcnt2, 0
  br i1 %is_refcnt_zero, label %refcnt_zero_after_release, label %end

refcnt_zero_after_release:                        ; preds = %entry
  %pointer_cast3 = bitcast i8* %0 to { i64, void (i8*)*, i64 }*
  %ptr_to_field4 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %pointer_cast3, i32 0, i32 1
  %field_value5 = load void (i8*)*, void (i8*)** %ptr_to_field4, align 8
  call void %field_value5(i8* %0)
  tail call void @free(i8* %0)
  br label %end

end:                                              ; preds = %refcnt_zero_after_release, %entry
  ret void
}

declare void @free(i8*)

define i64 @main() {
entry:
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([39 x i8], [39 x i8]* @name_of_obj.4, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.5, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda, i8* (i8*, i8*)** %ptr_to_field, align 8
  %pointer_cast1 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj to i8*
  call void @release_obj(i8* %pointer_cast1)
  %malloccall2 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj3 = bitcast i8* %malloccall2 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast4 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3 to i8*
  %call_runtime5 = call i64 @report_malloc(i8* %pointer_cast4, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.9, i32 0, i32 0))
  %ptr_to_control_block6 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3, i32 0, i32 0
  %ptr_to_refcnt7 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block6, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt7, align 4
  %ptr_to_dtor_field8 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block6, i32 0, i32 1
  store void (i8*)* @dtor.5, void (i8*)** %ptr_to_dtor_field8, align 8
  %ptr_to_obj_id9 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block6, i32 0, i32 2
  store i64 %call_runtime5, i64* %ptr_to_obj_id9, align 4
  %ptr_to_field10 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.6, i8* (i8*, i8*)** %ptr_to_field10, align 8
  %pointer_cast11 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3 to i8*
  call void @release_obj(i8* %pointer_cast11)
  %malloccall12 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj13 = bitcast i8* %malloccall12 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast14 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13 to i8*
  %call_runtime15 = call i64 @report_malloc(i8* %pointer_cast14, i8* getelementptr inbounds ([27 x i8], [27 x i8]* @name_of_obj.15, i32 0, i32 0))
  %ptr_to_control_block16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13, i32 0, i32 0
  %ptr_to_refcnt17 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt17, align 4
  %ptr_to_dtor_field18 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 1
  store void (i8*)* @dtor.5, void (i8*)** %ptr_to_dtor_field18, align 8
  %ptr_to_obj_id19 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 2
  store i64 %call_runtime15, i64* %ptr_to_obj_id19, align 4
  %ptr_to_field20 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.10, i8* (i8*, i8*)** %ptr_to_field20, align 8
  %pointer_cast21 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13 to i8*
  call void @release_obj(i8* %pointer_cast21)
  %malloccall22 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj23 = bitcast i8* %malloccall22 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast24 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23 to i8*
  %call_runtime25 = call i64 @report_malloc(i8* %pointer_cast24, i8* getelementptr inbounds ([28 x i8], [28 x i8]* @name_of_obj.21, i32 0, i32 0))
  %ptr_to_control_block26 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23, i32 0, i32 0
  %ptr_to_refcnt27 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt27, align 4
  %ptr_to_dtor_field28 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 1
  store void (i8*)* @dtor.5, void (i8*)** %ptr_to_dtor_field28, align 8
  %ptr_to_obj_id29 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 2
  store i64 %call_runtime25, i64* %ptr_to_obj_id29, align 4
  %ptr_to_field30 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.16, i8* (i8*, i8*)** %ptr_to_field30, align 8
  %pointer_cast31 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23 to i8*
  %malloccall32 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj33 = bitcast i8* %malloccall32 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast34 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj33 to i8*
  %call_runtime35 = call i64 @report_malloc(i8* %pointer_cast34, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.22, i32 0, i32 0))
  %ptr_to_control_block36 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj33, i32 0, i32 0
  %ptr_to_refcnt37 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt37, align 4
  %ptr_to_dtor_field38 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 1
  store void (i8*)* @dtor.19, void (i8*)** %ptr_to_dtor_field38, align 8
  %ptr_to_obj_id39 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 2
  store i64 %call_runtime35, i64* %ptr_to_obj_id39, align 4
  %ptr_to_field40 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj33, i32 0, i32 1
  store i64 3, i64* %ptr_to_field40, align 4
  %pointer_cast41 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj33 to i8*
  %pointer_cast42 = bitcast i8* %pointer_cast31 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field43 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast42, i32 0, i32 1
  %field_value = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field43, align 8
  %call_lambda = tail call i8* %field_value(i8* %pointer_cast41, i8* %pointer_cast31)
  %malloccall44 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj45 = bitcast i8* %malloccall44 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast46 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj45 to i8*
  %call_runtime47 = call i64 @report_malloc(i8* %pointer_cast46, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.23, i32 0, i32 0))
  %ptr_to_control_block48 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj45, i32 0, i32 0
  %ptr_to_refcnt49 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block48, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt49, align 4
  %ptr_to_dtor_field50 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block48, i32 0, i32 1
  store void (i8*)* @dtor.19, void (i8*)** %ptr_to_dtor_field50, align 8
  %ptr_to_obj_id51 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block48, i32 0, i32 2
  store i64 %call_runtime47, i64* %ptr_to_obj_id51, align 4
  %ptr_to_field52 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj45, i32 0, i32 1
  store i64 5, i64* %ptr_to_field52, align 4
  %pointer_cast53 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj45 to i8*
  %pointer_cast54 = bitcast i8* %call_lambda to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field55 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast54, i32 0, i32 1
  %field_value56 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field55, align 8
  %call_lambda57 = tail call i8* %field_value56(i8* %pointer_cast53, i8* %call_lambda)
  %pointer_cast58 = bitcast i8* %call_lambda57 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field59 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast58, i32 0, i32 1
  %field_value60 = load i64, i64* %ptr_to_field59, align 4
  call void @release_obj(i8* %call_lambda57)
  call void @check_leak()
  ret i64 %field_value60
}

define i8* @lambda(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([30 x i8], [30 x i8]* @name_of_obj.2, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.3, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.1, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.1(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @retain_obj(i8* %field_value)
  call void @release_obj(i8* %1)
  %pointer_cast1 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load i64, i64* %ptr_to_field2, align 4
  call void @release_obj(i8* %field_value)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, { i64, i8** } }* getelementptr ({ { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
  %pointer_cast4 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast4, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %array_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj, i32 0, i32 1
  %ptr_to_field5 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field, i32 0, i32 0
  store i64 %field_value3, i64* %ptr_to_field5, align 4
  %2 = trunc i64 %field_value3 to i32
  %mallocsize = mul i32 %2, ptrtoint (i1** getelementptr (i1*, i1** null, i32 1) to i32)
  %malloccall6 = tail call i8* @malloc(i32 %mallocsize)
  %buffer_ptr = bitcast i8* %malloccall6 to i8**
  %ptr_to_field7 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field, i32 0, i32 1
  store i8** %buffer_ptr, i8*** %ptr_to_field7, align 8
  %ptr_to_field8 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field, i32 0, i32 0
  %field_value9 = load i64, i64* %ptr_to_field8, align 4
  %ptr_to_field10 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field, i32 0, i32 1
  %field_value11 = load i8**, i8*** %ptr_to_field10, align 8
  %release_loop_counter = alloca i64, align 8
  store i64 0, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

loop_release_array_elements:                      ; preds = %loop_body, %entry
  %counter_val = load i64, i64* %release_loop_counter, align 4
  %is_end = icmp eq i64 %counter_val, %field_value9
  br i1 %is_end, label %after_loop, label %loop_body

loop_body:                                        ; preds = %loop_release_array_elements
  call void @retain_obj(i8* %0)
  %ptr_to_elem_of_array = getelementptr i8*, i8** %field_value11, i64 %counter_val
  store i8* %0, i8** %ptr_to_elem_of_array, align 8
  %incremented_counter_val = add i64 %counter_val, 1
  store i64 %incremented_counter_val, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

after_loop:                                       ; preds = %loop_release_array_elements
  call void @release_obj(i8* %0)
  %pointer_cast12 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj to i8*
  ret i8* %pointer_cast12
}

declare noalias i8* @malloc(i32)

define void @dtor(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
  %ptr_to_array = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast, i32 0, i32 1
  %ptr_to_field = getelementptr inbounds { i64, i8** }, { i64, i8** }* %ptr_to_array, i32 0, i32 0
  %field_value = load i64, i64* %ptr_to_field, align 4
  %ptr_to_field1 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %ptr_to_array, i32 0, i32 1
  %field_value2 = load i8**, i8*** %ptr_to_field1, align 8
  %release_loop_counter = alloca i64, align 8
  store i64 0, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

loop_release_array_elements:                      ; preds = %loop_body, %entry
  %counter_val = load i64, i64* %release_loop_counter, align 4
  %is_end = icmp eq i64 %counter_val, %field_value
  br i1 %is_end, label %after_loop, label %loop_body

loop_body:                                        ; preds = %loop_release_array_elements
  %ptr_to_elem_of_array = getelementptr i8*, i8** %field_value2, i64 %counter_val
  %elem_of_array = load i8*, i8** %ptr_to_elem_of_array, align 8
  call void @release_obj(i8* %elem_of_array)
  %incremented_counter_val = add i64 %counter_val, 1
  store i64 %incremented_counter_val, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

after_loop:                                       ; preds = %loop_release_array_elements
  %1 = bitcast i8** %field_value2 to i8*
  tail call void @free(i8* %1)
  ret void
}

define void @dtor.3(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value)
  ret void
}

define void @dtor.5(i8* %0) {
entry:
  ret void
}

define i8* @lambda.6(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([14 x i8], [14 x i8]* @name_of_obj.8, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.3, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.7, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.7(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @retain_obj(i8* %field_value)
  %pointer_cast1 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field2, align 8
  %call_lambda = tail call i8* %field_value3(i8* %1, i8* %field_value)
  %pointer_cast4 = bitcast i8* %call_lambda to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field5, align 8
  %call_lambda7 = tail call i8* %field_value6(i8* %0, i8* %call_lambda)
  ret i8* %call_lambda7
}

define i8* @lambda.10(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([19 x i8], [19 x i8]* @name_of_obj.14, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.3, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.11, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.11(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @retain_obj(i8* %field_value)
  call void @release_obj(i8* %1)
  %pointer_cast1 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load i64, i64* %ptr_to_field2, align 4
  %pointer_cast4 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load i64, i64* %ptr_to_field5, align 4
  %eq = icmp eq i64 %field_value3, %field_value6
  %eq_bool = sext i1 %eq to i8
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8 }* getelementptr ({ { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8 }*
  %pointer_cast7 = bitcast { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast7, i8* getelementptr inbounds ([11 x i8], [11 x i8]* @name_of_obj.12, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.13, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj, i32 0, i32 1
  store i8 %eq_bool, i8* %ptr_to_field8, align 1
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj to i8*
  ret i8* %pointer_cast9
}

define void @dtor.13(i8* %0) {
entry:
  ret void
}

define i8* @lambda.16(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.20, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.3, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.17, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.17(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @retain_obj(i8* %field_value)
  call void @release_obj(i8* %1)
  %pointer_cast1 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast1, i32 0, i32 1
  %field_value3 = load i64, i64* %ptr_to_field2, align 4
  %pointer_cast4 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load i64, i64* %ptr_to_field5, align 4
  %add = add i64 %field_value3, %field_value6
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast7 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast7, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @name_of_obj.18, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.19, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 1
  store i64 %add, i64* %ptr_to_field8, align 4
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  ret i8* %pointer_cast9
}

define void @dtor.19(i8* %0) {
entry:
  ret void
}
