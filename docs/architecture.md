# 内部実装

`ufodb_v0` の内部実装。Union-Find自体の一般的な理論は [`union-find.md`](./union-find.md) を参照。

## 3層構造

- `UnionFind`（`src/union_find.rs`）— インデックス（`usize`）だけを扱う、Union-Findのコアアルゴリズム層。文字列キーの存在は一切知らない
- `Ufdb`（`src/lib.rs`）— `HashMap<String, usize>` で文字列キーとインデックスを橋渡しし、`UnionFind` に処理を委譲する層
- `Db`（`src/db.rs`）— `HashMap<String, Ufdb>` で名前付きの複数`Ufdb`インスタンスを束ね、「現在選択中のDB名」を管理する層。`USE`/`CREATEDB`はこの層の操作

### `UnionFind`

内部は2つの配列で構成される:

- `parent: Vec<usize>` — 各要素がどの要素を親として指しているかを表す。`parent[i] == i` のとき、`i` はそのグループの代表元（root）
- `size: Vec<usize>` — 各要素が root のときにだけ意味を持つ、そのグループに属する要素数

主な操作と計算量:

| メソッド | 役割 | 計算量 |
| --- | --- | --- |
| `add()` | 新しい要素を1つ追加し、自分自身だけのグループ（サイズ1）として登録する。追加された要素のインデックスを返す | O(1) |
| `find(x)` | `x` が属するグループの代表元（root）のインデックスを返す。辿る過程で経路圧縮を行う | ならしO(α(n)) |
| `unite(x, y)` | `x` と `y` のグループを1つにまとめる。union by size（要素数が少ない方を多い方へ繋ぐ）で木の高さを抑える。すでに同じグループなら何もせず `false` を返す | ならしO(α(n)) |
| `same(x, y)` | `x` と `y` が同じグループに属するかを返す | ならしO(α(n)) |
| `size(x)` | `x` が属するグループの要素数を返す | O(1)（rootのsizeを引くだけ） |

要素の追加はすべて `add()` による1個ずつの動的な追加で、生成時に要素数をあらかじめ指定する必要はない。

### `Ufdb`

```rust
pub struct Ufdb {
    keys: HashMap<String, usize>, // キー文字列 → UnionFind内部のインデックス
    uf: union_find::UnionFind,    // インデックスベースの本体
}
```

呼び出し側は `"apple"` のような任意の文字列キーだけを扱い、`Ufdb` がその都度 `HashMap` を介してインデックスに変換してから `UnionFind` に処理を委譲する。

| メソッド | 役割 | 計算量 |
| --- | --- | --- |
| `make_set(key)` | キーが未登録なら `UnionFind::add()` で新しいインデックスを確保し `HashMap` に登録する（新規追加なら `true`、既存なら何もせず `false`） | ならしO(1)（HashMapの挿入） |
| `unite(key_a, key_b)` | `key_a`・`key_b` それぞれに `make_set` を呼んでから（未登録キーも自動で登録される）、両者のインデックスで `UnionFind::unite` を呼ぶ。すでに同じグループなら `false` | ならしO(α(n)) |
| `same(key_a, key_b)` | `key_a`・`key_b` が両方登録済みの場合のみ `UnionFind::same` に委譲する。どちらか一方でも未登録なら登録はせず `false` を返す | ならしO(α(n)) |

### `Db`

```rust
pub struct Db {
    current_db: String,           // 現在選択中のDB名
    db: HashMap<String, Ufdb>,    // DB名 → Ufdbインスタンス
}
```

`Ufdb`が文字列キーとインデックスを橋渡しするのと同じように、`Db`はDB名と`Ufdb`インスタンスを橋渡しする。各DBは完全に独立した`Ufdb`インスタンス（独自の`keys`・独自の`UnionFind`）なので、DBをまたいだ`MERGE`/`SAME`などは成立しない。

不変条件: `current_db`が指す名前は常に`db`の中に実在するキーである。この条件は`new`/`create_db`/`use_db`の3メソッドだけが`current_db`を書き換えることで維持されており、`current`メソッドはこの条件を前提に`unwrap()`している。

| メソッド | 役割 |
| --- | --- |
| `new()` | デフォルトDB `ufdb` を1つ持った状態で初期化し、`current_db`もそれにする |
| `current()` | 選択中の`Ufdb`への可変参照を返す。`INSERT`などのコマンドはこれ経由で`Ufdb`のメソッドに委譲する |
| `create_db(name)` | `db`に`name`が無ければ`Ufdb::new()`を挿入。あれば何もしない。いずれの場合も`current_db`を`name`に切り替える（戻り値は新規作成なら`true`） |
| `use_db(name)` | `db`に`name`があれば`current_db`を切り替えて`true`。無ければ切り替えずに`false`（自動作成しない） |

## 検証済みの規模

union by size + 経路圧縮により操作はならしO(α(n))で、キー数に対してほぼ線形にしかメモリを消費しない。理論上はメモリが許す限り動作するはずだが、実際に動作確認したのは10万件（`N = 100,000`）規模まで。それ以上の規模は未検証（詳細は [`ROADMAP.md`](./ROADMAP.md) の「既知の制約」を参照）。

`FIND`（代表元キーを直接返すコマンド）は採用しない。代表元は `unite` の内部ロジック（union by sizeの同点判定など）でどのキーになるか決まる、ユーザーからは予測できない値なので、単体のコマンドとして見せる意味が薄いため。同じグループかどうかは `SAME`、全体像を見たい場合は `GROUPS` を使う。

各コマンド実行後には、実行にかかった時間をミリ秒（小数点以下6桁）で表示する（例: `elapsed: 0.001500ms`）。