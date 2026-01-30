use rsbwapi::*;
use std::collections::HashMap;

use crate::{state::build_stages::BuildStage, utils::build_order::next_thing_to_build::BuildStatusMap};

pub struct GameState {
  pub unit_build_history: Vec<BuildHistoryEntry>,
  pub build_stages: Vec<BuildStage>,
  pub current_stage_index: usize,
  pub stage_item_status: BuildStatusMap,
  pub base_locations: Vec<BaseLocation>,
  pub squads: Vec<Squad>,
  pub worker_refinery_assignments: HashMap<usize, usize>,
}

impl Default for GameState {
  fn default() -> Self {
    Self {
      unit_build_history: Vec::new(),
      build_stages: crate::state::build_stages::get_build_stages(),
      current_stage_index: 0,
      stage_item_status: HashMap::new(),
      base_locations: Vec::new(),
      squads: Vec::new(),
      worker_refinery_assignments: HashMap::new(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildStatus {
  Assigned,
  Started,
}

#[derive(Clone, Debug)]
pub struct BuildHistoryEntry {
  pub unit_type: Option<UnitType>,
  pub upgrade_type: Option<UpgradeType>,
  pub assigned_unit_id: Option<usize>,
  pub tile_position: Option<rsbwapi::TilePosition>,
  pub status: BuildStatus,
}

#[derive(Clone, Debug)]
pub struct CheckedPosition {
  pub tile_position: TilePosition,
  pub is_valid: bool,
}

#[derive(Clone, Debug)]
pub struct BaseLocation {
  pub position: TilePosition,
  pub checked_positions: Vec<CheckedPosition>,
}

#[derive(Clone, Debug)]
pub struct Squad {
  pub unit_ids: Vec<usize>,
  pub target_position: Option<Position>,
}
