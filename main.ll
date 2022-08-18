; ModuleID = 'main'
source_filename = "main"

@rust_str = private unnamed_addr constant [40 x i8] c"The argument of writeArray! is shared!\0A\00", align 1
@name_of_obj = private unnamed_addr constant [28 x i8] c"writeArray! array idx value\00", align 1
@name_of_obj.3 = private unnamed_addr constant [38 x i8] c"\\value->(writeArray! array idx value)\00", align 1
@name_of_obj.5 = private unnamed_addr constant [46 x i8] c"\\idx->(\\value->(writeArray! array idx value))\00", align 1
@name_of_obj.7 = private unnamed_addr constant [56 x i8] c"\\array->(\\idx->(\\value->(writeArray! array idx value)))\00", align 1
@name_of_obj.12 = private unnamed_addr constant [27 x i8] c"writeArray array idx value\00", align 1
@name_of_obj.13 = private unnamed_addr constant [37 x i8] c"\\value->(writeArray array idx value)\00", align 1
@name_of_obj.14 = private unnamed_addr constant [45 x i8] c"\\idx->(\\value->(writeArray array idx value))\00", align 1
@name_of_obj.15 = private unnamed_addr constant [55 x i8] c"\\array->(\\idx->(\\value->(writeArray array idx value)))\00", align 1
@name_of_obj.18 = private unnamed_addr constant [28 x i8] c"\\idx->(readArray array idx)\00", align 1
@name_of_obj.19 = private unnamed_addr constant [38 x i8] c"\\array->(\\idx->(readArray array idx))\00", align 1
@name_of_obj.22 = private unnamed_addr constant [20 x i8] c"newArray size value\00", align 1
@name_of_obj.23 = private unnamed_addr constant [30 x i8] c"\\value->(newArray size value)\00", align 1
@name_of_obj.24 = private unnamed_addr constant [39 x i8] c"\\size->(\\value->(newArray size value))\00", align 1
@name_of_obj.27 = private unnamed_addr constant [14 x i8] c"\\x->(fix f x)\00", align 1
@name_of_obj.28 = private unnamed_addr constant [20 x i8] c"\\f->(\\x->(fix f x))\00", align 1
@name_of_obj.31 = private unnamed_addr constant [11 x i8] c"eq lhs rhs\00", align 1
@name_of_obj.33 = private unnamed_addr constant [19 x i8] c"\\rhs->(eq lhs rhs)\00", align 1
@name_of_obj.34 = private unnamed_addr constant [27 x i8] c"\\lhs->(\\rhs->(eq lhs rhs))\00", align 1
@name_of_obj.37 = private unnamed_addr constant [12 x i8] c"add lhs rhs\00", align 1
@name_of_obj.39 = private unnamed_addr constant [20 x i8] c"\\rhs->(add lhs rhs)\00", align 1
@name_of_obj.40 = private unnamed_addr constant [28 x i8] c"\\lhs->(\\rhs->(add lhs rhs))\00", align 1
@name_of_obj.41 = private unnamed_addr constant [3 x i8] c"31\00", align 1
@name_of_obj.42 = private unnamed_addr constant [2 x i8] c"0\00", align 1
@name_of_obj.43 = private unnamed_addr constant [2 x i8] c"0\00", align 1
@name_of_obj.44 = private unnamed_addr constant [2 x i8] c"0\00", align 1
@name_of_obj.45 = private unnamed_addr constant [2 x i8] c"1\00", align 1
@name_of_obj.46 = private unnamed_addr constant [2 x i8] c"1\00", align 1
@name_of_obj.50 = private unnamed_addr constant [3 x i8] c"31\00", align 1
@name_of_obj.51 = private unnamed_addr constant [3 x i8] c"-1\00", align 1
@name_of_obj.52 = private unnamed_addr constant [3 x i8] c"-2\00", align 1
@name_of_obj.53 = private unnamed_addr constant [2 x i8] c"1\00", align 1
@name_of_obj.54 = private unnamed_addr constant [230 x i8] c"\\n->(if ((eq) (n)) (31) then arr else (let x=((readArray) (arr)) (((add) (n)) (-1)) in (let y=((readArray) (arr)) (((add) (n)) (-2)) in (let arr=(((writeArray!) (arr)) (n)) (((add) (x)) (y)) in (((f) (arr)) (((add) (n)) (1)))))))\00", align 1
@name_of_obj.56 = private unnamed_addr constant [238 x i8] c"\\arr->(\\n->(if ((eq) (n)) (31) then arr else (let x=((readArray) (arr)) (((add) (n)) (-1)) in (let y=((readArray) (arr)) (((add) (n)) (-2)) in (let arr=(((writeArray!) (arr)) (n)) (((add) (x)) (y)) in (((f) (arr)) (((add) (n)) (1))))))))\00", align 1
@name_of_obj.58 = private unnamed_addr constant [244 x i8] c"\\f->(\\arr->(\\n->(if ((eq) (n)) (31) then arr else (let x=((readArray) (arr)) (((add) (n)) (-1)) in (let y=((readArray) (arr)) (((add) (n)) (-2)) in (let arr=(((writeArray!) (arr)) (n)) (((add) (x)) (y)) in (((f) (arr)) (((add) (n)) (1)))))))))\00", align 1
@name_of_obj.60 = private unnamed_addr constant [2 x i8] c"2\00", align 1
@name_of_obj.61 = private unnamed_addr constant [3 x i8] c"30\00", align 1

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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([56 x i8], [56 x i8]* @name_of_obj.7, i32 0, i32 0))
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
  %call_runtime5 = call i64 @report_malloc(i8* %pointer_cast4, i8* getelementptr inbounds ([55 x i8], [55 x i8]* @name_of_obj.15, i32 0, i32 0))
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
  %call_runtime15 = call i64 @report_malloc(i8* %pointer_cast14, i8* getelementptr inbounds ([38 x i8], [38 x i8]* @name_of_obj.19, i32 0, i32 0))
  %ptr_to_control_block16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13, i32 0, i32 0
  %ptr_to_refcnt17 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt17, align 4
  %ptr_to_dtor_field18 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field18, align 8
  %ptr_to_obj_id19 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block16, i32 0, i32 2
  store i64 %call_runtime15, i64* %ptr_to_obj_id19, align 4
  %ptr_to_field20 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.16, i8* (i8*, i8*)** %ptr_to_field20, align 8
  %pointer_cast21 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj13 to i8*
  %malloccall22 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj23 = bitcast i8* %malloccall22 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast24 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23 to i8*
  %call_runtime25 = call i64 @report_malloc(i8* %pointer_cast24, i8* getelementptr inbounds ([39 x i8], [39 x i8]* @name_of_obj.24, i32 0, i32 0))
  %ptr_to_control_block26 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23, i32 0, i32 0
  %ptr_to_refcnt27 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt27, align 4
  %ptr_to_dtor_field28 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field28, align 8
  %ptr_to_obj_id29 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block26, i32 0, i32 2
  store i64 %call_runtime25, i64* %ptr_to_obj_id29, align 4
  %ptr_to_field30 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.20, i8* (i8*, i8*)** %ptr_to_field30, align 8
  %pointer_cast31 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj23 to i8*
  %malloccall32 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj33 = bitcast i8* %malloccall32 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast34 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33 to i8*
  %call_runtime35 = call i64 @report_malloc(i8* %pointer_cast34, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.28, i32 0, i32 0))
  %ptr_to_control_block36 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33, i32 0, i32 0
  %ptr_to_refcnt37 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt37, align 4
  %ptr_to_dtor_field38 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field38, align 8
  %ptr_to_obj_id39 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block36, i32 0, i32 2
  store i64 %call_runtime35, i64* %ptr_to_obj_id39, align 4
  %ptr_to_field40 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.25, i8* (i8*, i8*)** %ptr_to_field40, align 8
  %pointer_cast41 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj33 to i8*
  %malloccall42 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj43 = bitcast i8* %malloccall42 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast44 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43 to i8*
  %call_runtime45 = call i64 @report_malloc(i8* %pointer_cast44, i8* getelementptr inbounds ([27 x i8], [27 x i8]* @name_of_obj.34, i32 0, i32 0))
  %ptr_to_control_block46 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43, i32 0, i32 0
  %ptr_to_refcnt47 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block46, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt47, align 4
  %ptr_to_dtor_field48 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block46, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field48, align 8
  %ptr_to_obj_id49 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block46, i32 0, i32 2
  store i64 %call_runtime45, i64* %ptr_to_obj_id49, align 4
  %ptr_to_field50 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.29, i8* (i8*, i8*)** %ptr_to_field50, align 8
  %pointer_cast51 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj43 to i8*
  %malloccall52 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* null, i32 1) to i32))
  %ptr_to_obj53 = bitcast i8* %malloccall52 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %pointer_cast54 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj53 to i8*
  %call_runtime55 = call i64 @report_malloc(i8* %pointer_cast54, i8* getelementptr inbounds ([28 x i8], [28 x i8]* @name_of_obj.40, i32 0, i32 0))
  %ptr_to_control_block56 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj53, i32 0, i32 0
  %ptr_to_refcnt57 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt57, align 4
  %ptr_to_dtor_field58 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 1
  store void (i8*)* @dtor.8, void (i8*)** %ptr_to_dtor_field58, align 8
  %ptr_to_obj_id59 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block56, i32 0, i32 2
  store i64 %call_runtime55, i64* %ptr_to_obj_id59, align 4
  %ptr_to_field60 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj53, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.35, i8* (i8*, i8*)** %ptr_to_field60, align 8
  %pointer_cast61 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %ptr_to_obj53 to i8*
  %malloccall62 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj63 = bitcast i8* %malloccall62 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast64 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj63 to i8*
  %call_runtime65 = call i64 @report_malloc(i8* %pointer_cast64, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.41, i32 0, i32 0))
  %ptr_to_control_block66 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj63, i32 0, i32 0
  %ptr_to_refcnt67 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block66, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt67, align 4
  %ptr_to_dtor_field68 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block66, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field68, align 8
  %ptr_to_obj_id69 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block66, i32 0, i32 2
  store i64 %call_runtime65, i64* %ptr_to_obj_id69, align 4
  %ptr_to_field70 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj63, i32 0, i32 1
  store i64 31, i64* %ptr_to_field70, align 4
  %pointer_cast71 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj63 to i8*
  %pointer_cast72 = bitcast i8* %pointer_cast31 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field73 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast72, i32 0, i32 1
  %field_value = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field73, align 8
  %call_lambda = tail call i8* %field_value(i8* %pointer_cast71, i8* %pointer_cast31)
  %malloccall74 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj75 = bitcast i8* %malloccall74 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast76 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj75 to i8*
  %call_runtime77 = call i64 @report_malloc(i8* %pointer_cast76, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.42, i32 0, i32 0))
  %ptr_to_control_block78 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj75, i32 0, i32 0
  %ptr_to_refcnt79 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block78, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt79, align 4
  %ptr_to_dtor_field80 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block78, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field80, align 8
  %ptr_to_obj_id81 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block78, i32 0, i32 2
  store i64 %call_runtime77, i64* %ptr_to_obj_id81, align 4
  %ptr_to_field82 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj75, i32 0, i32 1
  store i64 0, i64* %ptr_to_field82, align 4
  %pointer_cast83 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj75 to i8*
  %pointer_cast84 = bitcast i8* %call_lambda to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field85 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast84, i32 0, i32 1
  %field_value86 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field85, align 8
  %call_lambda87 = tail call i8* %field_value86(i8* %pointer_cast83, i8* %call_lambda)
  call void @retain_obj(i8* %pointer_cast1)
  %pointer_cast88 = bitcast i8* %pointer_cast1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field89 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast88, i32 0, i32 1
  %field_value90 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field89, align 8
  %call_lambda91 = tail call i8* %field_value90(i8* %call_lambda87, i8* %pointer_cast1)
  %malloccall92 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj93 = bitcast i8* %malloccall92 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast94 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj93 to i8*
  %call_runtime95 = call i64 @report_malloc(i8* %pointer_cast94, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.43, i32 0, i32 0))
  %ptr_to_control_block96 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj93, i32 0, i32 0
  %ptr_to_refcnt97 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block96, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt97, align 4
  %ptr_to_dtor_field98 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block96, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field98, align 8
  %ptr_to_obj_id99 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block96, i32 0, i32 2
  store i64 %call_runtime95, i64* %ptr_to_obj_id99, align 4
  %ptr_to_field100 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj93, i32 0, i32 1
  store i64 0, i64* %ptr_to_field100, align 4
  %pointer_cast101 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj93 to i8*
  %pointer_cast102 = bitcast i8* %call_lambda91 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field103 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast102, i32 0, i32 1
  %field_value104 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field103, align 8
  %call_lambda105 = tail call i8* %field_value104(i8* %pointer_cast101, i8* %call_lambda91)
  %malloccall106 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj107 = bitcast i8* %malloccall106 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast108 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj107 to i8*
  %call_runtime109 = call i64 @report_malloc(i8* %pointer_cast108, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.44, i32 0, i32 0))
  %ptr_to_control_block110 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj107, i32 0, i32 0
  %ptr_to_refcnt111 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block110, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt111, align 4
  %ptr_to_dtor_field112 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block110, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field112, align 8
  %ptr_to_obj_id113 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block110, i32 0, i32 2
  store i64 %call_runtime109, i64* %ptr_to_obj_id113, align 4
  %ptr_to_field114 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj107, i32 0, i32 1
  store i64 0, i64* %ptr_to_field114, align 4
  %pointer_cast115 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj107 to i8*
  %pointer_cast116 = bitcast i8* %call_lambda105 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field117 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast116, i32 0, i32 1
  %field_value118 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field117, align 8
  %call_lambda119 = tail call i8* %field_value118(i8* %pointer_cast115, i8* %call_lambda105)
  call void @retain_obj(i8* %pointer_cast1)
  %pointer_cast120 = bitcast i8* %pointer_cast1 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field121 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast120, i32 0, i32 1
  %field_value122 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field121, align 8
  %call_lambda123 = tail call i8* %field_value122(i8* %call_lambda119, i8* %pointer_cast1)
  %malloccall124 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj125 = bitcast i8* %malloccall124 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast126 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj125 to i8*
  %call_runtime127 = call i64 @report_malloc(i8* %pointer_cast126, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.45, i32 0, i32 0))
  %ptr_to_control_block128 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj125, i32 0, i32 0
  %ptr_to_refcnt129 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block128, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt129, align 4
  %ptr_to_dtor_field130 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block128, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field130, align 8
  %ptr_to_obj_id131 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block128, i32 0, i32 2
  store i64 %call_runtime127, i64* %ptr_to_obj_id131, align 4
  %ptr_to_field132 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj125, i32 0, i32 1
  store i64 1, i64* %ptr_to_field132, align 4
  %pointer_cast133 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj125 to i8*
  %pointer_cast134 = bitcast i8* %call_lambda123 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field135 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast134, i32 0, i32 1
  %field_value136 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field135, align 8
  %call_lambda137 = tail call i8* %field_value136(i8* %pointer_cast133, i8* %call_lambda123)
  %malloccall138 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj139 = bitcast i8* %malloccall138 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast140 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj139 to i8*
  %call_runtime141 = call i64 @report_malloc(i8* %pointer_cast140, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.46, i32 0, i32 0))
  %ptr_to_control_block142 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj139, i32 0, i32 0
  %ptr_to_refcnt143 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block142, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt143, align 4
  %ptr_to_dtor_field144 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block142, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field144, align 8
  %ptr_to_obj_id145 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block142, i32 0, i32 2
  store i64 %call_runtime141, i64* %ptr_to_obj_id145, align 4
  %ptr_to_field146 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj139, i32 0, i32 1
  store i64 1, i64* %ptr_to_field146, align 4
  %pointer_cast147 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj139 to i8*
  %pointer_cast148 = bitcast i8* %call_lambda137 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field149 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast148, i32 0, i32 1
  %field_value150 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field149, align 8
  %call_lambda151 = tail call i8* %field_value150(i8* %pointer_cast147, i8* %call_lambda137)
  %malloccall152 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* null, i32 1) to i32))
  %ptr_to_obj153 = bitcast i8* %malloccall152 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }*
  %pointer_cast154 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153 to i8*
  %call_runtime155 = call i64 @report_malloc(i8* %pointer_cast154, i8* getelementptr inbounds ([244 x i8], [244 x i8]* @name_of_obj.58, i32 0, i32 0))
  %ptr_to_control_block156 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153, i32 0, i32 0
  %ptr_to_refcnt157 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block156, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt157, align 4
  %ptr_to_dtor_field158 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block156, i32 0, i32 1
  store void (i8*)* @dtor.59, void (i8*)** %ptr_to_dtor_field158, align 8
  %ptr_to_obj_id159 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block156, i32 0, i32 2
  store i64 %call_runtime155, i64* %ptr_to_obj_id159, align 4
  %ptr_to_field160 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.47, i8* (i8*, i8*)** %ptr_to_field160, align 8
  call void @retain_obj(i8* %pointer_cast21)
  %ptr_to_field161 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153, i32 0, i32 2
  store i8* %pointer_cast21, i8** %ptr_to_field161, align 8
  %ptr_to_field162 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153, i32 0, i32 3
  store i8* %pointer_cast61, i8** %ptr_to_field162, align 8
  %ptr_to_field163 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153, i32 0, i32 4
  store i8* %pointer_cast1, i8** %ptr_to_field163, align 8
  %ptr_to_field164 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153, i32 0, i32 5
  store i8* %pointer_cast51, i8** %ptr_to_field164, align 8
  %pointer_cast165 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8* }* %ptr_to_obj153 to i8*
  %pointer_cast166 = bitcast i8* %pointer_cast41 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field167 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast166, i32 0, i32 1
  %field_value168 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field167, align 8
  %call_lambda169 = tail call i8* %field_value168(i8* %pointer_cast165, i8* %pointer_cast41)
  %pointer_cast170 = bitcast i8* %call_lambda169 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field171 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast170, i32 0, i32 1
  %field_value172 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field171, align 8
  %call_lambda173 = tail call i8* %field_value172(i8* %call_lambda151, i8* %call_lambda169)
  %malloccall174 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj175 = bitcast i8* %malloccall174 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast176 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj175 to i8*
  %call_runtime177 = call i64 @report_malloc(i8* %pointer_cast176, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.60, i32 0, i32 0))
  %ptr_to_control_block178 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj175, i32 0, i32 0
  %ptr_to_refcnt179 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block178, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt179, align 4
  %ptr_to_dtor_field180 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block178, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field180, align 8
  %ptr_to_obj_id181 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block178, i32 0, i32 2
  store i64 %call_runtime177, i64* %ptr_to_obj_id181, align 4
  %ptr_to_field182 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj175, i32 0, i32 1
  store i64 2, i64* %ptr_to_field182, align 4
  %pointer_cast183 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj175 to i8*
  %pointer_cast184 = bitcast i8* %call_lambda173 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field185 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast184, i32 0, i32 1
  %field_value186 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field185, align 8
  %call_lambda187 = tail call i8* %field_value186(i8* %pointer_cast183, i8* %call_lambda173)
  %pointer_cast188 = bitcast i8* %pointer_cast21 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field189 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast188, i32 0, i32 1
  %field_value190 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field189, align 8
  %call_lambda191 = tail call i8* %field_value190(i8* %call_lambda187, i8* %pointer_cast21)
  %malloccall192 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj193 = bitcast i8* %malloccall192 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast194 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj193 to i8*
  %call_runtime195 = call i64 @report_malloc(i8* %pointer_cast194, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.61, i32 0, i32 0))
  %ptr_to_control_block196 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj193, i32 0, i32 0
  %ptr_to_refcnt197 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block196, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt197, align 4
  %ptr_to_dtor_field198 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block196, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field198, align 8
  %ptr_to_obj_id199 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block196, i32 0, i32 2
  store i64 %call_runtime195, i64* %ptr_to_obj_id199, align 4
  %ptr_to_field200 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj193, i32 0, i32 1
  store i64 30, i64* %ptr_to_field200, align 4
  %pointer_cast201 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj193 to i8*
  %pointer_cast202 = bitcast i8* %call_lambda191 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field203 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast202, i32 0, i32 1
  %field_value204 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field203, align 8
  %call_lambda205 = tail call i8* %field_value204(i8* %pointer_cast201, i8* %call_lambda191)
  %pointer_cast206 = bitcast i8* %call_lambda205 to { { i64, void (i8*)*, i64 }, i64 }*
  %ptr_to_field207 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %pointer_cast206, i32 0, i32 1
  %field_value208 = load i64, i64* %ptr_to_field207, align 4
  call void @release_obj(i8* %call_lambda205)
  call void @check_leak()
  ret i64 %field_value208
}

define i8* @lambda(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([46 x i8], [46 x i8]* @name_of_obj.5, i32 0, i32 0))
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast1, i8* getelementptr inbounds ([38 x i8], [38 x i8]* @name_of_obj.3, i32 0, i32 0))
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
  %call_runtime = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([40 x i8], [40 x i8]* @rust_str, i32 0, i32 0))
  call void @abort()
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, { i64, i8** } }* getelementptr ({ { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, { i64, i8** } }*
  %pointer_cast11 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj to i8*
  %call_runtime12 = call i64 @report_malloc(i8* %pointer_cast11, i8* getelementptr inbounds ([28 x i8], [28 x i8]* @name_of_obj, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime12, i64* %ptr_to_obj_id, align 4
  %3 = getelementptr inbounds { { i64, void (i8*)*, i64 }, { i64, i8** } }, { { i64, void (i8*)*, i64 }, { i64, i8** } }* %ptr_to_obj, i32 0, i32 1
  %ptr_to_field13 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 0
  %field_value14 = load i64, i64* %ptr_to_field13, align 4
  %ptr_to_field15 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 1
  %field_value16 = load i8**, i8*** %ptr_to_field15, align 8
  %ptr_to_field17 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %3, i32 0, i32 0
  store i64 %field_value14, i64* %ptr_to_field17, align 4
  %4 = trunc i64 %field_value14 to i32
  %mallocsize = mul i32 %4, ptrtoint (i1** getelementptr (i1*, i1** null, i32 1) to i32)
  %malloccall18 = tail call i8* @malloc(i32 %mallocsize)
  %dst_buffer = bitcast i8* %malloccall18 to i8**
  %ptr_to_field19 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %3, i32 0, i32 1
  store i8** %dst_buffer, i8*** %ptr_to_field19, align 8
  %ptr_to_field20 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 0
  %field_value21 = load i64, i64* %ptr_to_field20, align 4
  %ptr_to_field22 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %2, i32 0, i32 1
  %field_value23 = load i8**, i8*** %ptr_to_field22, align 8
  %release_loop_counter = alloca i64, align 8
  store i64 0, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

cont_bb:                                          ; preds = %after_loop, %entry
  %array_phi = phi { { i64, void (i8*)*, i64 }, { i64, i8** } }* [ %pointer_cast7, %entry ], [ %ptr_to_obj, %after_loop ]
  %array_field_phi = phi { i64, i8** }* [ %2, %entry ], [ %3, %after_loop ]
  %ptr_to_field25 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field_phi, i32 0, i32 0
  %field_value26 = load i64, i64* %ptr_to_field25, align 4
  %ptr_to_field27 = getelementptr inbounds { i64, i8** }, { i64, i8** }* %array_field_phi, i32 0, i32 1
  %field_value28 = load i8**, i8*** %ptr_to_field27, align 8
  %ptr_to_elem_of_array = getelementptr i8*, i8** %field_value28, i64 %field_value6
  %elem = load i8*, i8** %ptr_to_elem_of_array, align 8
  call void @release_obj(i8* %elem)
  store i8* %0, i8** %ptr_to_elem_of_array, align 8
  %pointer_cast29 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %array_phi to i8*
  ret i8* %pointer_cast29

loop_release_array_elements:                      ; preds = %loop_body, %shared_bb
  %counter_val = load i64, i64* %release_loop_counter, align 4
  %is_end = icmp eq i64 %counter_val, %field_value21
  br i1 %is_end, label %after_loop, label %loop_body

loop_body:                                        ; preds = %loop_release_array_elements
  %ptr_to_src_elem = getelementptr i8*, i8** %field_value16, i64 %counter_val
  %ptr_to_dst_elem = getelementptr i8*, i8** %dst_buffer, i64 %counter_val
  %src_elem = load i8*, i8** %ptr_to_src_elem, align 8
  call void @retain_obj(i8* %src_elem)
  store i8* %src_elem, i8** %ptr_to_dst_elem, align 8
  %incremented_counter_val = add i64 %counter_val, 1
  store i64 %incremented_counter_val, i64* %release_loop_counter, align 4
  br label %loop_release_array_elements

after_loop:                                       ; preds = %loop_release_array_elements
  %pointer_cast24 = bitcast { { i64, void (i8*)*, i64 }, { i64, i8** } }* %pointer_cast7 to i8*
  call void @release_obj(i8* %pointer_cast24)
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([45 x i8], [45 x i8]* @name_of_obj.14, i32 0, i32 0))
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
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }*
  %pointer_cast1 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast1, i8* getelementptr inbounds ([37 x i8], [37 x i8]* @name_of_obj.13, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.4, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field2 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.11, i8* (i8*, i8*)** %ptr_to_field2, align 8
  %ptr_to_field3 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field3, align 8
  %ptr_to_field4 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %field_value, i8** %ptr_to_field4, align 8
  %pointer_cast5 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast5
}

define i8* @lambda.11(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast11, i8* getelementptr inbounds ([27 x i8], [27 x i8]* @name_of_obj.12, i32 0, i32 0))
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

define i8* @lambda.16(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([28 x i8], [28 x i8]* @name_of_obj.18, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
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

define i8* @lambda.20(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([30 x i8], [30 x i8]* @name_of_obj.23, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.21, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.21(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast4, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.22, i32 0, i32 0))
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

define i8* @lambda.25(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([14 x i8], [14 x i8]* @name_of_obj.27, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.26, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.26(i8* %0, i8* %1) {
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

define i8* @lambda.29(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([19 x i8], [19 x i8]* @name_of_obj.33, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.30, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.30(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast7, i8* getelementptr inbounds ([11 x i8], [11 x i8]* @name_of_obj.31, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.32, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8 }, { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj, i32 0, i32 1
  store i8 %eq_bool, i8* %ptr_to_field8, align 1
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, i8 }* %ptr_to_obj to i8*
  ret i8* %pointer_cast9
}

define void @dtor.32(i8* %0) {
entry:
  ret void
}

define i8* @lambda.35(i8* %0, i8* %1) {
entry:
  call void @release_obj(i8* %1)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* getelementptr ({ { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }*
  %pointer_cast = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast, i8* getelementptr inbounds ([20 x i8], [20 x i8]* @name_of_obj.39, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.6, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.36, i8* (i8*, i8*)** %ptr_to_field, align 8
  %ptr_to_field1 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %0, i8** %ptr_to_field1, align 8
  %pointer_cast2 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast2
}

define i8* @lambda.36(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast7, i8* getelementptr inbounds ([12 x i8], [12 x i8]* @name_of_obj.37, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field8 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 1
  store i64 %add, i64* %ptr_to_field8, align 4
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  %pointer_cast9 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  ret i8* %pointer_cast9
}

define void @dtor.38(i8* %0) {
entry:
  ret void
}

define i8* @lambda.47(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast10, i8* getelementptr inbounds ([238 x i8], [238 x i8]* @name_of_obj.56, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.57, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field11 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.48, i8* (i8*, i8*)** %ptr_to_field11, align 8
  %ptr_to_field12 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %field_value, i8** %ptr_to_field12, align 8
  %ptr_to_field13 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %field_value3, i8** %ptr_to_field13, align 8
  %ptr_to_field14 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 4
  store i8* %0, i8** %ptr_to_field14, align 8
  %ptr_to_field15 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 5
  store i8* %field_value6, i8** %ptr_to_field15, align 8
  %ptr_to_field16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 6
  store i8* %field_value9, i8** %ptr_to_field16, align 8
  %pointer_cast17 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast17
}

define i8* @lambda.48(i8* %0, i8* %1) {
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
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast13, i8* getelementptr inbounds ([230 x i8], [230 x i8]* @name_of_obj.54, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.55, void (i8*)** %ptr_to_dtor_field, align 8
  %ptr_to_obj_id = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 2
  store i64 %call_runtime, i64* %ptr_to_obj_id, align 4
  %ptr_to_field14 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 1
  store i8* (i8*, i8*)* @lambda.49, i8* (i8*, i8*)** %ptr_to_field14, align 8
  %ptr_to_field15 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 2
  store i8* %field_value, i8** %ptr_to_field15, align 8
  %ptr_to_field16 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 3
  store i8* %field_value3, i8** %ptr_to_field16, align 8
  %ptr_to_field17 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 4
  store i8* %field_value6, i8** %ptr_to_field17, align 8
  %ptr_to_field18 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 5
  store i8* %field_value9, i8** %ptr_to_field18, align 8
  %ptr_to_field19 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 6
  store i8* %field_value12, i8** %ptr_to_field19, align 8
  %ptr_to_field20 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj, i32 0, i32 7
  store i8* %0, i8** %ptr_to_field20, align 8
  %pointer_cast21 = bitcast { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)*, i8*, i8*, i8*, i8*, i8*, i8* }* %ptr_to_obj to i8*
  ret i8* %pointer_cast21
}

define i8* @lambda.49(i8* %0, i8* %1) {
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
  %pointer_cast16 = bitcast i8* %field_value12 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field17 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast16, i32 0, i32 1
  %field_value18 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field17, align 8
  %call_lambda = tail call i8* %field_value18(i8* %0, i8* %field_value12)
  %malloccall = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj = bitcast i8* %malloccall to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast19 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj to i8*
  %call_runtime = call i64 @report_malloc(i8* %pointer_cast19, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.50, i32 0, i32 0))
  %ptr_to_control_block = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj, i32 0, i32 0
  %ptr_to_refcnt = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt, align 4
  %ptr_to_dtor_field = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field, align 8
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
  call void @release_obj(i8* %field_value)
  call void @release_obj(i8* %0)
  call void @release_obj(i8* %field_value3)
  call void @release_obj(i8* %field_value6)
  call void @release_obj(i8* %field_value9)
  br label %cont

else:                                             ; preds = %entry
  call void @retain_obj(i8* %field_value)
  call void @retain_obj(i8* %field_value15)
  %pointer_cast29 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field30 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast29, i32 0, i32 1
  %field_value31 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field30, align 8
  %call_lambda32 = tail call i8* %field_value31(i8* %field_value15, i8* %field_value)
  call void @retain_obj(i8* %field_value3)
  call void @retain_obj(i8* %0)
  %pointer_cast33 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field34 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast33, i32 0, i32 1
  %field_value35 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field34, align 8
  %call_lambda36 = tail call i8* %field_value35(i8* %0, i8* %field_value3)
  %malloccall37 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj38 = bitcast i8* %malloccall37 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast39 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj38 to i8*
  %call_runtime40 = call i64 @report_malloc(i8* %pointer_cast39, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.51, i32 0, i32 0))
  %ptr_to_control_block41 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj38, i32 0, i32 0
  %ptr_to_refcnt42 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block41, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt42, align 4
  %ptr_to_dtor_field43 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block41, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field43, align 8
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
  call void @retain_obj(i8* %field_value15)
  %pointer_cast55 = bitcast i8* %field_value to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field56 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast55, i32 0, i32 1
  %field_value57 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field56, align 8
  %call_lambda58 = tail call i8* %field_value57(i8* %field_value15, i8* %field_value)
  call void @retain_obj(i8* %field_value3)
  call void @retain_obj(i8* %0)
  %pointer_cast59 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field60 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast59, i32 0, i32 1
  %field_value61 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field60, align 8
  %call_lambda62 = tail call i8* %field_value61(i8* %0, i8* %field_value3)
  %malloccall63 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj64 = bitcast i8* %malloccall63 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast65 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj64 to i8*
  %call_runtime66 = call i64 @report_malloc(i8* %pointer_cast65, i8* getelementptr inbounds ([3 x i8], [3 x i8]* @name_of_obj.52, i32 0, i32 0))
  %ptr_to_control_block67 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj64, i32 0, i32 0
  %ptr_to_refcnt68 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block67, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt68, align 4
  %ptr_to_dtor_field69 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block67, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field69, align 8
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
  %pointer_cast81 = bitcast i8* %field_value9 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field82 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast81, i32 0, i32 1
  %field_value83 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field82, align 8
  %call_lambda84 = tail call i8* %field_value83(i8* %field_value15, i8* %field_value9)
  call void @retain_obj(i8* %0)
  %pointer_cast85 = bitcast i8* %call_lambda84 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field86 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast85, i32 0, i32 1
  %field_value87 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field86, align 8
  %call_lambda88 = tail call i8* %field_value87(i8* %0, i8* %call_lambda84)
  call void @retain_obj(i8* %field_value3)
  %pointer_cast89 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field90 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast89, i32 0, i32 1
  %field_value91 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field90, align 8
  %call_lambda92 = tail call i8* %field_value91(i8* %call_lambda54, i8* %field_value3)
  %pointer_cast93 = bitcast i8* %call_lambda92 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field94 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast93, i32 0, i32 1
  %field_value95 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field94, align 8
  %call_lambda96 = tail call i8* %field_value95(i8* %call_lambda80, i8* %call_lambda92)
  %pointer_cast97 = bitcast i8* %call_lambda88 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field98 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast97, i32 0, i32 1
  %field_value99 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field98, align 8
  %call_lambda100 = tail call i8* %field_value99(i8* %call_lambda96, i8* %call_lambda88)
  %pointer_cast101 = bitcast i8* %field_value6 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field102 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast101, i32 0, i32 1
  %field_value103 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field102, align 8
  %call_lambda104 = tail call i8* %field_value103(i8* %call_lambda100, i8* %field_value6)
  %pointer_cast105 = bitcast i8* %field_value3 to { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }*
  %ptr_to_field106 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }, { { i64, void (i8*)*, i64 }, i8* (i8*, i8*)* }* %pointer_cast105, i32 0, i32 1
  %field_value107 = load i8* (i8*, i8*)*, i8* (i8*, i8*)** %ptr_to_field106, align 8
  %call_lambda108 = tail call i8* %field_value107(i8* %0, i8* %field_value3)
  %malloccall109 = tail call i8* @malloc(i32 ptrtoint ({ { i64, void (i8*)*, i64 }, i64 }* getelementptr ({ { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* null, i32 1) to i32))
  %ptr_to_obj110 = bitcast i8* %malloccall109 to { { i64, void (i8*)*, i64 }, i64 }*
  %pointer_cast111 = bitcast { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj110 to i8*
  %call_runtime112 = call i64 @report_malloc(i8* %pointer_cast111, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @name_of_obj.53, i32 0, i32 0))
  %ptr_to_control_block113 = getelementptr inbounds { { i64, void (i8*)*, i64 }, i64 }, { { i64, void (i8*)*, i64 }, i64 }* %ptr_to_obj110, i32 0, i32 0
  %ptr_to_refcnt114 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block113, i32 0, i32 0
  store i64 1, i64* %ptr_to_refcnt114, align 4
  %ptr_to_dtor_field115 = getelementptr inbounds { i64, void (i8*)*, i64 }, { i64, void (i8*)*, i64 }* %ptr_to_control_block113, i32 0, i32 1
  store void (i8*)* @dtor.38, void (i8*)** %ptr_to_dtor_field115, align 8
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
  %phi = phi i8* [ %field_value15, %then ], [ %call_lambda126, %else ]
  ret i8* %phi
}

define void @dtor.55(i8* %0) {
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

define void @dtor.57(i8* %0) {
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

define void @dtor.59(i8* %0) {
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
