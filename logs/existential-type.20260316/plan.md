Fixに存在量化型を追加することを検討しています。

存在量化型の典型的なユースケースは、トレイトを返す関数です。
例えば、`repeat`という、指定された引数を指定された回数だけ繰り返すイテレータを作成する関数を考えます。

```fix
repeat : a -> I64 -> Std::Iterator::MapIterator Std::Iterator::RangeIterator Std::I64 a = |x, n| range(0, n).map(|_| x);
```

この`repeat`の型を書くのは非常に面倒です。
また、型が「`range(0, n).map(|_| x)`」という内部実装に依存してしまう、という問題もあります。

そこで「**ある** 型であって、`Iterator`トレイトを実装していて、アイテムの型が`a`であるような型」
というものを表現できるように、存在量化型を導入します。

存在量化型の提案文法は以下の通りです。

```fix
repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it = |x, n| range(0, n).map(|_| x);
```

ここで、ハテナなしの型変数（`a`）は「任意の型」でありえるのに対し、ハテナ付きの型変数（`?it`）は（`a`に依存して決まる）「ある特定の型」であることを意味します。

===
AIによるコメント：

結論：**∀…∃…（forall先、exists後）の固定順で実用上十分**。

### 理論的な違い

- `∀a. ∃?it. P(a, ?it)` — 各`a`に対して（`a`に依存しうる）`?it`が存在する
- `∃?it. ∀a. P(a, ?it)` — すべての`a`に対して共通の単一の`?it`が存在する

後者のほうが強い主張。

### プログラミングでは問題にならない理由

**1. 存在型は呼び出し側から不透明**

呼び出し側は`?it`の具体型を知らず、トレイトのインターフェース（`Iterator`等）だけを通じて使う。そのため、「異なる`a`で呼んだ時の`?it`が同じ型かどうか」は呼び出し側にとって意味がない。

**2. Rust・Swiftと同じ設計**

- Rustの`impl Trait`（戻り値位置）は常に`∀…∃…`の順
- Swiftの`some`戻り値型も同様

これらの言語で`∃…∀…`の順が欲しくなって困る、という実例は知られていない。

**3. `∃…∀…`が必要になる仮想的ケース**

`∃?it. ∀a. a -> ?it` のような型、つまり「任意の型を受け取り、型パラメータに依存しない固定の型を返す」関数は理論上ありえる。しかし：

- `∀a. ∃?it. a -> ?it` と `∃?it. ∀a. a -> ?it` には実際に差がある。前者では異なる`a`での呼び出し結果が同じ`?it`である保証がないが、後者ではすべての`a`に対して同じ`?it`が返ることが保証される。
- ただし、この差が問題になるのは「異なる型引数で呼んだ結果を同一の型として統一したい」場合だけ。存在型は呼び出し側から不透明でトレイトインターフェース経由でしか使えないため、そのような場面では結局トレイトオブジェクト（動的ディスパッチ）を使うことになり、存在型の出番ではない。
- また、どの型を受け取っても同じ型を返すなら、その戻り値型は実用上は明示的に書けるはずなので、存在量化する必要がそもそもない。

**4. 本当に差が出るのは高階型（rank-2以上）**

```
f : (∃?it. ∀a. a -> ?it) -> ...
```

のように存在型を**引数位置**で使い、かつ高階の量化が必要な場合に初めて順序の制御が必要になるが、これは非常に稀なユースケースで、多くの言語が未サポート。

### まとめ

提案文法の「暗黙に∀が先、∃が後」という設計は：
- 既存のFix文法と整合的で使いやすい
- Rust/Swiftと同じ実績のあるアプローチ
- 実用上のユースケースをカバーしている

問題なく進めてよい。将来もし`∃…∀…`が必要になった場合でも、その時点で文法を拡張すれば後方互換に対応できる。

===

本機能の名前は「不透明型（opaque type）」とする。

===

## 他の応用例

```
trait c : ToIter {
    type Elem c;
    to_iter : [?it : Iterator, Iterator::Item ?it = ToIter::Elem c] c -> ?it;
}

impl Array a : ToIter {
    type Item (Array) a = a;
    to_iter = |arr| ArrayIterator { _idx : 0, _arr : arr };
}
```

## 文法・パーサーを更新
構文上は不透明型は型変数であるという扱いで良いと思われる。
ハテナマークから始まる型変数は不透明型となる。

## Schemeにopaque_tysを追加（predicates/equalitiesは統一保持）
不透明型はgen_varsに入れるべきではないため、`opaque_tys`フィールドを追加する。
一方、predicates/equalitiesはopaque/non-opaqueを混在して同じフィールドに保持する。これにより、resolve_namespace, set_kinds, LSP等の大半の使用箇所（30箇所）が変更不要となる。
opaque/non-opaqueの分離が必要な箇所（instantiate_scheme, validate_constraints）ではフィルタヘルパー（`opaque_predicates()`, `non_opaque_predicates()`, `opaque_equalities()`, `non_opaque_equalities()`）を使用する。
既存のgen_varsやpredicatesやequalitiesの使用箇所をすべて精査して、opaqueを含まないものだけを返すべきか、opaqueを含むものをまとめて返すべきか、を事前に確認し、計画しておくべきである（**TODO 1**）。
→ 実行済み。詳細は [todo1_scheme_field_analysis.md](todo1_scheme_field_analysis.md) を参照。要点：predicates/equalities統一方式により30箇所が変更不要、gen_varsの使用箇所は `all_tyvars()` ヘルパーで対応（5箇所）、dual（Assume/Requireで逆処理）は instantiate_scheme に集中（4箇所）、validate_constraints は special（3箇所）。

## instantiate_schemeの変更
以下のように変更すると良いと思う。
以下の考察からわかるように、assumeとrequireの処理には「双対性」があるようだ。
似たようなコードを2回ずつ書くことになるので、ヘルパー関数の追加が必要かも（**TODO 2**）。

### Assumeのとき
主な利用箇所：「x : Scheme」の実装式の型推論を始めるときに、SchemeをAssumeする。
fixed_tyvarsにはopaque typesを入れてはいけない。
opaque typesは、Requireでやっているように、型変数を発行する（new_tyvar_by）。
opaque typesを含むequalitiesやpredicatesは、assumeされる（型推論の証明において仮定として使える）のではなく、証明される必要がある。よって、Requireでやっているように、self.predicatesに追加、add_equalityに追加、する。
これにより、check_type関数で、opaque typeを含むpredicateやequalityが証明されたことのチェックが行われる。

### Require のとき
主な利用箇所：実装の中で「x : Scheme」を変数として使うとき、SchemeをRequireする。
Requireされるschemeにとっては、opaque typeは「型変数」ではなく「決定した型」である。
（型チェック中一時的にopaque typesをtype constructorに追加する、ということも考えたが、）おそらく、Assumeでやっているように、opaque typesはfixed_tyvarsに突っ込めば良いのではないか。
* ここでfixed_tyvarsに突っ込むときに、opaque typeを可読性のある良い名前に変更しておく必要があると思う。例えば`Std::Iterator::repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it` の「`?it`」は「`Std::Iterator::repeat::?it`」という名前にするのが良さそう。これは、fixed_tyvars内での名前の衝突を防ぎ、コンパイラのデバッグやエラーメッセージの可読性に効く。
 * この際opaque typesのリネームが発生するので、opaque typesを含むequalitiesやpredicatesをsubstituteする必要がある。
また、opaque typeを含むpredicateやequalityは、型推論の前提として使用可能になる。よって、self.assumed_preds, self.assumed_eqsに追加する。

 **TODO 3** local_assumed_eqsにも追加するべきだろうか？
→ 不要。local_assumed_eqsはinstantiate_symbolで使われるが、下のfix_typesについてのセクションでも述べているように、instantiate_symbolの時点でopaque typeは完全に解消されるため。

 **TODO 4**：check_typeではself.eqalitiesがゼロになったことを調べているが、check_scheme_equivalent_oneでやっていないのはなぜ？ミスか？
→ 調査済み。おそらくバグ（チェック漏れ）。詳細は [todo4_equalities_check.md](todo4_equalities_check.md) を参照。
→ 実行済み。`check_scheme_equivalent_one`にequalitiesチェックを追加した。

### check_type
各opaque typeの「値」が固定された（fixされた）、つまり、型コンストラクタかfixed_tyvars（=その値のSchemeのgen_vars）で表現されるものとunifyされたことをチェックする。
instantiate_scheme(Assume)でopaque typeに対して型変数を発行しているので、TypeCheckContextのsubstitutionのkeyにその型変数があり、その先が上述のような「fixedな型」になっていることを検証する。
、、、と思ったが、この処理はfix_typesの中に書いたほうが良いだろう。よってcheck_typeはおそらく本質的な修正はなし。

## fix_types
型推論・型チェックが終わったあと、各ExprNodeのtype_（推論された型を入れるフィールド）を「固定する」、いいかえれば、（fixed_tyversに含まれていない）型変数を消去する、という処理。
これは2度使われる：check_typeの中と、instantiate_symbolの中。
* check_typeでのfix_types呼び出し：このときはfixed_tyvarsが設定されている、つまり、そのグローバル値の型シグネチャ（スキーム）で導入されている型変数のみ残る。またAssocated Typeもこの時点では残る可能性がある。
* instantiate_symbolでのfix_types呼び出し：このときはfixed_tyvarsが設定されていないので、完全に型変数がなくなり、Associated Typeも解消される。
fix_typesに対する修正：
そのグローバル値の型スキームをAssumeするときに追加されたopaque typeが解消された（他のfixedな型にsubstituteされた）ことを検証し、そうでないときはエラーメッセージを出すべき。
注意：実際には使わない不透明型（例：`pi : [?t : ToString] F64` があるとき、`?t` が決定できない、というエラーが出るべき。

 **TODO 5**以前finalize_typesをfix_typesにリネームしたが、fix_typesから呼ばれるサブルーチンの中にfinalizeという単語が残っているようである。リネームするべき。
→ 実行済み。`finalize_types_for_pattern` を `fix_types_for_pattern` にリネームした（typecheck.rs内、定義1箇所＋呼び出し4箇所）。

## validate_constraints
パースのあと、型推論・型チェックに入る前に、constraint（equalityやpredicate）の書き方についてのvalidationする処理である。

predicates・equalityについて：
上述のように、opqaue typeを含むpredicate, equalityは、そうでないpredicate, equalityとは扱いが全然違う。Schemeの別のフィールドに格納されるべき。
この「opq predicate, opq equality に対する制約は以下の形を取ることができる：
* `?monad : * -> *` のような kind signature
	* Fixでは型変数に対するカインド推論は未実装。明示的にユーザがkind signature（e.g., `f : * -> *`）を書く必要がある。一旦はopaque typesでも同様でよいかと思う（便利ではないが）。
* `?t : MyTrait` のようなpredicate
* `MyAssocTy ?t a1 ... an = <type>` のような`?it` に対する関連型の指定。
	* 以下でgen_varsとは、この不透明型が出現する型シグネチャにおけるgeneralized variable、およびトレイトメンバーの型シグネチャの場合は実装型（`trait c : MyTrait`における`c`のこと）も含まれるものとする。
	* `a1`...`an`は`MyAssocTy`のarityの個数だけ用意された型変数である。
		* これらはgen_varsとdisjointでなければならない。
		* これらはopaque typeであってはならない（ハテナから始まってはいけない）
	* `<type>`に出現する（opaqueでない）型変数は、gen_varsあるいは`a1`,...,`an`である。
		* → そもそも、ここで新しい型変数をつかった場合、ユーザはその型変数をgen_varsとして扱うことを期待しているか。なので、この制約は実装しなくても良いかも。
		* Schemeのgen_varsを計算するとき「`a1`...`an`をgen_varsに追加しない」というのが大事か。
	* 同じAssociated Typeとopaque typeに対するこの類の制約を2回以上書いてはいけない。
以上の条件は、この型スキームを持つグローバル値（あるいはトレイトメソッド）を式として使用し、Requireされた状況で（→するとopaque typeに関する条件は証明の仮定として使われる、つまり実質的にAssumeされるわけだが）、Associated Typeを解消する処理reduce_type_by_equalityで適用する仮定が一意になり、合流性が保証されるようにするためのもの。

 **TODO 6** Associated typeのコード中の出現は基本的にsaturatedである必要があったはずなのだが、validate_constraintsにはそれが書かれていないな。どこに書いてあるんだろう？探す。それを満たしていないときのエラー出力コードはあるか？
→ 調査済み。saturationチェックは`validate_constraints`ではなく、名前解決フェーズ（`TypeNode::resolve_namespace`、[src/ast/types.rs](src/ast/types.rs) L691-699, L712-724）で行われている。未飽和の場合は「Associated type `...` has arity N, but supplied M types. All appearance of associated type has to be saturated.」というエラーが出る。arity情報は`TraitEnv::assoc_ty_to_arity()`（[src/ast/traits.rs](src/ast/traits.rs) L1274-1284）から`NameResolutionEnv`に渡される。`validate_constraints`はこのチェック済みを前提としている。

## LSP

opaque typeにホバーしたとき、それを解決した型が表示される必要がある。
例：
```
pi : ?f = 3.14;
```
この「`?f`」でホバーしたら「`F64`」と表示される。


 **TODO 7**：opaque typeの典型的なユースケースを挙げて、それはFixでどうかけるのか。上の方向性でそれが実現できるのか検証せよ。
→ 調査済み。[use_cases.md](use_cases.md) を参照。

## テスト計画
* 基本、test_opaque_typeというモジュールを作ってそこに実装
* 本文中に出てきているサンプルが動くこと
* トレイトメソッドの定義ではなく、実装において書けるアノテーションでもopaque typeを使う例
* higher kinded opaque typeを使う例
* 追加するvalidation一つ一つに対して、それにfailするコード例
* LSPに対するテスト（これはtest_opaque_typeではなく適切なフォルダ（lsp用のフォルダ）に配置）
	* opaque typeにホバーすると「解消された型」が表示されること

**TODO 8**：他に追加するべきテストがないか検討せよ。

 **TODO 9** tests/test_associated_typeを作り、ここに関連型についてのテストコードを移動。
→ 実行済み。`src/tests/test_associated_type.rs`を作成し、`test_basic.rs`から以下の10テストを移動：`test_associated_type_collects`, `test_associated_type_type_level_arithmetic`, `test_associated_type_equality_in_impl_context`, `test_associated_type_equality_in_impl_context_unsatisfied`, `test_regression_f28ea22`, `test_regression_on_associated_type_bug`, `test_associated_type_in_type_sign_lacking_assumption`, `test_associated_type_use_unknown_type_variable_in_associated_type_implementation`, `test_regression_issue_70`, `test_higher_kinded_associated_type`。全10テストパス確認済み。

 **TODO 10**：あらゆる箇所でのassociated typeのusageはsaturatedでなければならない。この条件を満たさないコード例をテストコードに追加し、エラーが出ることを確認せよ。特にimplでのassociated typeの実装の左辺。
→ 実行済み。`test_associated_type.rs`に以下の5テストを追加し、全パス確認済み：
  - `test_unsaturated_associated_type_in_global_function_signature`：グローバル関数型シグネチャでの未飽和
  - `test_unsaturated_associated_type_in_equality_constraint`：equality制約での未飽和（多引数assoc type）
  - `test_unsaturated_associated_type_in_impl_rhs`：impl関連型RHSでの未飽和
  - `test_unsaturated_associated_type_in_type_annotation`：式中の型アノテーションでの未飽和
  - `test_unsaturated_multi_param_associated_type`：多引数associated typeで引数不足
  ※ 既存の`test_regression_f28ea22`がトレイトメソッド型シグネチャでの未飽和をカバー。
  ※ impl関連型のLHSについては、パース時の`validate_as_associated_type_defn()`で構造チェックされるため、引数の個数が合わない場合はパースエラーとなる（別のチェック経路）。


**TODO 11**：opaque typeを追加したことで、Associated Typeのuse caseが増えるので、TODO10の視点でテストを更に増やすべきである。実行せよ。

**TODO 100**：この計画に穴・抜け・漏れがないか検討せよ。

**TODO 999**：すべてのTODOが実行済みであることを確認せよ。