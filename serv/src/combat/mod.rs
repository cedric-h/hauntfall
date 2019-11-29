use comn::{prelude::*, vec_of_pos};
use specs::{prelude::*, Component};

mod alignment;
pub use alignment::Alignment;

#[derive(Debug, Clone, Component)]
/// Entities with this component will chase other entities
/// who have an Alignment component that matches the one in
/// the `target` field.
/// In order for the chasing to begin and/or continue, these
/// entities must also be within the distance indicated in the
/// `distance` field.
pub struct Chaser {
    /// Entities
    pub target: Alignment,
    pub distance_squared: f32,
    pub speed: f32,
}
pub struct ChaserBuilder {
    pub target: Alignment,
    pub distance: f32,
    pub speed: f32,
}
impl ChaserBuilder {
    pub fn build(self) -> Chaser {
        let ChaserBuilder {
            target,
            distance,
            speed,
        } = self;
        Chaser {
            target,
            speed,
            distance_squared: distance.powi(2),
        }
    }
}

pub struct Chase;
impl<'a> System<'a> for Chase {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Pos>,
        ReadStorage<'a, Alignment>,
        ReadStorage<'a, Chaser>,
    );

    fn run(&mut self, (ents, mut poses, aligns, chasers): Self::SystemData) {
        use na::Translation2;
        (&chasers, &poses, &ents)
            .join()
            // returns (chaser ent, direction to go in)
            .filter_map(|(chaser, vec_of_pos!(chaser_vec), chaser_ent)| {
                (&poses, &aligns)
                    .join()
                    .filter_map(|(vec_of_pos!(chased_vec), align)| {
                        if align == &chaser.target {
                            let delta = chased_vec - chaser_vec;
                            let delta_distance_squared = delta.magnitude_squared();
                            if delta_distance_squared < chaser.distance_squared {
                                return Some((delta, delta_distance_squared));
                            }
                        }
                        None
                    })
                    // finds the entity with the shortest distance from the chaser
                    .min_by(|(_, dist_a), (_, dist_b)| dist_a.partial_cmp(&dist_b).unwrap())
                    .map(|(delta, _)| (chaser_ent, delta))
            })
            .collect::<Vec<_>>()
            .into_iter()
            // move the chaser in the specified direction
            .for_each(|(chaser_ent, dir)| {
                let chaser = chasers.get(chaser_ent).unwrap();
                let chaser_pos = poses.get_mut(chaser_ent).unwrap();
                chaser_pos.0.translation.vector += dir.normalize() * chaser.speed;
            });
    }
}
