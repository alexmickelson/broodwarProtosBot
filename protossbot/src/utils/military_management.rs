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
    for unit in military_units_not_in_squads {
      if matches!(unit.get_order(), Order::AttackUnit | Order::AttackMove) {
        continue;
      }
      println!(
        "Unit {} was {:?} now attacking due to nearby enemies",
        unit.get_id(),
        unit.get_order()
      );
      let enemy_units = game
        .get_all_units()
        .iter()
        .filter(|u| u.get_player().get_id() != player.get_id() && u.is_visible())
        .cloned()
        .collect::<Vec<Unit>>();
      if let Some(closest_enemy) = enemy_units.iter().min_by_key(|e| {
        let dx = (unit.get_tile_position().x - e.get_tile_position().x).abs();
        let dy = (unit.get_tile_position().y - e.get_tile_position().y).abs();
        dx + dy
      }) {
        unit.attack(closest_enemy.get_position());
      }
    }
    return;
  }

  if military_supply > 70 {
    send_out_new_squad(game, player, state, military_units_not_in_squads);
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
