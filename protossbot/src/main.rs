mod bot;
mod state;
mod web_server;
mod utils {
  pub mod build_buildings_utils;
  pub mod build_location_utils;
  pub mod build_manager;
  pub mod worker_management;
  pub mod debug_utils;
  pub mod base_location_utils;
  pub mod military_management;
}

use bot::ProtosBot;
use state::game_state::GameState;
use std::sync::{Arc, Mutex};
use web_server::{SharedBuildStatus, SharedGameSpeed};

fn main() {
  println!("Starting RustBot...");

  let game_state = Arc::new(Mutex::new(GameState::default()));
  let shared_speed = SharedGameSpeed::new(42); // Default speed (slowest)
  let build_status = SharedBuildStatus::new();

  // Start web server in a separate thread
  let shared_speed_clone = shared_speed.clone();
  let build_status_clone = build_status.clone();
  let game_state_for_thread = game_state.clone();
  std::thread::spawn(move || {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(web_server::start_web_server(
      shared_speed_clone,
      build_status_clone,
      game_state_for_thread,
    ));
  });

  rsbwapi::start(move |_game| {
    ProtosBot::new(
      game_state.clone(),
      shared_speed.clone(),
      build_status.clone(),
    )
  });
}
