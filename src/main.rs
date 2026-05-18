mod core;
mod ui;

use ui::renderer::Renderer;

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Lucy Tetris RS".to_owned(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = Renderer::new();
    app.run().await;
}
