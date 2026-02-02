use rsbwapi::*;
use std::collections::HashMap;

pub type MapTileInformation = HashMap<TilePosition, TileDisplayInformation>;

#[derive(Clone, Debug)]
pub struct MapInformation {
  pub map_width: i32,
  pub map_height: i32,
  pub tile_information: MapTileInformation,
}

#[derive(Clone, Debug)]
pub struct TileDisplayInformation {
  pub is_walkable: bool,
  pub is_buildable: bool,
}

#[derive(Clone, Debug)]
pub struct UnitDisplayInformation {
  pub unit_type: UnitType,
  pub position: Position, // Pixel position
  pub unit_id: usize,
  pub unit_width: i32,
  pub unit_height: i32,
  pub player_id: Option<i32>,
  pub player_name: Option<String>,
}

pub fn get_map_information_from_game(game: &Game) -> MapInformation {
  let map_width = game.map_width();
  let map_height = game.map_height();
  let mut tile_information = HashMap::new();

  // Iterate through all tiles on the map
  for x in 0..map_width {
    for y in 0..map_height {
      let tile_pos = TilePosition { x, y };

      let tile_info = TileDisplayInformation {
        is_walkable: game.is_walkable(tile_pos.to_walk_position()),
        is_buildable: game.is_buildable(tile_pos),
      };

      tile_information.insert(tile_pos, tile_info);
    }
  }

  MapInformation {
    map_width,
    map_height,
    tile_information,
  }
}

pub fn get_unit_display_information(game: &Game) -> Vec<UnitDisplayInformation> {
  game
    .get_all_units()
    .iter()
    .filter_map(|unit| {
      let unit_type = unit.get_type();

      // Skip unknown or invalid unit types
      if unit_type == UnitType::Unknown || unit_type == UnitType::None {
        return None;
      }

      let position = unit.get_position();
      let unit_id = unit.get_id();
      let player = unit.get_player();

      Some(UnitDisplayInformation {
        unit_type,
        position,
        unit_id,
        unit_width: unit_type.width(),
        unit_height: unit_type.height(),
        player_id: Some(player.get_id() as i32),
        player_name: Some(player.get_name()),
      })
    })
    .collect()
}
