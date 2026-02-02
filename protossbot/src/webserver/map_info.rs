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
pub struct MapInfoResponse {
  pub map_width: i32,
  pub map_height: i32,
  pub tiles: Vec<TileInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct TileInfo {
  pub x: i32,
  pub y: i32,
  pub is_walkable: bool,
  pub is_buildable: bool,
}

pub async fn get_map_info(State(game_state): State<Arc<Mutex<GameState>>>) -> Response {
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

  let Some(map_info) = &state_guard.map_information else {
    return (StatusCode::NOT_FOUND, Json("Map information not available")).into_response();
  };

  let tiles: Vec<TileInfo> = map_info
    .tile_information
    .iter()
    .map(|(pos, tile_data)| TileInfo {
      x: pos.x,
      y: pos.y,
      is_walkable: tile_data.is_walkable,
      is_buildable: tile_data.is_buildable,
    })
    .collect();

  let response = MapInfoResponse {
    map_width: map_info.map_width,
    map_height: map_info.map_height,
    tiles,
  };

  (StatusCode::OK, Json(response)).into_response()
}
