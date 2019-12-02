use crate::prelude::*;
use crate::combat::{Damage, Health, Hurtbox};
use crate::{collide, controls::Heading, Hitbox};
use specs::prelude::*;

/// Currently, Collision serves to prevent people who are trying to go through things
/// from going through those things.
pub struct Collision;
impl<'a> System<'a> for Collision {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Damage>,
        ReadStorage<'a, Hurtbox>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Hitbox>,
        ReadStorage<'a, Heading>,
    );

    fn run(&mut self, (ents, mut poses, mut dmgs, hurtboxes, hps, hitboxes, headings): Self::SystemData) {
        use collide::query::contact;
        use na::Translation2;

        // for everyone going somewhere...
        (&*ents, &poses, &hitboxes, &headings)
            .join()
            .filter_map(|(ent, Pos { iso }, Hitbox { cuboid: hb }, _)| {
                // for everything they could collide with...
                for (o_ent, Pos { iso: o_iso }, Hitbox { cuboid: o_hb }) in
                    (&*ents, &poses, &hitboxes).join()
                {
                    if ent != o_ent {
                        // they're touching the goer! goer goes back!
                        if let Some(c) = contact(iso, hb, o_iso, o_hb, 0.0) {
                            return Some((ent, o_ent, c.normal.into_inner() * c.depth));
                        }
                    }
                }
                None
            })
            .collect::<Vec<_>>()
            .iter()
            .for_each(|(goer_ent, o_ent, normal)| {
                let vec_of_pos!(loc) = poses.get_mut(*goer_ent).unwrap();
                *loc -= normal;

                // apply damage if the goer has a hurtbox
                // and the other has a health.
                let hurtbox = hurtboxes.get(*goer_ent);
                if let (Some(hurtbox), true) = (hurtbox, hps.get(*o_ent).is_some()) {
                    dmgs.insert(*o_ent, hurtbox.clone().into_damage(normal))
                        .expect("Couldn't insert Damage from Hurtbox!");
                }
            });
    }
}
