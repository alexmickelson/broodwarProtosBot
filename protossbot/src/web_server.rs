use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::state::game_state::{self, BuildHistoryEntry, BuildStatus};

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

#[derive(Clone, Default)]
pub struct BuildStatusData {
  pub stage_name: String,
  pub stage_index: usize,
  pub item_status: HashMap<String, String>,
}

#[derive(Clone)]
pub struct SharedBuildStatus {
  data: Arc<Mutex<BuildStatusData>>,
}

impl SharedBuildStatus {
  pub fn new() -> Self {
    Self {
      data: Arc::new(Mutex::new(BuildStatusData::default())),
    }
  }

  pub fn update(
    &self,
    stage_name: String,
    stage_index: usize,
    item_status: HashMap<String, String>,
  ) {
    let mut data = self.data.lock().unwrap();
    data.stage_name = stage_name;
    data.stage_index = stage_index;
    data.item_status = item_status;
  }

  pub fn get(&self) -> BuildStatusData {
    self.data.lock().unwrap().clone()
  }
}

#[derive(Clone)]
struct AppState {
  game_speed: SharedGameSpeed,
  build_status: SharedBuildStatus,
  game_state: Arc<Mutex<game_state::GameState>>,
}

#[derive(Serialize, Deserialize)]
pub struct GameSpeedResponse {
  pub speed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GameSpeedRequest {
  pub speed: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BuildStatusResponse {
  pub stage_name: String,
  pub stage_index: usize,
  pub items: Vec<BuildItemStatusInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct BuildItemStatusInfo {
  pub unit_name: String,
  pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct BuildHistoryInfo {
  pub unit_name: Option<String>,
  pub status: String,
  pub assigned_unit_id: Option<usize>,
}

async fn get_game_speed(State(app_state): State<AppState>) -> Response {
  let speed = app_state.game_speed.get();
  (StatusCode::OK, Json(GameSpeedResponse { speed })).into_response()
}

async fn set_game_speed(
  State(app_state): State<AppState>,
  Json(payload): Json<GameSpeedRequest>,
) -> Response {
  if payload.speed < -1 || payload.speed > 1000 {
    return (
      StatusCode::BAD_REQUEST,
      Json(GameSpeedResponse { speed: 0 }),
    )
      .into_response();
  }

  app_state.game_speed.set(payload.speed);

  (
    StatusCode::OK,
    Json(GameSpeedResponse {
      speed: payload.speed,
    }),
  )
    .into_response()
}

async fn get_build_status(State(app_state): State<AppState>) -> Response {
  let data = app_state.build_status.get();

  let items: Vec<BuildItemStatusInfo> = data
    .item_status
    .iter()
    .map(|(unit_name, status)| BuildItemStatusInfo {
      unit_name: unit_name.clone(),
      status: status.clone(),
    })
    .collect();

  let response = BuildStatusResponse {
    stage_name: data.stage_name,
    stage_index: data.stage_index,
    items,
  };

  (StatusCode::OK, Json(response)).into_response()
}

async fn get_build_history(State(app_state): State<AppState>) -> Response {
  let state_guard = match app_state.game_state.lock() {
    Ok(g) => g,
    Err(_) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json("Failed to lock game state"),
      )
        .into_response();
    }
  };

  let history: Vec<BuildHistoryInfo> = state_guard
    .unit_build_history
    .iter()
    .filter(|entry| entry.status == BuildStatus::Assigned)
    .map(|entry| BuildHistoryInfo {
      unit_name: entry.unit_type.map(|ut| ut.name().to_string()),
      status: match entry.status {
        BuildStatus::Assigned => "Assigned".to_string(),
        BuildStatus::Started => "Started".to_string(),
      },
      assigned_unit_id: entry.assigned_unit_id,
    })
    .collect();

  (StatusCode::OK, Json(history)).into_response()
}

pub async fn start_web_server(
  shared_speed: SharedGameSpeed,
  build_status: SharedBuildStatus,
  game_state: Arc<Mutex<game_state::GameState>>,
) {
  let static_dir = std::env::current_dir().unwrap().join("static");

  let cors = CorsLayer::very_permissive();

  let app_state = AppState {
    game_speed: shared_speed,
    build_status,
    game_state,
  };

  let app = Router::new()
    .route("/api/speed", get(get_game_speed))
    .route("/api/speed", post(set_game_speed))
    .route("/api/build-status", get(get_build_status))
    .route("/api/build-history", get(get_build_history))
    .layer(cors)
    .fallback_service(ServeDir::new(static_dir))
    .with_state(app_state);

  let addr = "127.0.0.1:3333";

  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  println!("Web server running at http://{}", addr);

  axum::serve(listener, app).await.unwrap();
}
