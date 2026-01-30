// Returns the Manhattan distance between two positions
fn position_distance(a: ScaledPosition<1>, b: ScaledPosition<1>) -> i32 {
  (a.x - b.x).abs() + (a.y - b.y).abs()
}
use crate::state::game_state::{GameState, Squad};
use rsbwapi::*;

pub fn military_onframe(game: &Game, player: &Player, state: &mut GameState) {
  let all_my_military_units: Vec<Unit> = player
    .get_units()
    .iter()
    .filter(|u| u.get_type().is_building() == false && u.get_type().is_worker() == false)
    .cloned()
    .collect();

  let enemies_near_my_buildings: Vec<Unit> = game
    .get_all_units()
    .iter()
    .filter(|u| {
      u.get_player().get_id() != player.get_id()
        && u.is_visible()
        && u.get_type().is_building() == false
        && player.get_units().iter().any(|my_u| {
          my_u.get_type().is_building() && {
            let dx = (u.get_tile_position().x - my_u.get_tile_position().x).abs();
            let dy = (u.get_tile_position().y - my_u.get_tile_position().y).abs();
            let manhattan = dx + dy;
            manhattan < 10
          }
        })
    })
    .cloned()
    .collect();

  if enemies_near_my_buildings.len() > 0 {
    defend_base(game, player, &all_my_military_units);
    keep_medics_close_to_other_units(game, player, state);
    return;
  }

  let military_units_not_in_squads = player
    .get_units()
    .iter()
    .filter(|u| {
      u.get_type().is_building() == false
        && u.get_type().is_worker() == false
        && !state
          .squads
          .iter()
          .any(|squad| squad.unit_ids.contains(&u.get_id()))
    })
    .cloned()
    .collect::<Vec<Unit>>();

  let military_supply = military_units_not_in_squads
    .iter()
    .map(|u| u.get_type().supply_required())
    .sum::<i32>();

  if military_supply > 70 {
    send_out_new_squad(game, player, state, military_units_not_in_squads);
  }
  keep_medics_close_to_other_units(game, player, state);
}

fn defend_base(game: &Game, player: &Player, military_units: &[Unit]) {
  // game.get_
  for unit in military_units {
    if unit.get_type() == UnitType::Terran_Medic {
      continue;
    }
    let enemy_units = game
      .get_all_units()
      .iter()
      .filter(|u| {
        u.get_player().get_id() != player.get_id() && !u.get_player().is_neutral()
      })
      .cloned()
      .collect::<Vec<Unit>>();
    if let Some(closest_enemy) = enemy_units.iter().cloned().min_by_key(|e| {
      let dx = (unit.get_position().x - e.get_position().x).abs();
      let dy = (unit.get_position().y - e.get_position().y).abs();
      dx + dy
    }) {
      // If already attacking this enemy, skip
      let unit_order = unit.get_order();

      if unit_order == Order::AttackUnit {
        let Some(order_target) = unit.get_order_target() else {
          continue;
        };

        if order_target.get_id() == closest_enemy.get_id() {
          continue;
        }

        if position_distance(closest_enemy.get_position(), order_target.get_position()) < (32 * 1) {
          continue;
        }
      }

      if unit_order == Order::AttackMove {
        let Some(order_target_pos) = unit.get_order_target_position() else {
          continue;
        };
        if position_distance(closest_enemy.get_position(), order_target_pos) < (32 * 1) {
          continue;
        }
      }
      let enemy_pos = closest_enemy.get_position();
      println!(
        "Unit {} was {:?} now attacking due to nearby enemies",
        unit.get_id(),
        unit.get_order()
      );
      match unit.is_in_weapon_range(&closest_enemy) {
        true => match unit.attack(&closest_enemy) {
          Ok(true) => {}
          Ok(false) => {
            let _ = unit.move_(enemy_pos);
          }
          Err(e) => {
            println!(
              "Failed to order unit {} to attack enemy {}: {:?}",
              unit.get_id(),
              closest_enemy.get_id(),
              e
            );
          }
        },
        false => {
          // If can't attack directly, attack the position of the nearest unit
          match unit.attack(enemy_pos) {
            Ok(true) => {}
            Ok(false) => {
              println!(
                "Unit {} failed to attack move to enemy position {:?}",
                unit.get_id(),
                enemy_pos
              );
            }
            Err(Error::Unable_To_Hit) => {
              // Walk to the closest enemy's position
              println!(
                "Unit {} unable to hit enemy at {:?} (Unable_To_Hit), moving to closest enemy position",
                unit.get_id(),
                enemy_pos
              );
              let _ = unit.move_(enemy_pos);
            }
            Err(e) => {
              println!(
                "Failed to order unit {} to attack move to enemy position {:?}: {:?}",
                unit.get_id(),
                enemy_pos,
                e
              );
            }
          }
        }
      }
    }
  }
}

fn send_out_new_squad(
  game: &Game,
  player: &Player,
  state: &mut GameState,
  military_units_not_in_squads: Vec<Unit>,
) {
  let enemy_start_locations = game
    .get_start_locations()
    .into_iter()
    .filter(|&loc| {
      let my_building_is_close = player
        .get_units()
        .into_iter()
        .filter(|u| u.get_type().is_building())
        .any(|u| {
          let dx = (u.get_tile_position().x - loc.x).abs();
          let dy = (u.get_tile_position().y - loc.y).abs();
          let manhattan = dx + dy;
          manhattan < 5
        });

      !my_building_is_close
    })
    .collect::<Vec<TilePosition>>();

  let new_squad = {
    let unit_ids = military_units_not_in_squads
      .iter()
      .map(|u| u.get_id())
      .collect::<Vec<usize>>();
    Squad {
      unit_ids,
      target_position: {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        enemy_start_locations
          .choose(&mut rng)
          .cloned()
          .map(|tp| tp.to_position())
      },
    }
  };

  println!(
    "new squad with {} units going to attack {:?}",
    military_units_not_in_squads.len(),
    new_squad.target_position
  );

  for unit in military_units_not_in_squads {
    println!(
      "Unit {} was {:?} now attacking",
      unit.get_id(),
      unit.get_order()
    );
    let _ = unit.attack(new_squad.target_position.unwrap());
  }
  state.squads.push(new_squad);
}

fn keep_medics_close_to_other_units(_game: &Game, player: &Player, _state: &mut GameState) {
  let medic_units: Vec<Unit> = player
    .get_units()
    .iter()
    .filter(|u: &&Unit| u.get_type() == UnitType::Terran_Medic)
    .cloned()
    .collect();

  let other_friendly_units: Vec<Unit> = player
    .get_units()
    .iter()
    .filter(|u| u.get_type() != UnitType::Terran_Medic && u.get_type().is_building() == false)
    .cloned()
    .collect();

  for medic in medic_units {
    let medic_pos: ScaledPosition<1> = medic.get_position();
    let closest_ally = other_friendly_units.iter().min_by_key(|ally| {
      let ally_pos = ally.get_position();
      let dx = (medic_pos.x - ally_pos.x).abs();
      let dy = (medic_pos.y - ally_pos.y).abs();
      dx + dy
    });

    if let Some(closest) = closest_ally {
      let closest_pos = closest.get_position();
      let dx = (medic_pos.x - closest_pos.x).abs();
      let dy = (medic_pos.y - closest_pos.y).abs();
      let manhattan = dx + dy;
      if manhattan > 5 * 32 {
        println!(
          "Medic {} is too far from allies, moving closer",
          medic.get_id()
        );
        let _ = medic.move_(closest_pos);
      }
    }
  }
}
