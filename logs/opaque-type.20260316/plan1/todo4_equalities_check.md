# TODO 4 調査：check_scheme_equivalent_one の equalities チェック漏れ

## 問題

`check_type`では`self.equalities`がゼロになったことを調べているが、`check_scheme_equivalent_one`でやっていないのはなぜ？ミスか？

## 結論

おそらくバグ（チェック漏れ）である。

## 状況

- `check_type`（[typecheck.rs](../src/elaboration/typecheck.rs) L1321-1324）では、`reduce_predicates()`後に`self.equalities.len() > 0`をチェックし、残存equalityがあればエラーにしている。
- `check_scheme_equivalent_one`（同 L1251-1267）では、`reduce_predicates()`後に`self.predicates`のチェックはあるが、`self.equalities`のチェックがない。

## なぜバグと判断するか

`check_scheme_equivalent_one`では`lhs`をAssume、`rhs`をRequireしている。Require時にrhs側のequalityは`self.equalities`に追加される。`add_equality`は簡約可能なequalityはunifyで解消するが、解消できないものは`self.equalities`に残る。例えば、lhsに`[a : Iterator, Item a = I64]`、rhsに`[a : Iterator, Item a = String]`のようなケースでは、predicateチェックは通るがequalityが不整合のまま残り得る。現状はこれを見逃す。

## なぜ今まで問題にならなかったか

`check_scheme_equivalent`はトレイト実装の型シグネチャ検証（[program.rs](../src/ast/program.rs) L1130）でのみ使われる。トレイト実装では通常、associated typeが一致しないとpredicateの段階やunifyの段階で先にエラーになるため、equalityだけが残る状況が起きにくかった。

## 対応

opaque type実装時に`check_scheme_equivalent_one`にequalitiesチェックを追加する。opaque typeではequalityの出番が増えるため、このバグが顕在化するリスクが高まる。
