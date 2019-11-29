use crate::prelude::*;
use comn::art::Appearance;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct RenderEntry {
    ent: u32,
    iso: Iso2,
}
stdweb::js_serializable!(RenderEntry);

#[derive(Serialize, Deserialize)]
struct AppearanceEntry {
    ent: u32,
    appearance_index: Appearance,
}
stdweb::js_serializable!(AppearanceEntry);

#[derive(Default)]
pub struct Render {
    pub reader_id: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for Render {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Appearance>,
        ReadStorage<'a, Pos>,
    );

    fn run(&mut self, (ents, appearances, poses): Self::SystemData) {
        let events = appearances.channel().read(self.reader_id.as_mut().unwrap());

        for event in events {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    js!(set_appearance(@{AppearanceEntry {
                        ent: *id,
                        appearance_index: appearances.get(ents.entity(*id))
                            .expect("Couldn't read appearance on modification/insert to give to JS")
                            .clone(),
                    }}));
                }
                ComponentEvent::Removed(id) => {
                    js!(clear_appearance(@{id}));
                }
            }
        }

        let render_entries = (&*ents, &poses)
            .join()
            .map(|(ent, Pos(i))| RenderEntry {
                ent: ent.id(),
                iso: i.clone(),
            })
            .collect::<Vec<_>>();

        js!(render(@{render_entries}));
    }

    fn setup(&mut self, res: &mut World) {
        Self::SystemData::setup(res);
        self.reader_id = Some(WriteStorage::<Appearance>::fetch(&res).register_reader());
    }
}
