#![recursion_limit = "256"]
#[macro_use]
extern crate stdweb;
use stdweb::web::window;

pub mod prelude {
    pub use super::net::Player;
    pub use comn::prelude::*;
    pub use comn::rmps;
    pub use log::*;
    pub use specs::{prelude::*, Component};
}
use prelude::*;

mod controls;
mod item;
mod net;
mod renderer;

fn main() {
    stdweb::initialize();

    #[cfg(feature = "stdweb-logger")]
    stdweb_logger::init_with_level(Level::Trace);

    // instantiate an ECS world to hold all of the systems, resources, and components.
    let mut world = World::new();

    world.insert(comn::Fps(80.0));

    // add systems and instantiate and order the other systems.
    #[rustfmt::skip]
    let mut dispatcher = DispatcherBuilder::new()
        // controls
        .with(comn::controls::MoveHeadings,         "heading",      &[])
        .with(controls::MovementControl::default(), "move",         &[])
        .with(controls::LaunchAttacks::default(),   "attack",       &[])
        .with(controls::PickupItems::default(),     "click",        &[])
        // phys
        .with(comn::phys::Collision,                "collision",    &[])
        .with(net::SyncPositions,                   "sync phys",    &[])
        // art
        .with(renderer::Render::default(),          "render",       &[])
        .with(comn::art::UpdateAnimations,          "animate",      &[])
        // util
        .with(net::HandleServerPackets::default(),  "packets",      &[])
        .with(comn::dead::ClearDead,                "clear dead",   &[])
        // items
        .with(item::DepositionItems,                "deposition",   &[])
        .with(item::UpdateInventory::default(),     "update items", &[])
        .build();

    // go through all of the systems and register components and resources accordingly
    dispatcher.setup(&mut world);

    info!("Starting game loop!");

    fn game_loop(mut dispatcher: specs::Dispatcher<'static, 'static>, mut world: specs::World) {
        // run all of the ECS systems
        dispatcher.dispatch(&mut world);
        world.maintain();

        // tell browser to repeat me the next time the monitor is going to refresh
        window().request_animation_frame(|_| game_loop(dispatcher, world));
    }

    game_loop(dispatcher, world);

    stdweb::event_loop();
}
