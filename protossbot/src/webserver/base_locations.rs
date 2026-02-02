use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::state::game_state::GameState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableTilePosition {
  pub x: i32,
  pub y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableCheckedPosition {
  pub tile_position: SerializableTilePosition,
  pub is_valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializableBaseLocation {
  pub id: usize,
  pub position: SerializableTilePosition,
  pub checked_positions: Vec<SerializableCheckedPosition>,
  pub path_to_location: Vec<SerializableTilePosition>,
}

pub async fn get_base_locations(
  State(game_state): State<Arc<Mutex<GameState>>>,
) -> Json<Vec<SerializableBaseLocation>> {
  let Ok(state) = game_state.lock() else {
    return Json(Vec::new());
  };

  let base_locations: Vec<SerializableBaseLocation> = state
    .base_locations
    .iter()
    .enumerate()
    .map(|(index, base)| SerializableBaseLocation {
      id: index,
      position: SerializableTilePosition {
        x: base.position.x,
        y: base.position.y,
      },
      checked_positions: base
        .checked_positions
        .iter()
        .map(|cp| SerializableCheckedPosition {
          tile_position: SerializableTilePosition {
            x: cp.tile_position.x,
            y: cp.tile_position.y,
          },
          is_valid: cp.is_valid,
        })
        .collect(),
      path_to_location: base
        .path_to_location
        .iter()
        .map(|tp| SerializableTilePosition { x: tp.x, y: tp.y })
        .collect(),
    })
    .collect();

  Json(base_locations)
}
