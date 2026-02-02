use rsbwapi::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type MapTileInformation = HashMap<TilePosition, TileDisplayInformation>;

#[derive(Clone, Debug)]
pub struct MapInformation {
  pub map_width: i32,
  pub map_height: i32,
  pub tile_information: MapTileInformation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileDisplayInformation {
  pub is_walkable: bool,
  pub is_buildable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializablePosition {
  pub x: i32,
  pub y: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitDisplayInformation {
  pub unit_type: String,
  pub pixel_position: SerializablePosition,
  pub unit_id: usize,
  pub unit_width: i32,
  pub unit_height: i32,
  pub player_id: Option<i32>,
  pub player_name: Option<String>,
  pub target_pixel_position: Option<SerializablePosition>,
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
  let neutral_unit_ids = game
    .get_static_neutral_units()
    .iter()
    .map(|u| u.get_id())
    .collect::<Vec<_>>();

  let non_neutral_units: Vec<UnitDisplayInformation> = game
    .get_all_units()
    .iter()
    .filter(|unit| !neutral_unit_ids.contains(&unit.get_id()))
    .filter_map(|unit| {
      let unit_type = unit.get_type();

      // Skip unknown or invalid unit types
      if unit_type == UnitType::Unknown
        || unit_type == UnitType::None
        || unit_type.is_neutral()
        || unit_type.is_mineral_field()
      {
        return None;
      }

      let position = unit.get_position();
      let unit_id = unit.get_id();
      let player = unit.get_player();

      // Skip neutral units
      if player.get_name() == "Neutral" {
        return None;
      }

      let target_position = unit.get_target_position().or_else(|| {
        unit.get_target().and_then(|target_unit| {
          if target_unit.exists() {
            Some(target_unit.get_position())
          } else {
            None
          }
        })
      });

      Some(UnitDisplayInformation {
        unit_type: unit_type.name().to_string(),
        pixel_position: SerializablePosition {
          x: position.x,
          y: position.y,
        },
        unit_id,
        unit_width: unit_type.width(),
        unit_height: unit_type.height(),
        player_id: Some(player.get_id() as i32),
        player_name: Some(player.get_name()),
        target_pixel_position: target_position.map(|p| SerializablePosition { x: p.x, y: p.y }),
      })
    })
    .collect();

  let minerals: Vec<UnitDisplayInformation> = game
    .get_static_minerals()
    .iter()
    .filter_map(|mineral| {
      let pos = mineral.get_initial_position();
      Some(UnitDisplayInformation {
        unit_type: UnitType::Resource_Mineral_Field.name().to_string(),
        pixel_position: SerializablePosition { x: pos.x, y: pos.y },
        unit_id: mineral.get_id(),
        unit_width: UnitType::Resource_Mineral_Field.width(),
        unit_height: UnitType::Resource_Mineral_Field.height(),
        player_id: None,
        player_name: Some("Neutral".to_owned()),
        target_pixel_position: None,
      })
    })
    .collect();

  let geysers: Vec<UnitDisplayInformation> = game
    .get_static_geysers()
    .iter()
    .filter_map(|geyser| {
      let pos = geyser.get_initial_position();
      Some(UnitDisplayInformation {
        unit_type: UnitType::Resource_Vespene_Geyser.name().to_string(),
        pixel_position: SerializablePosition { x: pos.x, y: pos.y },
        unit_id: geyser.get_id(),
        unit_width: UnitType::Resource_Vespene_Geyser.width(),
        unit_height: UnitType::Resource_Vespene_Geyser.height(),
        player_id: None,
        player_name: Some("Neutral".to_owned()),
        target_pixel_position: None,
      })
    })
    .collect();

  non_neutral_units
    .into_iter()
    .chain(minerals)
    .chain(geysers)
    .collect()
}
