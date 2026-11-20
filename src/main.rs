use bevy::app::AppExit;

fn main() -> AppExit {
    match std::env::args().nth(1).as_deref() {
        Some("--version" | "-V") => {
            println!("numhop {}", env!("CARGO_PKG_VERSION"));
            AppExit::Success
        }
        Some("--help" | "-h") => {
            println!("numhop — a 3D math platformer\n\nUsage: numhop [--version] [--help]\n\nPress ? in the game for shortcuts.");
            AppExit::Success
        }
        _ => numhop::game::run(),
    }
}
