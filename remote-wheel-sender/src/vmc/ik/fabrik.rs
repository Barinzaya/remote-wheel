use glam::{Quat, Vec3A};

use super::{Chain, Link};

pub struct Settings {
    pub max_iterations: u32,
    pub pos_tolerance: f32,
}

pub fn solve(settings: &Settings, chain: &mut impl Chain, target_pos: Vec3A) -> Result<u32, ()> {
    let num_links = chain.num_links();

    assert!(num_links > 0);
    let last_link = num_links - 1;

    for i in 0..settings.max_iterations {
        let mut old_next_pos = chain.link(last_link).pos();
        let mut new_next_pos = target_pos;

        for j in (1..last_link).into_iter().rev() {
            let base_rot = chain.link(j - 1).rot();

            let mut link = chain.link(j);
            let old_pos = link.pos();
            let old_rot = link.rot();

            let old_delta = old_next_pos - old_pos;
            let new_delta = new_next_pos - old_pos;

            let length = old_delta.length();
            let old_dir = old_delta.normalize_or_zero();
            let new_dir = new_delta.normalize_or_zero();

            let new_rot = Quat::from_rotation_arc(old_dir.into(), new_dir.into()) * old_rot;
            let new_rot = base_rot
                * link
                    .angular_constraint()
                    .apply(base_rot.inverse() * new_rot);
            link.set_rot(new_rot);

            let new_dir = (old_rot.inverse() * new_rot) * old_dir;
            let new_pos = new_next_pos - length * new_dir;
            old_next_pos = old_pos;
            new_next_pos = new_pos;
        }

        let mut end = chain.link(last_link);
        if end.pos().distance(target_pos) <= settings.pos_tolerance {
            return Ok(i + 1);
        }
    }

    Err(())
}
