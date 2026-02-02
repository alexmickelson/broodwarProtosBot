use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::state::game_state::GameState;

#[derive(Serialize, Deserialize)]
pub struct FrameStatsResponse {
  pub avg_frame_time_ms: f64,
}

pub async fn get_frame_stats(State(game_state): State<Arc<Mutex<GameState>>>) -> impl IntoResponse {
  let state = match game_state.lock() {
    Ok(s) => s,
    Err(_) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(FrameStatsResponse {
          avg_frame_time_ms: 0.0,
        }),
      )
    }
  };

  let avg_frame_time_ms = if !state.recent_frame_times_ns.is_empty() {
    let sum: u128 = state.recent_frame_times_ns.iter().sum();
    let avg_ns = sum / state.recent_frame_times_ns.len() as u128;
    avg_ns as f64 / 1_000_000.0
  } else {
    0.0
  };

  let response = FrameStatsResponse { avg_frame_time_ms };

  (StatusCode::OK, Json(response))
}
