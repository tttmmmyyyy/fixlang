// use super::*;

// const TASK_DATA_NAME: &str = "TaskData";
// const ASYNC_TASK_DATA_TASK_FUNC_IDX: u32 = 0;
// const ASYNC_TASK_DATA_FLAG_IDX: u32 = 1;
// const ASYNC_TASK_DATA_RESULT_SPACE: u32 = 2;

// fn task_data_tycon() -> Rc<TypeNode> {
//     type_tycon(&tycon(FullName::from_strs(
//         &[ASYNCTASK_NAME],
//         TASK_DATA_NAME,
//     )))
// }

// // A struct which stores a task function and task result and auxiliary data.
// // This struct is allocated on heap by `_unsafe_make_task_data` and deleted by `_unsafe_delete_task_data`.
// fn task_data_ty<'c, 'm>(res_ty: Rc<TypeNode>) -> Rc<TypeNode> {
//     type_tyapp(task_data_tycon(), res_ty)
// }

// A struct which stores a task function and task result and auxiliary data.
// This struct is allocated on heap by `_unsafe_make_task_data` and deleted by `_unsafe_delete_task_data`.
// fn task_data_ty<'c, 'm>(
//     gc: &mut GenerationContext<'c, 'm>,
//     res_ty: Rc<TypeNode>,
// ) -> StructType<'c> {
//     let task_func_ty = type_fun(make_unit_ty(), res_ty.clone()).get_embedded_type(gc, &vec![]);
//     let field_tys = [
//         task_func_ty.into(),                   // task_func : () -> a
//         gc.context.i8_type().into(),           // 8-bit flag.
//         res_ty.get_embedded_type(gc, &vec![]), // space to store result.
//     ];
//     gc.context.struct_type(&field_tys, false)
// }

// #[derive(Clone, Serialize, Deserialize)]
// pub struct InlineLLVMAsyncTaskMakeTaskData {
//     arg_name: String,
// }

// impl InlineLLVMAsyncTaskMakeTaskData {
//     pub fn generate<'c, 'm, 'b>(
//         &self,
//         gc: &mut GenerationContext<'c, 'm>,
//         _ty: &Rc<TypeNode>,
//         rvo: Option<Object<'c>>,
//     ) -> Object<'c> {
//         let arg_name = FullName::local(&self.arg_name);
//         let task_func = gc.get_var(&arg_name).ptr.get(gc);

//         // Task result type.
//         let res_ty = task_func.ty.get_lambda_dst();

//         // Allocate task data on heap.
//         allocate_obj(ty, capture, array_cap, gc, name)
//         let task_data_ty = task_data_ty(res_ty);
//         let task_data_ptr = gc
//             .builder()
//             .build_malloc(task_data_ty, "malloc_task_data")
//             .expect("allocate task data faile");
//     }
// }

// // _unsafe_make_task_data : (() -> a) -> Ptr
// pub fn async_task_make_task_data_function() -> (Rc<ExprNode>, Rc<Scheme>) {
//     const ARG_NAME: &str = "task";

//     let expr = expr_abs(
//         vec![var_local(ARG_NAME)],
//         expr_llvm(
//             LLVMGenerator::AsyncTaskMakeTaskData(InlineLLVMAsyncTaskMakeTaskData {
//                 arg_name: ARG_NAME.to_string(),
//             }),
//             vec![FullName::local(ARG_NAME)],
//             "_unsafe_make_task_data(task)".to_string(),
//             make_ptr_ty(),
//             None,
//         ),
//         None,
//     );
//     let task_ty = type_fun(make_unit_ty(), type_tyvar_star("a"));
//     let scm = Scheme::generalize(
//         HashMap::from([("a".to_string(), kind_star())]),
//         vec![],
//         type_fun(task_ty, make_ptr_ty()),
//     );
//     (expr, scm)
// }

// #[derive(Clone, Serialize, Deserialize)]
// pub struct InlineLLVMAsyncTaskDeleteTaskData {
//     arg_name: String,
// }

// impl InlineLLVMAsyncTaskDeleteTaskData {
//     pub fn generate<'c, 'm, 'b>(
//         &self,
//         gc: &mut GenerationContext<'c, 'm>,
//         _ty: &Rc<TypeNode>,
//         rvo: Option<Object<'c>>,
//     ) -> Object<'c> {
//     }
// }

// // _unsafe_delete_task_data : Ptr -> ()
// pub fn async_task_delete_task_data_function() -> (Rc<ExprNode>, Rc<Scheme>) {
//     const ARG_NAME: &str = "task_data";

//     let expr = expr_abs(
//         vec![var_local(ARG_NAME)],
//         expr_llvm(
//             LLVMGenerator::AsyncTaskDeleteTaskData(InlineLLVMAsyncTaskDeleteTaskData {
//                 arg_name: ARG_NAME.to_string(),
//             }),
//             vec![FullName::local(ARG_NAME)],
//             "_unsafe_delete_task_data(task_data)".to_string(),
//             make_unit_ty(),
//             None,
//         ),
//         None,
//     );
//     let scm = Scheme::generalize(
//         HashMap::default(),
//         vec![],
//         type_fun(make_ptr_ty(), make_unit_ty()),
//     );
//     (expr, scm)
// }

// #[derive(Clone, Serialize, Deserialize)]
// pub struct InlineLLVMAsyncTaskExtractTaskResult {
//     arg_name: String,
// }

// impl InlineLLVMAsyncTaskExtractTaskResult {
//     pub fn generate<'c, 'm, 'b>(
//         &self,
//         gc: &mut GenerationContext<'c, 'm>,
//         _ty: &Rc<TypeNode>,
//         rvo: Option<Object<'c>>,
//     ) -> Object<'c> {
//     }
// }

// // _unsafe_extract_task_result : Ptr -> a
// pub fn async_task_extract_task_result_function() -> (Rc<ExprNode>, Rc<Scheme>) {
//     const ARG_NAME: &str = "task_data";
//     const RESULT_TY: &str = "a";
//     let result_ty = type_tyvar_star(RESULT_TY);

//     let expr = expr_abs(
//         vec![var_local(ARG_NAME)],
//         expr_llvm(
//             LLVMGenerator::AsyncTaskDeleteTaskData(InlineLLVMAsyncTaskDeleteTaskData {
//                 arg_name: ARG_NAME.to_string(),
//             }),
//             vec![FullName::local(ARG_NAME)],
//             "_unsafe_extract_task_result(task_data)".to_string(),
//             result_ty,
//             None,
//         ),
//         None,
//     );
//     let scm = Scheme::generalize(
//         HashMap::from([(RESULT_TY.to_string(), kind_star())]),
//         vec![],
//         type_fun(make_ptr_ty(), result_ty),
//     );
//     (expr, scm)
// }
