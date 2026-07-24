# Handoff: consolidate the RC-unit-boundary predicate (do on `unique-check-elim`)

The Array/Buffer redesign (`design.md` §3.3) needs the new unbox `Array` added to the
"indivisible RC unit boundary" special-case. That special-case is currently a scattered,
inconsistent `is_box || is_union || is_punched_array` disjunction. Consolidating it into one
named predicate is a standalone cleanup of the RC/provenance machinery, whose home is
`unique-check-elim` — so it is delegated there (independent of the redesign; avoids
cross-branch conflicts). The redesign (bce) then only adds the unbox `Array` to the unified
predicate and picks up the refactor via the periodic `unique-check-elim` -> bce merge.

The copy-paste instruction handed to the `unique-check-elim` agent:

```
【タスク】RC-unit 境界判定の述語を1つに統一する(unique-check-elim ブランチで)

## 背景
RC 挿入(src/rc_ir/borrow.rs)は値を「RC unit(boxed leaf)」単位に分解して
retain/release を置く。ただし一部の型は「1つの不可分 unit」として扱い、中へ
descend せず値全体の custom traverser で RC する。その判定が
`is_box(type_env) || is_union(type_env) || is_punched_array()` という disjunction で
複数箇所に散在し、しかも不揃い(is_punched_array を含む所と含まない所がある)。
これを1つの名前付き述語に寄せ、不揃いを解消したい。

## 現状の散在箇所(grep で網羅すること)
`src/rc_ir/` で `is_punched_array` と、`is_box`/`is_union` の disjunction を grep。
確認済みの箇所:
- borrow.rs::rc_units_go       — `is_box || is_union || is_punched_array`(全部入り)
- borrow.rs::clamp_unit        — `is_box || is_union`(★ is_punched_array を含まない)
- borrow.rs の ownership-shape ビルダ(is_box||is_union のとき OwnershipShape::Boxed を
  返す箇所)— `is_box || is_union`(★ is_punched_array を含まない)
関連(粒度が違うので機械的に寄せず要判断):
- object.rs::build_traverse の is_punched_array 特別扱い(custom traverser dispatch)
- src/rc_ir/codegen.rs の RC-unit projection(project_rc_unit)
- src/rc_ir/provenance.rs::build_shape(is_box は leaf 境界だが union は variant へ
  descend する点が RC-unit と粒度が異なる。かつ最近「map of boxed leaves keyed by
  path」へリファクタ済みなので、現状を見てから判断)

## やること
1. TypeNode に「custom traverser を持つ不可分 RC unit か」を表す名前付き述語を1つ
   導入(例: `is_rc_unit_root(&self, type_env: &TypeEnv) -> bool`
   = is_box || is_union || is_punched_array)。
2. 上記の RC-unit 判定箇所をこの述語に寄せる(重複した disjunction を消す)。
3. ★ clamp_unit と ownership-shape ビルダが is_punched_array を含まないのが
   latent bug か意図的かを調査して統一する。punched array は rc_units_go では
   1 unit なので、その内部への leaf path は clamp_unit で punched-array 根に丸め
   られるべき/ownership-shape も1 Boxed leaf であるべき、という整合性を確認
   (実際に punched-array 内部の leaf path が生じるかも含めて)。含めるのが正しけ
   れば含める。挙動が変わる場合は memcheck で正しさを確認。
4. build_shape は union を variant へ展開する点で粒度が違うので機械的に寄せない
   (is_box 境界だけ共有できるかは判断に任せる)。
5. 拡張点だけ残す: 将来 Array/Buffer 再設計が unbox な Array をこの述語に1行足す。
   ★ その Array 対応は本タスクではやらない。

## 検証
- 全 opt レベル(max/basic/none)で `cargo test --release`。
- RC 正しさが肝なので minilib / project_euler を memcheck。
- 原則 挙動不変。punched の統一で挙動が変わる場合は、その差が正しい修正である
  ことを memcheck で確認する。

## ブランチ
unique-check-elim で行う(RC/provenance 機構の本拠。bce へは定期マージで入る)。
コミットは論点ごとに分ける(述語導入 / 各箇所の寄せ / clamp_unit 不揃いの修正)。
```
