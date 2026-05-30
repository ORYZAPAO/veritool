# veritool 実装日記

## 2026-05-25 (Sat) - 初期セットアップと基盤構築

### 実施内容

1. **プロジェクト構造の作成**
   - Cargo workspace structure を作成
   - `veritool-core` (ライブラリ) と `veritool-cli` (CLI) の分割
   - `crates/veritool-core/src/` 直下に設計モジュール群
   - `crates/veritool-cli/src/` に CLI 実装

2. **データモデルの定義** (`crates/veritool-core/src/design.rs`)
   - `Design` struct: モジュール集合とファイルリスト
   - `Module` struct: 名前、ファイル、パラメータ、端子、信号、インスタンス、FF宣言
   - `Port` struct: 名前、方向（input/output/inout）、ネット種別、データ型、幅情報
   - `Signal` struct: 信号名、ネット種別、データ型、幅情報
   - `Instance` struct: インスタンス名、参照モジュール、パラメータ上書き
   - `FfDecl` struct: FF宣言情報（信号名、幅、クロックエッジ、リセット種別）

3. **パーサーの設定**
   - `sv-parser = "0.13"` を採用（Cargo.toml 修正）
   - 初期は `sv-parser = "^5"` を記述したが、crates.io には存在しないため修正

### 発生した問題

#### sv-parser 0.13 API の大幅な変更
- API が大幅に変更され、既存のコードがコンパイルエラー
- `SourceRange`, `SvFile`, `ast` モジュール, `Visit`, `Visitor` などの API が変更
- モジュール宣言構造が大きく変更:
  - `ModuleDeclarationAnsi` と `ModuleDeclarationNonansi` に分かれる
  - `header` field ではなく、`nodes` を使用
  - モジュール名は `module.nodes.name.to_string()` で取得

#### Define 構造の違い
- DefineText の new() メソッドの引数が異なる
- `Define::new(name.clone(), v.clone())` ではなく、第三引数が必要

#### API の型エラー
- `ModuleDeclarationAnsi` と `ModuleDeclarationNonansi` で関数の引数が異なる
- 同一の関数で両方を処理できないため、別々の実装が必要

### 解決策

1. **sv-parser API を正確に理解**
   - GitHub の example code を参考に
   - `SyntaxTree` は `IntoIterator` を実装している
   - モジュール名: `module.nodes.name.to_string()`
   - パラメータとポートは `module.header.*` でアクセス

2. **一時的なモック実装**
   - API に合わせてコードを修正
   - 基本的には空のベクタを返す実装で先行実装
   - 今後の実装フェーズで具体的な実装を追加

### 現在のステータス

- ✅ Workspace structure 完了
- ✅ Data models 完了
- ✅ Loader.rs: ファイルリスト・インクルード・定義処理（空実装）
- ✅ Visit.rs: モジュール走査（空実装）
- ✅ Ports.rs: 端子抽出（空実装）
- ✅ Signals.rs: 信号抽出（空実装）
- ✅ FF.rs: FF抽出（空実装）
- ✅ Hierarchy.rs: 階層構築（空実装）
- ✅ Top.rs: トップモジュール検出（空実装）
- ✅ Params.rs: 定数式評価（空実装）
- ✅ Width.rs: ビット幅計算（空実装）
- ✅ Report.rs: レポートモデル（空実装）
- ✅ CLI: args.rs & main.rs（空実装）
- ✅ Formatters: text.rs & format/mod.rs（空実装）

### 次のアクション

1. **実際のモジュール情報抽出**
   - sv-parser の API を使用して、実際のポート情報を抽出
   - モジュールの items から信号を抽出
   - モジュールインスタンスを抽出

2. **テストの作成**
   - フィクスチャSVファイルを作成
   - 単体テストとスナップショットテスト

3. **機能実装**
   - 端子抽出の実装（ANSI/non-ANSI 両方対応）
   - 信号抽出の実装
   - インスタンス抽出の実装

4. **FFカウントの実装**
   - always_ff / always block の解析
   - 非ブロッキング代入のLHSを抽出
   - パラメータ依存の幅評価

5. **階層表示**
   - DFSによる階層ツリー構築
   - Text/Json/Markdown形式で出力

### 参考資料

- sv-parser 0.13.5 API
- Rust 2024 edition
- Cargo workspace
- Clap 4 CLI framework
- IndexMap for ordered maps
- Thiserror/Anyhow for error handling

## 進捗メモ

- Phase 1 (基盤構築): 一部完了 - API に合わせた構造はできたが、実装は空
- sv-parser API の学習が必要
- 実際のモジュール抽出コードの実装が必要

---

## 2026-05-25 (Sat) 続編 - 実装本体の完成 (Phase 1-2)

### 実施内容

#### sv-parser 0.13 API の全面解析

- `SyntaxTree` はイテレータとして `RefNode` を DFS で返す
- `unwrap_node!(node, TypeName)` マクロでサブツリーから最初の一致を探索
- `tree.get_str(locate)` でソーステキストを取得 (`&Locate: Into<RefNodes>`)
- イベントベースイテレータ `.event()` で Enter/Leave を追跡 → モジュールスコープ管理に活用

#### visit.rs の完全書き直し (イベントベース)

- `NodeEvent::Enter/Leave(RefNode::ModuleDeclarationAnsi/Nonansi)` でモジュールスタック管理
- `module_stack.len() == 1` ガードにより、ネストモジュールのアイテムを除外
- 以下の情報を一回のツリー走査で抽出:
  - ANSIポート (`AnsiPortDeclarationNet/Variable`)
  - 非ANSIポート (`InputDeclarationNet/Variable`, `OutputDeclarationNet/Variable`, `InoutDeclaration`)
  - 信号宣言 (`DataDeclarationVariable`, `NetDeclarationNetType`)
  - モジュールインスタンス (`ModuleInstantiation` → `HierarchicalInstance`)
  - FF宣言 (`AlwaysConstruct` + `NonblockingAssignment` でLHS信号名を収集)
  - パラメータ宣言 (`ParameterDeclaration`)

#### loader.rs の修正

- `Define::new(name, args, text)` + `DefineText::new(text, None)` の正しいAPI使用
- 複数ファイル間で `defines` を `extend()` で伝播

#### design.rs

- `DataType::Reg` variant を追加

#### analyze/ の整理

- `ports.rs`, `signals.rs` は visit.rs に処理移行のためコメントのみに
- `hierarchy.rs` は `build_hierarchy()` 関数で Design からツリー構築 (format/text.rs 内にインライン実装)
- `top.rs` はargs.rs の `detect_top_module()` に移行

### CLI の実装

**args.rs**: `Cli`, `Commands`, `OutputFormat`, `detect_top_module`, `parse_defines` を clap derive で実装

**format/text.rs**: 全出力フォーマット (text/json/markdown/csv) を統合
- `print_ports`, `print_signals`, `print_ff_module`, `print_ff_hierarchy`
- `print_hier`, `print_top`, `print_report`
- 階層表示: ASCII tree (root + `print_hier_instances` でインスタンスのみ再帰)
- FF 数: signal lookup → port fallback → 1 (未知)

**veritool-cli/Cargo.toml**: `anyhow`, `indexmap` を追加

### テスト fixtures と統合テスト

fixtures:
- `tests/fixtures/counter.sv`: パラメータ付き8bitカウンタ
- `tests/fixtures/fifo_sync.sv`: パラメータ付き同期FIFO
- `tests/fixtures/top_with_subs.sv`: 階層付きtop (alu + reg_file)

統合テスト (`crates/veritool-core/tests/integration_test.rs`):
- test_counter_ports: ポート名・方向・幅の確認
- test_counter_ff_decls: FF宣言 (q) の確認
- test_top_module_detection: top1 がtop候補
- test_hierarchy_instances: top1が2インスタンス
- test_fifo_signals: mem/wr_ptr/rd_ptr のシグナル抽出
- test_fifo_ff_decls: wr_ptr/rd_ptr がFF

全6テスト: ✅ PASS

### 動作確認 (E2E)

```
veritool ports tests/fixtures/counter.sv         # ポート表
veritool top tests/fixtures/top_with_subs.sv     # → top1
veritool hier tests/fixtures/top_with_subs.sv    # ASCII tree
veritool --format markdown hier ...              # markdown
veritool --format json ff tests/fixtures/fifo_sync.sv  # FF JSON
veritool signals tests/fixtures/fifo_sync.sv    # 信号一覧
veritool --format markdown report ... --top top1 # 総合レポート
```

### 現在のステータス

- ✅ Phase 1 (基盤): 完了
- ✅ Phase 2 (階層 & top): 完了
- 🔵 Phase 3 (FF集計): 基本実装済み (パラメータ依存幅は1bitとして扱う)
- 🔵 Phase 4 (パラメータ評価): 未実装 (params.rs スタブのまま)
- 🔵 Phase 5 (プリプロセッサ・ファイルリスト): filelist基本対応済み
- 🔵 Phase 6 (出力フォーマット): text/json/markdown/csv 基本実装済み

### 次のアクション (Phase 3-4 完了後)

1. **insta** スナップショットテストの追加
2. ~~**picorv32** などの大規模RTLでの動作確認 (Phase 5)~~ → 完了 (下記)
3. **generate ブロック対応** の基礎 (Phase 6)

---

## 2026-05-25 (Sat) - picorv32 動作確認 (Phase 5) と表示改善

### picorv32 動作確認結果

テスト対象: `/home/zosan/gowin/TangNano-20K-example/picorv32/src/picorv32.v` (3043行)

| テスト | 結果 |
|---|---|
| クラッシュなし | ✅ (3秒以内) |
| top 検出 | ✅ picorv32_regs / picorv32_axi / picorv32_wb |
| 階層表示 | ✅ picorv32_axi → picorv32 → [fast_mul, mul, div] |
| ポート抽出 | ✅ 27ポート (output reg [31:0] 等) |
| 信号抽出 | ✅ 394信号 (reg/wire) |
| FF カウント (picorv32_axi) | ✅ **2144 FF** |

#### 3ファイル同時解析 (system SoC)

`top.v` + `picorv32.v` + `simpleuart.v` のマルチファイル解析:

| モジュール | 自モジュール FF | 階層合算 FF |
|---|---|---|
| system | 131138 | **133410** |
| picorv32 | 1368 | 2140 |
| picorv32_pcpi_fast_mul | 266 | 266 |
| picorv32_pcpi_mul | 305 | 305 |
| picorv32_pcpi_div | 201 | 201 |
| simpleuart | 132 | 132 |

`system.memory[0:MEM_SIZE-1][31:0]` = 4096 × 32 = 131072 FF が正確にカウント ✅

### 発見した問題と対処

| 問題 | 対処 |
|---|---|
| 信号の `logic reg` 冗長表示 | `format_type()` 修正: Verilog `reg` は単独 `reg` と表示 |
| signals 表に unpacked 次元が未表示 | Unpacked 列を追加 (`[0:MEM_SIZE-1]` が見える) |

### 既知の制限 (Phase 6 以降)

- `generate if/else` 非評価: picorv32 の `pcpi_mul` が fast_mul と通常 mul の両方表示される
- → `ENABLE_FAST_MUL=1` 等で実際に有効なモジュールのみカウントする機能は未実装
- generate for ループも未展開

---

## 2026-05-25 (Sat) 続々編 - params.rs 実装と正確な FF カウント

### 実施内容

#### params.rs - 完全実装

再帰降下パーサで SV 定数式を評価:

- **演算子**: `+`, `-`, `*`, `/`, `%`, `**`, `<<`, `>>`, `&`, `|`, `^`, `~`, 単項 `-`, `~`
- **リテラル**: 整数 (`123`), SV ベース付き (`8'd255`, `8'hFF`, `8'b1010`)
- **システム関数**: `$clog2(n)` — ceiling(log2(n)), 最小 1
- **識別子**: パラメータ名 → HashMap から値を解決
- **括弧**: `(expr)`
- 評価不能 (未知識別子等) → `None` を返す

#### ParamEnv 構造体

```rust
ParamEnv::from_module(module)       // パラメータデフォルト値の逐次評価
    .with_overrides(&[("WIDTH", 16)]) // -P 上書き
```

localparam も `parameter` と同じく `ParamEnv` に追加 (sequential 評価なので `$clog2(DEPTH)` が解決可能)

#### visit.rs の改修

- `LocalParameterDeclaration` も抽出するよう追加
- `extract_param_assignments<T>` を共通化 (parameter/localparam 両対応)
- `extract_param_decl` のバグ修正: `tree.get_str(pa)` (全体テキスト `"WIDTH = 8"`) → `tree.get_str(cpe)` (RHS のみ `"8"`)

#### width.rs の拡張

- `calculate_width_with_params(signal, env)` — ParamEnv を使った正確な幅計算
- `calculate_width_with_map(signal, params)` — HashMap 直接指定版
- 旧 `calculate_width` / `calculate_ff_width` は後方互換のため残存

#### CLI の改修

- `print_ff_module(design, module, format, overrides: &[(String, i64)])`
- `print_ff_hierarchy(design, top, format, top_overrides: &[(String, i64)])`
- `parse_param_overrides(args)` — `-P NAME=VAL` 文字列を解析
- 階層 FF カウント: 親の ParamEnv でインスタンスのパラメータバインドを評価し子に伝播

### 検証結果

| ケース | 期待値 | 実際 |
|---|---|---|
| `counter` (WIDTH=8) | q=8bit → **8 FF** | 8 ✅ |
| `counter -P WIDTH=16` | q=16bit → **16 FF** | 16 ✅ |
| `fifo` (WIDTH=8, DEPTH=16) | wr_ptr=[4:0]=5, rd_ptr=5, mem=8×16=128 → **138 FF** | 138 ✅ |
| `fifo -P WIDTH=32 -P DEPTH=16` | wr_ptr=5, rd_ptr=5, mem=32×16=512 → **522 FF** | 522 ✅ |

### テスト

- `params::tests`: 9件 (リテラル/算術/シフト/ビット/パラメータ/$clog2/ネスト/未解決)
- `integration_test`: 8件 (新規追加: `test_counter_param_evaluation`, `test_fifo_localparam_evaluation`)

### 現在のステータス

- ✅ Phase 1 (基盤): 完了
- ✅ Phase 2 (階層 & top): 完了
- ✅ Phase 3 (FF集計): 完了 — `always_ff` + `always @(posedge)` の非ブロッキング代入から集計
- ✅ Phase 4 (パラメータ評価): 完了 — `$clog2`, 四則演算, localparam 連鎖, `-P` 上書き, 1層インスタンスバインディング
- 🔵 Phase 5 (プリプロセッサ・ファイルリスト): filelist基本対応済み
- ✅ Phase 6 (出力フォーマット): text/json/markdown/csv 実装済み

---

## 2026-05-25 (Sat) まとめ — セッション全体の成果

### 何を作ったか

PLAN.md に定義された SystemVerilog/Verilog 静的解析 CLI「veritool」をゼロから実装した。
一日のセッションで Phase 1 〜 5 を完走し、picorv32 実機動作確認まで到達した。

### 最終的なコード構成

```
veritool/
├── Cargo.toml                          # workspace (resolver = "2")
├── crates/
│   ├── veritool-core/                  # ライブラリ crate
│   │   └── src/
│   │       ├── design.rs               # データモデル (Design/Module/Port/Signal/...)
│   │       ├── loader.rs               # ファイルリスト・sv-parser 呼び出し
│   │       ├── visit.rs                # イベントベース SyntaxTree ウォーカー (中核)
│   │       ├── params.rs               # SV 定数式エバリュエータ + ParamEnv
│   │       ├── width.rs                # ビット幅計算 (packed × unpacked)
│   │       ├── report.rs               # Display impl (NetKind/DataType)
│   │       └── analyze/                # 後処理ユーティリティ (現在はスタブ)
│   └── veritool-cli/                   # バイナリ crate
│       └── src/
│           ├── main.rs                 # エントリポイント・サブコマンド振り分け
│           ├── args.rs                 # clap derive (Cli/Commands/OutputFormat/-P)
│           └── format/
│               └── text.rs             # 全出力フォーマット統合
└── tests/
    └── fixtures/
        ├── counter.sv                  # 8bit カウンタ (パラメータ付き)
        ├── fifo_sync.sv                # 同期 FIFO (WIDTH/DEPTH/localparam ADDR_W)
        └── top_with_subs.sv            # 階層テスト (top1 → alu + reg_file)
```

### 実装した機能

#### CLI サブコマンド

```bash
veritool ports  [-m MODULE] [FILES...]   # ポート一覧 (方向/型/幅/名前)
veritool signals [-m MODULE] [FILES...]  # 内部信号一覧 (型/packed幅/unpacked幅/名前)
veritool ff     [--top TOP] [FILES...]   # FF 数見積もり (階層合算)
veritool hier   [--top TOP] [FILES...]   # モジュールインスタンス階層ツリー
veritool top    [FILES...]               # トップモジュール候補一覧
veritool report [--top TOP] [FILES...]   # 上記をまとめて出力

# 共通オプション
-f filelist.f          # ファイルリスト
-I incdir/             # インクルードパス
-D NAME=VAL            # プリプロセッサマクロ定義
-P NAME=VAL            # パラメータ上書き (複数指定可)
--format text|json|markdown|csv   # 出力形式
--top MODULE           # 起点モジュール
-m MODULE              # 対象モジュール (ports/signals 用)
```

#### 情報抽出 (visit.rs — イベントベース走査)

| 抽出対象 | sv-parser RefNode |
|---|---|
| ANSI ポート | `AnsiPortDeclarationNet/Variable` |
| 非ANSIポート | `InputDeclarationNet/Variable`, `OutputDeclarationNet/Variable`, `InoutDeclaration` |
| 信号宣言 | `DataDeclarationVariable`, `NetDeclarationNetType` |
| モジュールインスタンス | `ModuleInstantiation` → `HierarchicalInstance` |
| FF 宣言 | `AlwaysConstruct`(always_ff/posedge検出) + `NonblockingAssignment` LHS |
| パラメータ | `ParameterDeclaration`, `LocalParameterDeclaration` |

#### パラメータ評価 (params.rs — 再帰降下パーサ)

- 演算子: `+` `-` `*` `/` `%` `**` `<<` `>>` `&` `|` `^` `~` 単項
- リテラル: 整数 / `8'd255` / `8'hFF` / `8'b1010`
- システム関数: `$clog2(n)` (ceiling log2、最小 1)
- `localparam ADDR_W = $clog2(DEPTH)` の連鎖評価
- `-P NAME=VAL` CLI フラグで上書き
- 親インスタンスのパラメータバインドを子に 1 層伝播

#### 幅計算 (width.rs)

`calculate_width_with_params(signal, env)` — packed × unpacked 積算

```
logic [WIDTH-1:0] mem [0:DEPTH-1]
  packed  = WIDTH bits
  unpacked = DEPTH elements
  total   = WIDTH × DEPTH bits
```

#### 出力フォーマット

| フォーマット | 内容 |
|---|---|
| text | comfy-table による整形テーブル / ASCII ツリー |
| json | serde_json ネスト構造 |
| markdown | GitHub 表 / 箇条書き階層 |
| csv | ヘッダ付き CSV |

### テスト結果 (最終)

```
cargo test --workspace
  params::tests        9 件  ✅  (リテラル/算術/シフト/ビット演算/$clog2/etc.)
  debug_params         2 件  ✅  (counter/fifo パラメータ抽出デバッグ)
  integration_test     8 件  ✅  (ポート/信号/FF/階層/パラメータ評価)
  ──────────────────────────────
  合計                19 件  全パス
```

### 動作確認 — picorv32 (Phase 5)

ファイル: `picorv32.v` (3043 行)、マルチファイル解析も確認

```
veritool top picorv32.v
  → picorv32_regs / picorv32_axi / picorv32_wb

veritool ports -m picorv32 picorv32.v
  → 27 ポート (output reg [31:0] mem_addr 等)

veritool ff --top picorv32_axi picorv32.v
  picorv32_axi       own=0     total=2144
  picorv32_axi_adapter own=4   total=4
  picorv32           own=1368  total=2140
  picorv32_pcpi_fast_mul 266   266
  picorv32_pcpi_mul      305   305
  picorv32_pcpi_div      201   201

veritool ff --top system top.v picorv32.v simpleuart.v
  system             own=131138  total=133410  ← memory[0:4095][31:0]=128KB
  picorv32           own=1368    total=2140
  simpleuart         own=132     total=132
```

クラッシュなし、全コマンド 3 秒以内完了。

### FF カウント検証

| ケース | 期待 | 実測 |
|---|---|---|
| `counter` (WIDTH=8) | q=[7:0] → 8 FF | **8** ✅ |
| `counter -P WIDTH=16` | q=[15:0] → 16 FF | **16** ✅ |
| `fifo` (WIDTH=8, DEPTH=16) | wr_ptr=[4:0]+rd_ptr=[4:0]+mem=128 → 138 FF | **138** ✅ |
| `fifo -P WIDTH=32 -P DEPTH=16` | wr_ptr=5+rd_ptr=5+mem=512 → 522 FF | **522** ✅ |
| `picorv32_axi` (デフォルト) | CPU + AXI adapter | **2144** |
| `system` SoC | CPU + UART + 128KB RAM | **133410** |

### 既知の制限 (次フェーズ以降)

| 制限 | フェーズ |
|---|---|
| `generate if/else` 条件未評価 → 両分岐のモジュールが表示される | Phase 6 |
| `generate for` ループ展開なし | Phase 6 |
| 深い階層へのパラメータ伝播 (現在 1 層のみ) | Phase 4 拡張 |
| `always @*` / `always_comb` (組合せ) の誤検出なし ✅ | — |
| ラッチ (`always_latch`) 集計なし | Out of scope |
| インターフェース・モジュールポート (modport) | Phase 6 |

---

## 2026-05-29 (Thu) — Phase 6: スナップショットテスト + generate if/else 評価

### 作業概要

Phase 6 の残項目をすべて実装した。

1. **`insta` スナップショットテスト** — CLI / コア両方にスナップショットを追加
2. **`generate if/else` 条件評価** — デフォルトパラメータによる分岐選択を実装
3. **`design.rs` Serialize 対応** — コアスナップショットのために全型に `#[derive(Serialize)]` を追加

### insta スナップショットテスト

`insta` を workspace dev-dependency に追加し (`features = ["json", "redactions"]`)、以下のスナップショットを整備した。

#### veritool-core スナップショット (3件)

| テスト名 | 内容 |
|---|---|
| `snap_counter_module` | counter モジュール全体の JSON (file/span は redact) |
| `snap_fifo_module` | fifo_sync モジュール全体の JSON |
| `snap_top_with_subs_instances` | top1 のインスタンスリスト |

#### veritool-cli スナップショット (14件)

`CARGO_BIN_EXE_veritool` を使った process::Command テスト。

| カテゴリ | テスト | fixture |
|---|---|---|
| ports × 4形式 | text / json / markdown / csv | counter.sv |
| signals × 2 | text / json | fifo_sync.sv |
| ff × 3 | module-json / hierarchy-text / hierarchy-json | counter.sv / fifo_sync.sv |
| hier × 3 | text / json / markdown | top_with_subs.sv |
| top × 2 | text / json | top_with_subs.sv |

合計テスト数: **37件** (旧 19 + 新 18)

### generate if/else 条件評価

`visit.rs` にフェーズ分けされたスキップ機構を実装した。

#### 実装方針

```
for event in tree.into_iter().event() {
    // Phase 1: skip モード — skip_depth > 0 なら Enter/Leave のたびに深さを増減して continue
    if skip_depth > 0 { match event { Enter → ++depth, Leave → --depth }; continue; }

    // Phase 2: 通常処理
    match event {
        Enter(IfGenerateConstruct) → 条件評価 → if_gen_stack に push
        Leave(IfGenerateConstruct) → pop
        Enter(LoopGenerateConstruct/CaseGenerateConstruct) → Other 番兵を push
        Leave(Loop/Case) → pop
        Enter(GenerateBlock) → if_gen_stack.last() が IfGen なら branch_idx++
                               → false 分岐なら skip_depth = 1
        ...
    }
}
```

`IfGenCtx` enum:
- `IfGen { cond: Option<bool>, block_idx: usize }` — if-generate の状態
- `Other` — for/case generate の番兵 (内部 GenerateBlock が branch_idx を変えないようにする)

#### 条件式の取得

```rust
fn eval_if_generate_cond(tree, node, design, module_stack) -> Option<bool> {
    // node.nodes.1 = Paren<ConstantExpression>、.nodes.1 = ConstantExpression
    let cond_text = tree.get_str(&node.nodes.1.nodes.1)?.trim();
    let env = ParamEnv::from_module(module);
    let value = evaluate_expr(&cond_text, env.as_map())?;
    Some(value != 0)
}
```

#### 動作検証

```
# gen_if.sv: parameter FAST=1, WIDE=0 (デフォルト)
veritool hier gen_if.sv
└─ gen_if
   ├─ u_fast (fast_core)    ← FAST=1 → fast 分岐を選択 ✅
   └─ u_narrow (narrow_bus) ← WIDE=0 → narrow 分岐を選択 ✅
```

#### picorv32 への影響

| 指標 | 修正前 (誤) | 修正後 (正) |
|---|---|---|
| picorv32 own FF | 1368 | 1269 (generate false 分岐の FF を除外) |
| picorv32 total FF | 2140 | 1269 (pcpi_mul が除外) |
| system total FF | 133410 | 132539 |

修正前は `generate if (ENABLE_MUL)` と `generate if (ENABLE_FAST_MUL)` の両分岐の
モジュールが重複表示されていた。修正後はデフォルト値 (ENABLE_MUL=0) で評価し、
false 分岐 (pcpi_mul) をスキップする。

#### 既知の制限 (更新)

| 制限 | 状態 |
|---|---|
| `generate if/else` 条件評価 (デフォルトパラメータ) | ✅ **実装済み** |
| `generate if/else` — インスタンス overrides を反映 | ⚠️ パース時評価のため未対応 |
| `generate for` ループ展開 | Phase 7 以降 |
| `case` generate の条件評価 | Phase 7 以降 |
| インターフェース・modport | Out of scope |

> **制限の補足**: generate if の条件はパース時にモジュールのデフォルトパラメータで評価する。
> インスタンス側の `#(.ENABLE_MUL(1))` 上書きはパース時には不明なため、
> 階層表示は常にデフォルトパラメータに基づく分岐を示す。
> FF カウントにはインスタンス別パラメータ伝播が適用されるが、
> 階層インスタンスの有無はデフォルト評価のまま。

### テスト結果

```
cargo test --workspace
37 passed, 0 failed
```

### 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `Cargo.toml` (workspace) | insta features = ["json", "redactions"] |
| `crates/veritool-core/Cargo.toml` | dev-dependencies: insta |
| `crates/veritool-cli/Cargo.toml` | dev-dependencies: insta |
| `crates/veritool-core/src/design.rs` | 全型に `#[derive(Serialize)]` 追加 |
| `crates/veritool-core/src/visit.rs` | IfGenCtx + skip_depth + eval_if_generate_cond |
| `crates/veritool-core/tests/snapshot_tests.rs` | 新規 (3 スナップショット) |
| `crates/veritool-core/tests/integration_test.rs` | `test_generate_if_default_params` 追加 |
| `crates/veritool-cli/tests/snapshot_tests.rs` | 新規 (14 スナップショット) |
| `tests/fixtures/gen_if.sv` | generate if テスト fixture |
| `crates/veritool-cli/tests/snapshots/*.snap` | 17 スナップショットファイル |

---

## 2026-05-31 (Sun) — Phase 7: generate case / generate for 実装

### 作業概要

Phase 6 で未対応だった 2 つの generate 構文を実装した。

1. **`generate case` 条件評価** — case 式とアイテム値をパラメータ式として評価し、マッチするブロックのみを処理
2. **`generate for` ループ展開** — ループ境界を評価してイテレーション数を算出し、モジュールインスタンスを N 倍に複製

### params.rs — 比較演算子の追加

generate for の条件式 (`i < N`, `i <= N-1` 等) を評価するため、トークナイザとパーサに比較演算子を追加した。

**追加したトークン:** `Lt(<)`, `Gt(>)`, `Le(<=)`, `Ge(>=)`, `Eq(==)`, `Ne(!=)`

**パーサ:** `parse_compare` 関数を最低優先度で追加（bitwise OR より下）。エントリポイント `evaluate_expr` が `parse_compare` を呼ぶよう変更。

比較演算子の優先度は SV 標準に準拠: `compare > bitwise_or > bitwise_xor > bitwise_and > shift > add > mul > power > unary > primary`

### visit.rs — case generate

```
IfGenCtx に 2 つのバリアントを追加:
  CaseGen { value: Option<i64>, matched: bool }  ← case 文のコンテキスト
  CaseItem { should_process: bool }               ← 各 case アイテムのコンテキスト
```

イベントシーケンス:
1. `Enter(CaseGenerateConstruct)` → case 式を評価して `CaseGen` を push
2. `Enter(CaseGenerateItemNondefault)` → case 値リストと評価値を比較して `CaseItem` を push
3. `Leave(CaseGenerateItemNondefault)` → マッチしていたら親の `matched = true` に更新
4. `Enter(CaseGenerateItemDefault)` → `matched == false` の場合のみ処理
5. `Enter(GenerateBlock)` → `CaseItem { should_process: false }` なら skip_depth = 1

case 値リストのアクセス: `CaseGenerateItemNondefault.nodes.0` は `List<Symbol, ConstantExpression>` 型。`List::contents()` で `&ConstantExpression` のイテレータを取得し、各値を評価して比較。

### visit.rs — generate for

```
LoopCtx { count: usize }  ← イテレーション数を保持
loop_stack: Vec<LoopCtx>  ← ネストループに対応
```

`eval_loop_generate` / `try_eval_loop`:
1. `GenvarInitialization` から変数名と初期値を取得
2. `GenvarExpression` から条件式テキストを取得
3. `GenvarIteration` から step delta を計算 (`++`/`--` → ±1, `+=`/`-=` → ±rhs)
4. `{module_params + genvar: i}` の HashMap でループをシミュレート

`ModuleInstantiation` ハンドラ:
```
let multiplier = loop_stack.iter().map(|c| c.count).product();
// 複数コピーを inst_name_0, inst_name_1, ... として追加
```

ループ境界が評価できない場合 (複雑な式、外部変数など) は `count=1` にフォールバックし、単一インスタンスとして処理する。

### 動作検証

```
veritool hier tests/fixtures/gen_case.sv
└─ gen_case
   └─ u_medium (medium_core)     ← MODE=1 → medium_core ✅

veritool hier tests/fixtures/gen_for.sv
└─ gen_for
   ├─ u_cell_0 (unit_cell)
   ├─ u_cell_1 (unit_cell)
   ├─ u_cell_2 (unit_cell)
   └─ u_cell_3 (unit_cell)       ← N=4 → 4 インスタンス ✅
```

### 既知の制限 (更新)

| 制限 | 状態 |
|---|---|
| `generate if/else` 条件評価 (デフォルトパラメータ) | ✅ 実装済み (Phase 6) |
| `generate case` 条件評価 (デフォルトパラメータ) | ✅ **実装済み** (Phase 7) |
| `generate for` ループ展開 (デフォルトパラメータ) | ✅ **実装済み** (Phase 7) |
| `generate case` — インスタンス overrides 反映 | ⚠️ パース時評価のため未対応 |
| `generate for` — genvar 値のパラメータ伝播 | ⚠️ 全コピー同一パラメータ |
| `generate for` — 乗算/除算ステップ (`*=`, `/=`) | ⚠️ 未対応 (稀なケース) |
| `case` generate の xCase (casex/casez) | ⚠️ 未対応 |
| インターフェース・modport | Out of scope |

### テスト結果

```
cargo test --workspace
40 passed, 0 failed
```

(新規追加: 比較演算子テスト +1, generate case テスト +1, generate for テスト +1)

### 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `crates/veritool-core/src/params.rs` | 比較演算子 (Lt/Gt/Le/Ge/Eq/Ne) + parse_compare 追加 |
| `crates/veritool-core/src/visit.rs` | CaseGen/CaseItem/LoopCtx + case/for generate ハンドラ |
| `crates/veritool-core/tests/integration_test.rs` | generate case + for テスト追加 |
| `tests/fixtures/gen_case.sv` | 新規: generate case テスト fixture |
| `tests/fixtures/gen_for.sv` | 新規: generate for テスト fixture |
