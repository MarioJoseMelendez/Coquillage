mod app;
mod assets;
mod camera;
mod player;
mod renderer;
mod world;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if let Err(err) = app::run() {
        log::error!("{err:#}");
        std::process::exit(1);
    }
}
