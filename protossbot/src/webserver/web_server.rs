use axum::{
  routing::{get, post},
  Router,
};
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::state::game_state;

use super::{
  base_locations, build_history, build_status, frame_stats, game_speed, map_info, unit_info,
};

pub use build_status::SharedBuildStatus;
pub use game_speed::SharedGameSpeed;

pub async fn start_web_server(
  shared_speed: SharedGameSpeed,
  build_status: SharedBuildStatus,
  game_state: Arc<Mutex<game_state::GameState>>,
) {
  let static_dir = std::env::current_dir().unwrap().join("static");

  let cors = CorsLayer::very_permissive();

  let app = Router::new()
    .route("/api/speed", get(game_speed::get_game_speed))
    .route("/api/speed", post(game_speed::set_game_speed))
    .with_state(shared_speed)
    .route("/api/build-status", get(build_status::get_build_status))
    .with_state(build_status)
    .route("/api/build-history", get(build_history::get_build_history))
    .route("/api/map-info", get(map_info::get_map_info))
    .route("/api/unit-info", get(unit_info::get_unit_info))
    .route(
      "/api/base-locations",
      get(base_locations::get_base_locations),
    )
    .route("/api/frame-stats", get(frame_stats::get_frame_stats))
    .with_state(game_state)
    .layer(cors)
    .fallback_service(ServeDir::new(static_dir));

  let addr = "127.0.0.1:3333";

  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  println!("Web server running at http://{}", addr);

  axum::serve(listener, app).await.unwrap();
}
