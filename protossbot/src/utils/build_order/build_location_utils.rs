use crate::state::game_state::GameState;
pub use crate::utils::build_order::base_location_utils::get_base_locations;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rsbwapi::{Game, Player, TilePosition, Unit, UnitType};

pub fn find_build_location_default(
  game: &Game,
  player: &Player,
  game_state: &GameState,
  builder: &Unit,
  building_type: UnitType,
) -> Option<TilePosition> {
  find_build_location(game, player, game_state, builder, building_type, 15)
}

pub fn find_build_location(
  game: &Game,
  player: &Player,
  game_state: &GameState,
  builder: &Unit,
  building_type: UnitType,
  max_range: i32,
) -> Option<TilePosition> {
  if building_type.is_refinery() {
    for geyser in game.get_geysers() {
      let tp = geyser.get_tile_position();
      if let Ok(true) = game.can_build_here(Some(builder), tp, building_type, false) {
        return Some(tp);
      }
    }
    return None;
  }

  if building_type == UnitType::Terran_Command_Center {
    return get_command_center_location(game, player, game_state);
  }

  let bases = game_state.base_locations.clone();

  let coordinates_to_check = generate_shuffled_coordinates(max_range);

  for base in bases.clone() {
    for (dx, dy) in &coordinates_to_check {
      let candidate = TilePosition {
        x: base.position.x + dx,
        y: base.position.y + dy,
      };
      if is_viable_location(game, player, builder, candidate, building_type, game_state) {
        return Some(candidate);
      }
    }
  }

  None
}

fn generate_shuffled_coordinates(max_range: i32) -> Vec<(i32, i32)> {
  let mut coords = Vec::new();
  for dx in -max_range..=max_range {
    for dy in -max_range..=max_range {
      coords.push((dx, dy));
    }
  }
  let mut rng = thread_rng();
  coords.as_mut_slice().shuffle(&mut rng);
  coords
}

fn get_command_center_location(
  game: &Game,
  player: &Player,
  game_state: &GameState,
) -> Option<TilePosition> {
  let bases = game_state.base_locations.iter().skip(1).collect::<Vec<_>>();
  let player_command_centers: Vec<Unit> = player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type() == UnitType::Terran_Command_Center && u.exists())
    .collect();

  let Some(next_base_without_close_command_center) = bases.iter().find(|base| {
    !player_command_centers.iter().any(|cc| {
      let cc_pos = cc.get_tile_position();
      let dist = ((cc_pos.x - base.position.x).pow(2) + (cc_pos.y - base.position.y).pow(2)) as f64;
      dist.sqrt() < 5.0 // 5 tiles threshold
    })
  }) else {
    return None;
  };

  Some(next_base_without_close_command_center.position)
}

fn is_viable_location(
  game: &Game,
  player: &Player,
  builder: &Unit,
  tile_position: TilePosition,
  building_type: UnitType,
  game_state: &GameState,
) -> bool {
  let is_buildable = match game.can_build_here(builder, tile_position, building_type, true) {
    Ok(can_build) => can_build,
    Err(_) => false,
  };
  if !is_buildable {
    return false;
  }

  if blocks_worker_path(game, &tile_position, building_type) {
    return false;
  }

  if is_next_to_other_building(player, tile_position, building_type) {
    return false;
  }
  if building_covers_command_center_location(tile_position, building_type, game_state) {
    return false;
  }
  true
}

fn building_covers_command_center_location(
  tile_position: TilePosition,
  building_type: UnitType,
  game_state: &GameState,
) -> bool {
  let building_width = building_type.tile_width();
  let building_height = building_type.tile_height();
  let cc_width = UnitType::Terran_Command_Center.tile_width();
  let cc_height = UnitType::Terran_Command_Center.tile_height();
  for base in &game_state.base_locations {
    let base_x = base.position.x;
    let base_y = base.position.y;
    // Check if any tile of the building would overlap any tile of the command center footprint
    for bx in 0..building_width {
      for by in 0..building_height {
        let px = tile_position.x + bx;
        let py = tile_position.y + by;
        for cc_bx in 0..cc_width {
          for cc_by in 0..cc_height {
            let cc_px = base_x + cc_bx;
            let cc_py = base_y + cc_by;
            if px == cc_px && py == cc_py {
              return true;
            }
          }
        }
      }
    }
  }
  false
}

fn blocks_worker_path(game: &Game, tile_position: &TilePosition, building_type: UnitType) -> bool {
  // Get all resource depots (command centers, etc.) for the player
  let command_centers: Vec<Unit> = game
    .self_()
    .map(|player| {
      player
        .get_units()
        .iter()
        .filter(|u| u.get_type().is_resource_depot())
        .cloned()
        .collect()
    })
    .unwrap_or_else(Vec::new);

  let building_size = (building_type.tile_width(), building_type.tile_height());

  for cc in &command_centers {
    let cc_pos = cc.get_tile_position();
    for bx in 0..building_size.0 {
      for by in 0..building_size.1 {
        let px = tile_position.x + bx;
        let py = tile_position.y + by;
        // Distance in tiles
        let dist = ((px - cc_pos.x).pow(2) + (py - cc_pos.y).pow(2)) as f64;
        // 96 pixels = 3 tiles (since 1 tile = 32 px)
        if dist.sqrt() < 3.0 {
          return true;
        }
      }
    }
  }
  false
}

fn is_next_to_other_building(
  player: &Player,
  tile_position: TilePosition,
  building_type: UnitType,
) -> bool {
  let building_width = building_type.tile_width();
  let building_height = building_type.tile_height();
  for unit in player.get_units() {
    if !unit.get_type().is_building() || !unit.exists() {
      continue;
    }
    let other_pos = unit.get_tile_position();
    let other_width = unit.get_type().tile_width();
    let other_height = unit.get_type().tile_height();
    // Calculate the bounding boxes with a 1-tile buffer around each building
    let min_x = tile_position.x - 1;
    let max_x = tile_position.x + building_width;
    let min_y = tile_position.y - 1;
    let max_y = tile_position.y + building_height;
    let other_min_x = other_pos.x - 1;
    let other_max_x = other_pos.x + other_width;
    let other_min_y = other_pos.y - 1;
    let other_max_y = other_pos.y + other_height;
    // Check if the bounding boxes touch or overlap (no gap)
    let overlap_x = min_x < other_max_x && max_x > other_min_x;
    let overlap_y = min_y < other_max_y && max_y > other_min_y;
    if overlap_x && overlap_y {
      return true;
    }
  }
  false
}
