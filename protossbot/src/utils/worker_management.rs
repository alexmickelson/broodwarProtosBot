use rsbwapi::{Game, Player, Unit};

use crate::state::game_state::{BuildStatus, GameState};

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

  for worker in workers {
    let already_assigned = state.unit_build_history.iter().any(|entry| {
      entry.assigned_unit_id == Some(worker.get_id()) && entry.status == BuildStatus::Assigned
    });
    if already_assigned {
      continue;
    }
    assign_worker_to_mineral(game, &worker, state);
  }
}

fn assign_worker_to_mineral(game: &Game, worker: &Unit, state: &mut GameState) {
  let worker_id = worker.get_id();

  if !worker.is_idle() || worker.is_gathering_minerals() || worker.is_gathering_gas() {
    return;
  }

  let Some(mineral) = find_available_mineral(game, worker, state) else {
    return;
  };

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
