use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::sync::{Arc, RwLock};

// Global static for game speed (shared with the bot)
lazy_static::lazy_static! {
    pub static ref GAME_SPEED: Arc<RwLock<i32>> = Arc::new(RwLock::new(20));
}

#[component]
pub fn App() -> impl IntoView {
  provide_meta_context();

  view! {
      <Stylesheet id="leptos" href="/pkg/main.css"/>
      <Title text="Protoss Bot Control"/>
      <Router>
          <main>
              <Routes>
                  <Route path="" view=HomePage/>
              </Routes>
          </main>
      </Router>
  }
}

#[component]
fn HomePage() -> impl IntoView {
  let (game_speed, set_game_speed) = create_signal(0);

  let set_speed = create_action(move |speed: &i32| {
    let speed = *speed;
    async move {
      set_game_speed.set(speed);
      let _ = set_game_speed_server(speed).await;
    }
  });

  view! {
      <div class="container">
          <h1>"Protoss Bot Control Panel"</h1>

          <div class="speed-control">
              <h2>"Game Speed Control"</h2>
              <p>"Current Speed: " {game_speed}</p>

              <div class="button-group">
                  <button
                      class:selected=move || game_speed.get() == -1
                      on:click=move |_| set_speed.dispatch(-1)>
                      "Fastest (-1)"
                  </button>
                  <button
                      class:selected=move || game_speed.get() == 0
                      on:click=move |_| set_speed.dispatch(0)>
                      "Fast (0)"
                  </button>
                  <button
                      class:selected=move || game_speed.get() == 1
                      on:click=move |_| set_speed.dispatch(1)>
                      "Normal (1)"
                  </button>
                  <button
                      class:selected=move || game_speed.get() == 42
                      on:click=move |_| set_speed.dispatch(42)>
                      "Slowest (42)"
                  </button>
              </div>
          </div>
      </div>
  }
}

#[server(SetGameSpeed, "/api")]
pub async fn set_game_speed_server(speed: i32) -> Result<(), ServerFnError> {
  if let Ok(mut game_speed) = GAME_SPEED.write() {
    *game_speed = speed;
    println!("Game speed set to: {}", speed);
  }
  Ok(())
}
