use crate::{
  state::game_state::{BaseLocation, CheckedPosition},
  utils::build_buildings_utils,
};
use rsbwapi::{Game, TilePosition, Unit, UnitType};

pub fn get_base_locations(game: &Game, player: &rsbwapi::Player) -> Vec<BaseLocation> {
  let clusters = get_resource_clusters(game);
  let mut base_positions = clusters
    .into_iter()
    .map(|cluster| get_base_position_for_cluster(game, cluster))
    .collect::<Vec<TilePosition>>();

  let Some(player_start_location) = get_player_start_location(game, player) else {
    return Vec::new();
  };

  base_positions.sort_by(|a, b| {
    let start_pos = player_start_location;
    let dist_a = ((a.x - start_pos.x).pow(2) + (a.y - start_pos.y).pow(2)) as i32;
    let dist_b = ((b.x - start_pos.x).pow(2) + (b.y - start_pos.y).pow(2)) as i32;
    dist_a.cmp(&dist_b)
  });

  let mut result = Vec::new();
  for base_pos in base_positions {
    let mut checked_positions = Vec::new();
    // Example: check a 15x15 tile area around the base
    let radius = 3;


    for dx in -radius..=radius {
      for dy in -radius..=radius {
        let candidate = TilePosition {
          x: base_pos.x + dx,
          y: base_pos.y + dy,
        };
        let is_valid = is_in_bounds(game, &candidate)
          && game
            .can_build_here(None, candidate, UnitType::Terran_Command_Center, false)
            .unwrap_or(false);
        checked_positions.push(CheckedPosition {
          tile_position: candidate,
          is_valid,
        });
      }
    }
    result.push(BaseLocation {
      position: base_pos,
      checked_positions,
    });
  }
  result
}

fn get_player_start_location(game: &Game, player: &rsbwapi::Player) -> Option<TilePosition> {
  let first_building = player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type().is_building())
    .min_by_key(|u| u.get_id())?;
  let start_locations: Vec<TilePosition> = game.get_start_locations().clone();
  start_locations
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
}

fn get_resource_clusters(game: &Game) -> Vec<Vec<Unit>> {
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
  clusters
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

fn is_in_bounds(game: &Game, pos: &TilePosition) -> bool {
  let w = game.map_width();
  let h = game.map_height();
  pos.x >= 0 && pos.x < w && pos.y >= 0 && pos.y < h
}
