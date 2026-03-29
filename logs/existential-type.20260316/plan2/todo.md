# TODOファイル

本ファイルは人間が管理するAIエージェントのTODOリストです。
対応が完了したら、セクション名の冒頭に「(DONE)」を追記してください。**それ以外の部分は編集しないでください。**
TODOを対応せよ、という指示を受けたら、DONEになっていないセクションを確認してください。
「作成中」というセクションは私が書いている途中なので、まだ確認しなくて良いです。

## (DONE) to_iterでも丁寧に説明する

plan2.md
```
- **型コンストラクタを生成する**

  例：`repeat : [?it : Iterator, Item ?it = a] a -> I64 -> ?it` の場合：

  ```
  型コンストラクタ ?it（カインド * → *）
  型引数：a（元のSchemeのgen_vars）
  ```

  - 名前：`{関数のフルネーム}::?{opaque名}` 例：`Std::Iterator::repeat::?it`
  - wrap 関数：`{関数のフルネーム}::#wrap` 例：`Std::Iterator::repeat::#wrap`（関数ごとに1つ）
  - 型引数：元の Scheme の gen_vars
  - opaque type は struct ではない。内部表現は型チェック後に決まる。

- **trait member の場合も型コンストラクタを生成する**

  ```
  trait c : ToIter の to_iter に ?it がある場合
  → 型コンストラクタ ToIter::to_iter::?it を生成（型引数：c）
  ```
```

グローバル値の場合(repeatの場合)と、trait memberの場合（to_iterの場合）は、両方とも、本計画において重要な例です。
上の書き方は、repeatの方は丁寧に説明し、to_iterの場合は簡略化して説明していますが、to_iterの方もrepeatと同程度に丁寧に説明しましょう。

## (DONE) 実装計画書にエラーメッセージの検証と改善の項目を追加する

impl_plan.md の「エラーメッセージの検証と改善」セクションに記載済み。Phase 6 完了後に opaque type の性質を破るコードをコンパイルし、エラーメッセージの確認・改善を行う計画。

## (DONE) LSPについて、実装計画書に追加する

内容：
opaque typeにホバーしたとき、それを解決した型が表示される必要がある。
例：
```
pi : ?f = 3.14;
```
この「`?f`」でホバーしたら「`F64`」と表示される。

## (DONE) テスト計画書を作成

最低限でも以下のようなテストが必要である。

* 基本、test_opaque_typeというモジュールを作ってそこに実装
* use_cases.md 中に出てきているサンプルが動くこと
* トレイトメソッドの定義ではなく、実装において書けるアノテーションでもopaque typeを使う例
* higher kinded opaque typeを使う例
* opaque type に対し higher arity associated type を使う例
* opaque type に対し higher kinded associated type を使う例
* 追加するvalidation一つ一つに対して、それにfailするコードがfailし適切なエラーメッセージを持つことを確認する。
* LSPに対するテスト（これはtest_opaque_typeではなく適切なフォルダ（lsp用のフォルダ）に配置）
	* opaque typeにホバーすると「解消された型」が表示されること

## (DONE) 詳細な実装計画を作成する

desugarの大まかな流れは決まった。
現在のFixの実装を見て、具体的にどのコードをどう変更すれば良いのか、計画を立てつつ、実行可能性を検証しよう。
新しいplanファイル（impl_plan.md）などを作成して、そこに詳細な実装計画を書く。

## (DONE) opaque type constructorの置換についての記述が正しいかチェック

plan2.md
```
- **opaque type constructor の置換**：`?it a` → `MapIterator (RangeIterator I64) a` のように、Phase 5 の情報に基づき opaque type constructor を具体型に置き換える。置換後の型は型変数を含みうるが、opaque type constructor や associated type は含まない。
```
ここについて。

＜前提知識：opaque type導入前のコードについての解説＞
fix_typesは2回呼ばれる：check_typeの中（先）と、instantiate_symbolの中（後）。
・check_typeから呼ばれた場合、各ExprNodeのtype_は、gen_vars、associated typesを含む。しかし（fixedでない、つまり、gen_varsでない）型変数は含まない、という状態に「fix」される。
・instantiate_symbolから呼ばれた場合、各ExprNodeのtype_は、一切の型変数を含まず、完全に具体的な型になる（associated typeもtype variableも残らない）、という状態に「fix」される。

＜opaque type実装でやるべきこと＞
opaque typeを解消する、という処理をするのは2回目のfix_typesの段階である。
で、その結果、完全に具体的な型（opaque type constructorもassociated typeもtype variableも残らない）という状態に「fix」される、というのが正しい状態である。
よって上の説明における「置換後の型は型変数を含みうるが、」は不適切。

以上の情報を用いてplan2の当該セクションおよび実装計画を修正・詳細化せよ。

## (DONE) plan1のTODO11

plan1.mdのTODO11を実行せよ。

## (DONE) TraitDefnを調べる必要はあるか？

impl_plan.md
```
各 GlobalValue の Scheme を走査し、`?` で始まる TyVar を opaque 型変数として検出する。
```

→ 正しい。`create_trait_member_symbols` の後なので trait member の情報も GlobalValue (`SymbolExpr::Method`) に含まれている。TraitDefn を別途走査する必要なし。impl_plan.md の 1-1 を修正済み。

## (DONE) 実装計画のKindの計算が間違っている

impl_plan.md の 1-3 の Kind 計算を修正：
- gen_vars の個数だけでなく、各 gen_var のカインドを反映するように修正。
- trait member（to_iter）の例と higher-kinded gen_var の例を追加。
- グローバル値と trait member で処理が統一できることを明記。

## (DONE) Schemeの型変数の収集処理をヘルパーメソッドに切り出す

Scheme::generalizeに以下のような処理がある：
```
let mut vars = vec![];
for pred in &preds {
    pred.free_vars_to_vec(&mut vars);
}
for eq in &eqs {
    eq.free_vars_to_vec(&mut vars);
}
ty.free_vars_to_vec(&mut vars);
```
これをヘルパーメソッドに切り出せ。
またプロジェクト中の似たような処理がもしあれば、このヘルパーメソッドに置き換えよ。

## (DONE) wrap関数の型変数の命名規則を決める

wrap関数では、不透明型に対して一つ一つgen_varを発行している。plan2.mdではxとかyとか書いている型変数である。この型変数の名前は実際にはどう決まっているか？
ユーザが定義する型や型変数と決して衝突しない名前にするべき。また不透明型との対応がわかりやすいよう、不透明型の名前を含むあるいは編集してつくるべき。
その命名方法は、instantiate_schemeの中でwrapをrequireするときに「不透明型に対応していた型変数」を発見する処理で利用するべき。

## テストを実装

テスト計画（test_plan.md）をもとに不透明型のテストを実装。

## テスト計画の内容がすべて実装されているか確認

テスト計画（test_plan.md）にかかれているテストが本当にすべて実装されているか確認。

## 作成中

## 作成中

## 作成中

## 作成中

## 作成中

## 作成中