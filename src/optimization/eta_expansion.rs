/*
# Eta expansion transform.

## 概要

この変換は、「ラムダ式を生成して返す」というexpressionを、ラムダ式そのものに変換する。

具体的には、以下のようなグローバル定義を考える。

```
f : T;
f = {expr};
```

`n`が与えられたとき、eta expansion は、まず、上記の定義を以下のように変換し、

```
f : T;
f = |x1,...,x_n| {expr}(x_1,...,x_n);
```

引き続いて application inlining を行う。

## 目的

この変換は、関数を uncurry 最適化し、`n`個の引数をとる関数を生成するために必要である。

## 適用範囲

この変換は、uncurry 最適化の前に適用されることを想定しているため、uncurry最適化が生成する multi-parameter lambdas を含む式をサポートしない。

## 例

```
g = |x| e2;
f = if c { |x| e1 } else { g };
```
は、
```
f = |y| if c { e1[x:=y] } else { g(y) };
```
に変換される。

## 問題点

```
f = let arr = Array::fill(1e6, 0); in |i| arr.@(i);
```

のような定義があり、`f`がプログラム中で頻繁に参照される場合、`Array::fill(1e6, 0)` が何度も評価されてしまい、パフォーマンスが著しく低下する可能性がある。

## 注意点

### `Std::fix`との相性

`fix` の定義にはこの変換を適用するべきでない。
`fix` は `|f||x| LLVM[fix(f,x)]` と定義されており、`LLVM[fix(f,x)]` 内で `get_insert_block().get_parent()` を用いて `fix(f)` 即ち `|x| LLVM[fix(f,x)]` の実装を取得している。
もし `fix` に eta expansion を適用して `|f||x||p| LLVM[fix(f,x)](p)` のように変換してしまうと、`get_insert_block().get_parent()` が `|p| LLVM[fix(f,x)](p)` の実装を取得してしまい、正しく動作しなくなる。

*/

use std::sync::Arc;

use crate::{
    ast::{
        expr::{var_var, ExprNode},
        name::FullName,
    },
    expr_abs_typed, expr_app_typed, expr_var,
    misc::Map,
    optimization::{application_inlining, let_elimination},
};

// Perform eta expansion to take `n` arguments.
//
// If the type does not allow taking `n` arguments, return None.
pub fn run_on_expr(expr: Arc<ExprNode>, n: usize) -> Option<Arc<ExprNode>> {
    let ty = expr.type_.clone().unwrap();

    // Count the number of arguments that can be taken from the type.
    let (doms_tys, _codom_ty) = ty.collect_app_src(usize::MAX);
    if doms_tys.len() < n {
        return None;
    }

    // Determine the names of the new parameters.
    let mut new_params = vec![];
    for i in 0..n {
        new_params.push(FullName::local(&format!("#v{}", i)));
    }

    // Create the new lambda expression.
    let mut body = expr;
    for (i, param) in new_params.iter().enumerate() {
        // Get the type of the additional parameter.
        let var_ty = doms_tys[i].clone();

        // Create the variable expression of the additional parameter.
        let var = expr_var(param.clone(), None).set_type(var_ty);

        // Create the application expression `{body}({var})`.
        body = expr_app_typed(body, vec![var]);
    }

    // Abstract `body` by `params` in reverse order.
    let mut expr = body;
    for (var, ty) in new_params.into_iter().zip(doms_tys.into_iter()).rev() {
        expr = expr_abs_typed(var_var(var), ty, expr);
    }

    // Apply application inlining to the generated expression.
    let mut expr = application_inlining::run_on_expr(expr);
    let_elimination::run_on_expr_once(&mut expr, &Map::default());

    Some(expr)
}
