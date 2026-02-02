use crate::{
  state::game_state::GameState,
  utils::{
    build_order::{base_location_utils, build_location_utils, build_manager, next_thing_to_build::BuildStatusMap},
    debug_utils, map_information, military_management, worker_management,
  },
};
use rsbwapi::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;


impl AiModule for ProtosBot {
  fn on_start(&mut self, game: &Game) {
    // SAFETY: rsbwapi uses interior mutability (RefCell) for the command queue.
    // enable_flag only adds a command to the queue.
    // This cast is safe in the single-threaded BWAPI callback context.
    unsafe {
      let game_ptr = game as *const Game as *mut Game;
      (*game_ptr).enable_flag(Flag::UserInput as i32);
    }

    let Some(mut locked_state) = self.game_state.lock().ok() else {
      return;
    };
    let Some(player) = game.self_() else {
      return;
    };

    locked_state.base_locations = base_location_utils::get_base_locations(game, &player);
    locked_state.map_information = Some(map_information::get_map_information_from_game(game));

    println!("Game started on map: {}", game.map_file_name());
  }

  fn on_frame(&mut self, game: &Game) {
    let frame_start = std::time::Instant::now();

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
    worker_management::worker_onframe(game, &player, &mut locked_state);

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

    military_management::military_onframe(game, &player, &mut locked_state);
    debug_utils::print_debug_build_status(game, &player, &locked_state);
    locked_state.unit_display_information = map_information::get_unit_display_information(game);

    draw_unit_ids(game);

    // Update frame timing statistics - keep last 100 frames
    update_frame_timing(&mut locked_state, frame_start);
  }

  fn on_unit_create(&mut self, game: &Game, unit: Unit) {
    if game.get_frame_count() < 1 {
      return;
    }
    println!("unit created: {:?}", unit.get_type());
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
  shared_speed: crate::webserver::SharedGameSpeed,
  build_status: crate::webserver::SharedBuildStatus,
}

impl ProtosBot {
  pub fn new(
    game_state: Arc<Mutex<GameState>>,
    shared_speed: crate::webserver::SharedGameSpeed,
    build_status: crate::webserver::SharedBuildStatus,
  ) -> Self {
    Self {
      game_state,
      shared_speed,
      build_status,
    }
  }
}

fn update_frame_timing(state: &mut GameState, frame_start: Instant) {
  let frame_duration = frame_start.elapsed();
  if state.recent_frame_times_ns.len() >= 100 {
    state.recent_frame_times_ns.pop_front();
  }
  state
    .recent_frame_times_ns
    .push_back(frame_duration.as_nanos());
}

fn draw_unit_ids(game: &Game) {
  for unit in game.get_all_units() {
    if unit.exists() {
      let pos = unit.get_position();
      let unit_id = unit.get_id();
      game.draw_text_map(pos, &format!("{}", unit_id));
    }
  }
}