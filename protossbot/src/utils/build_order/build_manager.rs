use std::collections::HashMap;

use rsbwapi::{Color, Game, Player, UnitType};

use crate::{
  state::game_state::{BuildHistoryEntry, BuildStatus, GameState},
  utils::{
    build_order::{build_buildings_utils, next_thing_to_build},
    unit_utils,
  },
};

pub fn on_frame(game: &Game, player: &Player, state: &mut GameState) {
  build_buildings_utils::check_if_building_started(game, player, state);
  check_and_advance_stage(player, state);
  state.stage_item_status = next_thing_to_build::get_status_for_stage_items(game, player, state);

  build_buildings_utils::try_restart_failed_builing_builds(game, player, state);
  try_start_next_build(game, player, state);
}

fn try_start_next_build(game: &Game, player: &Player, state: &mut GameState) {
  if !should_start_next_build(game, player, state) {
    return;
  }
  let Some(next_item) = next_thing_to_build::get_next_thing_to_build(game, player, state) else {
    return;
  };

  match next_item {
    NextBuildItem::Unit(unit_type) => {
      if unit_type.is_building() {
        build_buildings_utils::start_building_construction(game, player, state, unit_type);
      } else {
        start_unit_training(game, player, state, unit_type);
      }
    }
    NextBuildItem::Upgrade(upgrade_type) => {
      start_upgrade_research(game, player, state, upgrade_type);
    }
  }
}

fn start_upgrade_research(
  _game: &Game,
  player: &Player,
  state: &mut GameState,
  upgrade_type: UpgradeType,
) {
  // Find a building that can research this upgrade and is idle
  let building_type = upgrade_type.what_upgrades();
  // Assign to a variable to avoid temporary value issue
  let btype = building_type;
  let Some(building) = player
    .get_units()
    .into_iter()
    .find(|u| u.get_type() == btype && u.is_completed() && !u.is_upgrading() && u.is_idle())
  else {
    println!(
      "No available building to research upgrade: {:?}",
      upgrade_type
    );
    return;
  };

  let building_id = building.get_id();
  let status = match building.upgrade(upgrade_type) {
    Ok(true) => BuildStatus::Started,
    Ok(false) => {
      println!(
        "Upgrade command failed for {:?} by building {}",
        upgrade_type, building_id
      );
      BuildStatus::Assigned
    }
    Err(e) => {
      println!(
        "Upgrade command FAILED for {:?} by building {}: {:?}",
        upgrade_type, building_id, e
      );
      BuildStatus::Assigned
    }
  };

  let entry = BuildHistoryEntry {
    unit_type: None,
    upgrade_type: Some(upgrade_type),
    assigned_unit_id: Some(building_id),
    tile_position: None,
    status,
  };
  state.unit_build_history.push(entry);
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

use rsbwapi::UpgradeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextBuildItem {
  Unit(UnitType),
  Upgrade(UpgradeType),
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

fn check_and_advance_stage(player: &Player, state: &mut GameState) {
  let Some(current_stage) = state.build_stages.get(state.current_stage_index) else {
    return;
  };
  let units_complete = current_stage
    .desired_counts
    .iter()
    .all(|(unit_type, &desired_count)| {
      let current_count = unit_utils::count_units_of_type(player, *unit_type);
      current_count >= desired_count
    });

  let upgrades_complete = current_stage.desired_upgrades.iter().all(|upgrade_type| {
    state.unit_build_history.iter().any(|entry| {
      entry.upgrade_type == Some(*upgrade_type)
        && entry.status == crate::state::game_state::BuildStatus::Started
    })
  });

  if units_complete && upgrades_complete {
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
