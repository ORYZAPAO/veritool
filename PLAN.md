# veritool: SystemVerilog/Verilog 静的解析ツール

## Context

論理設計・検証エンジニアの ORYZA さんが、`/Users/zosan/veritool` をフレッシュなリポジトリとしてゼロから立ち上げる。SystemVerilog/Verilog の RTL ソースから「設計を素早く把握するための情報」を抽出する CLI ツールを作りたい。具体的には:

1. **モジュール端子一覧** (port: 方向/型/幅/名前)
2. **内部信号一覧** (logic/wire/reg/var の宣言、幅、配列次元)
3. **インスタンスモジュール単位の FF (Flip-Flop) 数の見積もり** (常時 RTL レビュー時にサイズ感を掴むのに使う)
4. **インスタンスモジュールの階層表示** (ツリー)
5. **トップモジュール検知** (どこからもインスタンス化されていないモジュール)

論理合成前の RTL レビューや、知らないコードベースに着手するときの「ざっと見」を支える、軽量で再現性のある CLI を目指す。

## 採用方針 (確定済み)

| 項目 | 採用案 |
|---|---|
| 実装言語 | **Rust (2024 edition)** |
| パーサー | **sv-parser** (IEEE 1800-2017 対応、Apache-2.0/MIT) |
| 配布 | 単一バイナリ (`cargo install veritool` を想定、`cross` 経由でクロスビルドも視野) |
| CLI 形式 | サブコマンド型 (`veritool <subcmd> [opts] FILES...`) |
| 出力形式 | `text` (デフォルト, comfy-table) / `json` (serde) / `markdown` / `csv` |
| 対象スコープ | 単一 `.v/.sv` ファイル + ファイルリスト (`-f filelist.f`) の両対応 |
| プリプロセッサ | フル対応 (`include / `define / `ifdef / +incdir+ / -D) — sv-parser の `parse_sv` をそのまま活用 |
| FF 見積もり精度 | デフォルトパラメータでの集計 + `-P NAME=VAL` で上書き可 |

## アーキテクチャ

### Cargo workspace 構成

```
veritool/
├── Cargo.toml                  # [workspace]
├── crates/
│   ├── veritool-core/          # ライブラリ: パース・解析・データモデル
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs       # filelist/incdir/define を sv-parser に渡す
│   │   │   ├── design.rs       # Design / Module / Port / Signal / Instance データ型
│   │   │   ├── visit.rs        # SyntaxTree を歩く共通ユーティリティ
│   │   │   ├── analyze/
│   │   │   │   ├── ports.rs    # ANSI/non-ANSI 両方の port 宣言抽出
│   │   │   │   ├── signals.rs  # logic/wire/reg/var 抽出
│   │   │   │   ├── ff.rs       # always_ff / always@(posedge|negedge) からFF抽出
│   │   │   │   ├── hierarchy.rs# module_instantiation を走査して階層構築
│   │   │   │   └── top.rs      # インスタンス化されていないモジュール検出
│   │   │   ├── params.rs       # 定数式評価 ($clog2 含む)
│   │   │   ├── width.rs        # ビット幅 + unpacked 次元の積算
│   │   │   └── report.rs       # 集計結果モデル (出力フォーマッタが食う)
│   │   └── tests/              # fixtures/*.sv を使ったスナップショットテスト (insta)
│   └── veritool-cli/           # バイナリ: clap でサブコマンド構築・出力フォーマット
│       ├── src/
│       │   ├── main.rs
│       │   ├── args.rs         # clap derive
│       │   └── format/
│       │       ├── text.rs     # comfy-table
│       │       ├── json.rs     # serde_json
│       │       ├── markdown.rs # 表 + mermaid (階層用)
│       │       └── csv.rs
└── tests/fixtures/             # サンプル .sv (counter, fifo, riscv-mini 抜粋など)
```

ライブラリ (`veritool-core`) と CLI (`veritool-cli`) を分けることで、後から VS Code 拡張や LSP・他言語バインディングに転用可能。

### データモデル (veritool-core::design)

```rust
pub struct Design {
    pub modules: IndexMap<String, Module>,  // 定義順を保つ
    pub files:   Vec<PathBuf>,
}

pub struct Module {
    pub name: String,
    pub file: PathBuf,
    pub span: SourceRange,                  // (start_line, end_line)
    pub params: Vec<ParamDecl>,             // パラメータと既定値
    pub ports:  Vec<Port>,
    pub signals: Vec<Signal>,
    pub instances: Vec<Instance>,           // child インスタンス
    pub ff_decls: Vec<FfDecl>,              // 1モジュール内のFF宣言 (再集計用に保持)
}

pub struct Port {
    pub name: String, pub direction: Direction,        // input/output/inout
    pub net_kind: NetKind,                              // wire/logic/reg/...
    pub data_type: DataType,                            // logic/bit/byte/.../signed?
    pub packed_width: Option<Range>,                    // [MSB:LSB]
    pub unpacked_dims: Vec<Range>,                      // [a:b][c:d]...
}

pub struct Signal { /* Port と同形 + net/var 区別 */ }

pub struct Instance {
    pub inst_name: String,
    pub module_ref: String,                 // 参照モジュール名
    pub param_overrides: Vec<(String, Expr)>,
}

pub struct FfDecl {
    pub signal_name: String,
    pub packed_width: Option<Range>,
    pub unpacked_dims: Vec<Range>,
    pub clock_edge: ClockEdge,              // posedge/negedge
    pub reset_kind: ResetKind,              // sync/async/none
}
```

### CLI 仕様 (veritool-cli)

```
veritool <SUBCOMMAND> [OPTIONS] [FILES...]

共通オプション:
  -f, --filelist <FILE>     -f 形式のファイルリスト (複数指定可)
  -I, --incdir <DIR>        インクルードパス (+incdir+ 相当, 複数可)
  -D <NAME[=VAL]>           マクロ定義 (複数可)
  -P <NAME=VAL>             パラメータ上書き (ff/signals 等で利用, 複数可)
  --format <FMT>            text(default) | json | markdown | csv
  --top <MODULE>            起点モジュール (省略時は自動検出)
  -m, --module <NAME>       対象モジュール (ports/signals 用)
  -v, --verbose             進捗ログを stderr に出力

サブコマンド:
  ports     モジュールの端子一覧
  signals   モジュール内部信号一覧
  ff        FF 数の見積もり (-m 指定なら当該モジュール単体、無指定なら top から階層合算)
  hier      モジュールインスタンス階層をツリー表示
  top       トップモジュール候補を一覧
  report    上記をまとめて出力 (--format json/markdown 推奨)
```

サブコマンドの例:

```bash
# 単一ファイルの top 検出
veritool top alu.sv

# プロジェクト全体の階層
veritool hier -f rtl/files.f -I rtl/include --top soc_top

# FF 見積もりを Markdown レポートに
veritool ff -f rtl/files.f --top soc_top -P CFG_WIDTH=64 --format markdown > ff_report.md
```

### FF カウントアルゴリズム

1. SyntaxTree から `AlwaysConstruct` を走査し、`always_ff` または `always @(posedge|negedge ...)` のみ抽出 (`always @*` / `always_comb` / `always_latch` は除外)。
2. ブロック内の `NonblockingAssignment` の LHS を取り、対象信号名を集める。同じ信号が複数 always で代入されても 1 度だけ数える。
3. 当該信号の宣言から `packed_width` と `unpacked_dims` を取り出し、`width(packed) * Π width(unpacked)` を FF 数とする。
4. パラメータ依存の幅は `params` モジュールで定数式評価 (整数演算 + `<<` `>>` `&` `|` `^` `~` `$clog2`)。評価不可は警告 + 当該信号スキップ。
5. モジュール単位の合計 = 内部 FF + Σ (子インスタンスの再帰合計 × インスタンス展開数)。
6. `-P` 指定はトップに適用し、子インスタンスのパラメータバインドは parent 評価値を伝播 (1 階層分で十分。深い伝播は MVP 後)。

### top モジュール検出

- 全モジュールを集合 `M` とする。
- 各モジュールの `instances[].module_ref` の和集合を `Referenced` とする。
- `M \ Referenced` を top 候補として返す (複数あり得る; primitives と外部モジュール参照は注意)。

### 階層表示

- 起点 (`--top` 指定 or 自動検出 top) から DFS で `Instance` を辿りツリーを構築。
- `text`: ASCII tree (`├─ ` `└─ ` `│  `)。
- `markdown`: 入れ子箇条書き + 任意で `mermaid graph TD;` を併記。
- `json`: 入れ子オブジェクト。
- 循環参照は検出して warning 表示後にそのエッジで打ち切り。

## 既存ユーティリティの再利用

ゼロからの新規リポジトリのため社内既存コードはなし。Crates.io の以下を採用:

| 用途 | クレート |
|---|---|
| パース | `sv-parser` |
| CLI | `clap` (derive) |
| ロギング | `tracing` + `tracing-subscriber` |
| 表出力 | `comfy-table` |
| 順序保持 Map | `indexmap` |
| シリアライズ | `serde`, `serde_json` |
| CSV | `csv` |
| エラー | `thiserror` + `anyhow` |
| スナップショットテスト | `insta` |
| Glob | `globset` |

## 実装フェーズ

| Phase | 中身 | 検証 |
|---|---|---|
| **P1: 基盤** | workspace 雛形、loader.rs、design 抽出 (modules + ports + signals) | `veritool ports alu.sv`, `veritool signals alu.sv` が動く |
| **P2: 階層 & top** | hierarchy.rs / top.rs | 小さな multi-module fixture で階層が出る・top 検出が当たる |
| **P3: FF 集計** | always_ff/always 抽出、デフォルトパラメータでの集計 | counter/fifo fixture で手計算と一致 |
| **P4: パラメータ評価** | params.rs (定数式・$clog2)、`-P` 上書き | パラメタライズドモジュールで再現性確認 |
| **P5: プリプロセッサ・ファイルリスト** | `-f` / `+incdir+` / `-D` 統合 | 大きめオープンソース RTL (例: picorv32) でクラッシュなく完走 |
| **P6: 出力フォーマット** | JSON / Markdown / CSV、`report` サブコマンド | 各フォーマットの snapshot test |

各フェーズで insta スナップショットを更新しつつ進める。

## 修正対象ファイル (新規作成)

すべて新規。主要パス:

- `Cargo.toml` (workspace)
- `crates/veritool-core/Cargo.toml`
- `crates/veritool-core/src/lib.rs`
- `crates/veritool-core/src/loader.rs`
- `crates/veritool-core/src/design.rs`
- `crates/veritool-core/src/visit.rs`
- `crates/veritool-core/src/analyze/{mod.rs,ports.rs,signals.rs,ff.rs,hierarchy.rs,top.rs}`
- `crates/veritool-core/src/params.rs`
- `crates/veritool-core/src/width.rs`
- `crates/veritool-core/src/report.rs`
- `crates/veritool-cli/Cargo.toml`
- `crates/veritool-cli/src/{main.rs,args.rs}`
- `crates/veritool-cli/src/format/{mod.rs,text.rs,json.rs,markdown.rs,csv.rs}`
- `tests/fixtures/*.sv` (counter, fifo_sync, top_with_subs, paramized など)
- `README.md` (使い方サンプル)

## 検証 (End-to-End)

1. **ビルド**: `cargo build --workspace` がエラーなく通る。
2. **単体テスト**: `cargo test --workspace`。各 analyze モジュールに最低 2 ケース。
3. **スナップショット**: `cargo insta test --review` で固定 fixture に対する出力差分確認。
4. **手動 E2E** (tests/fixtures を入力):
   - `veritool ports tests/fixtures/counter.sv` → counter の `clk/rst/en/q[7:0]` が表で出ること
   - `veritool top tests/fixtures/top_with_subs.sv` → top1 のみが返ること
   - `veritool hier -f tests/fixtures/top_with_subs.f --top top1 --format markdown` → mermaid 含む階層が出ること
   - `veritool ff -f tests/fixtures/fifo_sync.f --top fifo_top -P DEPTH=16 -P WIDTH=32` → 手計算 `(16*32 + 制御 FF)` と一致
5. **大規模 RTL 動作確認** (Phase 5 以降): picorv32 もしくは ibex を clone し、`veritool ff -f filelist --top <top>` がクラッシュせず妥当な値を返すこと。
6. **クロスプラットフォーム**: ArchLinux (主環境) と macOS で `cargo test` を通す。

## 非対象 (Out of Scope)

- ラッチ (`always_latch`) の集計
- generate ブロックの完全展開 (条件式評価のみ対応、for-generate のループ展開は Phase 6 以降)
- DPI / VPI / UDP
- インタフェース・モジュールポート (modport) — Phase 6 以降
- フォーマット支援 (整形/lint) — 別ツール領域
