# テキサスホールデム ポーカーアプリ

オンラインで対戦可能なテキサスホールデムのポーカーアプリ

## 機能一覧

- ゲストでのゲーム参加
- ユーザー登録・ログイン機能
- オンラインでのリアルタイム対戦
- 戦績の記録・閲覧（登録ユーザーのみ）

## 使用技術 (Tech Stack)

このプロジェクトで使用している主な技術スタック

### フロントエンド (Frontend)

| カテゴリ | 技術 | 備考 |
| :--- | :--- | :--- |
| フレームワーク | [Next.js](https://nextjs.org/) | Reactベースのフレームワーク |
| 言語 | [TypeScript](https://www.typescriptlang.org/) | 静的型付けによる開発効率と堅牢性の向上 |
| UIライブラリ | [shadcn/ui](https://ui.shadcn.com/) | Tailwind CSSベースのコンポーネント集 |
| 状態管理 | [Zustand](https://zustand-demo.pmnd.rs/) | シンプルで軽量な状態管理ライブラリ |
| 通信 | WebSocket | リアルタイム通信を実現 |

### バックエンド (Backend)

| カテゴリ | 技術 | 備考 |
| :--- | :--- | :--- |
| フレームワーク | [Axum](https://github.com/tokio-rs/axum) | Rust製のモダンで高速なWebフレームワーク |
| 言語 | [Rust](https://www.rust-lang.org/) | パフォーマンスと安全性を両立 |
| DB接続 | [SQLx](https://github.com/launchbadge/sqlx) | 非同期対応のSQLツールキット |
| リアルタイム通信| WebSocket | Axumの機能を利用して実装 |

### データベース (Database)

| カテゴリ | 技術 | 備考 |
| :--- | :--- | :--- |
| RDBMS | [PostgreSQL](https://www.postgresql.org/) | 高機能で信頼性の高いリレーショナルデータベース |

### インフラ (Infrastructure)

| 役割 | サービス | 備考 |
| :--- | :--- | :--- |
| フロントエンド | [Vercel](https://vercel.com/) | Next.jsとの親和性が高く、デプロイが容易 |
| バックエンド | [Shuttle](https://www.shuttle.rs/) | Rustネイティブのデプロイプラットフォーム |

### バージョン

| 技術 | バージョン|
| :--- | :--- |
| Node | v20.16.0 |
| npm | 10.8.1 |
| Rust | rustc 1.89.0 (29483883e 2025-08-04) |