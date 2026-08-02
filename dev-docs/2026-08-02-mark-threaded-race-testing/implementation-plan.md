# mark_threaded のデータ競合テスト基盤: 実装計画

issue #96。`Std::mark_threaded` を正しく使う限り参照カウントにデータ競合は起きない、という不変条件を CI で継続的に検証する。

## 完成条件

1. `--threaded` でビルドした Fix プログラムを ThreadSanitizer (TSan) の下で走らせる経路が `fix` にある。
2. 複数スレッドが同じ値を retain / release / read する統合テストが CI で走り、TSan clean であることを assert する。
3. そのテストに歯がある。`mark_threaded` を外した対照プログラムでは TSan が競合を報告する。
4. 「`mark_threaded(v)` は v を他スレッドへ共有するより happens-before していること」という precondition が Document.md にある。

## 事前調査

### 現状の threaded RC codegen

`--threaded` を有効にすると、各 RC 操作はオブジェクトのモードで分岐し、threaded 経路は次の 3 つの形を出す。

| 操作 | 生成される形 | 実装 |
| --- | --- | --- |
| retain | `atomicrmw add ... monotonic` | `Generator::build_retain` |
| release | `atomicrmw sub ... release`、0 到達時に `fence acquire` してから破棄 | `Generator::build_release_boxed_with` |
| is_unique | `load ... monotonic`、unique なら `fence acquire` してから local 化 | `Generator::build_branch_by_is_unique` |

いずれも RC IR back end から `src/rc_ir/codegen.rs` と `src/object.rs` 経由で到達する現役の経路である。

### TSan を動かすのに要る 3 条件

いずれもこの機械の LLVM 17.0.6 で実測した。

**1. `sanitize_thread` 属性。** TSan の pass は属性を持つ関数だけを instrument する。属性つき / なしの 2 関数を含む LLVM IR に `-passes='tsan-module,function(tsan)'` をかけると、`__tsan_func_entry` と `__tsan_unaligned_write8` が入るのは属性つきの関数だけだった。Fix の codegen は現在この属性を付けない。

**2. ASLR を落とすこと。** この機械では TSan が `FATAL: ThreadSanitizer: unexpected memory mapping` で起動に失敗する。`vm.mmap_rnd_bits` が大きいディストリビューションで起きる既知の症状で、`setarch $(uname -m) -R` の下では正常に動く。speedtest の harness が既に使っている手段がそのまま使える。

**3. fence を使わない memory order。** 下記。

### fence 形は TSan が誤検出する

2 スレッドがそれぞれ別のフィールドに書いてから release し、最後に release したスレッドの destructor が両方のフィールドを読む C harness で測った。この「敗者の書き込み」対「勝者の destructor の読み」の組が、release/acquire の辺で守られているかを問う形である。

| release の形 | TSan の報告 | 判定 |
| --- | ---: | --- |
| `fetch_sub(release)` + 0 到達時に `fence(acquire)` | 3 件 | **誤検出**。C11 としては正しいコード |
| `fetch_sub(acq_rel)` | 0 件 | clean |
| `fetch_sub(relaxed)` | 2 件 | 真の競合（対照） |

現状の Fix codegen は 1 行目の形なので、このまま TSan にかけると正しいコードが落ちる。TSan は standalone の `atomic_thread_fence` を happens-before として扱わないためで、issue が指摘するとおりである。

### 阻害要因ではなかったもの

- **`-no-pie`**: `fix` のリンカ設定は `-no-pie` だが、`-fsanitize=thread -no-pie` でビルドしたプログラムで TSan は正常に競合を報告した。
- **末尾呼び出し**: `-fsanitize=thread -O2` でも自己再帰はループに畳まれ、1 億段の末尾再帰が完走した。Fix は TCO に依存しているので、ここが潰れると harness が書けなくなるところだった。
- **TSan runtime の入手**: LLVM 17 の配布物に `libclang_rt.tsan.a` が同梱されている。CI が入れる tarball も同じ構成である。

## 段階

### P1: threaded RC の memory order を fence 形から RMW 形へ

TSan と無関係に単体で正しく、後段すべての前提になる。

- **release**: `atomicrmw sub ... release` + `threaded_destruction_bb` の `fence acquire` を、`atomicrmw sub ... acq_rel` 1 個に畳む。fence が消えるとそのブロックは分岐 1 本だけになるので、threaded 経路は他のモードと同じ破棄経路へ合流させる。
- **is_unique**: `load ... monotonic` + `unique_threaded_bb` の `fence acquire` を、`load ... acquire` に置き換える。

C11 の fence 規則の下でどちらも等価である（monotonic な読みの後の acquire fence は、その読みを acquire にしたのと同じ辺を張る）。

**コスト**: 両形を `llc -O3` にかけて生成命令を突き合わせた。

x86-64 では**機械語がバイト単位で同一**である。`lock decl` は既に full barrier で、acquire load は plain `mov` であり、acquire fence は命令を生まない（アセンブリに出る `#MEMBARRIER` はコメントで、コード生成器への並べ替え禁止の印にすぎない）。release 経路も is_unique 経路も、逆アセンブルしたバイト列が完全に一致した。

arm64 は交換になる。

| 経路 | 現在 | 変更後 |
| --- | --- | --- |
| release | `ldxr` / `sub` / `stlxr`、0 到達時のみ `dmb ishld` | `ldaxr` / `subs` / `stlxr`、`dmb` なし |
| is_unique | `ldr`、unique のときのみ `dmb ishld` | `ldar`、`dmb` なし |

すべての操作に acquire が乗る代わりに、`dmb` が消える。`dmb` は acquire 付きロードよりはるかに重いので、**is_unique 側は勝ち筋が濃い**。Fix では unique 経路が常道であり、そこから `dmb` が消えるためである。release 側は 0 到達が稀なので、稀な `dmb` を消す代わりに毎回の acquire を払う形になり、小さな負けの可能性がある。

付随して、fence 形は基本ブロックを分けることを強いるため、比較を `subs` / `cset` に畳む最適化が阻害されている。上の例では命令数が release で 9 対 5、is_unique で 9 対 4 と、変更後のほうが短い。

実機がないため、確認はこの命令列の突き合わせまでとする。

#### コード中に残す制約

acquire を RMW に畳んだ形は、読んだだけでは fence 形に戻せそうに見える。戻すと**正しいまま TSan で落ちる**ようになり、P2 以降が機能しなくなる。それが分かるコメントを 2 箇所に置く。

`build_release_boxed_with` の threaded な減算:

```rust
// The decrement acquires as well as releases, so that the thread that brings the count to
// zero sees every write the other holders made. Keep the acquire in the read-modify-write:
// ThreadSanitizer draws no happens-before edge from a standalone `fence acquire`, so an
// acquire moved into one on the way to destruction leaves the code correct while making the
// race detector report the destructor's reads as racing with those writes. The acquire is
// free on x86-64, where a `lock`-prefixed read-modify-write already orders both ways; on
// AArch64 it costs an acquire on every decrement and saves a `dmb` on the destruction path.
```

`build_branch_by_is_unique` の threaded な読み出し（この経路が acquire を要る理由は既存のコメントが述べているので、置き場所の制約を足す）:

```rust
// Keep the acquire on the load itself: ThreadSanitizer draws no happens-before edge from a
// standalone `fence acquire`, so an acquire moved into one on the unique path leaves the
// code correct while making the race detector report the writes that follow as racing.
```

測定した表は、この計画書が残るので、そちらを根拠として参照できる。

**検証**: `--emit-llvm` の差分で 3 形が意図どおりになっていること、`--threaded` を有効にした既存テスト一式、そして上の C harness と同じ構造の Fix プログラム。

**分割**: これ単独で 1 つの PR にする。

### P2: TSan ビルドの設定設計

TSan は実行時ツールではなくビルド時 instrumentation なので、`ValgrindTool` とは別の設定軸を立てる。

#### 設定の入口

memcheck の前例に倣う。valgrind は CLI フラグを持たず、`Configuration::set_valgrind` と project file の `[build.test] memcheck` の 2 経路だけで入る。診断のためのモードを利用者向けの CLI 表面に出さない設計である。

```rust
/// The sanitizer the generated program is instrumented with.
pub enum Sanitizer {
    /// Generate the program as it is built for use.
    None,
    /// Instrument every memory access so that ThreadSanitizer can report data races.
    Thread,
}
```

**値は 1 つで、project file と CLI で名前も形もそろえる。** `sanitize = "thread"` と `--sanitize thread` が同じ設定であることが見た目で分かる。`opt_level` と `--opt-level` が取っている形と同じである。

単一値にするのは、shadow メモリを使う sanitizer どうしが排他だからである。address / thread / memory は shadow メモリの配置とインターセプタが衝突し、clang は `-fsanitize=address,thread` のような指定をすべてエラーにする（3 通りの組を確認した）。単一値なら、表現できてはいけない組み合わせが型に載らず、検証を書かずに済む。

**優先順位は Overwrite。** `opt_level` と同じく、優先度の高い場所の値が低い場所の値を置き換える。

`Configuration::sanitizer` フィールドを足し、入口を 3 つ置く。

| 入口 | 用途 |
| --- | --- |
| `Configuration::set_sanitizer(Sanitizer::Thread)` | Rust のテストから |
| project file の `[build]` / `[build.test]` の `sanitize = "thread"` | Fix プロジェクトのテストから |
| `--sanitize thread` | Fix 利用者が自分の multi-thread プログラムを検査する |

3 番目は memcheck に無い入口である。P6 で文書化する precondition は利用者が破りうるものなので、利用者が自分のプログラムを検査する手段として置く。

**この設定は計装だけを行い、他の設定に触れない。** `--threaded` とは直交させる。`mark_threaded` を呼ぶプログラムを検査するには `--threaded` も指定する。呼ばずにスレッド間で値を共有するプログラムは `--threaded` 無しのまま検査でき、これは P3 の対照そのものである。`--threaded` 無しで `mark_threaded` を呼んだ場合は既存のエラーが出る。

#### キャッシュキー

**2 箇所ある。どちらを落としても検査が空回りする。**

- `object_generation_hash()` に `sanitizer` を足す。落とすと計装なしのオブジェクトが再利用され、「TSan clean」が「何も検査していない」を意味するようになる。#141 と同じ壊れ方で、そのときは `llvm_passes()` がキーから漏れていた。
- `runtime.c` のオブジェクト名を決める `runtime_obj_hash_source` にも足す。落とすと gcc でビルドされた未計装の runtime が再利用される。

#### pass を足す場所

`llvm_passes()` の**外**に置く。`optimize_and_verify` が、選ばれたパイプラインを走らせた後で `tsan-module,function(tsan)` を追加で走らせる。`llvm_passes()` の戻り値に混ぜると、`--llvm-passes-file` が計装ごと差し替えられてしまう。最適化の後に計装するのは clang と同じ順序である。

`sanitize_thread` 属性も同じく `optimize_and_verify` で、モジュールの定義済み関数すべてに付ける。

#### C コンパイラとリンカ

sanitizer は 2 つの半分でできている。**計装 pass は LLVM 本体**にあり（`opt -passes=tsan` が clang を介さず動くことがその証拠である）、`Module::run_passes` に渡すパイプライン文字列からそのまま使える。**ランタイムは compiler-rt** という別のサブプロジェクトにあり、`libclang_rt.tsan.a` として clang の配布物に同梱される。`fix` がリンクしている libLLVM には入っていない。

clang が担っているのは、この 2 つをつなぐ手続きである。`-fsanitize=thread` を渡すと、clang は全関数に `sanitize_thread` 属性を付け、パイプラインの正しい位置に pass を差し込み、compiler-rt の該当アーカイブを探してリンクする。**front end が clang でなければ、この 3 つは自分でやることになる。** P2 の作業がこの 3 つに分かれているのはそのためである。

したがって sanitizer が有効なときだけ、`runtime.c` のコンパイルとリンクを gcc から clang へ切り替え、`-fsanitize=thread` を渡す。計装と TSan runtime を同じ LLVM に揃えるためであり、リンクドライバを clang にすれば `libclang_rt.tsan.a` の場所も clang が解決する。

**これは clang のインストールへの依存になる。** LLVM を compiler-rt 抜きでビルドした環境や、libLLVM だけを入れた環境では TSan ビルドができない。CI が入れる公式 tarball と brew の `llvm@17` はどちらも compiler-rt を含む。clang が見つからなければ明示的なエラーにする。

属性を付ける手段は inkwell にある。`Attribute::get_named_enum_kind_id("sanitize_thread")` と `Context::create_enum_attribute` の組で、使用中の inkwell 0.5.0 に両方ある。

#### 実行

`commands/run.rs` は valgrind のとき実行コマンドを包んでいる。同じ場所で、sanitizer が有効なときは `setarch $(uname -m) -R` で包む。`fix build` で作ったバイナリを利用者が自分で走らせる場合に備えて、ビルド時にその必要を案内する。

#### プラットフォーム

`platform_valgrind_supported()` に倣って `platform_thread_sanitizer_supported()` を置く。対応しない環境では警告して無効化し、テスト側が明示的に skip する。これは memcheck のテストが既に取っている作法である。

**検証**: TSan ビルドした Fix プログラムの LLVM IR に `__tsan_` の呼び出しが入ること、リンクが通ること、単スレッドのプログラムが TSan clean で完走すること、そして**同じソースを sanitizer なしでビルドした直後に sanitizer 付きでビルドすると計装済みのオブジェクトができること**（キャッシュキーの回帰テスト）。

### P3: multi-thread harness と統合テスト

Fix にはスレッドを立てる関数がないので、Document.md が定める正規ルートで組む。C 側の driver を `--object` でリンクし、Fix から FFI で pthread を張り、`mark_threaded` した値を `boxed_to_retained_ptr` で渡し、スレッドの入口で `FFI_EXPORT` した Fix 関数を呼ぶ。

**スレッドはテスト自身の C driver で立て、外部ライブラリに依存させない。** `fixlang-asynctask` はこの経路の参照実装なので書くときに読むが、テストの依存には加えない。テストが外部リポジトリの取得と、そのリポジトリの都合に左右されるようになるためである。

- C driver が N スレッドで retain / release / read を高い contention で叩く。
- join 後に参照カウントと use-after-free を assert する。
- テストは TSan の出力に報告が無いことを assert する。
- **対照**: `mark_threaded` を呼ばない版を用意し、TSan が競合を報告することを assert する。これがテストの歯である。`test_memcheck.rs` の `test_use_undefined_value` が memcheck のテスト群に対して果たしているのと同じ役割を持たせる。

TSan は動的検出なので、スケジュールを稼ぐために反復回数を上げる。

**memcheck は外す。** `Configuration::develop_mode()` は memcheck を既定で有効にするが、TSan で計装したバイナリを valgrind の下で走らせる意味はない。このテスト群は memcheck を切った設定を使う。

テストは `src/tests/test_thread_safety.rs` に集め、名前で選べるようにする。P4 がこの名前で絞り込む。P1 が置いた `src/tests/test_threaded_rc.rs` は生成 IR を読むだけで TSan を要さないので、通常のスイートに残す。

### P4: CI 統合

**TSan で走らせるのは multi-thread のテストだけにする。** テストの大半は単一スレッドで、それを TSan にかけても競合は原理的に出ず、計装のぶん遅くなるだけである。

Linux 限定の job を 1 つ足し、既存の Test workflow の matrix には混ぜない。

- ubuntu のみ。macOS は P5 の runtime assert で代替する。
- 走らせるのは `cargo test --release tests::test_thread_safety` だけ。
- 既存の job 側は `--skip tests::test_thread_safety` で外す。両側で明示することで、どちらの job が何を持つかが読んで分かる。
- `setarch -R` の下で走らせる（`sysctl` に触らずに済む）。
- 最適化レベルは既存と同じ 3 通りを回す。対象がひと握りのテストなので費用が小さく、かつ**最適化レベルこそが効く**ためである。P5 が挙げるとおり、モード検査を落とすのは最適化であり、`none` と `max` では threaded RC 経路の残り方が違う。

### P5: develop_mode の runtime assert

TSan を回せない環境で効く安価な保険。

RC 操作はモードで分岐するので、分岐が残っている限り「threaded な値に非 atomic RC が走る」ことは構成上起こらない。**危ないのは、分岐を落とす最適化のほう**である。`specialize`、`unique_check_elim`、そして #122 の RC state inference は、いずれも「この値は local だ」と証明してモード検査を消す。証明が間違えばそこに非 atomic 操作が残る。

したがって assert を置くべき場所は、**モード検査を省いた非 atomic RC 操作の直前**であり、そこでオブジェクトのモードが threaded でないことを検査する。`develop_mode` でのみ有効にする。

### P6: Document.md に precondition を明記

「`mark_threaded` を使えば競合しない」は無条件ではない。`mark_threaded(v)` が「v を他スレッドへ共有する」より happens-before していることが前提で、これが崩れると片方が非 atomic RC 中にもう片方が atomic RC を始める。Multithreading の節に、この順序が利用者側の責任であることを書く。

## 範囲外: model checker による protocol 検証

issue が挙げている stateless model checker (CDSChecker / C11Tester / GenMC) による網羅検査は行わない。

この道具が与えるのは「retain/release protocol の**設計**が C11 の下で正しい」という網羅的な結論である。検証の対象は手で書き写した C の模型なので、codegen が実際にその protocol どおりに吐いているかは含まれない。そちらは P3 が受け持ち、そして P1 から P6 だけで完成条件の 4 つは満たされる。加えてこれらは研究ツールで、環境で動かすところから始まる。得るものに対して掛かるものが釣り合わないと判断した。

## 利用者に見える文言

以下 3 つは利用者の目に触れるので、実装前に確定させる。

### 1. project file のテンプレート (`src/docs/project_template.toml`)

`[build]` の `threaded` と `opt_level` の近くに置く。

```toml
## Sanitizer to instrument the built program with.
## One of "none" (default) or "thread".
## "thread" builds the program with ThreadSanitizer, which reports data races at run time.
## The instrumented program runs several times slower and uses much more memory, so use it while
## checking a program.
## Overwritten by the command line argument.
# sanitize = "thread"
```

### 2. コンパイラフラグのヘルプ (`src/main.rs`)

`--opt-level` と同じく、値ごとの説明を持たせる。

- オプションの説明: `Sanitizer to instrument the built program with.`
- `none`: `Build the program as it is built for use.`
- `thread`: `Instrument the program with ThreadSanitizer, which reports data races at run time. The instrumented program runs several times slower and uses much more memory.`

### 3. Document.md

**設定項目の表**に 1 行足す。`threaded` の直後に置く。

| Field | Option | Type | Dependent Project | Description |
| --- | --- | --- | --- | --- |
| sanitize | --sanitize | Overwrite | Does not affect | Sanitizer to instrument the program with |

Type が Overwrite なのは `opt_level` と同じ理由で、値が 1 つに決まるためである。Dependent Project が Does not affect なのは、検査のためのモードを依存ライブラリが強制すべきでないためである（`threaded` は依存ライブラリが必要とするなら強制されるので Affects になっている）。

**Multithreading の節**に、次の内容の小節を足す。

```markdown
### Checking for data races

`--sanitize thread`, or the `sanitize` field of the project file, builds the program with
ThreadSanitizer, which reports a data race when one occurs while the program runs. Use it to
check that every value another thread reaches has been passed through `Std::mark_threaded`, and
that the call happens before the value is shared.

The option instruments the program and changes nothing else, so pass `--threaded` as well to
check a program that calls `Std::mark_threaded`. The instrumented program runs several times
slower and uses much more memory, so use it while checking a program.

`fix run` runs the program the way ThreadSanitizer needs. Run a program built by `fix build` as
follows on Linux, where ThreadSanitizer requires address space layout randomization to be off:

    setarch $(uname -m) -R ./a.out

A race that no run performs goes unreported, so drive the program the way it is used, and run it
more than once.
```

## AddressSanitizer について

同じ仕組みで載る。実測で確かめた。

- LLVM 17 の配布物に `libclang_rt.asan.a` が入っている。
- pass の gate は同じ形で、`sanitize_address` 属性の付いた関数だけが計装される（属性つき / なしの 2 関数で確認した。pass 名は `asan` 1 つで、TSan と違いモジュール用と関数用に分かれていない）。
- **ASan は ASLR を落とさずに動く。** shadow のオフセットが動的に決まるためで、TSan との違いはここである。`-no-pie` でも動く。canonical な use-after-free が報告されることを確認した。

memcheck との関係は、置き換えではなく補完である。

| | memcheck | ASan |
| --- | --- | --- |
| 領域外・use-after-free・二重 free | 検出 | 検出 |
| リーク | 検出 | 検出 (LeakSanitizer) |
| **未初期化値の読み** | **検出** | 検出しない |
| 速度 | 20-50 倍遅い | 2 倍程度遅い |

未初期化値の読みは ASan の守備範囲の外である（MSan の担当）。fixlang のテストはこれに依存していて、`test_memcheck.rs` の `test_use_undefined_value` がその歯になっている。したがって memcheck は残す。

ASan の利点は速度で、memcheck では重すぎて回せない規模のテストに掛けられる。特に `--no-runtime-check` でビルドしたプログラムは、Fix 自身の境界検査が外れた状態なので ASan の対象として筋がよい。

**この issue の範囲には入れない。** ただし `Sanitizer` enum に `Address` を足せば済む形にしておく。`--tsan` ではなく `--sanitize thread` を推す理由がこれである。

### memcheck を ASan で置き換えることについて

テストを軽くする目的では成立しない。理由が 3 つある。

**UBSan は Fix から使えない。** ASan / TSan と成り立ちが違い、front end の機能である。LLVM 17 の pass 一覧に載る関連物は `bounds-checking` 1 つだけで、符号付きオーバーフローのような検査は clang の IR 生成が直接吐いている（`-O0` の出力に既に `__ubsan_handle_add_overflow` が入っており、pass の産物ではない）。Fix が同じものを得るには検査を自前で出すことになり、それは UBSan の採用ではない。配列の境界検査は Fix が既に自前で出している。

**未初期化値の読みが空く。** ASan の守備範囲の外で、UBSan で埋まらない以上どこも埋めない。この検査は #92 の `specialize` の未初期化値という実際のコンパイラバグを見つけている。MSan なら埋まるが、libc まで計装しないと誤検出が出るため現実的でない。

**軽くなる幅が小さい。** 典型的なテスト 1 個を測ると、コンパイルが 4.06 秒、memcheck 下の実行が 1.43 秒、素の実行が 0 秒である。テストのプログラムは小さいので memcheck の 1.43 秒はほぼ valgrind の固定起動費であり、テスト 1 個の所要時間の 4 分の 1 ほどにあたる。ASan にすればこの一部は戻るが、計装のぶんビルドが伸びて相殺される。CI 時間を縮めたいなら本丸はコンパイル側である。

## リスク

- **TSan の実行コスト**。CI 時間が延びる。反復回数と CI 時間のバランスは P4 で測って決める。
- **間接末尾呼び出し**。自己再帰の TCO は保たれることを確認したが、相互再帰や間接呼び出しの末尾呼び出しは未確認である。harness を深い再帰にしないことで回避できる。
- **arm64 の threaded RC が P1 で遅くなる**。実機がないので `llc` の出力までしか確認できない。
- **TSan は起きたスケジュールしか見ない**。反復回数と contention を上げることで確率を稼ぐ以上のことはしない。

## 決まったこと

1. **スコープ**。P1 から P6 まで。model checker による網羅検査は範囲外とする。
2. **P1 の適用範囲**。常に AcqRel を使う。 x86-64 は機械語が同一で払うものがなく、arm64 も `dmb` との交換で一方的な損ではない。CI と本番が同じコードになる。fence 形へ戻されないよう、上の「コード中に残す制約」を置く。
3. **`--sanitize`**。作る。 P6 で文書化する precondition は利用者が破りうるものなので、利用者が自分のプログラムを検査する手段を用意する。

## PR の分割

| PR | 段階 | 内容 |
| --- | --- | --- |
| 1 | P1 | memory order を RMW 形へ。単体で正しい変更 |
| 2 | P2 + P3 | TSan ビルド経路と統合テスト |
| 3 | P4 | CI job |
| 4 | P5 + P6 | develop_mode の assert と Document.md |
