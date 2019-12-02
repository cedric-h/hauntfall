use crate::prelude::*;
use bimap::BiMap;
use comn::{na::Translation2, vec_of_pos};
use comn::{NetComponent, NetMessage, Pos};
use std::sync::{Arc, Mutex};
use stdweb::{
    unstable::TryInto,
    web::{
        event::{SocketCloseEvent, SocketErrorEvent, SocketMessageEvent, SocketOpenEvent},
        ArrayBuffer, IEventTarget, WebSocket,
    },
    Value,
};

#[derive(Default)]
pub struct Player(pub Option<Entity>);

pub struct ServerConnection {
    ws: WebSocket,
    pub message_queue: Arc<Mutex<Vec<NetMessage>>>,
}
impl ServerConnection {
    #[inline]
    fn send(&self, msg: NetMessage) {
        self.ws
            .send_bytes(&rmps::encode::to_vec(&msg).expect("Couldn't encode NetMessage!"))
            .expect("Couldn't send NetMessage to server!");
    }

    /* I'm not sure why/when/how you'd ever even actually use this on the client.
    * The server should definitely be in control of when new things are made,
    * even if indirectly the Client ends up requesting that to happen.
    * For that reason, this is prevented from working on the serverside.
    #[inline]
    pub fn new_ent(&self, ent: specs::Entity) {
    self.send(NetMessage::NewEnt(ent.id()));
    }*/

    #[inline]
    pub fn insert_comp<C: Into<NetComponent>>(
        &self,
        // The client can only request that components are
        // inserted onto itself.
        // ent: specs::Entity,
        comp: C,
    ) {
        // just using a 0 here for the entity ID since they can
        // only insert components onto their own entity.
        self.send(NetMessage::InsertComp(0, comp.into()));
    }
}

impl Default for ServerConnection {
    fn default() -> Self {
        let ws = WebSocket::new("ws://127.0.0.1:3012")
            .unwrap_or_else(|e| panic!("couldn't reach server: {}", e));
        let message_queue = Arc::new(Mutex::new(Vec::new()));

        ws.add_event_listener(|_: SocketOpenEvent| {
            info!("Connected to server!");
        });

        ws.add_event_listener(|e: SocketErrorEvent| {
            error!("Errror connecting to {:?}s", e);
        });

        ws.add_event_listener(|e: SocketCloseEvent| {
            error!("Server Connection Closed: {}s", e.reason());
        });

        ws.add_event_listener({
            let msgs = message_queue.clone();

            move |msg: SocketMessageEvent| {
                let msgs = msgs.clone();

                let parse_msg_data = move |data: Value| {
                    let buf: ArrayBuffer = data
                        .try_into()
                        .expect("Couldn't turn server message into array buffer!");

                    let mut msgs = msgs.lock().expect("The Server Message Queue is locked!");
                    msgs.push(
                        rmps::from_read_ref::<Vec<u8>, _>(&buf.into())
                            .expect("couldn't read net message bytes"),
                    );
                };

                js! {
                    let reader = new FileReader();
                    reader.addEventListener("loadend", () => {
                        let parse = @{parse_msg_data};
                        parse(reader.result);
                        parse.drop();
                    });
                    reader.readAsArrayBuffer(@{msg}.data);
                };
            }
        });

        Self { ws, message_queue }
    }
}

use comn::net::UpdatePosition;
pub struct SyncPositions;
impl<'a> System<'a> for SyncPositions {
    type SystemData = (
        WriteStorage<'a, Pos>,
        ReadStorage<'a, UpdatePosition>,
        ReadStorage<'a, comn::controls::Heading>,
    );

    // the idea here is to get wherever the client thinks something is to where the server has
    // it at within 10 ms.
    // You want to do that transition gradually to avoid sudden jerking.
    // If the internet is being slow and the update is from a while ago, however, it's probably
    // more apt to just rely on the physics simulation on the client than on the last position
    // the server sent; that way things in the simulation will still move.
    fn run(&mut self, (mut currents, updates, headings): Self::SystemData) {
        for (
            vec_of_pos!(at),
            &UpdatePosition {
                iso: Iso2 {
                    translation: go, ..
                },
                ..
            },
            heading,
        ) in (&mut currents, &updates, headings.maybe()).join()
        {
            if let Some(heading) = heading {
                if heading.dir.magnitude() > 0.0 {
                    continue;
                }
            }
            *at = at.lerp(&go.vector, 0.03);
            /*
            current.rotation = na::UnitComplex::from_complex(
            current.rotation.complex()
            + current.rotation.rotation_to(&update.rotation).complex() * 0.06,
            );*/
        }
    }
}

#[derive(Default)]
pub struct ServerToLocalIds(pub BiMap<u32, u32>);

#[derive(Default)]
pub struct HandleServerPackets {
    connection_established: bool,
    /// This system caches this value until it recieves it from the server,
    /// then it can know the local id (not the server id) of the Player,
    /// so it can then write to the Resource.
    local_player_server_id: Option<u32>,
}
impl<'a> System<'a> for HandleServerPackets {
    type SystemData = (
        Entities<'a>,
        Write<'a, ServerToLocalIds>,
        Read<'a, LazyUpdate>,
        Read<'a, ServerConnection>,
    );

    fn run(&mut self, (ents, mut server_to_local_ids, lu, sc): Self::SystemData) {
        if let Ok(mut msgs) = sc.message_queue.try_lock() {
            for msg in msgs.drain(0..) {
                // you know the connection is established when
                // we first get a message.
                if !self.connection_established {
                    // immediately request to be put in the game
                    // (later on we might want to have this happen
                    //  after i.e. a menu is clicked through)
                    sc.insert_comp(comn::net::SpawnPlayer);
                    self.connection_established = true;
                }

                use NetMessage::*;

                match msg {
                    NewEnt(server) => {
                        let local: u32 = ents.create().id();
                        server_to_local_ids.0.insert(server, local);

                        // record address if the thing we're instantiating is the player
                        if let Some(true) = self.local_player_server_id.map(|id| id == server) {
                            trace!("found player!");
                            lu.exec(move |world| {
                                let mut player = world.write_resource::<Player>();
                                player.0 = Some(world.entities().entity(local));
                            });
                            // no need to cache it now.
                            self.local_player_server_id = None;
                        }
                    }

                    InsertComp(id, net_comp) => {
                        // figure out what that entity's id is on our side.
                        let ent = server_to_local_ids
                            .0
                            .get_by_left(&id)
                            .map(|ent| ents.entity(*ent))
                            .filter(|ent| {
                                if !ents.is_alive(*ent) {
                                    info!("filtering out dead ent");
                                }
                                ents.is_alive(*ent)
                            });

                        if let Some(ent) = ent {
                            // if they're alive, insert,
                            net_comp.insert(ent, &lu);
                        } else {
                            // otherwise just throw an error,
                            // why is the server telling us about dead people?
                            error!(
                                "Can't insert component for dead entity, component: {:?}",
                                net_comp
                            );
                        }
                    }

                    Establishment {
                        local_player,
                        appearance_record,
                    } => {
                        info!("establishment");

                        // start loading the assets we'll need
                        js!(load_assets(@{appearance_record.names.clone()}));

                        // store that in the ECS
                        lu.exec_mut(move |world| {
                            world.insert(appearance_record);
                        });

                        // store our server ID until the server tells us about it.
                        self.local_player_server_id = Some(local_player);
                    }
                }
            }
        }
    }
}
