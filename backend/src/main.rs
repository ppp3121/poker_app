use axum::http::{Method, header};
use axum::{
    Json, Router,
    extract::{
        FromRequestParts, Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use dashmap::DashMap;
use dotenvy::dotenv;
use futures_util::{SinkExt, stream::StreamExt};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use time;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

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

// WebSocket接続を管理するための状態
#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    rooms: Arc<DashMap<uuid::Uuid, broadcast::Sender<String>>>,
}

// WebSocket認証用のクエリパラメータ
#[derive(Deserialize)]
struct WebSocketAuth {
    token: Option<String>,
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

// JWT検証のヘルパー関数
fn verify_jwt(token: &str) -> Result<Claims, String> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let decoding_key = DecodingKey::from_secret(secret.as_ref());

    decode::<Claims>(token, &decoding_key, &Validation::default())
        .map(|data| data.claims)
        .map_err(|err| format!("Invalid token: {}", err))
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

    // AppStateを初期化
    let app_state = Arc::new(AppState {
        db_pool: pool.clone(),
        rooms: Arc::new(DashMap::new()),
    });

    // CORSの設定
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_credentials(true)
        .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(vec![
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::COOKIE,
            header::SET_COOKIE,
            header::UPGRADE,
            header::CONNECTION,
            header::SEC_WEBSOCKET_KEY,
            header::SEC_WEBSOCKET_VERSION,
            header::SEC_WEBSOCKET_PROTOCOL,
        ]);

    // ルーターの設定
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(get_me))
        .route("/api/rooms", post(create_room).get(get_rooms))
        .route("/api/rooms/{id}", get(get_room_by_id))
        .route("/api/ws/rooms/{room_id}", get(ws_handler))
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- API ハンドラ ---

// WebSocketハンドラ関数
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<uuid::Uuid>,
    Query(auth): Query<WebSocketAuth>,
    jar: CookieJar,
) -> Response {
    // まずCookieからトークンを取得を試す
    let token = if let Some(query_token) = auth.token {
        query_token
    } else if let Some(cookie) = jar.get("token") {
        cookie.value().to_string()
    } else {
        println!("WebSocket connection failed: No token found");
        return (StatusCode::UNAUTHORIZED, "Missing token").into_response();
    };

    // JWTを検証
    let claims = match verify_jwt(&token) {
        Ok(claims) => claims,
        Err(err) => {
            println!("WebSocket connection failed: {}", err);
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
    };

    println!("WebSocket connection established for user: {}", claims.sub);
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims, room_id))
}

// 実際のWebSocket通信を処理する関数
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    claims: Claims,
    room_id: uuid::Uuid,
) {
    // or_insert_withを使用して、存在しない場合のみ新しいchannelを作成する
    let tx = state
        .rooms
        .entry(room_id)
        .or_insert_with(|| broadcast::channel(100).0)
        .clone();

    // このルーム専用のSenderを取得または新規作成する
    let mut rx = tx.subscribe();
    let (mut sender, mut receiver) = socket.split();
    let username = claims.sub;

    // このクライアントへメッセージを送信するためのタスクを生成
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // このクライアントからのメッセージを受信するためのタスクを生成
    let tx_clone = tx.clone();
    let username_clone = username.clone();
    let mut recv_task = tokio::spawn(async move {
        // まず参加メッセージを送信
        let join_msg = format!("{}さんが入室しました。", username_clone);
        let _ = tx_clone.send(join_msg);

        // クライアントからのメッセージを待ち続ける
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx_clone.send(format!("{}: {}", username_clone, text));
        }
    });

    // どちらかのタスクが終了したら、もう片方も終了させる
    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    };

    // 退出メッセージをブロードキャスト (元の変数はここでまだ有効)
    let leave_msg = format!("{}さんが退出しました。", username);
    let _ = tx.send(leave_msg);

    // ★ もしルームに誰もいなくなったら、DashMapからSenderを削除する（メモリ解放）
    if tx.receiver_count() == 1 {
        state.rooms.remove(&room_id);
        println!("Room {} is now empty and removed.", room_id);
    }
}

//registerハンドラ
async fn register(
    State(state): State<Arc<AppState>>,
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
    .execute(&state.db_pool)
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

// loginハンドラ
async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserAuth>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db_pool)
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

// logoutハンドラ
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

// create_roomハンドラ
async fn create_room(
    State(state): State<Arc<AppState>>,
    claims: Claims, // 認証済みユーザー情報
    Json(payload): Json<CreateRoomPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // まず、claims.sub (username) から user_id を取得する
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash FROM users WHERE username = $1",
    )
    .bind(&claims.sub)
    .fetch_one(&state.db_pool)
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
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create room: {}", e),
        )
    })?;

    Ok((StatusCode::CREATED, Json(room)))
}

// get_roomsハンドラ
async fn get_rooms(
    State(state): State<Arc<AppState>>,
    _claims: Claims, // ログインしているユーザーのみアクセス可能にするため
) -> Result<Json<Vec<Room>>, (StatusCode, String)> {
    let rooms = sqlx::query_as::<_, Room>("SELECT * FROM rooms ORDER BY created_at DESC")
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch rooms: {}", e),
            )
        })?;

    Ok(Json(rooms))
}

// get_room_by_idハンドラ
async fn get_room_by_id(
    State(state): State<Arc<AppState>>,
    Path(room_id): Path<uuid::Uuid>, // ★ URLパスからroom_idを取得
    _claims: Claims,                 // 認証が必要
) -> Result<Json<Room>, (StatusCode, String)> {
    let room = sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE id = $1")
        .bind(room_id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch room: {}", e),
            )
        })?;

    match room {
        Some(room) => Ok(Json(room)),
        None => Err((StatusCode::NOT_FOUND, "Room not found".to_string())),
    }
}

// get_meハンドラ
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
