use axum::http::{Method, header};
use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use time;
use tower_http::cors::{Any, CorsLayer};

// --- 構造体の定義 ---

#[derive(Serialize, sqlx::FromRow)]
struct User {
    id: uuid::Uuid,
    username: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
struct UserAuth {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct CreateRoomPayload {
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct Room {
    id: uuid::Uuid,
    name: String,
    status: String,
    created_by: uuid::Uuid,
    created_at: time::OffsetDateTime,
}

// --- JWT Claims Extractor ---

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // CookieJar extractorを使ってリクエストからクッキーを安全に抽出
        let jar = CookieJar::from_request_parts(parts, _state)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Cookie handling error".to_string(),
                )
            })?;

        let token = jar
            .get("token")
            .map(|c| c.value().to_string())
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing token".to_string()))?;

        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        decode::<Claims>(&token, &decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|err| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", err)))
    }
}

// --- メイン関数 ---

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

    // CORSの設定
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_credentials(true) // Cookieをやり取りするために必要
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![header::CONTENT_TYPE]);

    // ルーターの設定
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(get_me))
        .route("/api/rooms", post(create_room))
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- API ハンドラ ---

async fn register(
    State(pool): State<PgPool>,
    Json(payload): Json<UserAuth>,
) -> Result<StatusCode, (StatusCode, String)> {
    println!("Registering user: {}", payload.username);
    let password_hash = match bcrypt::hash(&payload.password, 12) {
        Ok(h) => h,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password".to_string(),
            ));
        }
    };
    match sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        payload.username,
        password_hash
    )
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return Err((StatusCode::CONFLICT, "Username already exists".to_string()));
                }
            }
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ))
        }
    }
}

async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<UserAuth>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
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

    if bcrypt::verify(&payload.password, &user.password_hash).unwrap_or(false) {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp() as usize;
        let claims = Claims {
            sub: user.username,
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

        let cookie = Cookie::build(("token", token))
            .path("/")
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Lax)
            .build(); // .build() じゃないと警告出るからbuildで、finishはダメintoはもっとダメ、エラー出る

        let jar = CookieJar::new().add(cookie);
        Ok((StatusCode::OK, jar))
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ))
    }
}

async fn logout() -> Result<impl IntoResponse, (StatusCode, String)> {
    // Cookieを即座に無効にするために、過去の時間を設定
    let past_time = time::OffsetDateTime::UNIX_EPOCH;

    // 中身を空にし、有効期限を過去に設定したCookieを作成
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .http_only(true)
        .secure(false) // 開発環境。本番環境ではtrueに
        .same_site(SameSite::Lax)
        .expires(past_time) // expires を使って有効期限を過去にする
        .build();

    let jar = CookieJar::new().add(cookie);
    Ok((StatusCode::OK, jar, "Logged out successfully"))
}

async fn create_room(
    State(pool): State<PgPool>,
    claims: Claims, // 認証済みユーザー情報
    Json(payload): Json<CreateRoomPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // まず、claims.sub (username) から user_id を取得する
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash FROM users WHERE username = $1",
    )
    .bind(&claims.sub)
    .fetch_one(&pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to find user".to_string(),
        )
    })?;

    // rooms テーブルに新しいルームを挿入
    let room = sqlx::query_as::<_, Room>(
        "INSERT INTO rooms (name, created_by) VALUES ($1, $2) RETURNING *",
    )
    .bind(payload.name)
    .bind(user.id) // 取得した user.id を使う
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create room: {}", e),
        )
    })?;

    Ok((StatusCode::CREATED, Json(room)))
}

async fn get_me(claims: Claims) -> Json<Claims> {
    Json(claims)
}

#[derive(Serialize)]
struct HealthStatus {
    status: String,
}

async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}
