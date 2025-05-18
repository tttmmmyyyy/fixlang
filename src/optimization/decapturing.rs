use std::{rc::Rc, sync::Arc};

use crate::{
    ast::{
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_make_struct, expr_var, var_local,
            var_var, ExprNode,
        },
        name::FullName,
        pattern::PatternNode,
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
        types::{tycon, type_fun, TypeNode},
    },
    builtin::{make_tuple_name, make_tuple_ty},
    constants::{DECAP_NAME, TUPLE_SIZE_BASE},
    misc::{Map, Set},
    optimization::utils::replace_free_var_of_expr,
};

use super::{uncurry::internalize_let_to_var_at_head, unify_local_names};

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
    // いずれかのシンボルに対して最適化が行われたかどうか。
    let mut changed = false;

    let symbols = std::mem::take(&mut prg.symbols);

    // 特殊化可能な関数の集合を計算する
    let mut specializable_funcs: Map<FullName, SpecializableFunctionInfo> = Map::default();
    for (name, sym) in &symbols {
        if let Some(specialize_info) = is_specializable_func(sym) {
            specializable_funcs.insert(name.clone(), specialize_info);
        }
    }
    let specializable_funcs = Rc::new(specializable_funcs);

    // グローバル名の集合を作成しておく
    let mut global_names = Set::default();
    for (name, _) in &symbols {
        global_names.insert(name.clone());
    }

    // それぞれのシンボルに対してデキャプチャリング最適化を実行する
    let mut new_symbols: Map<FullName, Symbol> = Map::default();
    let mut specializations: Vec<SpecializationInfo> = Vec::new();
    for (name, mut sym) in symbols {
        // 既に何も変化しないことが知られているシンボルはスキップする
        if stable_symbols.contains(&name) {
            new_symbols.insert(name.clone(), sym.clone());
            continue;
        }

        let mut visitor = DecapturingVisitor::new(
            name.clone(),
            specializable_funcs.clone(),
            global_names.clone(),
        );

        // デキャプチャリング最適化を行う
        let expr = unify_local_names::run_on_expr(sym.expr.as_ref().unwrap(), Set::default()); // デキャプチャ最適化の実装はshadowingを考慮していない
        let trav_res = visitor.traverse(&expr);
        if !trav_res.changed {
            // デキャプチャリング最適化が行われなかったとき
            stable_symbols.insert(name.clone());
            new_symbols.insert(name.clone(), sym.clone());
            continue;
        }
        changed = true;
        sym.expr = Some(trav_res.expr.calculate_free_vars());
        specializations.append(&mut visitor.required_specializations); // 特殊化要求はあとで処理する

        // 生成されたデキャプチャリングラムダをグローバル関数として登録する
        for decap_lam in visitor.decap_lambdas {
            let decap_lam_sym = decap_lam.make_symbol();
            global_names.insert(decap_lam_sym.name.clone());
            new_symbols.insert(decap_lam_sym.name.clone(), decap_lam_sym);
        }

        new_symbols.insert(name.clone(), sym.clone());
    }
    let mut symbols = new_symbols;

    // 特殊化要求を処理する
    for specialize_info in specializations {
        // 特殊化された関数の名前と型を生成する
        let specialized_func_name = specialize_info.specialized_func_name();
        let specialized_func_ty = specialize_info.specialized_func_ty();

        // 実装済みならスキップ
        if symbols.contains_key(&specialized_func_name) {
            continue;
        }

        // 特殊化された関数を実装する
        let expr = symbols
            .get(&specialize_info.org_func_name)
            .unwrap()
            .expr
            .as_ref()
            .unwrap()
            .clone();
        let expr = unify_local_names::run_on_expr(&expr, Set::default()); // デキャプチャ最適化の実装はshadowingを考慮していない

        // 引数名からデキャプチャされたラムダ情報へのマップを生成する
        let mut local_decap_lambdas = Map::default();
        let (args, _) = expr.destructure_lam_sequence();
        for (i, decap_lam) in &specialize_info.specialized_args {
            assert!(*i < args.len());
            assert_eq!(args[*i].len(), 1);
            let arg_name = &args[*i][0].name;
            local_decap_lambdas.insert(arg_name.clone(), decap_lam.clone());
        }

        let mut visitor = DecapturingVisitor::new(
            specialized_func_name.clone(),
            specializable_funcs.clone(),
            global_names.clone(),
        );
        visitor.local_decap_lambdas = local_decap_lambdas;
        let trav_res = visitor.traverse(&expr);
        let expr = trav_res.expr.calculate_free_vars();

        // デキャプチャにより作成されたラムダをグローバル関数として登録する
        for decap_lam in visitor.decap_lambdas {
            let decap_lam_sym = decap_lam.make_symbol();
            global_names.insert(decap_lam_sym.name.clone());
            symbols.insert(decap_lam_sym.name.clone(), decap_lam_sym);
        }

        // 特殊化された関数を登録する
        let specialized_func = Symbol {
            name: specialized_func_name.clone(),
            generic_name: specialize_info.org_func_name.clone(),
            ty: specialized_func_ty,
            expr: Some(expr),
        };
        symbols.insert(specialized_func_name.clone(), specialized_func);
        global_names.insert(specialized_func_name.clone());

        // TODO: 実際にはここでまた新しい特殊化要求が来る場合があるので、ループで処理していく必要がある。
    }

    prg.symbols = symbols;
    changed
}

// シンボルが特殊化可能かどうか判定する。特殊化可能なときはSpecializableFunctionInfoを生成する。
fn is_specializable_func(sym: &Symbol) -> Option<SpecializableFunctionInfo> {
    // 仮実装。foldとloopだけとする。
    let name = sym.name.clone();
    let name_str = name.to_string();
    if name_str.starts_with("Std::Iterator::fold#") || name_str.starts_with("Std::loop#") {
        // 型情報を取得して第一引数が関数であることを確認する。
        let param_tys = sym.ty.collect_app_src(usize::MAX).0;
        if param_tys.len() < 2 {
            return None;
        }
        if !param_tys[1].is_closure() {
            return None;
        }
        // どちらも第一引数（0-indexed）が特殊化可能。
        Some(SpecializableFunctionInfo {
            func_name: name,
            func_ty: sym.ty.clone(),
            specializable_arg_indices: vec![1],
        })
    } else {
        None
    }
    // 発想：十分条件。
    // 呼び出しグラフを強連結成分分解しておく。
    // 自分より下流のノードしか呼び出していないときはOK
    // そうでないときは、自身をそのまま同じ引数で呼び出していればOK、それ以外はNG。
    // インライン化によって自己再帰が多くなっているはず。

    // TODO: 最終的に特殊化可能であることをチェックするためにはインラインコストも考慮する必要がある。
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
    specializable_funcs: Rc<Map<FullName, SpecializableFunctionInfo>>,
    // デキャプチャリングにより生成された特殊化要求
    required_specializations: Vec<SpecializationInfo>,

    /* ラムダ関数の名前の決定に関するフィールド */
    // ラムダ関数の名前を生成するために使われる番号
    lam_func_counter: u32,
    // 現在デキャプチャリングを行っているシンボルの名前
    // ラムダ関数の名前を生成するために使われる。
    current_symbol: FullName,
    // グローバル名の集合
    // これは、新しいグローバル名を生成するときの衝突を避けるために利用される。
    global_names: Set<FullName>,
}

impl DecapturingVisitor {
    // Visitorを生成する
    fn new(
        current_symbol: FullName,
        specializable_funcs: Rc<Map<FullName, SpecializableFunctionInfo>>,
        global_names: Set<FullName>,
    ) -> Self {
        DecapturingVisitor {
            decap_lambdas: Vec::new(),
            local_decap_lambdas: Map::default(),
            specializable_funcs,
            required_specializations: Vec::new(),
            lam_func_counter: 0,
            current_symbol,
            global_names,
        }
    }

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
    fn decapture_lambda(
        &mut self,
        mut lam: Arc<ExprNode>,
        state: &mut crate::ast::traverse::VisitState,
    ) -> (DecapturedLambdaInfo, Arc<ExprNode>) {
        // キャプチャリストとその型を取得する。
        let cap_names = lam.lambda_cap_names();
        assert!(cap_names.len() <= TUPLE_SIZE_BASE as usize); // TODO: いずれはこの制約を外すために専用の構造体定義にする。

        // もしデキャプチャされたラムダ式をキャプチャしている場合は、事前にlamを訪問しておく。
        for cap_name in &cap_names {
            if self.local_decap_lambdas.contains_key(cap_name) {
                let lam_visit_res = self.visit_expr(&lam, state);
                lam = self
                    .revisit_if_changed(lam_visit_res, state)
                    .expr
                    .calculate_free_vars();
                break;
            }
        }

        let cap_names_types = cap_names
            .iter()
            .map(|name| state.scope.get_local(&name.name).unwrap().unwrap())
            .collect::<Vec<_>>();

        // キャプチャリストの型を作成する。
        let cap_list_ty = make_tuple_ty(cap_names_types.clone());

        // キャプチャリストの式を作成する。
        let cap_list_expr = expr_make_struct(
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
            expr_var(FullName::local(DECAP_NAME), None).set_inferred_type(cap_list_ty.clone()),
            lam.clone(),
        );
        let new_arg = var_local(DECAP_NAME);
        let new_lam = expr_abs_typed(new_arg, cap_list_ty.clone(), new_body);
        let new_lam = internalize_let_to_var_at_head(&new_lam);

        let decap_lam = DecapturedLambdaInfo {
            cap_list_ty,
            lambda_func: new_lam,
            lambda_func_name: self.new_lambda_func_name(),
        };
        self.decap_lambdas.push(decap_lam.clone());
        (decap_lam, cap_list_expr)
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
            hash_data += &format!(",{}", i);
            hash_data += &format!(",{}", decap_lam.lambda_func_name.to_string());
        }
        *name += &format!("_{:x}", md5::compute(hash_data));
        full_name
    }

    // 特殊化された関数の型を作成する
    fn specialized_func_ty(&self) -> Arc<TypeNode> {
        // 関数の型 `A1 -> A2 -> ... -> An -> B` を `([A1, A2, ..., An], B)` に分解し、
        // 特殊化される引数の型をキャプチャリストの型に置き換える。
        let org_ty = self.org_func_ty.clone();
        let (mut doms, codom) = org_ty.collect_app_src(usize::MAX);
        for (i, decap_lam) in self.specialized_args.iter() {
            let cap_list_ty = decap_lam.cap_list_ty.clone();
            doms[*i] = cap_list_ty;
        }

        // 関数の型に戻す
        let mut func_ty = codom;
        for dom in doms.iter().rev() {
            func_ty = type_fun(dom.clone(), func_ty);
        }

        func_ty
    }

    // 特殊化された関数を参照する式を作成する
    fn specialized_func_expr(&self) -> Arc<ExprNode> {
        expr_var(self.specialized_func_name(), None).set_inferred_type(self.specialized_func_ty())
    }
}

// デキャプチャしたラムダ式の情報を保持する構造体
#[derive(Clone)]
struct DecapturedLambdaInfo {
    // キャプチャリストの型
    // キャプチャされる型のタプルである。
    cap_list_ty: Arc<TypeNode>,
    // ラムダ関数
    lambda_func: Arc<ExprNode>,
    // ラムダ関数の名前
    lambda_func_name: FullName,
}

impl DecapturedLambdaInfo {
    // デキャプチャにより作成されたラムダ関数のシンボルを作成する
    fn make_symbol(&self) -> Symbol {
        Symbol {
            name: self.lambda_func_name.clone(),
            generic_name: self.lambda_func_name.clone(),
            ty: self.lambda_func.ty.as_ref().unwrap().clone(),
            expr: Some(self.lambda_func.clone()),
        }
    }
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

        // 名前を取得
        let name = &expr.get_var().name;

        // 変数名がローカルであることを確認する。
        if !name.is_local() {
            return StartVisitResult::VisitChildren;
        }

        // この名前がデキャプチャされたラムダを示しているか確認する。
        let decap_lambda = self.local_decap_lambdas.get(name);
        if decap_lambda.is_none() {
            return StartVisitResult::VisitChildren;
        }
        let decap_lambda = decap_lambda.unwrap();

        // この式の型として要求されている型がすでにキャプチャリスト型と一致しているなら何もしない。
        let expr_ty = expr.ty.as_ref().unwrap().clone();
        let cap_list_ty = decap_lambda.cap_list_ty.clone();
        if expr_ty.to_string() == cap_list_ty.to_string() {
            return StartVisitResult::VisitChildren;
        }

        //　この式の型として要求されている型がラムダ関数の型のコドメインと一致していることを確認する。
        let lambda_ty = decap_lambda.lambda_func.ty.as_ref().unwrap();
        let lambda_codom_ty = lambda_ty.get_lambda_dst();
        assert_eq!(expr_ty.to_string(), lambda_codom_ty.to_string());

        // ラムダ関数にキャプチャリストを与える式に置き換える。
        let lam = expr_var(decap_lambda.lambda_func_name.clone(), None)
            .set_inferred_type(lambda_ty.clone());
        let expr = expr_app_typed(lam, vec![expr.set_inferred_type(cap_list_ty)]);
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
        llvm_expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // LLVM式に与えられている自由変数のうち、デキャプチャされたラムダを示しているものがあれば、
        // ラムダ関数を呼び出した値を与える式に置き換える。

        let mut replace = Map::default(); // LLVM式の自由変数の置き換えのデータ
        for free_name in llvm_expr.free_vars() {
            let opt_decap_lambda = self.local_decap_lambdas.get(free_name);
            if opt_decap_lambda.is_none() {
                continue;
            }
            let decap_lambda = opt_decap_lambda.unwrap();

            // ラムダ関数にキャプチャリストを与えて呼び出す式を作成しておく。
            let lambda_ty = decap_lambda.lambda_func.ty.as_ref().unwrap();
            let lam = expr_var(decap_lambda.lambda_func_name.clone(), None)
                .set_inferred_type(lambda_ty.clone());
            let name_expr = expr_var(free_name.clone(), None)
                .set_inferred_type(decap_lambda.cap_list_ty.clone());
            let expr = expr_app_typed(lam, vec![name_expr]);

            replace.insert(free_name.clone(), expr);
        }

        // LLVM式の自由変数のいずれもがデキャプチャされたラムダを示していないときは何もしない。
        if replace.is_empty() {
            return StartVisitResult::VisitChildren;
        }

        let make_new_name = |name: &FullName| {
            let mut new_name = name.clone();
            new_name.name_as_mut().push_str("#call_decap_lam");
            new_name
        };

        // LLVM式の中の自由変数をリネームする
        let mut llvm_expr = llvm_expr.clone();
        for (name, _) in replace.iter() {
            let new_name = make_new_name(name);
            llvm_expr = replace_free_var_of_expr(&llvm_expr, name, &new_name);
        }

        // LLVM式の前にlet (新変数) = (ラムダ関数の呼び出し); を挿入する
        let mut expr = llvm_expr.clone();
        for (name, call_lam_expr) in replace.iter() {
            let new_name = make_new_name(name);
            expr = expr_let_typed(
                PatternNode::make_var(var_var(new_name.clone()), None)
                    .set_inferred_type(call_lam_expr.ty.as_ref().unwrap().clone()),
                call_lam_expr.clone(),
                expr.clone(),
            );
        }

        StartVisitResult::ReplaceAndRevisit(expr)
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
        state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // このapplication expressionが以下の条件を満たしているときに、クロージャ特殊化を実行する：
        // - 呼び出される関数はグローバルであり、その特殊化可能な引数にラムダ式であるかあるいは既にデキャプチャされたラムダ式（=キャプチャリスト）を与えている。

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
                    decaptured_args[i] = arg.set_inferred_type(decap_info.cap_list_ty.clone());
                }
            } else if arg.is_lam() {
                let (decap_info, expr) = self.decapture_lambda(arg.clone(), state); // この中でargを訪問している
                specialized_args.insert(i, decap_info.clone());
                decaptured_args[i] = expr;
            }
        }
        if specialized_args.is_empty() {
            return StartVisitResult::VisitChildren;
        }

        // 特殊化を要求する。
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
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // 子を訪問する前に、もし引数がデキャプチャされたラムダを示しているときは、このラムダ式の型のドメイン部分が間違っているので、修正する。
        let arg = expr.get_lam_params();
        assert_eq!(arg.len(), 1);
        let arg = &arg[0];
        let arg_name = &arg.name;
        let opt_local_decap_lambda = self.local_decap_lambdas.get(arg_name);
        // 引数がデキャプチャされたラムダを示していないときは何もしない。
        if opt_local_decap_lambda.is_none() {
            return StartVisitResult::VisitChildren;
        }
        let local_decap_lambda = opt_local_decap_lambda.unwrap();
        let cap_list_ty = local_decap_lambda.cap_list_ty.clone();
        let lam_ty = expr.ty.as_ref().unwrap();
        let arg_ty = lam_ty.get_lambda_srcs()[0].clone();
        // 引数の型がすでにあっているときは何もしない。
        if cap_list_ty.to_string() == arg_ty.to_string() {
            return StartVisitResult::VisitChildren;
        }
        // このラムダ式の型を修正する
        let new_lambda_ty = type_fun(cap_list_ty, lam_ty.get_lambda_dst());
        let expr = expr.set_inferred_type(new_lambda_ty);
        return StartVisitResult::ReplaceAndRevisit(expr);
    }

    fn end_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        // 子を訪問したことによって、この式の型のコドメインが変化した可能性があるので、型を修正する。
        // 例：|x| |y| (...) のようなラムダ式があって、yがデキャプチャされたラムダ式だったとき、|y| (...) を訪問したことによってその式が変化しているので、
        // |x| |y| (...) のcodomainの型を修正する必要がある場合がある。
        let lam_ty = expr.ty.as_ref().unwrap();
        let dom_ty = lam_ty.get_lambda_srcs()[0].clone();
        let codom_ty = lam_ty.get_lambda_dst().clone();
        let lam_body = expr.get_lam_body();
        let impl_codom_ty = lam_body.ty.as_ref().unwrap();
        if codom_ty.to_string() == impl_codom_ty.to_string() {
            return EndVisitResult::unchanged(expr);
        }
        let new_lambda_ty = type_fun(dom_ty, impl_codom_ty.clone());
        let expr = expr.set_inferred_type(new_lambda_ty);
        EndVisitResult::changed(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let pat = expr.get_let_pat();
        let bound = expr.get_let_bound();
        let value = expr.get_let_value();
        if bound.is_lam() {
            // 束縛される式がラムダ式のとき、デキャプチャリングを行う
            assert!(pat.is_var());
            let var_name = pat.get_var().name.clone();
            let (decap_lam, cap_list) = self.decapture_lambda(bound, state); // この中でboundを訪問している
            self.decap_lambdas.push(decap_lam.clone());
            self.local_decap_lambdas.insert(var_name.clone(), decap_lam);
            let pat = pat
                .set_var_tyanno(None) // 型アノテーションは誤りになるかもしれない。今後必要ではないので捨てておく。
                .set_inferred_type(cap_list.ty.as_ref().unwrap().clone());
            let expr = expr_let_typed(pat, cap_list, value);
            return StartVisitResult::ReplaceAndRevisit(expr);
        } else if bound.is_var() {
            let name = &bound.get_var().name;
            let opt_local_decap_lambda = self.local_decap_lambdas.get(name);
            if opt_local_decap_lambda.is_none() {
                return StartVisitResult::VisitChildren;
            }
            // let式により束縛される式が変数で、それがデキャプチャされたラムダ式を示している場合。
            let local_decap_lambda = opt_local_decap_lambda.unwrap();
            // boundの型をキャプチャリストの型に設定しておく。
            let bound = bound.set_inferred_type(local_decap_lambda.cap_list_ty.clone());
            let expr = expr.set_let_bound(bound);
            // このlet束縛で導入される変数もself.local_decap_lambdasに追加する。
            self.local_decap_lambdas
                .insert(pat.get_var().name.clone(), local_decap_lambda.clone());
            return StartVisitResult::ReplaceAndRevisit(expr);
        } else {
            return StartVisitResult::VisitChildren;
        }
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
