mod bot;
pub mod game_state;

use bot::RustBot;
use std::sync::{Arc, Mutex};
use game_state::GameState;


fn main() {
  println!("Starting RustBot...");

  let game_state = Arc::new(Mutex::new(GameState::default()));

  rsbwapi::start(move |_game| RustBot::new(game_state.clone() ));
}
