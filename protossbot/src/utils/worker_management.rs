use rand::seq::SliceRandom;
use rsbwapi::{Game, Order, Player, Unit};

use crate::state::game_state::{BuildStatus, GameState};

pub fn worker_onframe(game: &Game, player: &Player, state: &mut GameState) {
  let all_units = player.get_units();
  let workers: Vec<Unit> = all_units
    .iter()
    .filter(|u| {
      // Worker must be a completed worker unit
      u.get_type().is_worker() && u.is_completed()
    })
    .cloned()
    .collect();

  prevent_too_many_gas_workers(game, player, state, &workers);

  for worker in workers {
    let already_assigned = state.unit_build_history.iter().any(|entry| {
      entry.assigned_unit_id == Some(worker.get_id()) && entry.status == BuildStatus::Assigned
    });
    if already_assigned {
      continue;
    }

    if should_assign_to_refinery(&worker, player, state) {
      assign_worker_to_refinery(game, player, &worker, state);
    } else {
      assign_worker_to_mineral(game, player, &worker, state);
    }
  }
}

fn prevent_too_many_gas_workers(
  game: &Game,
  player: &Player,
  state: &mut GameState,
  workers: &Vec<Unit>,
) {
  let workers_mining_gas: Vec<Unit> = workers
    .iter()
    .filter(|u| u.is_gathering_gas())
    .cloned()
    .collect();

  for worker in &workers_mining_gas {
    let refinery_assigned = state.worker_refinery_assignments.get(&worker.get_id());
    if !refinery_assigned.is_some() {
      assign_worker_to_mineral(game, player, worker, state);
    }
  }
}

fn should_assign_to_refinery(worker: &Unit, player: &Player, state: &GameState) -> bool {
  let refineries = player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type().is_refinery() && u.exists() && u.is_completed())
    .collect::<Vec<Unit>>();

  for refinery in refineries {
    let assigned_workers_count = state
      .worker_refinery_assignments
      .values()
      .filter(|&&r_id| r_id == refinery.get_id())
      .count();
    if assigned_workers_count < 3 {
      return true;
    }
  }
  false
}

fn assign_worker_to_refinery(game: &Game, player: &Player, worker: &Unit, state: &mut GameState) {
  let worker_id = worker.get_id();
  let Some(refinery) = player
    .get_units()
    .into_iter()
    .find(|u| u.get_type().is_refinery() && u.exists() && u.is_completed())
  else {
    return;
  };

  let Some(worker) = game.get_unit(worker_id) else {
    return;
  };

  if worker.gather(&refinery).is_ok() {
    state
      .worker_refinery_assignments
      .insert(worker_id, refinery.get_id());
  }
}

fn assign_worker_to_mineral(game: &Game, player: &Player, worker: &Unit, state: &mut GameState) {
  let worker_id = worker.get_id();

  if !worker.is_idle() || worker.is_gathering_minerals() || worker.is_gathering_gas() {
    return;
  }

  let Some(mineral) = find_available_mineral(game, player, worker, state) else {
    println!("No available mineral found for worker {}", worker_id);
    return;
  };

  if worker.gather(&mineral).is_ok() {
    println!("Assigned worker {} to mine from mineral", worker_id,);
  }
}

fn find_available_mineral(
  game: &Game,
  player: &Player,
  worker: &Unit,
  _state: &GameState,
) -> Option<Unit> {
  let command_centers: Vec<Unit> = player
    .get_units()
    .into_iter()
    .filter(|u| u.get_type() == rsbwapi::UnitType::Terran_Command_Center)
    .collect();

  let minerals_close_to_command_centers = game
    .get_static_minerals()
    .into_iter()
    .filter(|mineral| {
      if !mineral.exists() {
        return false;
      }
      let mineral_pos = mineral.get_position();
      command_centers.iter().any(|cc| {
        let cc_pos = cc.get_position();
        let dx = (mineral_pos.x - cc_pos.x) as f64;
        let dy = (mineral_pos.y - cc_pos.y) as f64;
        // 320 pixels = 10 tiles (32 px per tile)
        (dx * dx + dy * dy).sqrt() < 320.0
      })
    })
    .collect::<Vec<_>>();

  let random_mineral = minerals_close_to_command_centers
    .choose(&mut rand::thread_rng())
    .cloned();

  random_mineral
}
