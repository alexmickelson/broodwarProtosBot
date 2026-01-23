mod bot;
mod state;
mod utils;

use bot::ProtosBot;
use state::game_state::GameState;
use std::sync::{Arc, Mutex};

fn main() {
  println!("Starting RustBot...");

  let game_state = Arc::new(Mutex::new(GameState::default()));

  // // Start the webserver in a separate thread
  // std::thread::spawn(|| {
  //   let rt = tokio::runtime::Runtime::new().unwrap();
  //   rt.block_on(start_webserver());
  // });

  rsbwapi::start(move |_game| ProtosBot::new(game_state.clone()));
}

async fn start_webserver() {
  use axum::Router;
  use leptos::*;
  use leptos_axum::{generate_route_list, LeptosRoutes};
  use protoss_bot_web::App;

  let conf = get_configuration(None).await.unwrap();
  let leptos_options = conf.leptos_options;
  let addr = leptos_options.site_addr;
  let routes = generate_route_list(App);

  let app = Router::new()
    .leptos_routes(&leptos_options, routes, App)
    .with_state(leptos_options);

  println!("Web server listening on http://{}", &addr);
  let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
  axum::serve(listener, app.into_make_service())
    .await
    .unwrap();
}
