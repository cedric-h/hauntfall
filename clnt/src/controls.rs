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
        event::{ConcreteEvent, DoubleClickEvent, KeyPressEvent, KeyUpEvent},
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

pub struct MouseControl {
    mouse_events: Arc<Mutex<Vec<Vec2>>>,
}
impl Default for MouseControl {
    fn default() -> Self {
        let mouse_events = Arc::new(Mutex::new(Vec::new()));

        document().add_event_listener({
            use crate::stdweb::traits::IMouseEvent;
            let mouse_events = mouse_events.clone();

            move |e: DoubleClickEvent| {
                trace!("click!");
                mouse_events
                    .lock()
                    .expect("Can't lock mouse_events to insert event")
                    .push(Vec2::new(e.client_x() as f32, e.client_y() as f32));
            }
        });

        Self { mouse_events }
    }
}
impl<'a> System<'a> for MouseControl {
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
        const MAX_ITEM_TO_MOUSE_DISTANCE_SQUARED: f32 = {
            let f = 2.0;
            f * f
        };

        if let (Ok(mut mouse_events), Some(player_entity)) = (self.mouse_events.lock(), player.0) {
            let Pos(Iso2 {
                translation: player_translation,
                ..
            }) = match poses.get(player_entity) {
                Some(p) => p,
                // we have a player, but it doesn't have a location yet?
                // well that's fine, but clicking anything just isn't gonna work.
                _ => {
                    trace!("can't look for click; player no pos");
                    return;
                }
            };

            for screen_click in mouse_events.drain(..) {
                trace!("mouse event!");
                let click = screen_click / crate::renderer::TOTAL_ZOOM;
                if let Some(&id) = (&*ents, &poses, &items)
                    .join()
                    // returns (the entity of that item, that item's distance from the mouse)
                    .filter_map(
                        |(
                            item_entity,
                            Pos(Iso2 {
                                translation: item_translation,
                                ..
                            }),
                            _,
                        )| {
                            trace!("click detected: {}", click);
                            // first see if the item is close enough to the mouse
                            let item_to_click_distance_squared =
                                (item_translation.vector - click).magnitude_squared();

                            if item_to_click_distance_squared < MAX_ITEM_TO_MOUSE_DISTANCE_SQUARED
                                && ({
                                    trace!("mouse click was close enough to item...");
                                    // if that's true, make sure it's also close enough to the player.
                                    let item_to_player_distance_squared =
                                        (player_translation.vector - item_translation.vector)
                                            .magnitude_squared();

                                    item_to_player_distance_squared
                                        < MAX_INTERACTION_DISTANCE_SQUARED
                                })
                            {
                                trace!("click close enough to item and player!");
                                Some((item_entity, item_to_click_distance_squared))
                            } else {
                                trace!(
                                    "click too far away: {}\nitem: {}\ndistance: {}",
                                    click,
                                    item_translation.vector,
                                    item_to_click_distance_squared.sqrt()
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
        }
    }
}
