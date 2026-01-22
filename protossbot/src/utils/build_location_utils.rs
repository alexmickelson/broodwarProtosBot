use rsbwapi::{Game, TilePosition, Unit, UnitType};

pub fn find_build_location(
  game: &Game,
  builder: &Unit,
  building_type: UnitType,
  max_range: i32,
) -> Option<TilePosition> {

  let start_tile = builder.get_tile_position();

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
  if !game.can_build_here(builder, position, building_type, false).unwrap_or(false) {
    return false;
  }

  // if building_type.requires_psi() && !game.has_power(position, building_type) {
  //   return false;
  // }

  // if building_type.requires_creep() && !game.has_creep(position) {
  //   return false;
  // }

  true
}
