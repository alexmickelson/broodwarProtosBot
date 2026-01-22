use std::collections::HashMap;
use rsbwapi::{Order, Position, Unit, UnitType, UpgradeType};

use crate::state::build_stages::BuildStage;

pub struct GameState {
  pub intended_commands: HashMap<usize, IntendedCommand>,
  pub unit_build_history: Vec<BuildHistoryEntry>,
  pub build_stages: Vec<BuildStage>,
  pub current_stage_index: usize,
  pub desired_game_speed: i32,
}


impl Default for GameState {
  fn default() -> Self {
    Self {
      intended_commands: HashMap::new(),
      unit_build_history: Vec::new(),
      build_stages: crate::state::build_stages::get_build_stages(),
      current_stage_index: 0,
      desired_game_speed: 20,
    }
  }
}

#[derive(Clone, Debug)]
pub struct IntendedCommand {
  pub order: Order,
  pub target_position: Option<Position>,
  pub target_unit: Option<Unit>,
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
  // pub status: BuildStatus,
}
