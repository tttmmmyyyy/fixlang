; ModuleID = 'main'
source_filename = "main"

@name_of_obj = private unnamed_addr constant [27 x i8] c"writeArray array idx value\00", align 1
@name_of_obj.3 = private unnamed_addr constant [37 x i8] c"\\value->(writeArray array idx value)\00", align 1
@name_of_obj.5 = private unnamed_addr constant [45 x i8] c"\\idx->(\\value->(writeArray array idx value))\00", align 1
@name_of_obj.7 = private unnamed_addr constant [55 x i8] c"\\array->(\\idx->(\\value->(writeArray array idx value)))\00", align 1
@name_of_obj.11 = private unnamed_addr constant [28 x i8] c"\\idx->(readArray array idx)\00", align 1
@name_of_obj.12 = private unnamed_addr constant [38 x i8] c"\\array->(\\idx->(readArray array idx))\00", align 1
@name_of_obj.15 = private unnamed_addr constant [20 x i8] c"newArray size value\00", align 1
@name_of_obj.16 = private unnamed_addr constant [30 x i8] c"\\value->(newArray size value)\00", align 1
@name_of_obj.17 = private unnamed_addr constant [39 x i8] c"\\size->(\\value->(newArray size value))\00", align 1
@name_of_obj.20 = private unnamed_addr constant [14 x i8] c"\\x->(fix f x)\00", align 1
@name_of_obj.21 = private unnamed_addr constant [20 x i8] c"\\f->(\\x->(fix f x))\00", align 1
@name_of_obj.24 = private unnamed_addr constant [11 x i8] c"eq lhs rhs\00", align 1
@name_of_obj.26 = private unnamed_addr constant [19 x i8] c"\\rhs->(eq lhs rhs)\00", align 1
@name_of_obj.27 = private unnamed_addr constant [27 x i8] c"\\lhs->(\\rhs->(eq lhs rhs))\00", align 1
@name_of_obj.30 = private unnamed_addr constant [12 x i8] c"add lhs rhs\00", align 1
@name_of_obj.32 = private unnamed_addr constant [20 x i8] c"\\rhs->(add lhs rhs)\00", align 1
@name_of_obj.33 = private unnamed_addr constant [28 x i8] c"\\lhs->(\\rhs->(add lhs rhs))\00", align 1
@name_of_obj.34 = private unnamed_addr constant [2 x i8] c"3\00", align 1
@name_of_obj.35 = private unnamed_addr constant [2 x i8] c"5\00", align 1

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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([55 x i8], [55 x i8]* @name_of_obj.7, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda, i8* (i8*, i8*)** %ptr_to_field, align 8
  %pointer_cast1 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj to i8*
  call void @release_obj(i8* %pointer_cast1)
  %malloccall2 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj3 = bitcast i8* %malloccall2 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast4 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3 to i8*
  %call_runtime5 = call i64 @report_malloc(i8* %pointer_cast4, i8* getelementptr inbounds ([38 x i8], [38 x i8]* @name_of_obj.12, i32 0, i32 0))
  %ptr_to_control_block6 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3, i32 0, i32 0
  %ptr_to_refcnt7 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block6, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt7, align 4
  %ptr_to_dtor_field8 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block6, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field8, align 8
  %ptr_to_obj_id9 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block6, i32 0, i32 2
  store i64 %call_runtime5, i64* %ptr_to_obj_id9, align 4
  %ptr_to_field10 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.9, i8* (i8*, i8*)** %ptr_to_field10, align 8
  %pointer_cast11 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj3 to i8*
  call void @release_obj(i8* %pointer_cast11)
  %malloccall12 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj13 = bitcast i8* %malloccall12 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast14 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13 to i8*
  %call_runtime15 = call i64 @report_malloc(i8* %pointer_cast14, i8* getelementptr inbounds ([39 x i8], [39 x i8]* @name_of_obj.17, i32 0, i32 0))
  %ptr_to_control_block16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13, i32 0, i32 0
  %ptr_to_refcnt17 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt17, align 4
  %ptr_to_dtor_field18 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field18, align 8
  %ptr_to_obj_id19 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 2
  store i64 %call_runtime15, i64* %ptr_to_obj_id19, align 4
  %ptr_to_field20 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.13, i8* (i8*, i8*)** %ptr_to_field20, align 8
  %pointer_cast21 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13 to i8*
  call void @release_obj(i8* %pointer_cast21)
  %malloccall22 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj23 = bitcast i8* %malloccall22 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast24 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23 to i8*
  %call_runtime25 = call i64 @report_malloc(i8* %pointer_cast24, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.21, i32 0, i32 0))
  %ptr_to_control_block26 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23, i32 0, i32 0
  %ptr_to_refcnt27 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt27, align 4
  %ptr_to_dtor_field28 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field28, align 8
  %ptr_to_obj_id29 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 2
  store i64 %call_runtime25, i64* %ptr_to_obj_id29, align 4
  %ptr_to_field30 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.18, i8* (i8*, i8*)** %ptr_to_field30, align 8
  %pointer_cast31 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23 to i8*
  call void @release_obj(i8* %pointer_cast31)
  %malloccall32 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj33 = bitcast i8* %malloccall32 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast34 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33 to i8*
  %call_runtime35 = call i64 @report_malloc(i8* %pointer_cast34, i8* getelementptr inbounds ([27 x i8], [27 x i8]* @name_of_obj.27, i32 0, i32 0))
  %ptr_to_control_block36 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33, i32 0, i32 0
  %ptr_to_refcnt37 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt37, align 4
  %ptr_to_dtor_field38 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field38, align 8
  %ptr_to_obj_id39 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 2
  store i64 %call_runtime35, i64* %ptr_to_obj_id39, align 4
  %ptr_to_field40 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.22, i8* (i8*, i8*)** %ptr_to_field40, align 8
  %pointer_cast41 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33 to i8*
  call void @release_obj(i8* %pointer_cast41)
  %malloccall42 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj43 = bitcast i8* %malloccall42 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast44 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43 to i8*
  %call_runtime45 = call i64 @report_malloc(i8* %pointer_cast44, i8* getelementptr inbounds ([28 x i8], [28 x i8]* @name_of_obj.33, i32 0, i32 0))
  %ptr_to_control_block46 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43, i32 0, i32 0
  %ptr_to_refcnt47 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block46, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt47, align 4
  %ptr_to_dtor_field48 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block46, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field48, align 8
  %ptr_to_obj_id49 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block46, i32 0, i32 2
  store i64 %call_runtime45, i64* %ptr_to_obj_id49, align 4
  %ptr_to_field50 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.28, i8* (i8*, i8*)** %ptr_to_field50, align 8
  %pointer_cast51 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43 to i8*
  %malloccall52 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj53 = bitcast i8* %malloccall52 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast54 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53 to i8*
  %call_runtime55 = call i64 @report_malloc(i8* %pointer_cast54, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.34, i32 0, i32 0))
  %ptr_to_control_block56 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53, i32 0, i32 0
  %ptr_to_refcnt57 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt57, align 4
  %ptr_to_dtor_field58 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field58, align 8
  %ptr_to_obj_id59 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 2
  store i64 %call_runtime55, i64* %ptr_to_obj_id59, align 4
  %ptr_to_field60 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53, i32 0, i32 1
  store i64 3, i64* %ptr_to_field60, align 4
  %pointer_cast61 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53 to i8*
  %pointer_cast62 = bitcast i8* %pointer_cast51 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field63 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast62, i32 0, i32 1
  %field_value = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field63, align 8
  %call_lambda = tail call i8* %field_value(i8* %pointer_cast61, i8* %pointer_cast51)
  %malloccall64 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj65 = bitcast i8* %malloccall64 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast66 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj65 to i8*
  %call_runtime67 = call i64 @report_malloc(i8* %pointer_cast66, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.35, i32 0, i32 0))
  %ptr_to_control_block68 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj65, i32 0, i32 0
  %ptr_to_refcnt69 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block68, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt69, align 4
  %ptr_to_dtor_field70 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block68, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field70, align 8
  %ptr_to_obj_id71 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block68, i32 0, i32 2
  store i64 %call_runtime67, i64* %ptr_to_obj_id71, align 4
  %ptr_to_field72 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj65, i32 0, i32 1
  store i64 5, i64* %ptr_to_field72, align 4
  %pointer_cast73 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj65 to i8*
  %pointer_cast74 = bitcast i8* %call_lambda to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field75 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast74, i32 0, i32 1
  %field_value76 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field75, align 8
  %call_lambda77 = tail call i8* %field_value76(i8* %pointer_cast73, i8* %call_lambda)
  %pointer_cast78 = bitcast i8* %call_lambda77 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field79 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast78, i32 0, i32 1
  %field_value80 = load i64, i64* %ptr_to_field79, align 4
  call void @release_obj(i8* %call_lambda77)
  call void @check_leak()
  ret i64 %field_value80
}

define i8* @lambda(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([45 x i8], [45 x i8]* @name_of_obj.5, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
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
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }*
  %pointer_cast1 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast1, i8* getelementptr inbounds ([37 x i8], [37 x i8]* @name_of_obj.3, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.4, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.2, i8* (i8*, i8*)** %ptr_to_field2, align 8
  %ptr_to_field3 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field3, align 8
  %ptr_to_field4 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %field_value, i8** %ptr_to_field4, align 8
  %pointer_cast5 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast5
}

define i8* @lambda.2(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %pointer_cast1 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @retain_obj(i8* %field_value)
  call void @retain_obj(i8* %field_value3)
  call void @release_obj(i8* %1)
  %pointer_cast4 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load i64, i64* %ptr_to_field5, align 4
  call void @release_obj(i8* %field_value)
  %pointer_cast7 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
  %2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast7, i32 0, i32 1
  %pointer_cast8 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast7 to { i64, void (i8*)*, i64 }*
  %ptr_to_field9 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %pointer_cast8, i32 0, i32 0
  %field_value10 = load i64, i64* %ptr_to_field9, align 4
  %is_unique = icmp eq i64 %field_value10, 1
  br i1 %is_unique, label %cont_bb, label %shared_bb

shared_bb:                                        ; preds = %entry
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, { i64, i8** } }* getelementptr ({ { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
  %pointer_cast11 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast11, i8* getelementptr inbounds ([27 x i8], [27 x i8]* @name_of_obj, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %3 = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj, i32 0, i32 1
  %ptr_to_field12 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 0
  %field_value13 = load i64, i64* %ptr_to_field12, align 4
  %ptr_to_field14 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 1
  %field_value15 = load i8**, i8*** %ptr_to_field14, align 8
  %ptr_to_field16 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %3, i32 0, i32 0
  store i64 %field_value13, i64* %ptr_to_field16, align 4
  %4 = trunc i64 %field_value13 to i32
  %mallocsize = mul i32 %4, ptrtoint (i1** getelementptr (i1*, i1** null, i32 1) to i32)
  %malloccall17 = tail call i8* @malloc(i32 %mallocsize)
  %dst_buffer = bitcast i8* %malloccall17 to i8**
  %ptr_to_field18 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %3, i32 0, i32 1
  store i8** %dst_buffer, i8*** %ptr_to_field18, align 8
  %ptr_to_field19 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 0
  %field_value20 = load i64, i64* %ptr_to_field19, align 4
  %ptr_to_field21 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 1
  %field_value22 = load i8**, i8*** %ptr_to_field21, align 8
  %release_loop_counter = alloca i64, align 8
  store i64 0, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

cont_bb:                                          ; preds = %after_loop, %entry
  %array_phi = phi { { i64, void (i8*)*, i64 }, { i64, i8** } }* [ %pointer_cast7, %entry ], [ %ptr_to_obj, %after_loop ]
  %array_field_phi = phi { i64, i8** }* [ %2, %entry ], [ %3, %after_loop ]
  %ptr_to_field24 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field_phi, i32 0, i32 0
  %field_value25 = load i64, i64* %ptr_to_field24, align 4
  %ptr_to_field26 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field_phi, i32 0, i32 1
  %field_value27 = load i8**, i8*** %ptr_to_field26, align 8
  %ptr_to_elem_of_array = getelementptr i8*, i8** %field_value27, i64 %field_value6
  %elem = load i8*, i8** %ptr_to_elem_of_array, align 8
  call void @release_obj(i8* %elem)
  store i8* %0, i8** %ptr_to_elem_of_array, align 8
  %pointer_cast28 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %array_phi to i8*
  ret i8* %pointer_cast28

loop_release_array_elements:                      ; preds = %loop_body, %shared_bb
  %counter_val = load i64, i64* %release_loop_counter, align 4
  %is_end = icmp eq i64 %counter_val, %field_value20
  br i1 %is_end, label %after_loop, label %loop_body

loop_body:                                        ; preds = %loop_release_array_elements
  %ptr_to_src_elem = getelementptr i8*, i8** %field_value15, i64 %counter_val
  %ptr_to_dst_elem = getelementptr i8*, i8** %dst_buffer, i64 %counter_val
  %src_elem = load i8*, i8** %ptr_to_src_elem, align 8
  call void @retain_obj(i8* %src_elem)
  store i8* %src_elem, i8** %ptr_to_dst_elem, align 8
  %incremented_counter_val = add i64 %counter_val, 1
  store i64 %incremented_counter_val, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

after_loop:                                       ; preds = %loop_release_array_elements
  %pointer_cast23 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast7 to i8*
  call void @release_obj(i8* %pointer_cast23)
  br label %cont_bb
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

define void @dtor.4(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value)
  %pointer_cast1 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value3)
  ret void
}

define void @dtor.6(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value)
  ret void
}

define void @dtor.8(i8* %0) {
entry:
  ret void
}

define i8* @lambda.9(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([28 x i8], [28 x i8]* @name_of_obj.11, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.10, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.10(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @retain_obj(i8* %field_value)
  call void @release_obj(i8* %1)
  %pointer_cast1 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
  %array_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast1, i32 0, i32 1
  %pointer_cast2 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field3 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast2, i32 0, i32 1
  %field_value4 = load i64, i64* %ptr_to_field3, align 4
  call void @release_obj(i8* %0)
  %ptr_to_field5 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field, i32 0, i32 0
  %field_value6 = load i64, i64* %ptr_to_field5, align 4
  %ptr_to_field7 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field, i32 0, i32 1
  %field_value8 = load i8**, i8*** %ptr_to_field7, align 8
  %ptr_to_elem_of_array = getelementptr i8*, i8** %field_value8, i64 %field_value4
  %elem = load i8*, i8** %ptr_to_elem_of_array, align 8
  call void @retain_obj(i8* %elem)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast1 to i8*
  call void @release_obj(i8* %pointer_cast9)
  ret i8* %elem
}

define i8* @lambda.13(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([30 x i8], [30 x i8]* @name_of_obj.16, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.14, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.14(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast4, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.15, i32 0, i32 0))
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

define i8* @lambda.18(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([14 x i8], [14 x i8]* @name_of_obj.20, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.19, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.19(i8* %0, i8* %1) {
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

define i8* @lambda.22(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([19 x i8], [19 x i8]* @name_of_obj.26, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.23, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.23(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast7, i8* getelementptr inbounds ([11 x i8], [11 x i8]* @name_of_obj.24, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.25, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj, i32 0, i32 1
  store i8 %eq_bool, i8* %ptr_to_field8, align 1
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj to i8*
  ret i8* %pointer_cast9
}

define void @dtor.25(i8* %0) {
entry:
  ret void
}

define i8* @lambda.28(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.32, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.29, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.29(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast7, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @name_of_obj.30, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 1
  store i64 %add, i64* %ptr_to_field8, align 4
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  ret i8* %pointer_cast9
}

define void @dtor.31(i8* %0) {
entry:
  ret void
}
