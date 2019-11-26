use crate::prelude::*;
use bimap::BiMap;
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
    * Instead, it's used internally to register a new player; if you send
    * this request through hacking or some other means, you'll just get
    * your player reset :grin:
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
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, UpdatePosition>);

    // the idea here is to get wherever the client thinks something is to where the server has
    // it at within 10 ms.
    // You want to do that transition gradually to avoid sudden jerking.
    // If the internet is being slow and the update is from a while ago, however, it's probably
    // more apt to just rely on the physics simulation on the client than on the last position
    // the server sent; that way things in the simulation will still move.
    fn run(&mut self, (mut currents, updates): Self::SystemData) {
        for (
            Pos(Iso2 {
                translation: at, ..
            }),
            UpdatePosition {
                iso: Iso2 {
                    translation: go, ..
                },
                ..
            },
        ) in (&mut currents, &updates).join()
        {
            /*
            const LERP_DIST: f32 = 0.03;
            let to_go = go.vector - at.vector;

            if to_go.magnitude().abs() > 2.0 * LERP_DIST {
            at.vector += to_go.normalize() * LERP_DIST;
            } */
            at.vector = at.vector.lerp(&go.vector, 0.03);
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
    pub connection_established: bool,
}
impl<'a> System<'a> for HandleServerPackets {
    type SystemData = (
        Entities<'a>,
        Write<'a, ServerToLocalIds>,
        Write<'a, Player>,
        Read<'a, LazyUpdate>,
        Read<'a, ServerConnection>,
    );

    fn run(&mut self, (ents, mut server_to_local_ids, mut player, lu, sc): Self::SystemData) {
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
                    }
                    InsertComp(id, net_comp) => {
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
                            match net_comp {
                                // I should really have some sort of
                                // Establishment packet that deals with this.
                                NetComponent::LocalPlayer(_) => {
                                    player.0 = Some(ent);
                                }
                                _ => net_comp.insert(ent, &lu),
                            }
                        } else {
                            error!(
                                "Can't insert component for dead entity, component: {:?}",
                                net_comp
                            );
                        }
                    }
                }
            }
        }
    }
}
