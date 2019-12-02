// our code
use super::prelude::*;
use comn::prelude::*;
use comn::specs::prelude::*;
// crates
use log::*;

/// This system sends the world to all clients with the LoggingIn component.
pub struct SendWorldToNewPlayers;
impl<'a> System<'a> for SendWorldToNewPlayers {
    type SystemData = (
        // things we need to do networking
        Read<'a, ConnectionManager>,
        WriteStorage<'a, LoggingIn>,
        ReadStorage<'a, Client>,
        // things we need to tell new players about
        Entities<'a>,
        ReadStorage<'a, comn::Hitbox>,
        ReadStorage<'a, comn::art::Appearance>,
        ReadStorage<'a, comn::art::Animate>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Pos>,
    );

    fn run(
        &mut self,
        (cm, mut logging_ins, clients, ents, hitboxes, appearances, animates, items, isos): Self::SystemData,
    ) {
        for (_, Client(addr)) in (logging_ins.drain(), &clients).join() {
            debug!("We're about to tell a new player about the world.");
            // tell them about each new entity they need to add, and about
            // some crucial components it has.
            for (iso, ent, hitbox, appearance, animate, item) in (
                &isos,
                &*ents,
                hitboxes.maybe(),
                appearances.maybe(),
                animates.maybe(),
                items.maybe(),
            )
                .join()
            {
                trace!("telling new player about an existing entity");
                cm.new_ent(*addr, ent);
                cm.insert_comp(*addr, ent, iso.clone());

                // I should really do all of these using some more macro
                // abomination on net_component
                if let Some(hitbox) = hitbox {
                    cm.insert_comp(*addr, ent, hitbox.clone());
                }
                if let Some(appearance) = appearance {
                    cm.insert_comp(*addr, ent, appearance.clone());
                }
                if let Some(item) = item {
                    cm.insert_comp(*addr, ent, item.clone());
                }
                if let Some(animate) = animate {
                    cm.insert_comp(*addr, ent, animate.clone());
                }
            }
        }
    }
}

pub struct SpawnNewPlayers;
impl<'a> System<'a> for SpawnNewPlayers {
    type SystemData = (
        Entities<'a>,
        Read<'a, ConnectionManager>,
        Read<'a, comn::art::AppearanceRecord>,
        Read<'a, LazyUpdate>,
        WriteStorage<'a, comn::net::SpawnPlayer>,
        ReadStorage<'a, Client>,
    );

    fn run(
        &mut self,
        (ents, cm, appear_record, lu, mut players_to_spawn, clients): Self::SystemData,
    ) {
        use crate::combat;
        use comn::{art, combat::Health, controls, item, Hitbox};
        for (_, ent, Client(new_player_addr)) in (players_to_spawn.drain(), &*ents, &clients).join()
        {
            trace!("spawning new player!");
            // these are the components the entity will have.
            let appearance = appear_record.try_appearance_of("Player").unwrap();
            let iso = Pos::vec(Vec2::new(1.0, 1.0));
            let speed = controls::Speed { speed: 0.115 };
            //let animate = Animate::new();
            let hitbox = Hitbox::vec(Vec2::new(0.5, 0.25));

            // give them player components
            lu.insert(ent, iso.clone());
            lu.insert(ent, speed.clone());
            lu.insert(ent, appearance.clone());
            //lu.insert(ent, animate.clone());
            lu.insert(ent, hitbox.clone());
            lu.insert(ent, art::PlayerAnimationController);
            lu.insert(ent, combat::Alignment::Players);
            lu.insert(ent, Health::full(5));
            lu.insert(ent, item::Inventory::character());

            // tell everyone 'bout the new kid on the block
            for Client(addr) in (&clients).join() {
                cm.new_ent(*addr, ent);
                cm.insert_comp(*addr, ent, iso.clone());
                cm.insert_comp(*addr, ent, speed.clone());
                cm.insert_comp(*addr, ent, appearance.clone());
                //cm.insert_comp(*addr, ent, animate.clone());
                cm.insert_comp(*addr, ent, hitbox.clone());
                cm.insert_comp(*addr, ent, art::PlayerAnimationController);
                if addr == new_player_addr {
                    debug!("so we did tell them about themself");
                }
            }
        }
    }
}
