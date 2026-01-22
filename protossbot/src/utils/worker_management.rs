use rsbwapi::{Game, Order, Player, Unit};

use crate::state::game_state::{GameState, IntendedCommand};

pub fn assign_idle_workers_to_minerals(game: &Game, player: &Player, state: &mut GameState) {
  let all_units = player.get_units();
  let workers: Vec<Unit> = all_units
    .iter()
    .filter(|u| u.get_type().is_worker() && u.is_completed())
    .cloned()
    .collect();

  // First, clean up workers that finished building
  reassign_finished_builders(game, &workers, state);

  // Then assign idle workers to mining
  for worker in workers {
    assign_worker_to_mineral(game, &worker, state);
  }
}

fn reassign_finished_builders(_game: &Game, workers: &[Unit], state: &mut GameState) {
  for worker in workers {
    let worker_id = worker.get_id();
    
    if let Some(cmd) = state.intended_commands.get(&worker_id) {
      // For building commands, only remove if the worker was constructing and now is not
      // This prevents premature removal when the worker is still moving to the build site
      if cmd.order == Order::PlaceBuilding {
        // Worker finished if it WAS constructing but no longer is and is idle
        // We check the actual order to see if it's moved on from building
        let current_order = worker.get_order();
        if worker.is_idle() && current_order != Order::PlaceBuilding && current_order != Order::ConstructingBuilding {
          println!("Worker {} finished building, reassigning to minerals", worker_id);
          state.intended_commands.remove(&worker_id);
        }
      } else if cmd.order == Order::Train && worker.is_idle() && !worker.is_training() {
        // For training, the original logic is fine
        state.intended_commands.remove(&worker_id);
      }
    }
  }
}

fn assign_worker_to_mineral(game: &Game, worker: &Unit, state: &mut GameState) {
  let worker_id = worker.get_id();

  // Don't reassign workers that have a non-mining intended command
  if let Some(cmd) = state.intended_commands.get(&worker_id) {
    if cmd.order != Order::MiningMinerals {
      return;
    }
  }

  // Only assign idle workers
  if !worker.is_idle() {
    return;
  }

  if worker.is_gathering_minerals() || worker.is_gathering_gas() {
    return;
  }

  let Some(mineral) = find_available_mineral(game, worker, state) else {
    return;
  };

  let intended_cmd = IntendedCommand {
    order: Order::MiningMinerals,
    target_position: None,
    target_unit: Some(mineral.clone()),
  };

  state.intended_commands.insert(worker_id, intended_cmd);

  println!(
    "Worker {} current order: {:?}, assigning to mine from mineral at {:?}",
    worker_id,
    worker.get_order(),
    mineral.get_position()
  );

  if worker.gather(&mineral).is_ok() {
    println!(
      "Assigned worker {} to mine from mineral at {:?}",
      worker_id,
      mineral.get_position()
    );
  }
}

fn find_available_mineral(game: &Game, worker: &Unit, state: &GameState) -> Option<Unit> {
  let worker_pos = worker.get_position();
  let minerals = game.get_static_minerals();
  let mut mineral_list: Vec<Unit> = minerals.iter().filter(|m| m.exists()).cloned().collect();

  mineral_list.sort_by_key(|m| {
    let pos = m.get_position();
    ((pos.x - worker_pos.x).pow(2) + (pos.y - worker_pos.y).pow(2)) as i32
  });

  for mineral in mineral_list.iter() {
    let mineral_id: usize = mineral.get_id();

    let worker_count = state
      .intended_commands
      .values()
      .filter(|cmd| {
        if let Some(target) = &cmd.target_unit {
          target.get_id() == mineral_id
        } else {
          false
        }
      })
      .count();

    if worker_count < 2 {
      return Some(mineral.clone());
    }
  }
  mineral_list.first().cloned()
}
