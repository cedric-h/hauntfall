#![feature(stmt_expr_attributes)]
use comn::specs::{self, prelude::*};
use log::*;
use specs::WorldExt;
mod combat;
mod config;
mod net;
mod pickup;
use config::{Level, ServerConfig};

// launch webserver to serve client files
#[cfg(feature = "webserver")]
fn host_client() {
    use warp::Filter;

    let index = warp::get2()
        .and(warp::path::end())
        .and(warp::fs::file("./deploy/index.html"));

    // dir already requires GET...
    let other = warp::fs::dir("./deploy/");

    // GET / => index.html
    // GET /ex/... => ./examples/..
    let routes = index.or(other);

    warp::serve(routes).run(([127, 0, 0, 1], 3030));
}

fn main() {
    #[rustfmt::skip]
    {
        use log::LevelFilter::*;
        let mut builder = pretty_env_logger::formatted_builder();

        #[cfg(feature = "webserver")]
        builder
            .filter_module("hyper",            Info)
            .filter_module("tungstenite",      Info)
            .filter_module("tokio_reactor",    Info);

        builder
            .filter(None,   Debug)
            .init();
    }

    #[cfg(feature = "webserver")]
    std::thread::spawn(|| host_client());

    let mut world = specs::World::new();
    world.insert(comn::Fps(20.0));
    #[rustfmt::skip]
    let mut dispatcher = DispatcherBuilder::new()
        .with(combat::Chase,                "chase",            &[])
        .with(pickup::ItemPickupDrop,       "pickup",           &[])
        .with(comn::art::UpdateAnimations,  "animate",          &[])
        .with(comn::phys::Collision,        "collision",        &[])
        .with(comn::controls::MoveHeadings, "heading",          &[])
        .with(net::SendWorldToNewPlayers,   "send world",       &[])
        .with(net::HandleClientPackets,     "client packets",   &["send world"])
        .with(net::SpawnNewPlayers,         "new players",      &["client packets"])
        .with(comn::dead::ClearDead,        "clear dead",       &["client packets"])
        .with(net::SendNewPositions,        "send pos",         &["clear dead"])
        .build();

    dispatcher.setup(&mut world);

    // parsing config file
    let config = ServerConfig::parse();
    let mut level = Level::from_name(config.level.clone());
    level
        .load_map(&mut world, &config)
        .unwrap_or_else(|e| panic!("Couldn't load map: {}", e));

    world.insert(config.appearance_record);

    info!("starting game loop!");

    let mut fixedstep = fixedstep::FixedStep::start(20.0); // 20.0Hz

    loop {
        while fixedstep.update() {
            dispatcher.dispatch(&mut world);
            world.maintain();
        }
    }
}
