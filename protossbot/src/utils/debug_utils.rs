use rsbwapi::*;

use crate::{
  state::game_state::{BuildStatus, GameState},
  utils::build_manager,
};
pub fn print_debug_build_status(game: &Game, player: &Player, state: &GameState) {
  print_next_build_item(game, player, state);
  print_pending_buildings(game, state);
  print_base_locations(game, state);
}

pub fn print_next_build_item(game: &Game, player: &Player, state: &GameState) {
    let next_build = build_manager::get_next_thing_to_build(game, player, state);
    use crate::utils::build_manager::NextBuildItem;
    let next_build_str = match next_build {
        Some(NextBuildItem::Unit(unit_type)) => {
            format!(
                "Next: {} ({}/{} M, {}/{} G)",
                unit_type.name(),
                player.minerals(),
                unit_type.mineral_price(),
                player.gas(),
                unit_type.gas_price()
            )
        }
        Some(NextBuildItem::Upgrade(upgrade_type)) => {
          // Use 0 (Terran) as the race argument for upgrade costs
          let minerals = upgrade_type.mineral_price(0);
          let gas = upgrade_type.gas_price(0);
          format!(
            "Next: Upgrade {:?} ({} M, {} G)",
            upgrade_type,
            minerals,
            gas
          )
        }
        None => "Next: None".to_string(),
    };
    game.draw_text_screen((0, 10), &next_build_str);
}

pub fn print_pending_buildings(game: &Game, state: &GameState) {
  for entry in &state.unit_build_history {
    if entry.status == BuildStatus::Started {
      continue;
    }
    if let Some(building_type) = entry.unit_type {
      if !building_type.is_building() {
        continue;
      }
      let Some(target_location) = entry.tile_position else {
        continue;
      };
      let tile_size = building_type.tile_size();
      // Convert tile coordinates to pixel coordinates (32 pixels per tile)
      let top_left = (target_location.x * 32, target_location.y * 32);
      let bottom_right = (
        (target_location.x + tile_size.x as i32) * 32,
        (target_location.y + tile_size.y as i32) * 32,
      );

      game.draw_box_map(top_left, bottom_right, Color::Green, false);
    }
  }
}
pub fn print_base_locations(game: &Game, state: &GameState) {
  for (index, base) in state.base_locations.iter().enumerate() {
    let pos = (base.position.x * 32, base.position.y * 32);
    game.draw_text_map(pos, &format!("Base {}", index));
    game.draw_circle_map(pos, 4, Color::Blue, true);

    // for checked in &base.checked_positions {
    //   let cpos = (checked.tile_position.x * 32, checked.tile_position.y * 32);
    //   if checked.is_valid {
    //     game.draw_circle_map(cpos, 2, Color::Green, true);
    //   } else {
    //     game.draw_circle_map(cpos, 2, Color::Red, true);
    //   }
    // }
  }
}
