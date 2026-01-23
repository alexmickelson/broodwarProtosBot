mod bot;
mod state;
mod utils;
mod web_server;

use bot::ProtosBot;
use state::game_state::GameState;
use std::sync::{Arc, Mutex};

fn main() {
  println!("Starting RustBot...");

  let game_state = Arc::new(Mutex::new(GameState::default()));

  // Start web server in a separate thread
  let game_state_clone = game_state.clone();
  std::thread::spawn(move || {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(web_server::start_web_server(game_state_clone));
  });

  rsbwapi::start(move |_game| ProtosBot::new(game_state.clone()));
}
