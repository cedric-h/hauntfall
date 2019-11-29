#![feature(stmt_expr_attributes)]
use comn::specs::{self, prelude::*};
use log::*;
use specs::WorldExt;
mod net;
mod pickup;

mod config;
use config::{Level, ServerConfig};

fn main() {
    {
        use log::LevelFilter::*;

        #[rustfmt::skip]
        pretty_env_logger::formatted_builder()
            .filter(None,                   Debug)
            .init();
    }

    let mut world = specs::World::new();
    world.insert(comn::Fps(20.0));
    #[rustfmt::skip]
    let mut dispatcher = DispatcherBuilder::new()
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
    level.load_map(&mut world, &config);

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
