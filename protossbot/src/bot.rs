use crate::{
  state::game_state::GameState,
  utils::{build_manager, worker_management},
};
use rsbwapi::*;
use std::sync::{Arc, Mutex};

fn draw_unit_ids(game: &Game) {
  for unit in game.get_all_units() {
    if unit.exists() {
      let pos = unit.get_position();
      let unit_id = unit.get_id();
      game.draw_text_map(pos, &format!("{}", unit_id));
    }
  }
}

impl AiModule for ProtosBot {
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

    let Some(player) = game.self_() else {
      return;
    };

    // Apply desired game speed from shared state
    let desired_speed = self.shared_speed.get();
    unsafe {
      let game_ptr = game as *const Game as *mut Game;
      (*game_ptr).set_local_speed(desired_speed);
    }

    build_manager::on_frame(game, &player, &mut locked_state);
    worker_management::assign_idle_workers_to_minerals(game, &player, &mut locked_state);

    // Update web server with current build status
    let stage_name = locked_state
      .build_stages
      .get(locked_state.current_stage_index)
      .map(|s| s.name.clone())
      .unwrap_or_else(|| "Unknown".to_string());
    self.build_status.update(
      stage_name,
      locked_state.current_stage_index,
      locked_state.stage_item_status.clone(),
    );

    build_manager::print_debug_build_status(game, &player, &locked_state);
    draw_unit_ids(game);
  }

  fn on_unit_create(&mut self, game: &Game, unit: Unit) {
    if game.get_frame_count() < 1 {
      return;
    }
    println!("unit created: {:?}", unit.get_type());

    // If the created unit is a building, handle building creation; otherwise handle unit creation (e.g., trained units).
    let Ok(mut locked_state) = self.game_state.lock() else {
      return;
    };

    if unit.get_type().is_building() {
      build_manager::on_building_create(&unit, &mut locked_state);
    } else {
      build_manager::on_unit_create(&unit, &mut locked_state);
    }
  }

  fn on_unit_morph(&mut self, _game: &Game, _unit: Unit) {}

  fn on_unit_destroy(&mut self, _game: &Game, _unit: Unit) {}

  fn on_unit_complete(&mut self, _game: &Game, _unit: Unit) {
    // let Some(player) = game.self_() else {
    //   return;
    // };
  }

  fn on_end(&mut self, _game: &Game, is_winner: bool) {
    if is_winner {
      println!("Victory!");
    } else {
      println!("Defeat!");
    }
  }
}

pub struct ProtosBot {
  game_state: Arc<Mutex<GameState>>,
  shared_speed: crate::web_server::SharedGameSpeed,
  build_status: crate::web_server::SharedBuildStatus,
}

impl ProtosBot {
  pub fn new(
    game_state: Arc<Mutex<GameState>>,
    shared_speed: crate::web_server::SharedGameSpeed,
    build_status: crate::web_server::SharedBuildStatus,
  ) -> Self {
    Self {
      game_state,
      shared_speed,
      build_status,
    }
  }
}
