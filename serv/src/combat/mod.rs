pub mod alignment;
pub use alignment::Alignment;

pub mod chase;
pub use chase::{Chase, Chaser};

mod damage {
    use crate::net::prelude::*;
    use comn::combat::{Damage, Health};
    use comn::prelude::*;
    use comn::Dead;
    use comn::{na::Translation2, vec_of_pos};
    use specs::prelude::*;

    /// How little the knockback needs to get before it's just gotten rid of.
    const MIN_KNOCKBACK: f32 = 0.05;
    /// Squared so that it can be compared to squared vector lengths,
    /// which can be calculated faster.
    const MIN_KNOCKBACK_SQUARED: f32 = MIN_KNOCKBACK * MIN_KNOCKBACK;
    /// How quickly the knockback fades away.
    const KNOCKBACK_DECAY: f32 = 0.85;

    pub struct DealDamage;
    impl<'a> System<'a> for DealDamage {
        type SystemData = (
            Entities<'a>,
            Read<'a, ConnectionManager>,
            WriteStorage<'a, Damage>,
            WriteStorage<'a, Health>,
            WriteStorage<'a, Pos>,
            WriteStorage<'a, Dead>,
            ReadStorage<'a, Client>,
        );

        fn run(
            &mut self,
            (ents, cm, mut damages, mut hps, mut poses, mut dead, clients): Self::SystemData,
        ) {
            for (ent, ref mut dmg, ref mut hp, vec_of_pos!(loc)) in
                (&*ents, &mut damages, &mut hps, &mut poses).join()
            {
                if dmg.hp != 0 {
                    log::info!("doin' damage!");
                    match hp.current.checked_sub(dmg.hp) {
                        Some(new_hp) => hp.current = new_hp,
                        // hp below 0!!!
                        None => {
                            // u ded
                            dead.insert(ent, Dead)
                                .expect("Couldn't kill below 0 hp entity!");
                            log::trace!("damage killin' 'em[{}]!", ent.id());
                            for Client(addr) in (&clients).join() {
                                cm.insert_comp(*addr, ent, Dead);
                            }
                        }
                    }
                    dmg.hp = 0;
                }
                if dmg.knockback.magnitude_squared() > MIN_KNOCKBACK_SQUARED {
                    *loc -= dmg.knockback;
                    dmg.knockback *= KNOCKBACK_DECAY;
                }
            }
        }
    }
}
pub use damage::DealDamage;

mod attack {
    use super::Alignment;
    // comn
    use comn::combat::{AttackRequest, Damage, Health};
    use comn::item::{Inventory, WEAPON_SLOT};
    use comn::prelude::*;
    use comn::{na::Translation2, vec_of_pos};
    // crates
    use specs::prelude::*;

    pub struct LaunchAttacks;
    impl<'a> System<'a> for LaunchAttacks {
        type SystemData = (
            Entities<'a>,
            WriteStorage<'a, AttackRequest>,
            WriteStorage<'a, Damage>,
            ReadStorage<'a, Health>,
            ReadStorage<'a, Alignment>,
            ReadStorage<'a, Pos>,
            ReadStorage<'a, Inventory>,
        );

        fn run(
            &mut self,
            (ents, mut attacks, mut damages, hps, aligns, poses, inventories): Self::SystemData,
        ) {
            const RANGE_SQUARED: f32 = 2.0 * 2.0;
            for (_, vec_of_pos!(atkr_loc), inv, atkr_align) in
                (attacks.drain(), &poses, &inventories, &aligns).join()
            {
                // attempting attack!
                if let Some(_wep_ent) = inv.slot(&WEAPON_SLOT).expect("attacker no wep slot!?!") {
                    // iterate over everything close enough to hit that's on
                    // another team
                    for (delta, attacked_ent) in (&*ents, &poses, &aligns, &hps).join().filter_map(
                        // returns (vector between atkr - atkee, atkee_ent)
                        |(ent, vec_of_pos!(loc), align, _)| {
                            // can't be on our team
                            if atkr_align != align {
                                let delta = atkr_loc - loc;
                                // gotta be close enough
                                if delta.magnitude_squared() < RANGE_SQUARED {
                                    return Some((delta.normalize(), ent));
                                } else {
                                    // not close enough
                                }
                            } else {
                                // that'd be friendly fire!
                            }
                            // ruled out
                            None
                        },
                    ) {
                        log::trace!("damaging[{}]; attack requirements met!", attacked_ent.id());
                        damages
                            .insert(
                                attacked_ent,
                                Damage {
                                    hp: 1,
                                    knockback: delta * 0.7,
                                },
                            )
                            .expect("Couldn't insert Damage onto Attacked entity.");
                    }
                }
            }
        }
    }
}
pub use attack::LaunchAttacks;
