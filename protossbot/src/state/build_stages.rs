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
      .with_unit(UnitType::Protoss_Probe, 10)
      .with_unit(UnitType::Protoss_Pylon, 1),

    BuildStage::new("Basic Production")
      .with_unit(UnitType::Protoss_Probe, 12)
      .with_unit(UnitType::Protoss_Pylon, 2)
      .with_unit(UnitType::Protoss_Gateway, 1)
      .with_unit(UnitType::Protoss_Forge, 1),


    // Stage 2: Defense cannons
    BuildStage::new("Defense Cannons")
      .with_unit(UnitType::Protoss_Probe, 16)
      .with_unit(UnitType::Protoss_Pylon, 3)
      .with_unit(UnitType::Protoss_Nexus, 1)
      .with_unit(UnitType::Protoss_Gateway, 1)
      .with_unit(UnitType::Protoss_Forge, 1)
      .with_unit(UnitType::Protoss_Photon_Cannon, 4),
  ]
}
