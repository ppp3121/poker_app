// backend/src/main.rs

// ... 既存のuse宣言 ...
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, Cookie, authorization::Bearer},
};
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
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

// ★ JWTのクレーム（中身）を定義する構造体
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // ユーザー名
    exp: usize,  // 有効期限
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // ★ 戻り値の型を変更
    println!("Logging in user: {}", payload.username);

    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid username or password".to_string(),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ));
        }
    };

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
        // ★ JWTを生成する処理
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + Duration::hours(24)).timestamp() as usize; // 24時間有効
        let claims = Claims {
            sub: user.username.clone(),
            exp,
        };
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create token".to_string(),
            )
        })?;

        // ★ Cookieを作成
        let cookie = axum_extra::headers::Cookie::new("token", token)
            .with_path("/")
            .with_http_only(true)
            .with_secure(false) // 開発環境なのでfalse。本番ではtrue
            .with_same_site(axum_extra::headers::SameSite::Lax);

        // ★ Cookieをヘッダーにセットしてレスポンスを返す
        let mut response = StatusCode::OK.into_response();
        response.headers_mut().insert(
            axum::http::header::SET_COOKIE,
            cookie.to_string().parse().unwrap(),
        );
        Ok(response)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ))
    }
}

// ... health_check関数 ...
#[derive(Serialize)]
struct HealthStatus {
    status: String,
}

async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}
