use rand::seq::SliceRandom;
use rand::thread_rng;
use rsbwapi::{Game, Player, TilePosition, Unit, UnitType};
pub fn find_build_location_default(
  game: &Game,
  player: &Player,
  builder: &Unit,
  building_type: UnitType,
) -> Option<TilePosition> {
  find_build_location(game, player, builder, building_type, 10)
}

pub fn find_build_location(
  game: &Game,
  player: &Player,
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

  let start = builder.get_tile_position();

  for (dx, dy) in generate_shuffled_coordinates(max_range) {
    let cand = TilePosition {
      x: start.x + dx,
      y: start.y + dy,
    };
    if !is_in_bounds(game, &cand) {
      continue;
    }
    if is_viable_location(game, player, builder, cand, building_type) {
      return Some(cand);
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

fn is_in_bounds(game: &Game, pos: &TilePosition) -> bool {
  let w = game.map_width();
  let h = game.map_height();
  pos.x >= 0 && pos.x < w && pos.y >= 0 && pos.y < h
}

fn is_viable_location(
  game: &Game,
  _player: &Player,
  builder: &Unit,
  tile_position: TilePosition,
  building_type: UnitType,
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

  true
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

pub fn get_base_locations(game: &Game, player: &Player) -> Vec<TilePosition> {
  let minerals: Vec<Unit> = game
    .get_static_minerals()
    .iter()
    .filter(|m| m.exists())
    .cloned()
    .collect();
  let mut clusters: Vec<Vec<Unit>> = Vec::new();
  let mut assigned = vec![false; minerals.len()];
  let threshold = 5.0; // tiles

  for (i, mineral) in minerals.iter().enumerate() {
    if assigned[i] {
      continue;
    }
    let mut cluster = vec![mineral.clone()];
    assigned[i] = true;

    let mut changed = true;
    while changed {
      changed = false;
      for (j, other) in minerals.iter().enumerate() {
        if assigned[j] {
          continue;
        }
        // If any mineral in cluster is close to this one, add it
        if cluster.iter().any(|m| {
          let a = m.get_tile_position();
          let b = other.get_tile_position();
          let dist = ((a.x - b.x).pow(2) + (a.y - b.y).pow(2)) as f64;
          dist.sqrt() <= threshold
        }) {
          cluster.push(other.clone());
          assigned[j] = true;
          changed = true;
        }
      }
    }
    clusters.push(cluster);
  }

  let base_locations = clusters
    .into_iter()
    .map(|cluster| get_base_position_for_cluster(game, cluster))
    .collect::<Vec<TilePosition>>();

  let Some(first_building) = player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type().is_building())
    .min_by_key(|u| u.get_id())
  else {
    println!("No buildings found for player {}", player.get_name());
    return Vec::new();
  };

  let start_locations: Vec<TilePosition> = game.get_start_locations().clone();
  let Some(player_start_location) = start_locations
    .iter()
    .min_by(|a, b| {
      let a_pos = *a;
      let b_pos = *b;
      let first_building_pos = first_building.get_tile_position();
      let dist_a =
        ((a_pos.x - first_building_pos.x).pow(2) + (a_pos.y - first_building_pos.y).pow(2)) as i32;
      let dist_b =
        ((b_pos.x - first_building_pos.x).pow(2) + (b_pos.y - first_building_pos.y).pow(2)) as i32;
      dist_a.cmp(&dist_b)
    })
    .cloned()
  else {
    println!("No start locations found in game");
    return Vec::new();
  };

  let mut ordered_locations_by_distance_from_start_location = base_locations;
  ordered_locations_by_distance_from_start_location.sort_by(|a, b| {
    let start_pos = player_start_location;
    let dist_a = ((a.x - start_pos.x).pow(2) + (a.y - start_pos.y).pow(2)) as i32;
    let dist_b = ((b.x - start_pos.x).pow(2) + (b.y - start_pos.y).pow(2)) as i32;
    dist_a.cmp(&dist_b)
  });
  ordered_locations_by_distance_from_start_location
}

fn get_base_position_for_cluster(game: &Game, cluster: Vec<Unit>) -> TilePosition {
  if cluster.is_empty() {
    return TilePosition { x: 0, y: 0 };
  }

  let mut candidate_positions = std::collections::HashSet::new();
  for unit in &cluster {
    let pos = unit.get_tile_position();
    for dx in -36..=36 {
      for dy in -36..=36 {
        let cand = TilePosition {
          x: pos.x + dx,
          y: pos.y + dy,
        };
        if game
          .can_build_here(None, cand, UnitType::Terran_Command_Center, false)
          .unwrap_or(false)
        {
          candidate_positions.insert(cand);
        }
      }
    }
  }

  candidate_positions
    .into_iter()
    .min_by_key(|cand| {
      cluster
        .iter()
        .map(|u| {
          let upos = u.get_tile_position();
          let dx = cand.x - upos.x;
          let dy = cand.y - upos.y;
          dx * dx + dy * dy
        })
        .sum::<i32>()
    })
    .unwrap_or_else(|| cluster[0].get_tile_position())
}
