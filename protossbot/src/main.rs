mod bot;
mod state;
mod utils;

use bot::ProtosBot;
use state::game_state::GameState;
use std::sync::{Arc, Mutex};

fn main() {
  println!("Starting RustBot...");

  let game_state = Arc::new(Mutex::new(GameState::default()));

  // Start the webserver in a separate thread
  std::thread::spawn(|| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
      start_webserver().await;
      // Keep the runtime alive indefinitely to prevent TLS cleanup issues
      std::future::pending::<()>().await;
    });
  });

  rsbwapi::start(move |_game| ProtosBot::new(game_state.clone()));
}

async fn start_webserver() {
  use axum::Router;
  use leptos::*;
  use leptos_axum::{generate_route_list, LeptosRoutes};
  use protoss_bot_web::App;
  use tower_http::services::ServeDir;

  let leptos_options = LeptosOptions {
    output_name: "protoss-bot-web".to_string(),
    site_root: "target/site".to_string(),
    site_pkg_dir: "pkg".to_string(),
    env: leptos_config::Env::DEV,
    site_addr: "127.0.0.1:3333".parse().unwrap(),
    reload_port: 3001,
    hash_file: "".to_string(),
    hash_files: false,
    ..Default::default()
  };
  let addr = leptos_options.site_addr;
  let routes = generate_route_list(App);

  let app = Router::new()
    .leptos_routes(&leptos_options, routes, App)
    .nest_service("/pkg", ServeDir::new("web/style"))
    .with_state(leptos_options);

  match tokio::net::TcpListener::bind(&addr).await {
    Ok(listener) => {
      println!("Web server listening on http://{}", &addr);
      if let Err(e) = axum::serve(listener, app.into_make_service()).await {
        eprintln!("Web server error: {}", e);
      }
    }
    Err(e) => {
      eprintln!(
        "Failed to bind to {}: {}. Is the port already in use?",
        &addr, e
      );
      eprintln!("Skipping web server startup.");
    }
  }
}
