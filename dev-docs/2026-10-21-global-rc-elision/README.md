# グローバル値から読んだ値の参照カウントを省く変更の検証

## この変更

グローバル値から到達できるオブジェクトは、初期化子が値を作り終えたところで、すべて参照カウントの状態が `GLOBAL` になる。`GLOBAL` のオブジェクトは参照カウントの対象外で、retain も release も実行時には何もしない。

locality 推論（`src/rc_ir/locality.rs`）は、参照カウントの操作ごとに、対象のオブジェクトの状態をコンパイル時に推論するパスである。この変更で、次のどちらかの値には、「入力によらず `GLOBAL` である」という事実（`LeafCond::global`）を持たせた。

- グローバル値そのもの
- 関数本体が、グローバル値から `Destructure` かボックス型のユニオンの `Match` で取り出した値

その値に対する retain と release には `RcState::Global` を付け、コード生成では操作を生成しない。

正しさが懸かっているのは、「`GLOBAL` でないオブジェクトを `global` と判断する」誤りがないことだけである。`global` と判断できない値には、変更前と同じく、実行時に状態を読んで分岐するコードが残る。そのコードは、オブジェクトが `GLOBAL` なら何もしないので、判断できないことは最適化を取りこぼすだけで済む。

この誤りがないことを、次の 4 つの方法で確かめた。

## 確かめたこと

### 1. 証明

`proof.md` は、`RcState::Global` を付けた箇所が数えるはずだったオブジェクトが、実行時にすべて `GLOBAL` であることを、コードを引いて示す。省いた操作は、変更前のコードでも何もしない分岐だったことも示す。

**範囲と扱い:**

- **対象:** commit `aac039931`。後のコードの変更に合わせた保守はしない。性質を保証し続けるのは、下の 2 と 3 の検査とテストである。
- **網羅していない点:** 自分で分かっているものを、`proof.md` の「穴」の節に挙げた。型ごとの traverser の網羅と、その場で書き換える op の網羅などである。

**レビュー:** 文脈を渡さない別のエージェントが 1 周だけ行い、各段を引いたコードと突き合わせた。

- **反例:** 0 件
- **事実の誤り:** 1 件。証明を強める向きのもので、直した。

### 2. develop mode の実行時の検査

`--develop-mode`（テストでは `Configuration::develop_mode()`）で建てると、`RcState::Global` を付けた箇所ごとに、対象のオブジェクトの状態が `GLOBAL` かを確かめるコードが入る（`Generator::build_assert_refcnt_state_global`）。違えば、プログラムは「A reference-counting operation inferred global reached a non-global object.」と出して異常終了する。

テストスイートはすべて develop mode で建つので、この検査はテスト全体で働く。

### 3. 普通の人が書かない形のプログラム

テストスイートの入力は、目的に沿った素直なコードに偏っている。この領域は過去に何度も壊れていて、原因はいつも入力の不足だった。そこで、グローバル値のまわりで、言語としては正しいが普通は書かない形のプログラムを、2 つの視点から書いた。

- 「`global` と判断されるのに実は `GLOBAL` でない値」を作ることを狙ったもの
- 型の形、合流、op を経た値、クロージャ、`Destructor`、再帰などを広く変えたもの

各プログラムは、次の 10 通りで建てて走らせた。

- develop mode で、最適化レベル 4 つ（none / basic / max / experimental）× `--threaded` のあり・なしの 8 通り
- 変更前のコンパイラで `-O max`
- `-O max` の通常のビルドを valgrind memcheck にかけたもの

出力がすべて一致し、assert も memcheck の誤りも出ないことを確かめた。この変更による不具合は見つからなかった。

そのうち 66 本を `src/tests/test_locality/global_reads/` に置き、`test_unusual_reads_of_globals_hold_at_run_time` が develop mode と memcheck で走らせて、記録した出力と比べる。新しいプログラムは、ディレクトリを足せば加わる。

### 4. 実コーパスの memcheck

外の実プログラムを、`-O max` で valgrind memcheck にかけた。

- **デフォルトのレジストリにある、テストを持つライブラリ 36 本の `fix test`:** 35 本で誤りは 0 件だった。`cairo-rsvg-fix` の 11 件は、C ライブラリ librsvg の内部の未初期化読みで、変更前のコンパイラでも同じ件数が出る。
- **Project Euler の Fix の解法 127 本:** 全解法の答えの assert が通った。素の実行が 10 秒以内の 97 本は memcheck にもかけ、誤りは 0 件だった。

## 変更とは別に見つかった、既存の問題

どちらも変更前のコンパイラで同じように起きる。

- **初期化子の契約が、`boxed_from_retained_ptr` の文書にない。**
  - 契約の中身: グローバル値の初期化子は、プログラムの別の場所が持っているオブジェクトを、その結果に取り込んではならない。
  - いま書かれている場所: `Array::mutate_elements` の doc コメントだけである。`FFI::mutate_boxed` と `FFI::boxed_from_retained_ptr` にも書かれていたが、別の作業の commit `cb51c3a03` で、理由の説明なく削除された。
  - 起きること: `main` が `boxed_to_retained_ptr` で C 側に預けたローカルの値を、初期化子が `boxed_from_retained_ptr` で取り込むと、develop mode の「inferred local」の検査で止まる。
- **自分自身を読む初期化子。**
  - 作り方: `unsafe_perform` と FFI で外に状態を持たせ、初期化子が初期化の途中で自分自身をもう一度読むようにする。
  - 起きること: `--threaded` でないビルドでは初期化が 2 回走り、先の値がリークする。`--threaded` のビルドでは `pthread_once` の中で再入してデッドロックする。

## 付録: 再現の手順

- **3 のハーネス:** 1 本のプログラムを上の 10 通りで建て、比べるシェルスクリプトを使った。手順は、各構成で `fix build --develop-mode -O <level> [--threaded] --disable-cpu-feature 'avx512.*' --disable-cpu-feature gfni` を実行して走らせ、標準出力の hash を比べるだけである。
- **4 のレジストリ:** 写しの `fixproj.toml` の `[build.test]` に `memcheck = true` を足し、リポジトリの `valgrind.supp` を改行を挟んで追記して、`fix test -O max --allow-preliminary-commands` を実行する。
- **valgrind と CPU 命令:** AVX-512 や GFNI の命令を含むバイナリは、valgrind の下で SIGILL で止まる。手で valgrind にかけるときは、上の 2 つの `--disable-cpu-feature` を付けて建てる。
