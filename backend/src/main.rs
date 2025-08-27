// backend/src/main.rs

// ... 既存のuse宣言 ...
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

// ★ ユーザー情報を格納する構造体を追加
#[derive(Serialize, sqlx::FromRow)]
struct User {
    id: uuid::Uuid,
    username: String,
    password_hash: String,
}

// ... main関数 ...
#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/register", post(register))
        // ★ ログイン用のルートを追加
        .route("/api/login", post(login))
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ... RegisterUser 構造体 ...
// ★ ログイン用に使い回すため、名前を UserAuth に変更
#[derive(Deserialize)]
struct UserAuth {
    username: String,
    password: String,
}

// ... register関数 ...
// ★ 引数の型を UserAuth に変更
async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<UserAuth>,
) -> Result<StatusCode, (StatusCode, String)> {
    // ... (関数の中身は変更なし) ...
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

// ★ ログイン処理を行うハンドラ関数を追加
async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<UserAuth>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Logging in user: {}", payload.username);

    // 1. ユーザー名でデータベースからユーザーを検索
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            // ユーザーが見つからない場合は認証失敗
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".to_string(),
            ));
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ));
        }
    };

    // 2. パスワードのハッシュを検証
    let is_valid = match bcrypt::verify(&payload.password, &user.password_hash) {
        Ok(v) => v,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to verify password".to_string(),
            ));
        }
    };

    if is_valid {
        // 3. パスワードが正しければ成功
        println!("User {} logged in successfully", payload.username);
        Ok(StatusCode::OK) // 200 OK
    } else {
        // 4. パスワードが間違っていれば認証失敗
        Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ))
    }
}

// ... health_check関数 (変更なし) ...
#[derive(Serialize)]
struct HealthStatus {
    status: String,
}

async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}
