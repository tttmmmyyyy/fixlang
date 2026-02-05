このプロジェクトはプログラミング言語Fixのコンパイラ及び関連ツールを提供しています。

Fixのユーザはプロジェクトファイルfixproj.tomlの[[dependencies]]セクションに依存を追加します。
今回、テスト用の依存というものを追加します。[[dependencies.test]]セクションです。
このセクションに追加された依存は、Fixのテストを実行する際（fix testコマンド）やLanguage Server Protocolで診断を行う際などに利用します（テスト用依存が使われる状況のことをテストモードと呼びましょう）。

テストモードでは、[[dependencies]]と[[dependencies.test]]をマージした依存リストを使います。

ロックファイルはfixdeps.lockという名前になっています。テストモードの場合、fixdeps.test.lockという名前にしましょう。

実装計画を立ててください。
