# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

`ufodb_v0`: Union-Find（DSU / 素集合データ構造）を核にしたRust製のDB。CLI(REPL)経由でキーのみを管理する。

## バージョニング方針

メジャーバージョンごとにリポジトリを分ける、という方針自体は維持する。ただし**ストレージ（永続化）・TCPサーバー化・ユーザーアカウント（TCP接続時のログイン用）・configファイル**の4機能は、当初v1（別リポジトリ）を想定していたが、v0（このリポジトリ）に前倒しして実装する（詳細は`docs/ROADMAP.md`参照）。

- **v0（このリポジトリ）**: オンメモリのUnion-Find/DB機能に加え、上記4機能を含む
- **v1（別リポジトリ）**: v0からさらに先の大きな方針転換が必要になった場合に検討する（現時点で具体的な内容は未定）

## 構成

- `src/union_find.rs` — `UnionFind`構造体。`usize`インデックスのみを扱う純粋なUnion-Find実装（`parent`/`size`の2つの`Vec`）。文字列キーの存在は知らない
- `src/lib.rs` — `Ufdb`構造体。`HashMap<String, usize>`でキーとインデックスを橋渡しし、`UnionFind`に処理を委譲する
- `src/main.rs` — REPL本体。`clap`（derive API）でコマンドをパースし、標準入力を1行ずつ読んで実行する

ユーザー向けの概要・使い方・用語集は `README.md`、内部実装・計算量は `docs/architecture.md`、Union-Find自体の一般論は `docs/union-find.md`、実装計画・進捗は `docs/ROADMAP.md` を参照。

## コマンド

- `cargo build`
- `cargo test`
- `cargo run`（起動後はREPLで `INSERT` / `MERGE` / `SAME` / `GROUPS` / `EXIT` などを1行ずつ入力する）

## 作業の進め方

このプロジェクトの実装コードは基本的にユーザー自身が書く。Claude Codeの役割は:

- `README.md` / `docs/`配下のドキュメント（`ROADMAP.md`・`architecture.md`・`union-find.md`）/ `CLAUDE.md` の作成・更新
- 設計上の概念について説明する（コードを渡すのではなくSocratic的に）
- ユーザーが書いたコードのレビュー・指摘
- `cargo build` / `cargo test` によるビルド・動作確認

ユーザーから明示的に依頼されない限り、実装コードを直接書かない。
