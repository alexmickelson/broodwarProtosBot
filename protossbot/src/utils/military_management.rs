use crate::state::game_state::{GameState, Squad};
use rsbwapi::*;

pub fn military_onframe(game: &Game, player: &Player, state: &mut GameState) {
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
    defend_base(game, player, &military_units_not_in_squads);
    keep_medics_close_to_other_units(game, player, state);
    return;
  }
  if military_supply > 70 {
    send_out_new_squad(game, player, state, military_units_not_in_squads);
  }
  keep_medics_close_to_other_units(game, player, state);
}

fn defend_base(game: &Game, player: &Player, military_units: &[Unit]) {
  for unit in military_units {
    if unit.get_type() == UnitType::Terran_Medic {
      continue;
    }
    let enemy_units = game
      .get_all_units()
      .iter()
      .filter(|u| u.get_player().get_id() != player.get_id() && u.is_visible())
      .cloned()
      .collect::<Vec<Unit>>();
    if let Some(closest_enemy) = enemy_units.iter().min_by_key(|e| {
      let dx = (unit.get_position().x - e.get_position().x).abs();
      let dy = (unit.get_position().y - e.get_position().y).abs();
      dx + dy
    }) {
      let enemy_pos = closest_enemy.get_position();
      if matches!(unit.get_order(), Order::AttackUnit | Order::AttackMove) {
        if let Some(order_target) = unit.get_order_target() {
          if order_target.get_id() == closest_enemy.get_id() {
            continue;
          }
        }
      }
      // Always attack the closest enemy unit
      println!(
        "Unit {} was {:?} now attacking due to nearby enemies",
        unit.get_id(),
        unit.get_order()
      );
      let _ = unit.attack(closest_enemy);
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
    unit.attack(new_squad.target_position.unwrap());
  }
  state.squads.push(new_squad);
}

fn keep_medics_close_to_other_units(game: &Game, player: &Player, state: &mut GameState) {
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
        medic.move_(closest_pos);
      }
    }
  }
}
