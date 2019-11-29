use super::net::ServerConnection;
use crate::prelude::*;
use comn::controls::Heading;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use stdweb::{
    traits::IKeyboardEvent,
    web::{
        document,
        event::{ConcreteEvent, KeyPressEvent, KeyUpEvent},
        IEventTarget,
    },
};

//(key direction, key down)
type KeyMap = Arc<Mutex<HashMap<char, bool>>>;

pub struct MovementControl {
    keys: KeyMap,
    current_heading: Vec2,
}
impl MovementControl {
    fn handle_key_event<K: IKeyboardEvent + ConcreteEvent>(keys: KeyMap, key_down: bool) {
        document().add_event_listener(move |e: K| {
            if !e.repeat() {
                let first_letter = e.key().chars().next().expect("zero length key name");
                if "wsad".contains(first_letter) {
                    keys.lock()
                        .expect("Can't lock keys")
                        .insert(first_letter, key_down);
                }
            }
        });
    }
}
impl Default for MovementControl {
    fn default() -> Self {
        let keys = Arc::new(Mutex::new(HashMap::new()));

        Self::handle_key_event::<KeyPressEvent>(keys.clone(), true);
        Self::handle_key_event::<KeyUpEvent>(keys.clone(), false);

        Self {
            keys,
            current_heading: na::zero(),
        }
    }
}
impl<'a> System<'a> for MovementControl {
    type SystemData = (
        Read<'a, ServerConnection>,
        Read<'a, Player>,
        WriteStorage<'a, Heading>,
    );

    fn run(&mut self, (sc, player, mut headings): Self::SystemData) {
        // if keys isn't being used by the listener, and the player character has been added.
        if let (Ok(keys), Some(player)) = (self.keys.try_lock(), player.0) {
            // these variables are needed to determine direction from key names.
            if keys.len() > 0 {
                let move_vec = keys.iter().fold(na::zero(), |vec: Vec2, key| match key {
                    ('w', true) => vec + Vec2::new(-1.0, 1.0),
                    ('s', true) => vec + Vec2::new(1.0, -1.0),
                    ('a', true) => vec + Vec2::new(-1.0, -1.0),
                    ('d', true) => vec + Vec2::new(1.0, 1.0),
                    _ => vec,
                });

                if move_vec != self.current_heading {
                    self.current_heading = move_vec;
                    let heading = Heading {
                        dir: na::Unit::new_normalize(move_vec),
                    };

                    // now that we know, tell the server where we'd like to go
                    sc.insert_comp(heading.clone());

                    // and record that locally for clientside prediction
                    headings.insert(player, heading.clone()).expect(
                        "couldn't insert heading to player for clientside movement prediction",
                    );
                }
            }
        }
    }
}

/// The player pressed the key for picking up an item.
pub struct PickupItems {
    pickup_presses: Arc<Mutex<usize>>,
}
impl Default for PickupItems {
    fn default() -> Self {
        let pickup_presses = Arc::new(Mutex::new(0));

        document().add_event_listener({
            let pickup_presses = pickup_presses.clone();

            move |e: KeyPressEvent| {
                if let (true, Some('e')) = (!e.repeat(), e.key().chars().next()) {
                    *pickup_presses
                        .lock()
                        .expect("Can't lock pickup_presses to insert event") += 1;
                }
            }
        });

        Self { pickup_presses }
    }
}
impl<'a> System<'a> for PickupItems {
    type SystemData = (
        Entities<'a>,
        Read<'a, ServerConnection>,
        Read<'a, crate::net::ServerToLocalIds>,
        Read<'a, Player>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Pos>,
    );

    fn run(&mut self, (ents, sc, server_to_local_ids, player, items, poses): Self::SystemData) {
        use comn::item::{PickupRequest, MAX_INTERACTION_DISTANCE_SQUARED};

        if let (Ok(mut pickup_presses), Some(player_entity)) =
            (self.pickup_presses.lock(), player.0)
        {
            let Pos(Iso2 {
                translation: player_translation,
                ..
            }) = match poses.get(player_entity) {
                Some(p) => p,
                // we have a player, but it doesn't have a location yet?
                // well that's fine, but tryna pick up anything just isn't gonna work.
                _ => {
                    trace!("can't look for pickup press; player no pos");
                    return;
                }
            };

            for _pickup_press in 0..*pickup_presses {
                trace!("pickup press event!");

                // grab the entity that's closest to the player
                if let Some(&id) = (&*ents, &poses, &items)
                    .join()
                    // returns (the entity of that item, that item's distance from the player^2)
                    .filter_map(
                        |(
                            item_entity,
                            Pos(Iso2 {
                                translation: item_translation,
                                ..
                            }),
                            _,
                        )| {
                            // first see if the item is close enough to the mouse
                            let item_to_player_distance_squared = (item_translation.vector
                                - player_translation.vector)
                                .magnitude_squared();

                            if item_to_player_distance_squared < MAX_INTERACTION_DISTANCE_SQUARED {
                                trace!("item close enough!");
                                Some((item_entity, item_to_player_distance_squared))
                            } else {
                                trace!(
                                    "click too far away: {}\nitem: {}\ndistance: {}",
                                    player_translation.vector,
                                    item_translation.vector,
                                    item_to_player_distance_squared.sqrt()
                                );
                                None
                            }
                        },
                    )
                    // finds the item with the shortest distance from the click
                    .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(&dist_b).unwrap())
                    // we care about the item's id on the server, not its distance from the player.
                    .and_then(|(item_entity, _)| {
                        server_to_local_ids.0.get_by_right(&item_entity.id())
                    })
                {
                    trace!("sending request for picking up item with id {}", id);
                    sc.insert_comp(PickupRequest { id });
                }
            }
            *pickup_presses = 0;
        }
    }
}
