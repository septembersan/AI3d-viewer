# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

wGPUを使用したRust製3Dポイントクラウドビューアー。現在は50³（125,000ポイント）のデモ立方体グリッドを表示する初期プロトタイプ段階。

## ビルド・実行コマンド

```bash
cargo build              # デバッグビルド
cargo build --release    # リリースビルド（opt-level=3, LTO有効）
cargo run --release      # リリースモードで実行
cargo test               # テスト実行（現時点ではテスト未実装）
```

## アーキテクチャ

### コンポーネント構成

- **main.rs** — Winitイベントループ、`ApplicationHandler`実装、マウス入力処理（左ドラッグ=回転、右ドラッグ=パン、ホイール=ズーム）
- **renderer.rs** — wGPUデバイス/キュー/サーフェス/パイプライン管理、ユニフォームバッファ（View-Projection行列）、ポイント頂点バッファ、深度テクスチャ
- **point_cloud.rs** — `PointCloud`と`GpuPoint`（16バイト: position `[f32;3]` + packed RGBA8 `u32`）の定義、デモデータ生成
- **camera.rs** — 球面座標系の軌道カメラ（yaw/pitch/distance/target）、View-Projection行列計算（`glam`使用）
- **shader.wgsl** — WGSL頂点/フラグメントシェーダー、ポイントリストトポロジー用

### データフロー

1. `PointCloud`がCPU側で`GpuPoint`配列を保持
2. `Renderer::new()`でGPUバッファに転送
3. 毎フレーム: `Camera`からView-Projection行列を計算 → ユニフォームバッファ更新 → `PointList`として描画

### 主要な依存ライブラリ

| ライブラリ | 用途 |
|---|---|
| wgpu 23 | GPU レンダリング |
| winit 0.30 | ウィンドウ・イベント管理 |
| glam 0.29 | 線形代数（行列・ベクトル） |
| bytemuck 1 | GPU用メモリレイアウト変換（Pod/Zeroable） |
| pollster 0.4 | async→sync ブロック化 |

### 設計上の注意点

- `GpuPoint`は`#[repr(C)]`で16バイトアライメント。GPUバッファレイアウトと一致させる必要あり
- カメラのpitchは±1.5ラジアン、distanceは0.1〜500.0にクランプ
- サーフェスフォーマットはsRGBを使用（`TextureFormat::*Srgb`優先）
- リサイズ時は`renderer.resize()`で深度テクスチャとサーフェスを再構成
