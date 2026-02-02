use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::state::game_state::{self, BuildStatus, GameState};

#[derive(Serialize, Deserialize)]
pub struct BuildHistoryInfo {
  pub unit_name: Option<String>,
  pub status: String,
  pub assigned_unit_id: Option<usize>,
}

pub async fn get_build_history(State(game_state): State<Arc<Mutex<GameState>>>) -> Response {
  let state_guard = match game_state.lock() {
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
