use rsbwapi::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub fn get_paths_between(
  game: &Game,
  start: TilePosition,
  destinations: &[TilePosition],
) -> Option<Vec<Vec<TilePosition>>> {
  let mut all_paths = Vec::new();
  let mut current_start = start;
  let mut remaining_destinations: Vec<TilePosition> = destinations.to_vec();
  let mut destination_index = 1;

  while !remaining_destinations.is_empty() {
    let (path, found_dest, attempts) =
      find_first_destination(game, current_start, &remaining_destinations)?;

    println!(
      "Found destination {} after {} attempts",
      destination_index, attempts
    );
    destination_index += 1;

    all_paths.push(path);
    remaining_destinations.retain(|&d| d != found_dest);
    current_start = found_dest;
  }

  Some(all_paths)
}

fn find_first_destination(
  game: &Game,
  start: TilePosition,
  destinations: &[TilePosition],
) -> Option<(Vec<TilePosition>, TilePosition, usize)> {
  let mut queue = VecDeque::new();
  let mut came_from = HashMap::new();
  let mut visited = HashSet::new();

  queue.push_back(start);
  visited.insert(start);

  let mut attempts = 0;
  while let Some(current) = queue.pop_front() {
    attempts += 1;

    // Check if current position is any of the destinations
    if let Some(&found_dest) = destinations.iter().find(|&&d| d == current) {
      // reconstruct path
      let mut path = Vec::new();
      let mut node = current;
      path.push(node);
      while let Some(prev) = came_from.get(&node) {
        node = *prev;
        path.push(node);
      }
      path.reverse();
      return Some((path, found_dest, attempts));
    }

    for neighbor in neighbors(game, current) {
      if visited.contains(&neighbor) {
        continue;
      }
      queue.push_back(neighbor);
      visited.insert(neighbor);
      came_from.insert(neighbor, current);
    }
  }

  println!(
    "Failed to find path after {} attempts from {:?} to any remaining destinations, {:?}",
    attempts, start, destinations
  );
  None
}

fn neighbors(game: &Game, tile: TilePosition) -> Vec<TilePosition> {
  let mut result = Vec::new();

  let skip_ratio = 1;

  let deltas = [
    (-skip_ratio, 0),
    (skip_ratio, 0),
    (0, -skip_ratio),
    (0, skip_ratio),
  ];
  let map_width = game.map_width();
  let map_height = game.map_height();

  for (dx, dy) in deltas.iter() {
    let nx = tile.x + dx;
    let ny = tile.y + dy;
    if nx >= 0 && ny >= 0 && nx < map_width && ny < map_height {
      let position = TilePosition { x: nx, y: ny };
      if game.is_walkable(position.to_walk_position()) {
        result.push(position);
      }
    }
  }
  result
}
