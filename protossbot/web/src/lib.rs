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
        <Stylesheet id="leptos" href="/pkg/protoss-bot-web.css"/>
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
    let (game_speed, set_game_speed) = create_signal(20);

    let set_speed = move |speed: i32| {
        set_game_speed.set(speed);
        spawn_local(async move {
            let _ = set_game_speed_server(speed).await;
        });
    };

    view! {
        <div class="container">
            <h1>"Protoss Bot Control Panel"</h1>
            
            <div class="speed-control">
                <h2>"Game Speed Control"</h2>
                <p>"Current Speed: " {game_speed}</p>
                
                <div class="button-group">
                    <button on:click=move |_| set_speed(0)>"Slowest (0)"</button>
                    <button on:click=move |_| set_speed(10)>"Slower (10)"</button>
                    <button on:click=move |_| set_speed(20)>"Normal (20)"</button>
                    <button on:click=move |_| set_speed(30)>"Fast (30)"</button>
                    <button on:click=move |_| set_speed(42)>"Fastest (42)"</button>
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
