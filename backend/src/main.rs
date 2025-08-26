// backend/src/main.rs

use axum::{
    Json,
    Router,
    extract::State, // 共有データをハンドラで受け取るために必要
    http::StatusCode,
    routing::{get, post}, // POSTリクエストを扱うために追加
};
use serde::{Deserialize, Serialize}; // Deserializeを追加
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

// sqlx を使ってPostgreSQLに接続するためのライブラリ
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

// .env ファイルを読み込むためのライブラリ
use dotenvy::dotenv;
use std::env;

// Tokioを非同期ランタイムとしてメイン関数を定義する
#[tokio::main]
async fn main() {
    // .env ファイルから環境変数を読み込む
    dotenv().ok();

    // データベース接続URLを環境変数から取得
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // データベース接続プールを作成
    // PgPoolは複数のDB接続を効率的に管理してくれる
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    println!("Database connected successfully.");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ルーティングを定義
    let app = Router::new()
        .route("/api/health", get(health_check))
        // POSTリクエストで "/api/register" にアクセスされたら register 関数を呼ぶ
        .route("/api/register", post(register))
        .layer(cors)
        // .with_state を使って、すべてのハンドラでDB接続プールを共有する
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// フロントエンドから受け取るJSONの型を定義
#[derive(Deserialize)]
struct RegisterUser {
    username: String,
    password: String,
}

// ユーザー登録の処理を行うハンドラ関数
async fn register(
    State(pool): State<PgPool>,        // 共有しているDB接続プールを受け取る
    Json(payload): Json<RegisterUser>, // 送られてきたJSONを RegisterUser 型に変換して受け取る
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Registering user: {}", payload.username);

    // パスワードをハッシュ化する (コスト=12は推奨される強度)
    let password_hash = match bcrypt::hash(&payload.password, 12) {
        Ok(h) => h,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password".to_string(),
            ));
        }
    };

    // SQLを使ってDBにユーザーを挿入する
    // sqlx::query! マクロはコンパイル時にSQLの構文をチェックしてくれるので安全
    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        payload.username,
        password_hash
    )
    .execute(&pool)
    .await;

    match result {
        // 成功した場合
        Ok(_) => Ok(StatusCode::CREATED), // 201 Created ステータスを返す
        // 失敗した場合
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            // ユーザー名が重複している場合のエラーを判定
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return Err((
                        StatusCode::CONFLICT, // 409 Conflict
                        "Username already exists".to_string(),
                    ));
                }
            }
            // その他のDBエラー
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

// health_check関数は変更なし
#[derive(Serialize)]
struct HealthStatus {
    status: String,
}

async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}
