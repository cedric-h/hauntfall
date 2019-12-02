use crate::prelude::*;
use comn::art::Appearance;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct RenderEntry {
    ent: u32,
    iso: Iso2,
    rot: f32,
}
stdweb::js_serializable!(RenderEntry);

#[derive(Serialize, Deserialize)]
struct AppearanceEntry {
    ent: u32,
    appearance_index: usize,
}
stdweb::js_serializable!(AppearanceEntry);

#[derive(Serialize, Deserialize)]
struct PlayerPos {
    vec: Vec2,
}
stdweb::js_serializable!(PlayerPos);

#[derive(Default)]
pub struct Render {
    pub reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for Render {
    type SystemData = (
        Entities<'a>,
        Read<'a, Player>,
        ReadStorage<'a, Appearance>,
        ReadStorage<'a, Pos>,
    );

    fn run(&mut self, (ents, player, appearances, poses): Self::SystemData) {
        let events = appearances.channel().read(self.reader_id.as_mut().unwrap());

        for event in events {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    js!(set_appearance(@{AppearanceEntry {
                        ent: *id,
                        appearance_index: appearances.get(ents.entity(*id))
                            .expect("Couldn't read appearance on modification/insert to give to JS")
                            .index,
                    }}));
                }
                ComponentEvent::Removed(id) => {
                    js!(clear_appearance(@{id}));
                }
            }
        }

        let render_entries = (&*ents, &poses)
            .join()
            .map(|(ent, Pos { iso })| RenderEntry {
                ent: ent.id(),
                iso: iso.clone(),
                rot: iso.rotation.angle(),
            })
            .collect::<Vec<_>>();

        let player_pos = PlayerPos {
            vec: player.0
            .and_then(|x| {
                poses
                    .get(x)
                    .map(|x| x.iso.translation.vector)
            })
            .unwrap_or_else(|| na::zero())
        };

        js!(render(@{render_entries}, @{player_pos}));
    }

    fn setup(&mut self, res: &mut World) {
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<Appearance>::fetch(&res).register_reader());
    }
}
