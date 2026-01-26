use rsbwapi::*;

use crate::{
  state::game_state::{BuildHistoryEntry, BuildStatus, GameState},
  utils::build_location_utils,
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
  for entry in state.unit_build_history.iter_mut() {
    if entry.status != BuildStatus::Assigned {
      continue;
    }
    let Some(unit_type) = entry.unit_type else {
      continue;
    };
    let Some(builder_id) = entry.assigned_unit_id else {
      continue;
    };
    let Some(builder) = game.get_unit(builder_id) else {
      continue;
    };

    if !matches!(
      builder.get_order(),
      Order::ConstructingBuilding | Order::PlaceBuilding | Order::ResetCollision
    ) {
      println!(
        "Builder {} is not constructing (order: {:?}) for assigned build of {}",
        builder_id,
        builder.get_order(),
        unit_type.name()
      );
      let Some(new_location) =
        build_location_utils::find_build_location_default(game, player, &builder, unit_type)
      else {
        println!(
          "Cannot build {}: no valid location found for builder {}",
          unit_type.name(),
          builder_id
        );
        continue;
      };

      match builder.build(unit_type, new_location) {
        Ok(true) => {
          println!(
            "Restarted building {} by builder {}",
            unit_type.name(),
            builder_id
          );
          entry.tile_position = Some(new_location);
        }
        Ok(false) => {
          println!(
            "Restart build order failed for {} by builder {}",
            unit_type.name(),
            builder_id
          );
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
    build_location_utils::find_build_location_default(game, player, &builder, unit_type)
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
  _state: &GameState,
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
    })
    .cloned()
}
