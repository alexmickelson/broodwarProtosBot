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
  let Some(next_item) = get_next_thing_to_build(game, player, state) else {
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

  // Add upgrade status for this stage
  for upgrade_type in &current_stage.desired_upgrades {
    let upgrade_name = format!("{:?}", upgrade_type);
    let maybe_entry = state
      .unit_build_history
      .iter()
      .find(|entry| entry.upgrade_type == Some(*upgrade_type));
    let minerals = upgrade_type.mineral_price(0); // 0 = Terran, adjust if needed
    let gas = upgrade_type.gas_price(0);
    let player_minerals = player.minerals();
    let player_gas = player.gas();
    let building_type = upgrade_type.what_upgrades();
    // Find an idle, completed building that can do the upgrade
    let has_idle_building = player.get_units().iter().any(|u| {
      u.get_type() == building_type && u.is_completed() && u.is_idle() && !u.is_upgrading()
    });
    let has_building = player
      .get_units()
      .iter()
      .any(|u| u.get_type() == building_type && u.is_completed());
    let status = if let Some(entry) = maybe_entry {
      match entry.status {
        BuildStatus::Started => format!("Complete ({}M/{}G)", minerals, gas),
        BuildStatus::Assigned => format!("Pending ({}M/{}G)", minerals, gas),
      }
    } else if player_minerals < minerals || player_gas < gas {
      format!(
        "Need {} minerals, {} gas ({}M/{}G)",
        (minerals - player_minerals).max(0),
        (gas - player_gas).max(0),
        minerals,
        gas
      )
    } else if !has_building {
      format!("No building available ({}M/{}G)", minerals, gas)
    } else if !has_idle_building {
      format!("No idle building available ({}M/{}G)", minerals, gas)
    } else {
      format!("Ready to research ({}M/{}G)", minerals, gas)
    };
    status_map.insert(upgrade_name, status);
  }

  status_map
}

use rsbwapi::UpgradeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextBuildItem {
  Unit(UnitType),
  Upgrade(UpgradeType),
}

pub fn get_next_thing_to_build(
  game: &Game,
  player: &Player,
  state: &GameState,
) -> Option<NextBuildItem> {
  let current_stage = state.build_stages.get(state.current_stage_index)?;
  if need_more_supply(game, player, state) {
    println!("Decided to build supply depot next");
    return Some(NextBuildItem::Unit(UnitType::Terran_Supply_Depot));
  }
  let status_map = get_status_for_stage_items(game, player, state);

  let upgrade_candidates =
    get_ready_upgrade_candidates(&current_stage.desired_upgrades, &status_map);
  if let Some(upgrade) = upgrade_candidates.first() {
    return Some(NextBuildItem::Upgrade(*upgrade));
  }

  let mut unit_candidates = Vec::new();
  for (unit_type, &desired_count) in &current_stage.desired_counts {
    let current_count = count_units_of_type(player, state, *unit_type);
    if current_count >= desired_count {
      continue;
    }
    let status = status_map.get(&unit_type.name().to_string());
    if status.is_some() && status.unwrap().starts_with("Ready to build") {
      unit_candidates.push(*unit_type);
    }
  }

  if let Some(unit_type) = unit_candidates
    .into_iter()
    .max_by_key(|unit_type| unit_type.mineral_price() + unit_type.gas_price())
  {
    return Some(NextBuildItem::Unit(unit_type));
  }
  None
}

fn get_ready_upgrade_candidates(
  desired_upgrades: &[UpgradeType],
  status_map: &HashMap<String, String>,
) -> Vec<UpgradeType> {
  let mut candidates = Vec::new();
  for upgrade_type in desired_upgrades {
    let upgrade_name = format!("{:?}", upgrade_type);
    if let Some(status) = status_map.get(&upgrade_name) {
      if status.starts_with("Ready to research") {
        candidates.push(*upgrade_type);
      }
    }
  }
  candidates
}

fn need_more_supply(_game: &Game, player: &Player, _state: &GameState) -> bool {
  let supply_used = player.supply_used();
  let supply_total = player.supply_total();
  if supply_total == 0 {
    return false;
  }
  let supply_depots_in_progress: i32 = player
    .get_units()
    .iter()
    .filter(|u| {
      u.get_type() == UnitType::Terran_Supply_Depot && !u.is_completed() && u.exists()
    })
    .count() as i32;

  let supply_ratio = supply_used as f32 / (supply_total + supply_depots_in_progress * 8) as f32;
  return supply_ratio >= 0.85 || (supply_total - supply_used) <= 1;
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
  let units_complete = current_stage
    .desired_counts
    .iter()
    .all(|(unit_type, &desired_count)| {
      let current_count = count_units_of_type(player, state, *unit_type);
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
