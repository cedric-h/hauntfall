#![feature(stmt_expr_attributes)]
use comn::{
    prelude::*,
    specs::{self, prelude::*},
};
use log::*;
use specs::WorldExt;
mod net;
mod pickup;


mod map {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub struct MapEntry {
        pub location: [f32; 3],
        pub hitbox_dimensions: Option<[f32; 2]>,
        pub appearance: comn::art::Appearance,
    }
}

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

    let file = std::fs::File::open("./map.json")
        .expect("couldn't open map.json");
    let map_json: Vec<map::MapEntry> = serde_json::from_reader(file)
        .expect("map file isn't proper JSON");

    for map::MapEntry { location: loc, appearance, hitbox_dimensions: hb } in map_json.into_iter() {
        let mut builder = world.create_entity()
            .with(Pos::vec(Vec2::new(loc[0], loc[1])))
            .with(appearance);
        if let Some(hb) = hb {
            builder = builder
                .with(comn::Hitbox(comn::Cuboid::new(Vec2::new(hb[0], hb[1])/2.0)));
        }
        builder.build();
    }

    info!("starting game loop!");

    let mut fixedstep = fixedstep::FixedStep::start(20.0); // 20.0Hz

    loop {
        while fixedstep.update() {
            dispatcher.dispatch(&mut world);
            world.maintain();
        }
    }
}
