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
@name_of_obj.34 = private unnamed_addr constant [3 x i8] c"31\00", align 1
@name_of_obj.35 = private unnamed_addr constant [2 x i8] c"0\00", align 1
@name_of_obj.36 = private unnamed_addr constant [2 x i8] c"0\00", align 1
@name_of_obj.37 = private unnamed_addr constant [2 x i8] c"0\00", align 1
@name_of_obj.38 = private unnamed_addr constant [2 x i8] c"1\00", align 1
@name_of_obj.39 = private unnamed_addr constant [2 x i8] c"1\00", align 1
@name_of_obj.43 = private unnamed_addr constant [3 x i8] c"31\00", align 1
@name_of_obj.44 = private unnamed_addr constant [3 x i8] c"-1\00", align 1
@name_of_obj.45 = private unnamed_addr constant [3 x i8] c"-2\00", align 1
@name_of_obj.46 = private unnamed_addr constant [2 x i8] c"1\00", align 1
@name_of_obj.47 = private unnamed_addr constant [229 x i8] c"\\n->(if ((eq) (n)) (31) then arr else (let x=((readArray) (arr)) (((add) (n)) (-1)) in (let y=((readArray) (arr)) (((add) (n)) (-2)) in (let arr=(((writeArray) (arr)) (n)) (((add) (x)) (y)) in (((f) (arr)) (((add) (n)) (1)))))))\00", align 1
@name_of_obj.49 = private unnamed_addr constant [237 x i8] c"\\arr->(\\n->(if ((eq) (n)) (31) then arr else (let x=((readArray) (arr)) (((add) (n)) (-1)) in (let y=((readArray) (arr)) (((add) (n)) (-2)) in (let arr=(((writeArray) (arr)) (n)) (((add) (x)) (y)) in (((f) (arr)) (((add) (n)) (1))))))))\00", align 1
@name_of_obj.51 = private unnamed_addr constant [243 x i8] c"\\f->(\\arr->(\\n->(if ((eq) (n)) (31) then arr else (let x=((readArray) (arr)) (((add) (n)) (-1)) in (let y=((readArray) (arr)) (((add) (n)) (-2)) in (let arr=(((writeArray) (arr)) (n)) (((add) (x)) (y)) in (((f) (arr)) (((add) (n)) (1)))))))))\00", align 1
@name_of_obj.53 = private unnamed_addr constant [2 x i8] c"2\00", align 1
@name_of_obj.54 = private unnamed_addr constant [3 x i8] c"30\00", align 1

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
  %call_runtime55 = call i64 @report_malloc(i8* %pointer_cast54, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.34, i32 0, i32 0))
  %ptr_to_control_block56 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53, i32 0, i32 0
  %ptr_to_refcnt57 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt57, align 4
  %ptr_to_dtor_field58 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field58, align 8
  %ptr_to_obj_id59 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 2
  store i64 %call_runtime55, i64* %ptr_to_obj_id59, align 4
  %ptr_to_field60 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53, i32 0, i32 1
  store i64 31, i64* %ptr_to_field60, align 4
  %pointer_cast61 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj53 to i8*
  %pointer_cast62 = bitcast i8* %pointer_cast21 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field63 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast62, i32 0, i32 1
  %field_value = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field63, align 8
  %call_lambda = tail call i8* %field_value(i8* %pointer_cast61, i8* %pointer_cast21)
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
  store i64 0, i64* %ptr_to_field72, align 4
  %pointer_cast73 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj65 to i8*
  %pointer_cast74 = bitcast i8* %call_lambda to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field75 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast74, i32 0, i32 1
  %field_value76 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field75, align 8
  %call_lambda77 = tail call i8* %field_value76(i8* %pointer_cast73, i8* %call_lambda)
  call void @retain_obj(i8* %pointer_cast1)
  %pointer_cast78 = bitcast i8* %pointer_cast1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field79 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast78, i32 0, i32 1
  %field_value80 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field79, align 8
  %call_lambda81 = tail call i8* %field_value80(i8* %call_lambda77, i8* %pointer_cast1)
  %malloccall82 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj83 = bitcast i8* %malloccall82 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast84 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj83 to i8*
  %call_runtime85 = call i64 @report_malloc(i8* %pointer_cast84, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.36, i32 0, i32 0))
  %ptr_to_control_block86 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj83, i32 0, i32 0
  %ptr_to_refcnt87 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block86, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt87, align 4
  %ptr_to_dtor_field88 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block86, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field88, align 8
  %ptr_to_obj_id89 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block86, i32 0, i32 2
  store i64 %call_runtime85, i64* %ptr_to_obj_id89, align 4
  %ptr_to_field90 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj83, i32 0, i32 1
  store i64 0, i64* %ptr_to_field90, align 4
  %pointer_cast91 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj83 to i8*
  %pointer_cast92 = bitcast i8* %call_lambda81 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field93 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast92, i32 0, i32 1
  %field_value94 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field93, align 8
  %call_lambda95 = tail call i8* %field_value94(i8* %pointer_cast91, i8* %call_lambda81)
  %malloccall96 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj97 = bitcast i8* %malloccall96 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast98 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj97 to i8*
  %call_runtime99 = call i64 @report_malloc(i8* %pointer_cast98, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.37, i32 0, i32 0))
  %ptr_to_control_block100 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj97, i32 0, i32 0
  %ptr_to_refcnt101 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block100, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt101, align 4
  %ptr_to_dtor_field102 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block100, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field102, align 8
  %ptr_to_obj_id103 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block100, i32 0, i32 2
  store i64 %call_runtime99, i64* %ptr_to_obj_id103, align 4
  %ptr_to_field104 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj97, i32 0, i32 1
  store i64 0, i64* %ptr_to_field104, align 4
  %pointer_cast105 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj97 to i8*
  %pointer_cast106 = bitcast i8* %call_lambda95 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field107 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast106, i32 0, i32 1
  %field_value108 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field107, align 8
  %call_lambda109 = tail call i8* %field_value108(i8* %pointer_cast105, i8* %call_lambda95)
  call void @retain_obj(i8* %pointer_cast1)
  %pointer_cast110 = bitcast i8* %pointer_cast1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field111 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast110, i32 0, i32 1
  %field_value112 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field111, align 8
  %call_lambda113 = tail call i8* %field_value112(i8* %call_lambda109, i8* %pointer_cast1)
  %malloccall114 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj115 = bitcast i8* %malloccall114 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast116 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj115 to i8*
  %call_runtime117 = call i64 @report_malloc(i8* %pointer_cast116, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.38, i32 0, i32 0))
  %ptr_to_control_block118 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj115, i32 0, i32 0
  %ptr_to_refcnt119 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block118, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt119, align 4
  %ptr_to_dtor_field120 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block118, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field120, align 8
  %ptr_to_obj_id121 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block118, i32 0, i32 2
  store i64 %call_runtime117, i64* %ptr_to_obj_id121, align 4
  %ptr_to_field122 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj115, i32 0, i32 1
  store i64 1, i64* %ptr_to_field122, align 4
  %pointer_cast123 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj115 to i8*
  %pointer_cast124 = bitcast i8* %call_lambda113 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field125 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast124, i32 0, i32 1
  %field_value126 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field125, align 8
  %call_lambda127 = tail call i8* %field_value126(i8* %pointer_cast123, i8* %call_lambda113)
  %malloccall128 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj129 = bitcast i8* %malloccall128 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast130 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj129 to i8*
  %call_runtime131 = call i64 @report_malloc(i8* %pointer_cast130, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.39, i32 0, i32 0))
  %ptr_to_control_block132 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj129, i32 0, i32 0
  %ptr_to_refcnt133 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block132, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt133, align 4
  %ptr_to_dtor_field134 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block132, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field134, align 8
  %ptr_to_obj_id135 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block132, i32 0, i32 2
  store i64 %call_runtime131, i64* %ptr_to_obj_id135, align 4
  %ptr_to_field136 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj129, i32 0, i32 1
  store i64 1, i64* %ptr_to_field136, align 4
  %pointer_cast137 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj129 to i8*
  %pointer_cast138 = bitcast i8* %call_lambda127 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field139 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast138, i32 0, i32 1
  %field_value140 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field139, align 8
  %call_lambda141 = tail call i8* %field_value140(i8* %pointer_cast137, i8* %call_lambda127)
  %malloccall142 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* null, i32 1) to i32))
  %ptr_to_obj143 = bitcast i8* %malloccall142 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %pointer_cast144 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143 to i8*
  %call_runtime145 = call i64 @report_malloc(i8* %pointer_cast144, i8* getelementptr inbounds ([243 x i8], [243 x i8]* @name_of_obj.51, i32 0, i32 0))
  %ptr_to_control_block146 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143, i32 0, i32 0
  %ptr_to_refcnt147 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block146, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt147, align 4
  %ptr_to_dtor_field148 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block146, i32 0, i32 1
  store void (i8*)* @dtor.52, void (i8*)** %ptr_to_dtor_field148, align 8
  %ptr_to_obj_id149 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block146, i32 0, i32 2
  store i64 %call_runtime145, i64* %ptr_to_obj_id149, align 4
  %ptr_to_field150 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.40, i8* (i8*, i8*)** %ptr_to_field150, align 8
  %ptr_to_field151 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143, i32 0, i32 2
  store i8* %pointer_cast1, i8** %ptr_to_field151, align 8
  call void @retain_obj(i8* %pointer_cast11)
  %ptr_to_field152 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143, i32 0, i32 3
  store i8* %pointer_cast11, i8** %ptr_to_field152, align 8
  %ptr_to_field153 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143, i32 0, i32 4
  store i8* %pointer_cast51, i8** %ptr_to_field153, align 8
  %ptr_to_field154 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143, i32 0, i32 5
  store i8* %pointer_cast41, i8** %ptr_to_field154, align 8
  %pointer_cast155 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj143 to i8*
  %pointer_cast156 = bitcast i8* %pointer_cast31 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field157 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast156, i32 0, i32 1
  %field_value158 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field157, align 8
  %call_lambda159 = tail call i8* %field_value158(i8* %pointer_cast155, i8* %pointer_cast31)
  %pointer_cast160 = bitcast i8* %call_lambda159 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field161 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast160, i32 0, i32 1
  %field_value162 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field161, align 8
  %call_lambda163 = tail call i8* %field_value162(i8* %call_lambda141, i8* %call_lambda159)
  %malloccall164 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj165 = bitcast i8* %malloccall164 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast166 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj165 to i8*
  %call_runtime167 = call i64 @report_malloc(i8* %pointer_cast166, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.53, i32 0, i32 0))
  %ptr_to_control_block168 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj165, i32 0, i32 0
  %ptr_to_refcnt169 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block168, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt169, align 4
  %ptr_to_dtor_field170 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block168, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field170, align 8
  %ptr_to_obj_id171 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block168, i32 0, i32 2
  store i64 %call_runtime167, i64* %ptr_to_obj_id171, align 4
  %ptr_to_field172 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj165, i32 0, i32 1
  store i64 2, i64* %ptr_to_field172, align 4
  %pointer_cast173 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj165 to i8*
  %pointer_cast174 = bitcast i8* %call_lambda163 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field175 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast174, i32 0, i32 1
  %field_value176 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field175, align 8
  %call_lambda177 = tail call i8* %field_value176(i8* %pointer_cast173, i8* %call_lambda163)
  %pointer_cast178 = bitcast i8* %pointer_cast11 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field179 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast178, i32 0, i32 1
  %field_value180 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field179, align 8
  %call_lambda181 = tail call i8* %field_value180(i8* %call_lambda177, i8* %pointer_cast11)
  %malloccall182 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj183 = bitcast i8* %malloccall182 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast184 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj183 to i8*
  %call_runtime185 = call i64 @report_malloc(i8* %pointer_cast184, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.54, i32 0, i32 0))
  %ptr_to_control_block186 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj183, i32 0, i32 0
  %ptr_to_refcnt187 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block186, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt187, align 4
  %ptr_to_dtor_field188 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block186, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field188, align 8
  %ptr_to_obj_id189 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block186, i32 0, i32 2
  store i64 %call_runtime185, i64* %ptr_to_obj_id189, align 4
  %ptr_to_field190 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj183, i32 0, i32 1
  store i64 30, i64* %ptr_to_field190, align 4
  %pointer_cast191 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj183 to i8*
  %pointer_cast192 = bitcast i8* %call_lambda181 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field193 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast192, i32 0, i32 1
  %field_value194 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field193, align 8
  %call_lambda195 = tail call i8* %field_value194(i8* %pointer_cast191, i8* %call_lambda181)
  %pointer_cast196 = bitcast i8* %call_lambda195 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field197 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast196, i32 0, i32 1
  %field_value198 = load i64, i64* %ptr_to_field197, align 4
  call void @release_obj(i8* %call_lambda195)
  call void @check_leak()
  ret i64 %field_value198
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
  store i8* %field_value, i8** %ptr_to_field3, align 8
  %ptr_to_field4 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %0, i8** %ptr_to_field4, align 8
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
  %pointer_cast4 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast4, i32 0, i32 1
  %field_value6 = load i64, i64* %ptr_to_field5, align 4
  call void @release_obj(i8* %field_value3)
  %pointer_cast7 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
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

define i8* @lambda.40(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %pointer_cast1 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %pointer_cast4 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast4, i32 0, i32 4
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  %pointer_cast7 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast7, i32 0, i32 5
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  call void @retain_obj(i8* %field_value)
  call void @retain_obj(i8* %field_value3)
  call void @retain_obj(i8* %field_value6)
  call void @retain_obj(i8* %field_value9)
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %pointer_cast10 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast10, i8* getelementptr inbounds ([237 x i8], [237 x i8]* @name_of_obj.49, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.50, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field11 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.41, i8* (i8*, i8*)** %ptr_to_field11, align 8
  %ptr_to_field12 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %field_value, i8** %ptr_to_field12, align 8
  %ptr_to_field13 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %field_value3, i8** %ptr_to_field13, align 8
  %ptr_to_field14 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 4
  store i8* %field_value6, i8** %ptr_to_field14, align 8
  %ptr_to_field15 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 5
  store i8* %0, i8** %ptr_to_field15, align 8
  %ptr_to_field16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 6
  store i8* %field_value9, i8** %ptr_to_field16, align 8
  %pointer_cast17 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast17
}

define i8* @lambda.41(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %pointer_cast1 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %pointer_cast4 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast4, i32 0, i32 4
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  %pointer_cast7 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast7, i32 0, i32 5
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  %pointer_cast10 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field11 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast10, i32 0, i32 6
  %field_value12 = load i8*, i8** %ptr_to_field11, align 8
  call void @retain_obj(i8* %field_value)
  call void @retain_obj(i8* %field_value3)
  call void @retain_obj(i8* %field_value6)
  call void @retain_obj(i8* %field_value9)
  call void @retain_obj(i8* %field_value12)
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %pointer_cast13 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast13, i8* getelementptr inbounds ([229 x i8], [229 x i8]* @name_of_obj.47, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.48, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field14 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.42, i8* (i8*, i8*)** %ptr_to_field14, align 8
  %ptr_to_field15 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %field_value, i8** %ptr_to_field15, align 8
  %ptr_to_field16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %field_value3, i8** %ptr_to_field16, align 8
  %ptr_to_field17 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 4
  store i8* %0, i8** %ptr_to_field17, align 8
  %ptr_to_field18 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 5
  store i8* %field_value6, i8** %ptr_to_field18, align 8
  %ptr_to_field19 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 6
  store i8* %field_value9, i8** %ptr_to_field19, align 8
  %ptr_to_field20 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 7
  store i8* %field_value12, i8** %ptr_to_field20, align 8
  %pointer_cast21 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast21
}

define i8* @lambda.42(i8* %0, i8* %1) {
entry:
  %pointer_cast = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  %pointer_cast1 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  %pointer_cast4 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast4, i32 0, i32 4
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  %pointer_cast7 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast7, i32 0, i32 5
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  %pointer_cast10 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field11 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast10, i32 0, i32 6
  %field_value12 = load i8*, i8** %ptr_to_field11, align 8
  %pointer_cast13 = bitcast i8* %1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field14 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast13, i32 0, i32 7
  %field_value15 = load i8*, i8** %ptr_to_field14, align 8
  call void @retain_obj(i8* %field_value)
  call void @retain_obj(i8* %field_value3)
  call void @retain_obj(i8* %field_value6)
  call void @retain_obj(i8* %field_value9)
  call void @retain_obj(i8* %field_value12)
  call void @retain_obj(i8* %field_value15)
  call void @release_obj(i8* %1)
  call void @retain_obj(i8* %0)
  %pointer_cast16 = bitcast i8* %field_value15 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field17 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast16, i32 0, i32 1
  %field_value18 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field17, align 8
  %call_lambda = tail call i8* %field_value18(i8* %0, i8* %field_value15)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast19 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast19, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.43, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field20 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 1
  store i64 31, i64* %ptr_to_field20, align 4
  %pointer_cast21 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  %pointer_cast22 = bitcast i8* %call_lambda to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field23 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast22, i32 0, i32 1
  %field_value24 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field23, align 8
  %call_lambda25 = tail call i8* %field_value24(i8* %pointer_cast21, i8* %call_lambda)
  %pointer_cast26 = bitcast i8* %call_lambda25 to { { i64, void (i8*)*, i64 }, i8 }*
  %ptr_to_field27 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %pointer_cast26, i32 0, i32 1
  %field_value28 = load i8, i8* %ptr_to_field27, align 1
  call void @release_obj(i8* %call_lambda25)
  %cond_val_i1 = trunc i8 %field_value28 to i1
  br i1 %cond_val_i1, label %then, label %else

then:                                             ; preds = %entry
  call void @release_obj(i8* %field_value9)
  call void @release_obj(i8* %0)
  call void @release_obj(i8* %field_value12)
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %field_value3)
  br label %cont

else:                                             ; preds = %entry
  call void @retain_obj(i8* %field_value3)
  call void @retain_obj(i8* %field_value6)
  %pointer_cast29 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field30 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast29, i32 0, i32 1
  %field_value31 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field30, align 8
  %call_lambda32 = tail call i8* %field_value31(i8* %field_value6, i8* %field_value3)
  call void @retain_obj(i8* %field_value9)
  call void @retain_obj(i8* %0)
  %pointer_cast33 = bitcast i8* %field_value9 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field34 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast33, i32 0, i32 1
  %field_value35 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field34, align 8
  %call_lambda36 = tail call i8* %field_value35(i8* %0, i8* %field_value9)
  %malloccall37 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj38 = bitcast i8* %malloccall37 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast39 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj38 to i8*
  %call_runtime40 = call i64 @report_malloc(i8* %pointer_cast39, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.44, i32 0, i32 0))
  %ptr_to_control_block41 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj38, i32 0, i32 0
  %ptr_to_refcnt42 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block41, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt42, align 4
  %ptr_to_dtor_field43 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block41, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field43, align 8
  %ptr_to_obj_id44 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block41, i32 0, i32 2
  store i64 %call_runtime40, i64* %ptr_to_obj_id44, align 4
  %ptr_to_field45 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj38, i32 0, i32 1
  store i64 -1, i64* %ptr_to_field45, align 4
  %pointer_cast46 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj38 to i8*
  %pointer_cast47 = bitcast i8* %call_lambda36 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field48 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast47, i32 0, i32 1
  %field_value49 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field48, align 8
  %call_lambda50 = tail call i8* %field_value49(i8* %pointer_cast46, i8* %call_lambda36)
  %pointer_cast51 = bitcast i8* %call_lambda32 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field52 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast51, i32 0, i32 1
  %field_value53 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field52, align 8
  %call_lambda54 = tail call i8* %field_value53(i8* %call_lambda50, i8* %call_lambda32)
  call void @retain_obj(i8* %field_value6)
  %pointer_cast55 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field56 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast55, i32 0, i32 1
  %field_value57 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field56, align 8
  %call_lambda58 = tail call i8* %field_value57(i8* %field_value6, i8* %field_value3)
  call void @retain_obj(i8* %field_value9)
  call void @retain_obj(i8* %0)
  %pointer_cast59 = bitcast i8* %field_value9 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field60 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast59, i32 0, i32 1
  %field_value61 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field60, align 8
  %call_lambda62 = tail call i8* %field_value61(i8* %0, i8* %field_value9)
  %malloccall63 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj64 = bitcast i8* %malloccall63 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast65 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj64 to i8*
  %call_runtime66 = call i64 @report_malloc(i8* %pointer_cast65, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.45, i32 0, i32 0))
  %ptr_to_control_block67 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj64, i32 0, i32 0
  %ptr_to_refcnt68 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block67, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt68, align 4
  %ptr_to_dtor_field69 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block67, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field69, align 8
  %ptr_to_obj_id70 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block67, i32 0, i32 2
  store i64 %call_runtime66, i64* %ptr_to_obj_id70, align 4
  %ptr_to_field71 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj64, i32 0, i32 1
  store i64 -2, i64* %ptr_to_field71, align 4
  %pointer_cast72 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj64 to i8*
  %pointer_cast73 = bitcast i8* %call_lambda62 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field74 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast73, i32 0, i32 1
  %field_value75 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field74, align 8
  %call_lambda76 = tail call i8* %field_value75(i8* %pointer_cast72, i8* %call_lambda62)
  %pointer_cast77 = bitcast i8* %call_lambda58 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field78 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast77, i32 0, i32 1
  %field_value79 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field78, align 8
  %call_lambda80 = tail call i8* %field_value79(i8* %call_lambda76, i8* %call_lambda58)
  %pointer_cast81 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field82 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast81, i32 0, i32 1
  %field_value83 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field82, align 8
  %call_lambda84 = tail call i8* %field_value83(i8* %field_value6, i8* %field_value)
  call void @retain_obj(i8* %0)
  %pointer_cast85 = bitcast i8* %call_lambda84 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field86 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast85, i32 0, i32 1
  %field_value87 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field86, align 8
  %call_lambda88 = tail call i8* %field_value87(i8* %0, i8* %call_lambda84)
  call void @retain_obj(i8* %field_value9)
  %pointer_cast89 = bitcast i8* %field_value9 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field90 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast89, i32 0, i32 1
  %field_value91 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field90, align 8
  %call_lambda92 = tail call i8* %field_value91(i8* %call_lambda54, i8* %field_value9)
  %pointer_cast93 = bitcast i8* %call_lambda92 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field94 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast93, i32 0, i32 1
  %field_value95 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field94, align 8
  %call_lambda96 = tail call i8* %field_value95(i8* %call_lambda80, i8* %call_lambda92)
  %pointer_cast97 = bitcast i8* %call_lambda88 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field98 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast97, i32 0, i32 1
  %field_value99 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field98, align 8
  %call_lambda100 = tail call i8* %field_value99(i8* %call_lambda96, i8* %call_lambda88)
  %pointer_cast101 = bitcast i8* %field_value12 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field102 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast101, i32 0, i32 1
  %field_value103 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field102, align 8
  %call_lambda104 = tail call i8* %field_value103(i8* %call_lambda100, i8* %field_value12)
  %pointer_cast105 = bitcast i8* %field_value9 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field106 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast105, i32 0, i32 1
  %field_value107 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field106, align 8
  %call_lambda108 = tail call i8* %field_value107(i8* %0, i8* %field_value9)
  %malloccall109 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj110 = bitcast i8* %malloccall109 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast111 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj110 to i8*
  %call_runtime112 = call i64 @report_malloc(i8* %pointer_cast111, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.46, i32 0, i32 0))
  %ptr_to_control_block113 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj110, i32 0, i32 0
  %ptr_to_refcnt114 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block113, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt114, align 4
  %ptr_to_dtor_field115 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block113, i32 0, i32 1
  store void (i8*)* @dtor.31, void (i8*)** %ptr_to_dtor_field115, align 8
  %ptr_to_obj_id116 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block113, i32 0, i32 2
  store i64 %call_runtime112, i64* %ptr_to_obj_id116, align 4
  %ptr_to_field117 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj110, i32 0, i32 1
  store i64 1, i64* %ptr_to_field117, align 4
  %pointer_cast118 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj110 to i8*
  %pointer_cast119 = bitcast i8* %call_lambda108 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field120 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast119, i32 0, i32 1
  %field_value121 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field120, align 8
  %call_lambda122 = tail call i8* %field_value121(i8* %pointer_cast118, i8* %call_lambda108)
  %pointer_cast123 = bitcast i8* %call_lambda104 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field124 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast123, i32 0, i32 1
  %field_value125 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field124, align 8
  %call_lambda126 = tail call i8* %field_value125(i8* %call_lambda122, i8* %call_lambda104)
  br label %cont

cont:                                             ; preds = %else, %then
  %phi = phi i8* [ %field_value6, %then ], [ %call_lambda126, %else ]
  ret i8* %phi
}

define void @dtor.48(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value)
  %pointer_cast1 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value3)
  %pointer_cast4 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast4, i32 0, i32 4
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  call void @release_obj(i8* %field_value6)
  %pointer_cast7 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast7, i32 0, i32 5
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  call void @release_obj(i8* %field_value9)
  %pointer_cast10 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field11 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast10, i32 0, i32 6
  %field_value12 = load i8*, i8** %ptr_to_field11, align 8
  call void @release_obj(i8* %field_value12)
  %pointer_cast13 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field14 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast13, i32 0, i32 7
  %field_value15 = load i8*, i8** %ptr_to_field14, align 8
  call void @release_obj(i8* %field_value15)
  ret void
}

define void @dtor.50(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value)
  %pointer_cast1 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value3)
  %pointer_cast4 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast4, i32 0, i32 4
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  call void @release_obj(i8* %field_value6)
  %pointer_cast7 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast7, i32 0, i32 5
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  call void @release_obj(i8* %field_value9)
  %pointer_cast10 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }*
  %ptr_to_field11 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %pointer_cast10, i32 0, i32 6
  %field_value12 = load i8*, i8** %ptr_to_field11, align 8
  call void @release_obj(i8* %field_value12)
  ret void
}

define void @dtor.52(i8* %0) {
entry:
  %pointer_cast = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast, i32 0, i32 2
  %field_value = load i8*, i8** %ptr_to_field, align 8
  call void @release_obj(i8* %field_value)
  %pointer_cast1 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast1, i32 0, i32 3
  %field_value3 = load i8*, i8** %ptr_to_field2, align 8
  call void @release_obj(i8* %field_value3)
  %pointer_cast4 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field5 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast4, i32 0, i32 4
  %field_value6 = load i8*, i8** %ptr_to_field5, align 8
  call void @release_obj(i8* %field_value6)
  %pointer_cast7 = bitcast i8* %0 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %pointer_cast7, i32 0, i32 5
  %field_value9 = load i8*, i8** %ptr_to_field8, align 8
  call void @release_obj(i8* %field_value9)
  ret void
}
