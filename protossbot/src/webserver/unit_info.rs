use axum::{extract::State, Json};
use std::sync::{Arc, Mutex};

use crate::{state::game_state::GameState, utils::map_information::UnitDisplayInformation};

pub async fn get_unit_info(
  State(game_state): State<Arc<Mutex<GameState>>>,
) -> Json<Vec<UnitDisplayInformation>> {
  let state_guard = match game_state.lock() {
    Ok(g) => g,
    Err(_) => return Json(vec![]),
  };

  Json(state_guard.unit_display_information.clone())
}
