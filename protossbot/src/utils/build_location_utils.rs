use rsbwapi::{Game, Player, TilePosition, Unit, UnitType};

// Spiral iterator similar to Styx2's approach
struct Spiral {
  center: TilePosition,
  x: i32,
  y: i32,
  dx: i32,
  dy: i32,
  segment_length: i32,
  segment_passed: i32,
}

impl Spiral {
  fn new(center: TilePosition) -> Self {
    Self {
      center,
      x: 0,
      y: 0,
      dx: 0,
      dy: -1,
      segment_length: 1,
      segment_passed: 0,
    }
  }
}

impl Iterator for Spiral {
  type Item = TilePosition;

  fn next(&mut self) -> Option<Self::Item> {
    let result = TilePosition {
      x: self.center.x + self.x,
      y: self.center.y + self.y,
    };

    // Move to next position
    self.x += self.dx;
    self.y += self.dy;
    self.segment_passed += 1;

    if self.segment_passed == self.segment_length {
      self.segment_passed = 0;

      // Turn 90 degrees clockwise
      let temp = self.dx;
      self.dx = -self.dy;
      self.dy = temp;

      // Increase segment length every two turns
      if self.dy == 0 {
        self.segment_length += 1;
      }
    }

    Some(result)
  }
}

pub fn find_build_location(
  game: &Game,
  player: &Player,
  builder: &Unit,
  building_type: UnitType,
  max_range: i32,
) -> Option<TilePosition> {
  // Find the base to start spiral search from
  let start_tile = if let Some(nexus) = player
    .get_units()
    .iter()
    .find(|u| u.get_type() == UnitType::Protoss_Nexus)
  {
    nexus.get_tile_position()
  } else {
    builder.get_tile_position()
  };

  let map_width = game.map_width();
  let map_height = game.map_height();
  let max_tiles = (max_range * max_range) as usize;

  // Use spiral search like Styx2
  Spiral::new(start_tile)
    .take(max_tiles.min(300)) // Limit to 300 tiles like Styx2
    .filter(|&tile| {
      // Check bounds
      tile.x >= 0 && tile.y >= 0 && tile.x < map_width && tile.y < map_height
    })
    .find(|&tile| is_valid_build_location(game, player, building_type, tile, builder))
}

fn is_valid_build_location(
  game: &Game,
  player: &Player,
  building_type: UnitType,
  position: TilePosition,
  builder: &Unit,
) -> bool {
  // Get building dimensions
  let width = building_type.tile_width();
  let height = building_type.tile_height();

  let map_width = game.map_width();
  let map_height = game.map_height();

  // Check if building would fit on the map
  if position.x + width > map_width || position.y + height > map_height {
    return false;
  }

  // Use BWAPI's can_build_here like Styx2 does (it handles most validation)
  if !game
    .can_build_here(Some(builder), position, building_type, false)
    .unwrap_or(false)
  {
    return false;
  }

  let center = position.to_position()
    + rsbwapi::Position {
      x: (width * 32) / 2,
      y: (height * 32) / 2,
    };

  // Check no resource containers nearby (minerals/geysers) - 128 pixel radius like Styx2
  let has_resources_nearby = game.get_all_units().iter().any(|u| {
    (u.get_type().is_mineral_field() || u.get_type().is_refinery())
      && u.get_position().distance(center) < 128.0
  });

  if has_resources_nearby {
    return false;
  }

  // Check no resource depots nearby - 128 pixel radius like Styx2
  let has_depot_nearby = player
    .get_units()
    .iter()
    .any(|u| u.get_type().is_resource_depot() && u.get_position().distance(center) < 128.0);

  if has_depot_nearby {
    return false;
  }

  true
}
