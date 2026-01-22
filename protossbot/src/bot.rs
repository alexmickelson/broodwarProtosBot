use std::sync::{Arc, Mutex};

use rsbwapi::*;

use crate::game_state::GameState;

impl AiModule for RustBot {
  fn on_start(&mut self, game: &Game) {
    // SAFETY: rsbwapi uses interior mutability (RefCell) for the command queue.
    // enable_flag only adds a command to the queue.
    // This cast is safe in the single-threaded BWAPI callback context.
    unsafe {
      let game_ptr = game as *const Game as *mut Game;
      (*game_ptr).enable_flag(Flag::UserInput as i32);
    }

    println!("Game started on map: {}", game.map_file_name());

  }

  fn on_frame(&mut self, game: &Game) {
    // println!("Frame {}", game.get_frame_count());
    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };

  }

  fn on_unit_create(&mut self, game: &Game, unit: Unit) {
    if game.get_frame_count() < 1 {
      return;
    }
    println!("unit created: {:?}", unit.get_type());
  }

  fn on_unit_morph(&mut self, game: &Game, unit: Unit) {

  }

  fn on_unit_destroy(&mut self, _game: &Game, unit: Unit) {
  
  }

  fn on_unit_complete(&mut self, game: &Game, unit: Unit) {
    let Some(player) = game.self_() else {
      return;
    };

  }

  fn on_end(&mut self, _game: &Game, is_winner: bool) {
    if is_winner {
      println!("Victory!");
    } else {
      println!("Defeat!");
    }
  }
}
pub struct RustBot {
  game_state: Arc<Mutex<GameState>>,
}

impl RustBot {
  pub fn new(game_state: Arc<Mutex<GameState>>) -> Self {
    Self {
      game_state,
    }
  }
}