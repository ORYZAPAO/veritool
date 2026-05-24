---
name: project-veritool-status
description: veritool プロジェクトの実装状況と次のアクション
metadata:
  type: project
---

Phase 1-5 完了 (2026-05-25)。`cargo test --workspace` 全 19 件パス。

**Why:** RTLレビュー・解析ツールをゼロから作成中

**How to apply:** 次の作業は generate 対応 (Phase 6) か insta スナップショットテスト追加

## 完了済み フェーズ

- **Phase 1-2**: 基盤、階層、top 検出
- **Phase 3-4**: FF 集計、パラメータ評価 ($clog2、四則演算、localparam)
- **Phase 5**: picorv32 で動作確認済み (3043行、3秒以内、クラッシュなし)

## picorv32 動作確認結果

ファイル: `/home/zosan/gowin/TangNano-20K-example/picorv32/src/picorv32.v`

- FF count (picorv32_axi): **2144 FF** (own=0, children=2144)
- FF count (system SoC): **133410 FF** (memory 128KB=131072 FF 含む)
- Ports: 27 ✅ / Signals: ~394 ✅ / Hierarchy: 正常 ✅

## 既知の制限

- generate if/else 非評価: picorv32 の pcpi_mul が fast_mul と通常 mul の両方表示
- generate for ループ展開は未実装 (Phase 6)

## 表示改善 (2026-05-25)

- `format_type()`: Verilog `reg` を "logic reg" → "reg" に統一
- signals テーブルに Unpacked 列を追加

## 次のアクション

1. `insta` スナップショットテストの追加
2. generate if 条件評価の基礎実装 (Phase 6)
