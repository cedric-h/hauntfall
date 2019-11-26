use crate::prelude::*;
use comn::art::Appearance;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct RenderEntry {
    ent: u32,
    appearance: Appearance,
    iso: Iso2,
}
stdweb::js_serializable!(RenderEntry);

pub const ZOOM: f32 = 20.0;
pub const CANVAS_ZOOM: f32 = 2.0; //change this in renderer.js
pub const TOTAL_ZOOM: f32 = ZOOM * CANVAS_ZOOM;

pub struct Render;

impl Default for Render {
    fn default() -> Self {
        Render
    }
}

impl<'a> System<'a> for Render {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Appearance>,
        ReadStorage<'a, Pos>,
    );

    fn run(&mut self, (ents, appearances, poses): Self::SystemData) {
        // tiles are rendered as if their origin was their center on the X and Y.
        // also, tiles are rendered first so that everything else can step on them.
        let render_entries = (&*ents, &appearances, &poses)
            .join()
            .map(|(ent, a, Pos(i))| RenderEntry {
                ent: ent.id(),
                appearance: a.clone(),
                iso: i.clone(),
            })
            .collect::<Vec<_>>();

        js!(render(@{render_entries}));
    }
}
