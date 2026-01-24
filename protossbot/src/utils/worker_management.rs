use rsbwapi::{Game, Player, Unit};

use crate::state::game_state::GameState;

pub fn assign_idle_workers_to_minerals(game: &Game, player: &Player, state: &mut GameState) {
  let all_units = player.get_units();
  let workers: Vec<Unit> = all_units
    .iter()
    .filter(|u| {
      // Worker must be a completed worker unit
      u.get_type().is_worker() && u.is_completed()
    })
    .cloned()
    .collect();

  // Assign idle workers that are not already assigned in the build history
  for worker in workers {
    // Skip if this worker is already recorded as assigned in any ongoing build history entry
    let already_assigned = state.unit_build_history.iter().any(|entry| {
      entry.assigned_unit_id == Some(worker.get_id())
        && entry.unit_type.map(|ut| ut.is_building()).unwrap_or(false)
    });
    if already_assigned {
      continue;
    }
    assign_worker_to_mineral(game, &worker, state);
  }
}

fn assign_worker_to_mineral(game: &Game, worker: &Unit, state: &mut GameState) {
  let worker_id = worker.get_id();

  // Skip if the worker is currently assigned to an in‑progress building construction.
  let has_in_progress_build = state.unit_build_history.iter().any(|entry| {
    entry.assigned_unit_id == Some(worker_id)
      && entry.unit_type.map(|ut| ut.is_building()).unwrap_or(false)
  });
  if has_in_progress_build {
    return;
  }

  // Worker must be idle and not already gathering resources.
  if !worker.is_idle() || worker.is_gathering_minerals() || worker.is_gathering_gas() {
    return;
  }

  // Find a mineral patch to assign.
  let Some(mineral) = find_available_mineral(game, worker, state) else {
    return;
  };

  // Assign worker to gather minerals (ignore intended command tracking).
  if worker.gather(&mineral).is_ok() {
    println!("Assigned worker {} to mine from mineral", worker_id,);
  }
}

fn find_available_mineral(game: &Game, worker: &Unit, _state: &GameState) -> Option<Unit> {
  let worker_pos = worker.get_position();
  let minerals = game.get_static_minerals();
  let mut mineral_list: Vec<Unit> = minerals.iter().filter(|m| m.exists()).cloned().collect();

  // Sort minerals by distance to the worker.
  mineral_list.sort_by_key(|m| {
    let pos = m.get_position();
    ((pos.x - worker_pos.x).pow(2) + (pos.y - worker_pos.y).pow(2)) as i32
  });

  // Return the closest mineral, ignoring any intended command tracking.
  mineral_list.first().cloned()
}
