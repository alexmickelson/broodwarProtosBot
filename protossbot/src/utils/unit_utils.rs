use rsbwapi::*;

use crate::state::game_state::GameState;

pub fn find_builder_for_unit(
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
        && (u.is_idle() || u.is_gathering_minerals())
    })
    .cloned()
}

pub fn count_units_of_type(player: &Player, unit_type: UnitType) -> i32 {
  player
    .get_units()
    .iter()
    .filter(|u| u.get_type() == unit_type)
    .count() as i32
}


pub fn count_completed_units_of_type(player: &Player, unit_type: UnitType) -> i32 {
  player
    .get_units()
    .iter()
    .filter(|u| u.get_type() == unit_type && u.is_completed())
    .count() as i32
}
