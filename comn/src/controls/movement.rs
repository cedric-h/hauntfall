use crate::{
    art::{player_anim::PlayerAnimation, Animate, PlayerAnimationController},
    controls::{Heading, Speed},
    prelude::*,
    Fps,
};
use specs::prelude::*;

pub struct MoveHeadings;
impl<'a> System<'a> for MoveHeadings {
    type SystemData = (
        Read<'a, Fps>,
        WriteStorage<'a, Pos>,
        WriteStorage<'a, Animate>,
        WriteStorage<'a, Heading>,
        ReadStorage<'a, Speed>,
        ReadStorage<'a, PlayerAnimationController>,
    );

    fn run(&mut self, (fps, mut isos, mut animates, mut heads, speeds, anim_controls): Self::SystemData) {
        for (pos, &mut Heading { mut dir }, speed, player_anim_control, animaybe) in (
            &mut isos,
            &mut heads,
            &speeds,
            anim_controls.maybe(),
            (&mut animates).maybe(),
        )
            .join()
        {
            if dir.magnitude() > 0.0 {
                // TODO: optimize this
                dir.renormalize();

                // 20 fps = 3, 60 fps = 1
                let update_granularity = 1.0 / fps.0 * 60.0;
                pos.iso.translation.vector += dir.into_inner() * speed.speed * update_granularity;

                if let (true, Some(anim)) = (player_anim_control.is_some(), animaybe) {
                    use crate::art::player_anim::Direction::*;

                    let direction = if dir.x > 0.0 {
                        Right
                    } else if dir.x < 0.0 {
                        Left
                    } else if dir.y > 0.0 {
                        Down
                    } else {
                        Up
                    };

                    anim.row = PlayerAnimation::Walk(direction).into();
                }
            } else {
                if let (true, Some(anim)) = (player_anim_control.is_some(), animaybe) {
                    anim.current_frame = 0;
                }
            }
        }
    }
}
