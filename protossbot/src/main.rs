mod bot;
mod state;
mod utils;

use bot::ProtosBot;
use state::game_state::GameState;
use std::sync::{Arc, Mutex};

fn main() {
  println!("Starting RustBot...");

  let game_state = Arc::new(Mutex::new(GameState::default()));

  rsbwapi::start(move |_game| ProtosBot::new(game_state.clone()));
}
