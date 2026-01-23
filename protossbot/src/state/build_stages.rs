use rsbwapi::UnitType;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BuildStage {
  pub name: String,
  pub desired_counts: HashMap<UnitType, i32>,
}

impl BuildStage {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      desired_counts: HashMap::new(),
    }
  }

  pub fn with_unit(mut self, unit_type: UnitType, count: i32) -> Self {
    self.desired_counts.insert(unit_type, count);
    self
  }
}

pub fn get_build_stages() -> Vec<BuildStage> {
  vec![
    BuildStage::new("Start")
      .with_unit(UnitType::Terran_SCV, 10)
      .with_unit(UnitType::Terran_Supply_Depot, 1),
    BuildStage::new("Basic Production")
      .with_unit(UnitType::Terran_SCV, 12)
      .with_unit(UnitType::Terran_Supply_Depot, 2)
      .with_unit(UnitType::Terran_Barracks, 1)
      .with_unit(UnitType::Terran_Refinery, 1),
    // Stage 2: Defense bunker
    BuildStage::new("Defense Bunker")
      .with_unit(UnitType::Terran_SCV, 16)
      .with_unit(UnitType::Terran_Supply_Depot, 3)
      .with_unit(UnitType::Terran_Command_Center, 1)
      .with_unit(UnitType::Terran_Barracks, 1)
      .with_unit(UnitType::Terran_Refinery, 1)
      .with_unit(UnitType::Terran_Bunker, 2),
  ]
}
