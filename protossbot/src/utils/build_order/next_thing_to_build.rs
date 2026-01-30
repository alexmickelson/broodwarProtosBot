use std::collections::HashMap;

use rsbwapi::*;

use crate::{state::game_state::{BuildStatus, GameState}, utils::{build_order::build_manager::NextBuildItem, unit_utils}};



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
    .filter(|u| u.get_type() == UnitType::Terran_Supply_Depot && !u.is_completed() && u.exists())
    .count() as i32;

  let supply_ratio = supply_used as f32 / (supply_total + supply_depots_in_progress * 8) as f32;
  return supply_ratio >= 0.85 || (supply_total - supply_used) <= 1;
}
pub fn get_status_for_stage_items(
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
    let current_count = unit_utils::count_units_of_type(player, *unit_type);
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
      if unit_utils::find_builder_for_unit(player, *unit_type, state).is_none() {
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


fn can_afford_unit(player: &Player, unit_type: UnitType) -> bool {
  let minerals = player.minerals();
  let gas = player.gas();
  minerals >= unit_type.mineral_price() && gas >= unit_type.gas_price()
}
