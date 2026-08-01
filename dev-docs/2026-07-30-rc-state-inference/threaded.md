# threaded ビルドの locality 推論

`design.md` は `config.threaded` が偽のビルドを対象にしている。この文書はその続きで、threaded
ビルドへ広げるための作戦と設計を扱う。`design.md` の性質・束・転送・手続き間・健全性の論証は
そのまま前提とし、差分だけを書く。

## 何が取れるか

`build_branch_by_refcnt_state` は threaded ビルドで 3 分岐（local / threaded / global）、
非 threaded で 2 分岐（local / global）を出す。実行時に `LOCAL` 状態のオブジェクトは threaded
ビルドでも非 atomic な local アームを通るので、**`Local` を証明して得られるのはディスパッチの
除去である。** atomic が消えるのは実際に `THREADED` なオブジェクトの場合だけで、それは定義上
`Local` と証明できない。取り分は非 threaded と同種で、分岐が 3 分岐なぶん 1 site あたり
やや大きい。

`Threaded` 側を証明して分岐なしで atomic を出す道もあるが、これは must 性質なので別の解析に
なる。しかも `mark_threaded_one` は「既に threaded か global のオブジェクトは状態を保つ」ので
`mark_threaded` の結果ですら `THREADED` と断定できず、素直には書けない。

## 測定対象が無い

```
benchmark/speedtest/cases/*/fixproj.toml   46 件すべて threaded の行はコメントアウト
mark_threaded を使うコード                 src/tests/test_provenance/cases/mark_threaded/ のみ
```

`plan.md` の「threaded アームは 1 度も取られない」は非 threaded ビルドでの測定なので、threaded
ビルドの分布については何も言っていない。**この作業の効果を測る対象がリポジトリに 1 つも無い。**

## 作戦

### 第 0 手 — 測る

設計の分岐点は 1 つだけである: threaded プログラムの RC 操作のうち、実行時に `LOCAL` なものと
`THREADED` なものの比。

- `LOCAL` が支配的なら `Local` 証明が狙いで、論点は下の origin 汚染の粗さになる。
- `THREADED` が支配的なら `Local` 証明はほとんど無価値で、狙うべきは上に書いた `Threaded` の
  must 証明になる。

推測はできないし、外すと escape 推論を書く労力が丸ごと無駄になる。測定対象が無いので、
`Document.md` の正規ルート（FFI で pthread を張り、`mark_threaded` した値を
`boxed_to_retained_ptr` で共有する）に沿ったベンチを 1 本書くのが最初のタスクになる。これは
#96 のハーネスと大部分が重なるので、合流させるのが得である。

### 第 1 手 — 検証手段の位置づけを決める

「誤証明はデータ競合なので単スレッドのテストには見えない」は、`develop_mode` の assert には
当てはまらない可能性が高い。論証:

1. `LOCAL` なオブジェクトが 2 スレッドから到達可能になるには、先に `mark_threaded` と
   `boxed_to_retained_ptr` を通る必要がある。
2. よって `LOCAL` から `THREADED` への遷移は、そのオブジェクトを所有している 1 スレッドの中で
   逐次に起きる。
3. したがって「`Local` と注釈した site で状態バイトを読む」assert は、スケジュール依存でなく
   その場で誤りを捕まえる。

これが正しければ #96 は前提条件ではなく補完（codegen の memory order の検査）になり、順序制約
が 1 つ外れる。第 0 手のベンチができた時点で確かめられる。

### 第 2 手 — escape 推論（下の設計）を測定の後に詰める

骨格は `design.md` の再利用で、追加は「何が `mark_threaded` に流れ込みうるか」の 1 つだけ。
束は 2 点のまま（`Local` / `MayExt`）で、扉が 1 つ増えてエイリアスの問題が出る、という差分に
とどまる。

### 第 3 手 — share-nothing が入るなら書かない

後述のとおり、`mark_threaded` を deep clone にするとこの文書の escape 推論は不要になる。
投資の前にそのモードを入れるかを決める。

## 設計: origin 汚染と未 escape 窓

追加で要るのは `design.md` の「健全性は `threaded = false` に依存する」で述べた時間性への
対処で、問題の形はこうなる:

```
let a = ...;              -- a の値はここで決まる（Local）
retain a;
let b = a.mark_threaded;  -- b は MayExt。だがオブジェクトは THREADED になった
release a;                -- a は Local のまま -> Local 注釈 -> 非 atomic release。不健全
```

結果だけを `MayExt` にする forward 伝播では、マークされたオブジェクトに届く既存の束縛
（エイリアス）が漏れる。エイリアスはオリジン（確保点・パラメータ）を共有するので、
**オリジンを `Always` にする**のがエイリアス解析なしで漏れなく覆う最も粗い過大近似
（「`mark_threaded` に流れ込みうる値は生まれた時点から `Always`」）。機構は `borrow_ify` の
`infer_ownership` の第 3 の実例として作れる: あちらは consume site を種に、`origin`
（本体内の逆向き値追跡）でパラメータまで遡り、own フラグをプログラム全体の不動点まで単調に
育てる。こちらは種を `mark_threaded` op のオペランドに、フラグを「`mark_threaded` に流れ込み
うる」に替える。`origin` の遡り先にはパラメータ（手続き間の伝播）とローカルの確保束縛
（`Always` の付け先）の両方が出る。

ただしオリジン汚染は保守的すぎる面がある: マークより**前**に実行される操作は実行時には
`LOCAL` を見るので、本当は `Local` にできる。とくに「単スレッドで構築してから公開する」形では
構築フェーズの操作が全部それに当たる。これはフロー感度の精密化で健全に拾える —
**値が公開されうる位置（`mark_threaded` のオペランド、または公開されうる値への取り込み）に
まだ流れていない区間**では、オブジェクトを保持するのは現スレッドだけなので `Local` を出せる。
窓が閉じるのはマークの時点ではなく「公開されうる値に取り込まれた時点」（コンテナ経由の
間接マークがあるため）。ループ・再帰でマークを跨ぐ site — ある反復でマークされ次の反復で
同じ site が触る形 — は、再帰呼び出しがパラメータに「公開済みかもしれない」を join する
ことで自動的に `MayExt` 側へ落ちる。

ベースライン = オリジン汚染、精密化 = 未 escape 窓、の 2 層で設計し、精密化は構築フェーズの
実測が要求してから足す。

## escape 推論が要るのは `mark_threaded` だけである

3 つの扉のうち、既存の束縛が指しているオブジェクトの状態を変えるのはこれだけで、`mark_global`
は初期化子の結果グラフにしか掛からず、`boxed_from_retained_ptr` は新しいハンドルを作るだけ
なので、どちらも前向きの規則で足りる。

したがって、スレッド間で共有せず複製するモードを入れて `mark_threaded : a -> a` を deep clone
として実装する場合 — 引数のオブジェクトは `LOCAL` のまま、結果は新しいグラフ — この文書の
escape 推論は不要になり、threaded ビルドの locality は `design.md` と同じ前向き 1 パスに
なる。シグネチャが値を返す形になっているので、この差し替えは呼び出し側から見て互換である。
