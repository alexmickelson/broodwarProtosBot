use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
pub struct SharedGameSpeed {
  speed: Arc<Mutex<i32>>,
}

impl SharedGameSpeed {
  pub fn new(initial_speed: i32) -> Self {
    Self {
      speed: Arc::new(Mutex::new(initial_speed)),
    }
  }

  pub fn get(&self) -> i32 {
    *self.speed.lock().unwrap()
  }

  pub fn set(&self, speed: i32) {
    *self.speed.lock().unwrap() = speed;
  }
}

#[derive(Serialize, Deserialize)]
pub struct GameSpeedResponse {
  pub speed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GameSpeedRequest {
  pub speed: i32,
}

async fn get_game_speed(State(state): State<SharedGameSpeed>) -> Response {
  let speed = state.get();
  (StatusCode::OK, Json(GameSpeedResponse { speed })).into_response()
}

async fn set_game_speed(
  State(state): State<SharedGameSpeed>,
  Json(payload): Json<GameSpeedRequest>,
) -> Response {
  if payload.speed < -1 || payload.speed > 1000 {
    return (
      StatusCode::BAD_REQUEST,
      Json(GameSpeedResponse { speed: 0 }),
    )
      .into_response();
  }

  state.set(payload.speed);

  (
    StatusCode::OK,
    Json(GameSpeedResponse {
      speed: payload.speed,
    }),
  )
    .into_response()
}

pub async fn start_web_server(shared_speed: SharedGameSpeed) {
  let static_dir = std::env::current_dir().unwrap().join("static");

  let cors = CorsLayer::very_permissive();

  let app = Router::new()
    .route("/api/speed", get(get_game_speed))
    .route("/api/speed", post(set_game_speed))
    .layer(cors)
    .fallback_service(ServeDir::new(static_dir))
    .with_state(shared_speed);

  let addr = "127.0.0.1:3333";

  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  println!("Web server running at http://{}", addr);

  axum::serve(listener, app).await.unwrap();
}
