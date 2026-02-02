use std::collections::HashMap;

use rsbwapi::*;

use crate::{
  state::game_state::{BuildStatus, GameState},
  utils::{build_order::build_manager::NextBuildItem, unit_utils},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WantToBuildStatus {
  ReadyToBuild,
  CannotAfford { minerals_short: i32, gas_short: i32 },
  NoBuilderAvailable,
  HaveAllNeeded,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnitOrUpgradeType {
  Unit(UnitType),
  Upgrade(UpgradeType),
}

pub type BuildStatusMap = HashMap<UnitOrUpgradeType, WantToBuildStatus>;

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
    let current_count = unit_utils::count_units_of_type(player, *unit_type);
    if current_count >= desired_count {
      continue;
    }

    if let Some(status) = status_map.get(&UnitOrUpgradeType::Unit(*unit_type)) {
      if matches!(status, WantToBuildStatus::ReadyToBuild) {
        unit_candidates.push(*unit_type);
      }
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
  status_map: &BuildStatusMap,
) -> Vec<UpgradeType> {
  let mut candidates = Vec::new();
  for upgrade_type in desired_upgrades {
    if let Some(status) = status_map.get(&UnitOrUpgradeType::Upgrade(*upgrade_type)) {
      if matches!(status, WantToBuildStatus::ReadyToBuild) {
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
    .filter(|u| u.get_type() == UnitType::Terran_Supply_Depot && !u.is_completed() && u.exists())
    .count() as i32;

  let supply_ratio = supply_used as f32 / (supply_total + supply_depots_in_progress * 8) as f32;
  return supply_ratio >= 0.8 || (supply_total - supply_used) <= 1;
}

pub fn get_status_for_stage_items(
  _game: &Game,
  player: &Player,
  state: &GameState,
) -> BuildStatusMap {
  let mut status_map = BuildStatusMap::new();
  let Some(current_stage) = state.build_stages.get(state.current_stage_index) else {
    return status_map;
  };
  for (unit_type, &desired_count) in &current_stage.desired_counts {
    let status = get_unit_status(player, state, unit_type, desired_count);
    status_map.insert(UnitOrUpgradeType::Unit(unit_type.clone()), status);
  }

  for upgrade_type in &current_stage.desired_upgrades {
    let status = get_upgrade_status(player, state, upgrade_type);
    status_map.insert(UnitOrUpgradeType::Upgrade(upgrade_type.clone()), status);
  }

  status_map
}

fn can_afford_unit(player: &Player, unit_type: UnitType) -> bool {
  let minerals = player.minerals();
  let gas = player.gas();
  minerals >= unit_type.mineral_price() && gas >= unit_type.gas_price()
}

fn get_upgrade_status(
  player: &Player,
  state: &GameState,
  upgrade_type: &UpgradeType,
) -> WantToBuildStatus {
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
  let has_idle_building = player
    .get_units()
    .iter()
    .any(|u| u.get_type() == building_type && u.is_completed() && u.is_idle() && !u.is_upgrading());
  let has_building = player
    .get_units()
    .iter()
    .any(|u| u.get_type() == building_type && u.is_completed());

  if let Some(entry) = maybe_entry {
    match entry.status {
      BuildStatus::Started => WantToBuildStatus::HaveAllNeeded,
      BuildStatus::Assigned => WantToBuildStatus::ReadyToBuild,
    }
  } else if player_minerals < minerals || player_gas < gas {
    WantToBuildStatus::CannotAfford {
      minerals_short: (minerals - player_minerals).max(0),
      gas_short: (gas - player_gas).max(0),
    }
  } else if !has_building {
    WantToBuildStatus::NoBuilderAvailable
  } else if !has_idle_building {
    WantToBuildStatus::NoBuilderAvailable
  } else {
    WantToBuildStatus::ReadyToBuild
  }
}

fn get_unit_status(
  player: &Player,
  state: &GameState,
  unit_type: &UnitType,
  desired_count: i32,
) -> WantToBuildStatus {
  let current_count = unit_utils::count_units_of_type(player, *unit_type);
  if current_count >= desired_count {
    return WantToBuildStatus::HaveAllNeeded;
  }
  if !can_afford_unit(player, *unit_type) {
    let minerals_short = unit_type.mineral_price() - player.minerals();
    let gas_short = unit_type.gas_price() - player.gas();
    return WantToBuildStatus::CannotAfford {
      minerals_short,
      gas_short,
    };
  }
  
  if unit_type.required_tech() != TechType::None {
    if !player.has_researched(unit_type.required_tech()) {
      return WantToBuildStatus::NoBuilderAvailable;
    }
  }

  let has_all_required_buildings = unit_type
    .required_units()
    .iter()
    .all(|(req_unit, count)| unit_utils::count_completed_units_of_type(player, *req_unit) >= *count);
  
  if !has_all_required_buildings {
    return WantToBuildStatus::NoBuilderAvailable;
  }

  if unit_utils::find_builder_for_unit(player, *unit_type, state).is_none() {
    return WantToBuildStatus::NoBuilderAvailable;
  }

  WantToBuildStatus::ReadyToBuild
}
