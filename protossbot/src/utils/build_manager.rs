use rsbwapi::{Game, Order, Player, Unit, UnitType};

use crate::{
  state::game_state::{BuildHistoryEntry, GameState, IntendedCommand},
  utils::build_location_utils,
};

pub fn on_frame(game: &Game, player: &Player, state: &mut GameState) {
  cleanup_stale_commands(player, state);
  check_and_advance_stage(player, state);
  state.stage_item_status = get_status_for_stage_items(game, player, state);
  try_start_next_build(game, player, state);
}

pub fn on_building_create(unit: &Unit, state: &mut GameState) {
  if let Some(entry) = state
    .unit_build_history
    .iter()
    .rev()
    .find(|e| e.unit_type == Some(unit.get_type()))
  {
    if let Some(probe_id) = entry.assigned_unit_id {
      state.intended_commands.remove(&probe_id);
      println!(
        "Building {} started. Removed assignment for probe {}",
        unit.get_type().name(),
        probe_id
      );
    }
  }
}

fn cleanup_stale_commands(player: &Player, state: &mut GameState) {
  let unit_ids: Vec<usize> = player.get_units().iter().map(|u| u.get_id()).collect();
  state.intended_commands.retain(|unit_id, cmd| {
    if !unit_ids.contains(unit_id) {
      return false;
    }
    if let Some(unit) = player.get_units().iter().find(|u| u.get_id() == *unit_id) {
      if cmd.order == Order::PlaceBuilding {
        return unit.is_constructing() || unit.get_order() == Order::PlaceBuilding;
      }
      if cmd.order == Order::Train {
        return unit.is_training();
      }
    }
    false
  });
}

fn try_start_next_build(game: &Game, player: &Player, state: &mut GameState) {
  if !should_start_next_build(game, player, state) {
    return;
  }
  let Some(unit_type) = get_next_thing_to_build(game, player, state) else {
    return;
  };
  let Some(builder) = find_builder_for_unit(player, unit_type, state) else {
    return;
  };
  let builder_id = builder.get_id();
  if let Some((success, tile_pos)) =
    assign_builder_to_construct(game, player, &builder, unit_type, state)
  {
    if success {
      let entry = BuildHistoryEntry {
        unit_type: Some(unit_type),
        upgrade_type: None,
        assigned_unit_id: Some(builder_id),
        tile_position: tile_pos,
      };
      state.unit_build_history.push(entry);
      let current_stage = &state.build_stages[state.current_stage_index];
      println!(
        "Started building {} with unit {} (Stage: {})",
        unit_type.name(),
        builder_id,
        current_stage.name
      );
    }
  }
}

fn should_start_next_build(_game: &Game, _player: &Player, state: &mut GameState) -> bool {
  !has_pending_assignment(state)
}

fn has_pending_assignment(state: &GameState) -> bool {
  state.unit_build_history.iter().any(|entry| {
    if let Some(unit_id) = entry.assigned_unit_id {
      state.intended_commands.contains_key(&unit_id)
    } else {
      false
    }
  })
}

fn has_ongoing_constructions(state: &GameState, player: &Player) -> bool {
  state.unit_build_history.iter().any(|entry| {
    if let Some(unit_id) = entry.assigned_unit_id {
      if let Some(unit) = player.get_units().iter().find(|u| u.get_id() == unit_id) {
        return unit.is_constructing() || unit.is_training();
      }
    }
    false
  })
}

fn get_status_for_stage_items(
  _game: &Game,
  player: &Player,
  state: &GameState,
) -> std::collections::HashMap<String, String> {
  let mut status_map = std::collections::HashMap::new();
  let Some(current_stage) = state.build_stages.get(state.current_stage_index) else {
    return status_map;
  };
  for (unit_type, &desired_count) in &current_stage.desired_counts {
    let unit_name = unit_type.name().to_string();
    let current_count = count_units_of_type(player, state, *unit_type);
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
      if find_builder_for_unit(player, *unit_type, state).is_none() {
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
  status_map
}

fn get_next_thing_to_build(game: &Game, player: &Player, state: &GameState) -> Option<UnitType> {
  let current_stage = state.build_stages.get(state.current_stage_index)?;
  if let Some(pylon) = check_need_more_supply(game, player, state) {
    return Some(pylon);
  }
  let status_map = get_status_for_stage_items(game, player, state);
  let mut candidates = Vec::new();
  for (unit_type, &desired_count) in &current_stage.desired_counts {
    let current_count = count_units_of_type(player, state, *unit_type);
    if current_count >= desired_count {
      continue;
    }
    let status = status_map.get(&unit_type.name().to_string());
    if status.is_some() && status.unwrap().starts_with("Ready to build") {
      candidates.push(*unit_type);
    }
  }
  candidates
    .into_iter()
    .max_by_key(|unit_type| unit_type.mineral_price() + unit_type.gas_price())
}

fn check_need_more_supply(_game: &Game, player: &Player, _state: &GameState) -> Option<UnitType> {
  let supply_used = player.supply_used();
  let supply_total = player.supply_total();
  if supply_total == 0 {
    return None;
  }
  let supply_remaining = supply_total - supply_used;
  let threshold = ((supply_total as f32) * 0.15).ceil() as i32;
  if supply_remaining <= threshold && supply_total < 400 {
    let pylon_type = UnitType::Protoss_Pylon;
    if can_afford_unit(player, pylon_type) {
      return Some(pylon_type);
    }
  }
  None
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
        && !state.intended_commands.contains_key(&u.get_id())
    })
    .cloned()
}

fn assign_builder_to_construct(
  game: &Game,
  player: &Player,
  builder: &rsbwapi::Unit,
  unit_type: UnitType,
  state: &mut GameState,
) -> Option<(bool, Option<rsbwapi::TilePosition>)> {
  let builder_id = builder.get_id();
  if unit_type.is_building() {
    let build_location =
      build_location_utils::find_build_location(game, player, builder, unit_type, 42);
    if let Some(pos) = build_location {
      println!(
        "Attempting to build {} at {:?} with worker {} (currently at {:?})",
        unit_type.name(),
        pos,
        builder_id,
        builder.get_position()
      );
      match builder.build(unit_type, pos) {
        Ok(_) => {
          println!("Build command succeeded for {}", unit_type.name());
          let intended_cmd = IntendedCommand {
            order: Order::PlaceBuilding,
            target_position: Some(pos.to_position()),
            target_unit: None,
          };
          state.intended_commands.insert(builder_id, intended_cmd);
          Some((true, Some(pos)))
        }
        Err(e) => {
          println!("Build command FAILED for {}: {:?}", unit_type.name(), e);
          Some((false, None))
        }
      }
    } else {
      println!(
        "No valid build location found for {} by builder {}",
        unit_type.name(),
        builder.get_id()
      );
      Some((false, None))
    }
  } else {
    match builder.train(unit_type) {
      Ok(_) => {
        let intended_cmd = IntendedCommand {
          order: Order::Train,
          target_position: None,
          target_unit: None,
        };
        state.intended_commands.insert(builder_id, intended_cmd);
        Some((true, None))
      }
      Err(e) => {
        println!("Train command FAILED for {}: {:?}", unit_type.name(), e);
        Some((false, None))
      }
    }
  }
}

fn count_units_of_type(player: &Player, _state: &GameState, unit_type: UnitType) -> i32 {
  player
    .get_units()
    .iter()
    .filter(|u| u.get_type() == unit_type)
    .count() as i32
}

fn can_afford_unit(player: &Player, unit_type: UnitType) -> bool {
  let minerals = player.minerals();
  let gas = player.gas();
  minerals >= unit_type.mineral_price() && gas >= unit_type.gas_price()
}

fn check_and_advance_stage(player: &Player, state: &mut GameState) {
  let Some(current_stage) = state.build_stages.get(state.current_stage_index) else {
    return;
  };
  let stage_complete = current_stage
    .desired_counts
    .iter()
    .all(|(unit_type, &desired_count)| {
      let current_count = count_units_of_type(player, state, *unit_type);
      current_count >= desired_count
    });
  if stage_complete {
    let next_stage_index = state.current_stage_index + 1;
    if next_stage_index < state.build_stages.len() {
      println!(
        "Stage '{}' complete! Advancing to stage {}",
        current_stage.name, state.build_stages[next_stage_index].name
      );
      state.current_stage_index = next_stage_index;
    }
  }
}

pub fn print_debug_build_status(game: &Game, player: &Player, state: &GameState) {
  let mut y = 10;
  let x = 3;
  if let Some(current_stage) = state.build_stages.get(state.current_stage_index) {
    let next_build = get_next_thing_to_build(game, player, state);
    let next_build_str = if let Some(unit_type) = next_build {
      format!(
        "Next: {} ({}/{} M, {}/{} G)",
        unit_type.name(),
        player.minerals(),
        unit_type.mineral_price(),
        player.gas(),
        unit_type.gas_price()
      )
    } else {
      "Next: None".to_string()
    };
    game.draw_text_screen((x, y), &next_build_str);
    y += 10;
    if let Some(last_entry) = state.unit_build_history.last() {
      let unit_name = if let Some(unit_type) = last_entry.unit_type {
        unit_type.name()
      } else if let Some(_upgrade) = last_entry.upgrade_type {
        "Upgrade"
      } else {
        "Unknown"
      };
      game.draw_text_screen((x, y), &format!("Last Built: {}", unit_name));
    } else {
      game.draw_text_screen((x, y), "Last Built: None");
    }
    y += 10;
    game.draw_text_screen((x, y), "Stage Progress:");
    y += 10;
    for (unit_type, &desired_count) in &current_stage.desired_counts {
      let current_count = count_units_of_type(player, state, *unit_type);
      game.draw_text_screen(
        (x + 10, y),
        &format!("{}: {}/{}", unit_type.name(), current_count, desired_count),
      );
      y += 10;
    }
  }
  let has_pending = state.unit_build_history.iter().any(|entry| {
    if let Some(unit_id) = entry.assigned_unit_id {
      state.intended_commands.contains_key(&unit_id)
    } else {
      false
    }
  });
  for entry in &state.unit_build_history {
    if let (Some(unit_type), Some(tile_pos), Some(unit_id)) =
      (entry.unit_type, entry.tile_position, entry.assigned_unit_id)
    {
      if state.intended_commands.contains_key(&unit_id) {
        let pixel_x = tile_pos.x * 32;
        let pixel_y = tile_pos.y * 32;
        let width = unit_type.tile_width() * 32;
        let height = unit_type.tile_height() * 32;
        use rsbwapi::{Color, Position};
        let top_left = Position {
          x: pixel_x,
          y: pixel_y,
        };
        let top_right = Position {
          x: pixel_x + width,
          y: pixel_y,
        };
        let bottom_left = Position {
          x: pixel_x,
          y: pixel_y + height,
        };
        let bottom_right = Position {
          x: pixel_x + width,
          y: pixel_y + height,
        };
        let color = Color::Yellow;
        game.draw_line_map(top_left, top_right, color);
        game.draw_line_map(top_right, bottom_right, color);
        game.draw_line_map(bottom_right, bottom_left, color);
        game.draw_line_map(bottom_left, top_left, color);
        let center = Position {
          x: pixel_x + width / 2,
          y: pixel_y + height / 2,
        };
        game.draw_text_map(center, &format!("Pending: {}", unit_type.name()));
      }
    }
  }
  if has_pending {
    use rsbwapi::{Color, Position, TilePosition};
    if let Some(nexus) = player
      .get_units()
      .iter()
      .find(|u| u.get_type() == UnitType::Protoss_Nexus)
    {
      let nexus_tile = nexus.get_tile_position();
      let radius = 30;
      for dx in -radius..=radius {
        for dy in -radius..=radius {
          let tile_x = nexus_tile.x + dx;
          let tile_y = nexus_tile.y + dy;
          if tile_x < 0 || tile_y < 0 {
            continue;
          }
          let tile_pos = TilePosition {
            x: tile_x,
            y: tile_y,
          };
          let dist_sq = (dx * dx + dy * dy) as f32;
          if dist_sq > (radius * radius) as f32 {
            continue;
          }
          let is_buildable = game.is_buildable(tile_pos);
          if !is_buildable {
            continue;
          }
          let has_power = game.has_power(tile_pos, (1, 1));
          let pixel_x = tile_x * 32;
          let pixel_y = tile_y * 32;
          let center = Position {
            x: pixel_x + 16,
            y: pixel_y + 16,
          };
          let color = if has_power { Color::Green } else { Color::Blue };
          game.draw_circle_map(center, 3, color, true);
        }
      }
    }
  }
}
