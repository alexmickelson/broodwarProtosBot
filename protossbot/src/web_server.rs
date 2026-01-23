use crate::state::game_state::GameState;
use axum::{
  extract::State,
  http::StatusCode,
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
pub struct AppState {
  pub game_state: Arc<Mutex<GameState>>,
}

#[derive(Serialize, Deserialize)]
pub struct GameSpeedResponse {
  pub speed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GameSpeedRequest {
  pub speed: i32,
}

// GET /api/speed - Get current game speed
async fn get_game_speed(State(state): State<AppState>) -> impl IntoResponse {
  let game_state = match state.game_state.lock() {
    Ok(gs) => gs,
    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(GameSpeedResponse { speed: 0 })),
  };

  (
    StatusCode::OK,
    Json(GameSpeedResponse {
      speed: game_state.desired_game_speed,
    }),
  )
}

// POST /api/speed - Set game speed
async fn set_game_speed(
  State(state): State<AppState>,
  Json(payload): Json<GameSpeedRequest>,
) -> impl IntoResponse {
  // Validate speed is within reasonable range (0-1000)
  if payload.speed < 0 || payload.speed > 1000 {
    return (
      StatusCode::BAD_REQUEST,
      Json(GameSpeedResponse { speed: 0 }),
    );
  }

  let mut game_state = match state.game_state.lock() {
    Ok(gs) => gs,
    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(GameSpeedResponse { speed: 0 })),
  };

  game_state.desired_game_speed = payload.speed;

  (
    StatusCode::OK,
    Json(GameSpeedResponse {
      speed: game_state.desired_game_speed,
    }),
  )
}

pub async fn start_web_server(game_state: Arc<Mutex<GameState>>) {
  let app_state = AppState { game_state };

  // Serve static files from the "static" directory
  let static_dir = std::env::current_dir()
    .unwrap()
    .join("static");

  let app = Router::new()
    .route("/api/speed", get(get_game_speed))
    .route("/api/speed", post(set_game_speed))
    .fallback_service(ServeDir::new(static_dir))
    .layer(CorsLayer::permissive())
    .with_state(app_state);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    .await
    .unwrap();

  println!("Web server running at http://127.0.0.1:3000");

  axum::serve(listener, app).await.unwrap();
}
