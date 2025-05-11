use std::sync::Arc;

use crate::{
    ast::{
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_make_struct, expr_var, var_local,
            var_var, ExprNode,
        },
        name::FullName,
        pattern::PatternNode,
        program::Program,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
        types::{tycon, TypeNode},
    },
    builtin::{make_tuple_name, make_tuple_ty},
    constants::DECAP_NAME,
    misc::{Map, Set},
};

use super::{
    find_free_name_type::find_types_of_free_names, uncurry::internalize_let_to_var_at_head,
};

/*
# Decapturing optimization

## 概要

### デキャプチャリング

ラムダ式ごとに専用の構造体を定義する。
その構造体はラムダ式がキャプチャする値をフィールドに持つ。
またラムダ式の処理をグローバル関数として定義する。

例：
```
let f = |x| x + n;
```
があるとき、
```
type #Cap0_f = unbox struct { n: I64 };

#lam0_f : #Cap0_f -> I64 -> I64;
#lam0_f = |{ n : n }, x| x + n;
```
が定義される。

ラムダの定義は、`#Cap0_f { n : n }` に置き換えられる。

`#lam0_f`がグローバル関数となることで、後続のuncurry最適化が適用されるようになり、高速化が見込める。

### ラムダの使用個所の書き換え

`f(x)`は、以下のコードに変換される。
```
#lam0_f(f, x)
```
fにより多数の引数がある場合、fの使用箇所が部分適用となっている式は、`#lam0_f` に部分適用を行う式に置き換えられる。

特に、fに0個の引数が与えられる場合、すなわちfそのものが出現する場所では、原則、`#lam0_f(f)` というコードになる。
ただし、そのうち「クロージャ特殊化」が適用できる個所では、`#lam0_f` に置き換えられる。

### クロージャ特殊化

グローバル関数の引数にラムダを与えている場合を考える。
例として、`fold`の第2引数に、上記の`f`を与えている場合を考える。

```
fold : S -> (A -> S -> S) -> Iter -> S;
fold = |s, op, iter| (
    match iter.advance {
        none() => s,
        some((iter, a)) => iter.fold(op(a, s), op)
    }
);
```

これをもとに以下のようなバージョンのfoldを定義する。
これをクロージャ特殊化と呼ぶ。

```
fold#lam0_f : S -> #Cap0_f -> Iter -> S;
fold#lam0_f = |s, op, iter| (
    match iter.advance {
        none() => s,
        some((iter, a)) => iter.fold#lam0_f(#lam0_f(op, a, s), op)
    }
);
```

そして、`iter.fold(s0, f)`を`iter.fold#lam0_f(s0, f)`に置き換える。

## 適用範囲と制限

### ラムダを定義してから使うまでの経路

#### 例

ラムダを定義してそのまま使う式にはこの最適化が適用される。
例： `iter.fold(s0, |acm, i| acm + i)`

#### 例2

ラムダを定義して一度ローカル変数に入れてから使う使う式にもこの最適化が適用される。
例： `let f = |acm, i| acm + i; iter.fold(s0, f)`

ただし、ラムダを定義してから使うまでの経路がこれより複雑な場合はこの最適化は適用されない。
例： `let (_, f) = (0, |acm, i| acm + i); iter.fold(s0, f)`
このような場合に対処するためには、なんらかの他の最適化を実装し、事前に適用しておく必要があるだろう。

## 他の最適化との関係

* 本最適化の前にインライン化を行っておくべき。例えば、式`f >> g`がインライン化によりラムダ式に置き換わることで、本最適化の対象となる。
* 本最適化の後にもインライン化を行う価値があるかもしれない。本最適化によりグローバル関数が増えるためである。
* 本最適化の前にエータ展開を行っておくべきかもしれない。エータ展開により特殊化の対象となる引数が増えるためである。
* 本最適化により生成されたグローバル関数の性能を向上させるため、アンカリー化は本最適化の後に行うべきである。

TODO:
とりあえずラムダが直にfoldに渡されている場合に実装してみよう。
変数を通して引き渡される場合にはそのあとに対処しよう。
*/

pub fn run(prg: &mut Program) {
    let mut stable_symbols = Set::default();
    while run_one(prg, &mut stable_symbols) {}
}

// 全シンボルに対して最適化を行う。
// 何らかの最適化が行われた場合はtrueを返す。
//
// * `stable_symbols`: これ以上最適化が進まないことが確定しているシンボルの集合。
pub fn run_one(prg: &mut Program, stable_symbols: &mut Set<FullName>) -> bool {
    // TODO: まずはremove_shadowingを実行しておくこと。
}

// デキャプチャリング最適化を行うためのExpreesion Visitor
struct DecapturingVisitor {
    /* デキャプチャリング */
    // デキャプチャリングにより作成されたグローバルなラムダ関数
    decap_lambdas: Vec<DecapturedLambdaInfo>,
    // デキャプチャされたラムダに名前が与えられたとき、ここに保存される。
    local_decap_lambdas: Map<FullName, DecapturedLambdaInfo>,

    /* 特殊化 */
    // 特殊化可能な関数
    specializable_funcs: Map<FullName, SpecializableFunctionInfo>,
    // デキャプチャリングにより生成された特殊化要求
    required_specializations: Vec<SpecializationInfo>,

    /* ラムダ関数の名前の決定に関するフィールド */
    // ラムダ関数の名前を生成するために使われる番号
    // 複数のデキャプチャリングを行うとき、この番号を必ずしも引き継ぐ必要はないが、引き継ぐほうが名前決定処理の効率が良くなる。
    lam_func_counter: u32,
    // 現在デキャプチャリングを行っているシンボルの名前
    // ラムダ関数の名前を生成するために使われる。
    current_symbol: FullName,
    // グローバル名の集合
    // これは、新しいグローバル名を生成するときの衝突を避けるために利用される。
    global_names: Set<FullName>,
}

impl DecapturingVisitor {
    // 新しいラムダ関数の名前を生成する。
    fn new_lambda_func_name(&mut self) -> FullName {
        loop {
            let mut full_name = self.current_symbol.clone();
            *full_name.name_as_mut() += &format!("#decap_lam{}", self.lam_func_counter);
            self.lam_func_counter += 1;
            if self.global_names.contains(&full_name) {
                continue;
            }
            self.global_names.insert(full_name.clone());
            return full_name;
        }
    }

    // ラムダ式をデキャプチャする
    //
    // `DecapturedLambdaInfo`、および、キャプチャリストを生成する式を返す。
    fn decapture_lambda(&mut self, lam: &Arc<ExprNode>) -> (DecapturedLambdaInfo, Arc<ExprNode>) {
        // キャプチャリストとその型を取得する。
        let cap_names = lam.lambda_cap_names();
        let cap_names_types = find_types_of_free_names(lam, &cap_names);
        let cap_names_types = cap_names
            .iter()
            .map(|name| {
                let ty = cap_names_types.get(name).unwrap();
                ty.clone()
            })
            .collect::<Vec<_>>();

        // キャプチャリストの型を作成する。
        let cap_list_ty = make_tuple_ty(cap_names_types.clone());

        // キャプチャリストの式を作成する。
        let cap_list = expr_make_struct(
            tycon(make_tuple_name(cap_names.len() as u32)),
            cap_names
                .iter()
                .zip(cap_names_types.iter())
                .enumerate()
                .map(|(i, (name, ty))| {
                    let var = expr_var(name.clone(), None).set_inferred_type(ty.clone());
                    (i.to_string(), var)
                })
                .collect(),
        )
        .set_inferred_type(cap_list_ty.clone());

        // ラムダ関数を作成する。
        // このために、`lam`にキャプチャリストを受け取るための引数を加え、lamの本体の冒頭でキャプチャリストを分解するlet式を挿入する。
        let cap_pats = cap_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let var = var_var(name.clone());
                let ty = cap_names_types.get(i).unwrap();
                let pat = PatternNode::make_var(var, Some(ty.clone()));
                (i.to_string(), pat)
            })
            .collect::<Vec<_>>();
        let cap_pat =
            PatternNode::make_struct(tycon(make_tuple_name(cap_names.len() as u32)), cap_pats)
                .set_inferred_type(cap_list_ty.clone());
        let new_body = expr_let_typed(
            cap_pat,
            expr_var(FullName::local(DECAP_NAME), None),
            lam.clone(),
        );
        let new_arg = var_local(DECAP_NAME);
        let new_lam = expr_abs_typed(new_arg, cap_list_ty.clone(), new_body);
        let new_lam = internalize_let_to_var_at_head(&new_lam);

        let decap_lam = DecapturedLambdaInfo {
            cap_list_ty,
            cap_names: cap_names.clone(),
            lambda_func: new_lam,
            lambda_func_name: self.new_lambda_func_name(),
        };
        self.decap_lambdas.push(decap_lam.clone());
        (decap_lam, cap_list)
    }
}

// 特殊化可能な関数の情報
#[derive(Clone)]
struct SpecializableFunctionInfo {
    // 特殊化される関数の名前
    func_name: FullName,
    // 特殊化される関数の型
    func_ty: Arc<TypeNode>,
    // 特殊化可能な引数のインデックス（昇順）
    specializable_arg_indices: Vec<usize>,
}

// 特殊化の情報
struct SpecializationInfo {
    // 特殊化される関数の名前
    org_func_name: FullName,
    // 特殊化される関数の型
    org_func_ty: Arc<TypeNode>,
    // 特殊化される引数のインデックスに対して、そこに渡されるデキャプチャされたラムダを格納しているマップ
    specialized_args: Map<usize, DecapturedLambdaInfo>,
}

impl SpecializationInfo {
    // 特殊化された関数の名前を生成する。
    fn specialized_func_name(&self) -> FullName {
        let mut full_name = self.org_func_name.clone();
        let name = full_name.name_as_mut();
        *name += "#specialized";
        let mut hash_data = String::new();
        for (i, decap_lam) in self.specialized_args.iter() {
            hash_data += &format!("_{}", i);
            hash_data += &format!("_{}", decap_lam.lambda_func_name.to_string());
        }
        *name += &format!("{:x}", md5::compute(hash_data));
        full_name
    }

    // 特殊化された関数の型を作成する
    fn specialized_func_ty(&self) -> Arc<TypeNode> {
        // org_func_tyを分解 → 特殊化される引数の型をDecapturedLambdaInfoのcap_list_tyに置き換える。
        todo!();
    }

    // 特殊化された関数を参照する式を作成する
    fn specialized_func_expr(&self) -> Arc<ExprNode> {
        expr_var(self.specialized_func_name(), None).set_inferred_type(self.specialized_func_ty())
    }
}

// TODO: 特殊化可能であることをチェックするためにはインラインコストも考慮すること：
//
// `func`のインラインコストが小さいことを確認する：
// let complexity = self.inline_costs.get_complexity(func_name);
// if complexity.is_none() {
//     return StartVisitResult::VisitChildren;
// }
// let complexity = complexity.unwrap();
// let call_count = self.inline_costs.get_call_count(func_name);
// if complexity * call_count > INLINE_COST_THRESHOLD {
//     return StartVisitResult::VisitChildren;
// }

// デキャプチャしたラムダ式の情報を保持する構造体
#[derive(Clone)]
struct DecapturedLambdaInfo {
    // キャプチャされた名前の列
    cap_names: Vec<FullName>,
    // キャプチャリストの型
    // キャプチャされる型のタプルである。
    cap_list_ty: Arc<TypeNode>,
    // ラムダ関数
    lambda_func: Arc<ExprNode>,
    // ラムダ関数の名前
    lambda_func_name: FullName,
}

impl ExprVisitor for DecapturingVisitor {
    fn start_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // もしexprがデキャプチャされたラムダを示していて、かつ
        // この式の型がTで、ラムダ関数の型がC->Tのときは（Cはキャプチャリストの型）、
        // ラムダ関数にキャプチャリストを渡す式に置き換える。

        // 変数名を取得
        let name = &expr.get_var().name;

        // 変数名がローカルであることを確認する。
        if !name.is_local() {
            return StartVisitResult::VisitChildren;
        }

        // デキャプチャされたラムダを示しているか確認する。
        let decap_lambda = self.local_decap_lambdas.get(name);
        if !decap_lambda.is_none() {
            return StartVisitResult::VisitChildren;
        }
        let decap_lambda = decap_lambda.unwrap();

        //　この式の型として要求されている型がラムダ関数の型のコドメインと一致するか確認する。
        let var_ty = expr.ty.as_ref().unwrap().clone();
        let lambda_ty = decap_lambda.lambda_func.ty.as_ref().unwrap();
        let lambda_codom_ty = lambda_ty.get_lambda_dst();
        assert_eq!(var_ty.to_string(), lambda_codom_ty.to_string());
        // if var_ty.to_string() != lambda_codom_ty.to_string() {
        //     return StartVisitResult::VisitChildren;
        // }

        // ラムダ関数にキャプチャリストを与える式に置き換える。
        let lam = expr_var(decap_lambda.lambda_func_name.clone(), None)
            .set_inferred_type(lambda_ty.clone());
        let expr = expr_app_typed(lam, vec![expr.clone()]);
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // このapplication expressionが以下の条件を満たしているときに、クロージャ特殊化を実行する：
        // - 呼び出される関数はグローバルである。
        // - 特殊化可能な引数が少なくとも一つある：つまり、ある引数は、ラムダ式であるか、またはデキャプチャされたラムダ式（=キャプチャリスト）である。
        // - 関数のインラインコストが小さい。

        let (func, args) = expr.destructure_app();

        // `func`がグローバル関数であることを確認する。
        if !func.is_var() {
            return StartVisitResult::VisitChildren;
        }
        let func_name = &func.get_var().name;
        if !func_name.is_global() {
            return StartVisitResult::VisitChildren;
        }

        // `func`が特殊化可能な関数であることを確認する。
        let specializable_func = self.specializable_funcs.get(func_name);
        if specializable_func.is_none() {
            return StartVisitResult::VisitChildren;
        }
        let specialize_info = specializable_func.unwrap().clone();

        // `func`において特殊化可能なそれぞれの引数に対し、デキャプチャ情報を取得あるいは生成する。
        let mut specialized_args = Map::default();
        let mut decaptured_args = args.clone();
        for (i, arg) in args.iter().enumerate() {
            // 特殊化可能な引数であることを確認する。
            if !specialize_info.specializable_arg_indices.contains(&i) {
                continue;
            }
            // デキャプチャ情報を取得あるいは生成する。
            if arg.is_var() {
                let arg_name = &arg.get_var().name;
                if let Some(decap_info) = self.local_decap_lambdas.get(arg_name) {
                    specialized_args.insert(i, decap_info.clone());
                }
            } else if arg.is_lam() {
                let (decap_info, expr) = self.decapture_lambda(arg);
                specialized_args.insert(i, decap_info.clone());
                decaptured_args[i] = expr;
            }
        }
        if specialized_args.is_empty() {
            return StartVisitResult::VisitChildren;
        }

        // 特殊化を生成する
        let specialization = SpecializationInfo {
            org_func_name: func_name.clone(),
            org_func_ty: func.ty.as_ref().unwrap().clone(),
            specialized_args,
        };
        let specialized_func_expr = specialization.specialized_func_expr();
        self.required_specializations.push(specialization);

        // 特殊化された関数を呼び出す式に置き換える。
        let mut expr = specialized_func_expr.clone();
        for arg in decaptured_args {
            expr = expr_app_typed(expr, vec![arg]);
        }

        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
