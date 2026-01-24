use rsbwapi::{Game, Player, TilePosition, Unit, UnitType};

pub fn find_build_location(
  game: &Game,
  _player: &Player,
  builder: &Unit,
  building_type: UnitType,
  max_range: i32,
) -> Option<TilePosition> {
  if building_type.is_refinery() {
    for geyser in game.get_geysers() {
      let tp = geyser.get_tile_position();
      if let Ok(true) = game.can_build_here(Some(builder), tp, building_type, false) {
        return Some(tp);
      }
    }
    return None;
  }

  let start = builder.get_tile_position();
  for radius in 0..=max_range {
    for dx in -radius..=radius {
      let dy = radius - dx.abs();
      for &dy_sign in &[-1, 1] {
        let cand = TilePosition {
          x: start.x + dx,
          y: start.y + dy * dy_sign,
        };
        if let Ok(true) = game.can_build_here(Some(builder), cand, building_type, false) {
          return Some(cand);
        }
        if dy == 0 {
          continue;
        }
        let cand2 = TilePosition {
          x: start.x + dx,
          y: start.y - dy * dy_sign,
        };
        if let Ok(true) = game.can_build_here(Some(builder), cand2, building_type, false) {
          return Some(cand2);
        }
      }
    }
  }
  None
}
