use rsbwapi::*;

use crate::{
  state::game_state::{BuildHistoryEntry, BuildStatus, GameState},
  utils::build_order::build_location_utils,
};

pub fn check_if_building_started(game: &Game, player: &Player, state: &mut GameState) {
  let started_history_items = state
    .unit_build_history
    .iter_mut()
    .filter(|entry| entry.status == BuildStatus::Assigned)
    .collect::<Vec<&mut BuildHistoryEntry>>();

  let buildings_under_construction: Vec<Unit> = player
    .get_units()
    .iter()
    .filter(|u| u.is_constructing())
    .cloned()
    .collect();

  for entry in started_history_items {
    let Some(unit_type) = entry.unit_type else {
      continue;
    };
    let Some(builder_id) = entry.assigned_unit_id else {
      continue;
    };
    let Some(builder) = game.get_unit(builder_id) else {
      continue;
    };

    let builder_order = builder.get_order();

    let building_type_is_under_construction = buildings_under_construction
      .iter()
      .any(|b| b.get_type() == unit_type);

    // println!(
    //   "Builder {} assigned to build {:?}, under construction: {:?}, order: {:?}",
    //   builder_id, unit_type, building_type_is_under_construction, builder_order
    // );
    if builder_order == Order::ConstructingBuilding && building_type_is_under_construction {
      println!(
        "Builder {} has started constructing {:?}",
        builder_id, unit_type
      );
      entry.status = BuildStatus::Started;
    }
  }
}

pub fn try_restart_failed_builing_builds(game: &Game, player: &Player, state: &mut GameState) {
  let mut to_restart = Vec::new();

  for (idx, entry) in state.unit_build_history.iter().enumerate() {
    if entry.status != BuildStatus::Assigned {
      continue;
    }
    let (Some(unit_type), Some(builder_id)) = (entry.unit_type, entry.assigned_unit_id) else {
      continue;
    };
    let Some(builder) = game.get_unit(builder_id) else {
      continue;
    };

    if !matches!(
      builder.get_order(),
      Order::ConstructingBuilding | Order::PlaceBuilding | Order::ResetCollision
    ) {
      to_restart.push((idx, unit_type, builder_id));
    }
  }

  for (idx, unit_type, builder_id) in to_restart {
    let Some(builder) = game.get_unit(builder_id) else {
      continue;
    };

    let Some(new_location) =
      build_location_utils::find_build_location_default(game, player, state, &builder, unit_type)
    else {
      println!(
        "Cannot build {}: no valid location found for builder {}, tick {}",
        unit_type.name(),
        builder_id,
        game.get_frame_count()
      );
      continue;
    };

    let old_order = builder.get_order();

    if unit_type == UnitType::Terran_Command_Center && old_order == Order::Move {
      println!(
        "Skipping restart of Command Center build by builder {} is moving",
        builder_id
      );
      continue;
    }

    match builder.build(unit_type, new_location) {
      Ok(true) => {
        println!(
          "Restarted building {} by builder {}",
          unit_type.name(),
          builder_id
        );
        state.unit_build_history[idx].tile_position = Some(new_location);
      }
      Ok(false) => {
        println!(
          "Restart build order failed for {} by builder {} (old order: {:?})",
          unit_type.name(),
          builder_id,
          old_order
        );
        explore_location_for_building(game, &new_location, builder_id, unit_type);
      }
      Err(Error::Incompatible_UnitType) => {
        println!(
          "Incompatible UnitType for {} by builder {}, reassigning",
          unit_type.name(),
          builder_id,
        );
        let new_builder = find_builder_for_unit(player, unit_type, state);
        if let Some(new_builder) = new_builder {
          state.unit_build_history[idx].assigned_unit_id = Some(new_builder.get_id());
        }
      }
      Err(e) => {
        println!(
          "Restart build order FAILED for {} by builder {}: {:?}",
          unit_type.name(),
          builder_id,
          e
        );
      }
    }
  }
}

pub fn start_building_construction(
  game: &Game,
  player: &Player,
  state: &mut GameState,
  unit_type: UnitType,
) {
  let Some(builder) = find_builder_for_unit(player, unit_type, state) else {
    println!("No builder available to train {:?}", unit_type);
    return;
  };
  let builder_id = builder.get_id();

  let Some(building_location) =
    build_location_utils::find_build_location_default(game, player, state, &builder, unit_type)
  else {
    state.unit_build_history.push(BuildHistoryEntry {
      unit_type: Some(unit_type),
      upgrade_type: None,
      assigned_unit_id: Some(builder_id),
      tile_position: None,
      status: BuildStatus::Assigned,
    });
    return;
  };
  state.unit_build_history.push(BuildHistoryEntry {
    unit_type: Some(unit_type),
    upgrade_type: None,
    assigned_unit_id: Some(builder_id),
    tile_position: Some(building_location),
    status: BuildStatus::Assigned,
  });

  match builder.build(unit_type, building_location) {
    Ok(true) => {
      println!(
        "Started building {} by builder {}",
        unit_type.name(),
        builder_id
      );
    }
    Ok(false) => {
      println!(
        "Build order failed for {} by builder {}",
        unit_type.name(),
        builder_id
      );
      explore_location_for_building(game, &building_location, builder_id, unit_type);
    }
    Err(e) => {
      println!(
        "Build order FAILED for {} by builder {}: {:?}",
        unit_type.name(),
        builder_id,
        e
      );
    }
  }
}

fn find_builder_for_unit(
  player: &Player,
  unit_type: UnitType,
  state: &GameState,
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
        && !state.worker_refinery_assignments.contains_key(&u.get_id())
    })
    .cloned()
}

fn explore_location_for_building(
  game: &Game,
  location: &TilePosition,
  builder_id: usize,
  unit_type: UnitType,
) {
  let Some(builder) = game.get_unit(builder_id) else {
    return;
  };
  println!(
    "moving builder {} next to build location to build {}",
    builder_id,
    unit_type.name()
  );
  let build_pos = Position {
    x: (location.x * 32) - 16,
    y: (location.y * 32),
  };
  builder.move_(build_pos);
}
