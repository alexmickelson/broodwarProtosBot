use rsbwapi::{UnitType, UpgradeType};
use std::collections::HashMap;

use crate::state::build_stages::BuildStage;

pub struct GameState {
  pub unit_build_history: Vec<BuildHistoryEntry>,
  pub build_stages: Vec<BuildStage>,
  pub current_stage_index: usize,
  pub desired_game_speed: i32,
  pub stage_item_status: HashMap<String, String>,
}

impl Default for GameState {
  fn default() -> Self {
    Self {
      unit_build_history: Vec::new(),
      build_stages: crate::state::build_stages::get_build_stages(),
      current_stage_index: 0,
      desired_game_speed: 20,
      stage_item_status: HashMap::new(),
    }
  }
}

#[derive(Clone, Debug)]
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
  // pub status: BuildStatus,
}
