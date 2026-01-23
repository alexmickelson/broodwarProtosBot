use rsbwapi::{Game, TilePosition, Unit, UnitType};

pub fn find_build_location(
  game: &Game,
  builder: &Unit,
  building_type: UnitType,
  max_range: i32,
) -> Option<TilePosition> {
  let start_tile = builder.get_tile_position();
  let map_width = game.map_width();
  let map_height = game.map_height();

  for distance in 0..max_range {
    for dx in -distance..=distance {
      for dy in -distance..=distance {
        if dx.abs() != distance && dy.abs() != distance {
          continue;
        }

        let tile = TilePosition {
          x: start_tile.x + dx,
          y: start_tile.y + dy,
        };

        if tile.x < 0 || tile.y < 0 || tile.x >= map_width || tile.y >= map_height {
          continue;
        }

        if is_valid_build_location(game, building_type, tile, builder) {
          return Some(tile);
        }
      }
    }
  }

  None
}

fn is_valid_build_location(
  game: &Game,
  building_type: UnitType,
  position: TilePosition,
  builder: &Unit,
) -> bool {
  game
    .can_build_here(builder, position, building_type, false)
    .unwrap_or(false)
}
