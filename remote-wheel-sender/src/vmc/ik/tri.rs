use std::f32::consts::PI;

use glam::{Quat, Vec3, Vec3A};

use super::{Chain, Link};

pub struct Settings {
    pub elbow_axis: Vec3A,
    pub max_iterations: u32,
    pub rot_tolerance: f32,
}

pub fn solve(
    settings: &Settings,
    chain: &mut impl Chain,
    target_pos: Vec3A,
    target_rot: Quat,
) -> Result<u32, ()> {
    let num_links = chain.num_links();
    debug_assert!(num_links == 4);

    let ((shoulder_pos, mut shoulder_rot), shoulder_constraint) = {
        let mut link = chain.link(1);
        (link.state(), link.angular_constraint())
    };
    let (elbow_pos, elbow_constraint) = {
        let mut link = chain.link(2);
        (link.pos(), link.angular_constraint())
    };
    let wrist_pos = chain.link(3).pos();

    let target_offset = target_pos - shoulder_pos;
    let target_dist = target_offset.length();
    let target_dir = target_offset / target_dist;

    let upper_length = (elbow_pos - shoulder_pos).length();
    let lower_length = (wrist_pos - elbow_pos).length();

    if upper_length == 0.0 || lower_length == 0.0 {
        return Err(());
    }

    let elbow_angle = if target_dist < upper_length + lower_length {
        let num =
            upper_length * upper_length + lower_length * lower_length - target_dist * target_dist;
        let den = 2.0 * upper_length * lower_length;
        PI - (num / den).acos()
    } else {
        0.0
    };

    let elbow_rot = elbow_constraint.apply(Quat::from_axis_angle(
        settings.elbow_axis.into(),
        elbow_angle,
    ));

    chain.link(2).set_rot(shoulder_rot * elbow_rot);

    let base_rot = chain.link(0).rot();
    let base_inv_rot = base_rot.inverse();

    for i in 0..settings.max_iterations {
        let wrist_pos = chain.link(3).pos();
        let wrist_dir = (wrist_pos - shoulder_pos).normalize_or_zero();

        let mut ideal_rot =
            Quat::from_rotation_arc(wrist_dir.into(), target_dir.into()) * shoulder_rot;

        if i == 0 {
            let twist = Quat::from_rotation_arc(ideal_rot * Vec3::Y, target_rot * Vec3::Y)
                .to_scaled_axis()
                .dot(target_dir.into());
            ideal_rot = Quat::from_axis_angle(target_dir.into(), 0.5 * twist) * ideal_rot;
        }

        let constrained_rot = base_rot * shoulder_constraint.apply(base_inv_rot * ideal_rot);

        shoulder_rot = base_rot * constrained_rot;
        chain.link(1).set_rot(shoulder_rot);

        if ideal_rot.angle_between(constrained_rot) <= settings.rot_tolerance {
            chain.link(3).set_rot(target_rot);
            return Ok(i + 1);
        }
    }

    chain.link(3).set_rot(target_rot);
    Err(())
}
