mod app;
mod config;
mod launcher;
mod search;
mod ui;

use app::App;

#[tokio::main]
async fn main() {
    env_logger::init();
    let app = App::new();
    app.run();
}
