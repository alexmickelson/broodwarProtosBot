use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::utils::build_order::next_thing_to_build::BuildStatusMap;

#[derive(Clone, Default)]
pub struct BuildStatusData {
  pub stage_name: String,
  pub stage_index: usize,
  pub item_status: BuildStatusMap,
}

#[derive(Clone)]
pub struct SharedBuildStatus {
  data: Arc<Mutex<BuildStatusData>>,
}

impl SharedBuildStatus {
  pub fn new() -> Self {
    Self {
      data: Arc::new(Mutex::new(BuildStatusData::default())),
    }
  }

  pub fn update(&self, stage_name: String, stage_index: usize, item_status: BuildStatusMap) {
    let mut data = self.data.lock().unwrap();
    data.stage_name = stage_name;
    data.stage_index = stage_index;
    data.item_status = item_status;
  }

  pub fn get(&self) -> BuildStatusData {
    self.data.lock().unwrap().clone()
  }
}

#[derive(Serialize, Deserialize)]
pub struct BuildStatusResponse {
  pub stage_name: String,
  pub stage_index: usize,
  pub items: Vec<BuildItemStatusInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct BuildItemStatusInfo {
  pub unit_name: String,
  pub status: String,
}

pub async fn get_build_status(State(build_status): State<SharedBuildStatus>) -> Response {
  let data = build_status.get();

  let items: Vec<BuildItemStatusInfo> = data
    .item_status
    .iter()
    .map(|(key, status)| BuildItemStatusInfo {
      unit_name: match key {
        crate::utils::build_order::next_thing_to_build::UnitOrUpgradeType::Unit(u) => {
          u.name().to_string()
        }
        crate::utils::build_order::next_thing_to_build::UnitOrUpgradeType::Upgrade(up) => {
          format!("Upgrade: {:?}", up)
        }
      },
      status: format!("{:?}", status),
    })
    .collect();

  let response = BuildStatusResponse {
    stage_name: data.stage_name,
    stage_index: data.stage_index,
    items,
  };

  (StatusCode::OK, Json(response)).into_response()
}
