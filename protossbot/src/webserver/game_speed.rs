use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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

pub async fn get_game_speed(State(game_speed): State<SharedGameSpeed>) -> Response {
  let speed = game_speed.get();
  (StatusCode::OK, Json(GameSpeedResponse { speed })).into_response()
}

pub async fn set_game_speed(
  State(game_speed): State<SharedGameSpeed>,
  Json(payload): Json<GameSpeedRequest>,
) -> Response {
  if payload.speed < -1 || payload.speed > 1000 {
    return (
      StatusCode::BAD_REQUEST,
      Json(GameSpeedResponse { speed: 0 }),
    )
      .into_response();
  }

  game_speed.set(payload.speed);

  (
    StatusCode::OK,
    Json(GameSpeedResponse {
      speed: payload.speed,
    }),
  )
    .into_response()
}
