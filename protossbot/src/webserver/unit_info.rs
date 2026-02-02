use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::state::game_state::GameState;

#[derive(Serialize, Deserialize)]
pub struct UnitInfoResponse {
  pub units: Vec<UnitInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct UnitInfo {
  pub unit_id: usize,
  pub unit_type_id: i32,
  pub x: i32,
  pub y: i32,
  pub width: i32,
  pub height: i32,
  pub player_id: Option<i32>,
  pub player_name: Option<String>,
}

pub async fn get_unit_info(State(game_state): State<Arc<Mutex<GameState>>>) -> Response {
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

  let units: Vec<UnitInfo> = state_guard
    .unit_display_information
    .iter()
    .map(|unit_data| UnitInfo {
      unit_id: unit_data.unit_id,
      unit_type_id: unit_data.unit_type as i32,
      x: unit_data.position.x,
      y: unit_data.position.y,
      width: unit_data.unit_width,
      height: unit_data.unit_height,
      player_id: unit_data.player_id,
      player_name: unit_data.player_name.clone(),
    })
    .collect();

  let response = UnitInfoResponse { units };

  (StatusCode::OK, Json(response)).into_response()
}
