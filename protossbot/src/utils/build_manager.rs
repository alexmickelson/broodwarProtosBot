use std::collections::HashMap;

use rsbwapi::{Color, Game, Player, UnitType};

use crate::{
  state::game_state::{BuildHistoryEntry, BuildStatus, GameState},
  utils::build_buildings_utils,
};

pub fn on_frame(game: &Game, player: &Player, state: &mut GameState) {
  build_buildings_utils::check_if_building_started(game, player, state);
  check_and_advance_stage(player, state);
  state.stage_item_status = get_status_for_stage_items(game, player, state);

  build_buildings_utils::try_restart_failed_builing_builds(game, player, state);
  try_start_next_build(game, player, state);
}

fn try_start_next_build(game: &Game, player: &Player, state: &mut GameState) {
  if !should_start_next_build(game, player, state) {
    return;
  }
  let Some(unit_type) = get_next_thing_to_build(game, player, state) else {
    return;
  };

  if unit_type.is_building() {
    build_buildings_utils::start_building_construction(game, player, state, unit_type);
  } else {
    start_unit_training(game, player, state, unit_type);
  }
}

fn start_unit_training(_game: &Game, player: &Player, state: &mut GameState, unit_type: UnitType) {
  let Some(trainer) = find_builder_for_unit(player, unit_type, state) else {
    return;
  };
  let trainer_id = trainer.get_id();
  let entry = BuildHistoryEntry {
    unit_type: Some(unit_type),
    upgrade_type: None,
    assigned_unit_id: Some(trainer_id),
    tile_position: None,
    status: match trainer.train(unit_type) {
      Ok(true) => BuildStatus::Started,
      Ok(false) => {
        println!(
          "Train command failed for {} by trainer {}",
          unit_type.name(),
          trainer_id
        );
        BuildStatus::Assigned
      }
      Err(e) => {
        println!(
          "Train command FAILED for {} by trainer {}: {:?}",
          unit_type.name(),
          trainer_id,
          e
        );
        BuildStatus::Assigned
      }
    },
  };

  state.unit_build_history.push(entry);
}

fn should_start_next_build(game: &Game, player: &Player, state: &mut GameState) -> bool {
  if state
    .unit_build_history
    .iter()
    .any(|entry: &BuildHistoryEntry| entry.status == BuildStatus::Assigned)
  {
    return false;
  }

  true
}

fn get_status_for_stage_items(
  _game: &Game,
  player: &Player,
  state: &GameState,
) -> HashMap<String, String> {
  let mut status_map = HashMap::new();
  let Some(current_stage) = state.build_stages.get(state.current_stage_index) else {
    return status_map;
  };
  for (unit_type, &desired_count) in &current_stage.desired_counts {
    let unit_name = unit_type.name().to_string();
    let current_count = count_units_of_type(player, state, *unit_type);
    if current_count >= desired_count {
      status_map.insert(
        unit_name,
        format!("Complete ({}/{})", current_count, desired_count),
      );
      continue;
    }
    if !can_afford_unit(player, *unit_type) {
      let minerals_short = unit_type.mineral_price() - player.minerals();
      let gas_short = unit_type.gas_price() - player.gas();
      status_map.insert(
        unit_name,
        format!(
          "Need {} minerals, {} gas ({}/{})",
          minerals_short.max(0),
          gas_short.max(0),
          current_count,
          desired_count
        ),
      );
      continue;
    }
    if unit_type.is_building() {
      if find_builder_for_unit(player, *unit_type, state).is_none() {
        status_map.insert(
          unit_name,
          format!("No builder available ({}/{})", current_count, desired_count),
        );
        continue;
      }
    }
    status_map.insert(
      unit_name,
      format!("Ready to build ({}/{})", current_count, desired_count),
    );
  }
  status_map
}

pub fn get_next_thing_to_build(game: &Game, player: &Player, state: &GameState) -> Option<UnitType> {
  let current_stage = state.build_stages.get(state.current_stage_index)?;
  if let Some(pylon) = check_need_more_supply(game, player, state) {
    return Some(pylon);
  }
  let status_map = get_status_for_stage_items(game, player, state);
  let mut candidates = Vec::new();
  for (unit_type, &desired_count) in &current_stage.desired_counts {
    let current_count = count_units_of_type(player, state, *unit_type);
    if current_count >= desired_count {
      continue;
    }
    let status = status_map.get(&unit_type.name().to_string());
    if status.is_some() && status.unwrap().starts_with("Ready to build") {
      candidates.push(*unit_type);
    }
  }
  candidates
    .into_iter()
    .max_by_key(|unit_type| unit_type.mineral_price() + unit_type.gas_price())
}

fn check_need_more_supply(_game: &Game, player: &Player, _state: &GameState) -> Option<UnitType> {
  let supply_used = player.supply_used();
  let supply_total = player.supply_total();
  if supply_total == 0 {
    return None;
  }
  let supply_remaining = supply_total - supply_used;
  let threshold = ((supply_total as f32) * 0.15).ceil() as i32;
  if supply_remaining <= threshold && supply_total < 400 {
    let unit_supply_type = UnitType::Terran_Supply_Depot;
    if can_afford_unit(player, unit_supply_type) {
      return Some(unit_supply_type);
    }
  }
  None
}

fn find_builder_for_unit(
  player: &Player,
  unit_type: UnitType,
  _state: &GameState,
) -> Option<rsbwapi::Unit> {
  let builder_type = unit_type.what_builds().0;
  player
    .get_units()
    .iter()
    .find(|u| {
      u.get_type() == builder_type
        && !u.is_constructing()
        && !u.is_training()
        && (u.is_idle() || u.is_gathering_minerals() || u.is_gathering_gas())
    })
    .cloned()
}

fn count_units_of_type(player: &Player, _state: &GameState, unit_type: UnitType) -> i32 {
  player
    .get_units()
    .iter()
    .filter(|u| u.get_type() == unit_type)
    .count() as i32
}

fn can_afford_unit(player: &Player, unit_type: UnitType) -> bool {
  let minerals = player.minerals();
  let gas = player.gas();
  minerals >= unit_type.mineral_price() && gas >= unit_type.gas_price()
}

fn check_and_advance_stage(player: &Player, state: &mut GameState) {
  let Some(current_stage) = state.build_stages.get(state.current_stage_index) else {
    return;
  };
  let stage_complete = current_stage
    .desired_counts
    .iter()
    .all(|(unit_type, &desired_count)| {
      let current_count = count_units_of_type(player, state, *unit_type);
      current_count >= desired_count
    });
  if stage_complete {
    let next_stage_index = state.current_stage_index + 1;
    if next_stage_index < state.build_stages.len() {
      println!(
        "Stage '{}' complete! Advancing to stage {}",
        current_stage.name, state.build_stages[next_stage_index].name
      );
      state.current_stage_index = next_stage_index;
    }
  }
}
