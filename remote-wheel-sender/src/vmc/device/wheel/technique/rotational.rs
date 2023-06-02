use std::{cmp::Ordering, f32::consts::TAU};

use anyhow::{ensure, Error as AnyError, Result as AnyResult};
use glam::{EulerRot, Quat, Vec2, Vec3A};
use serde::{Deserialize, Deserializer};

use crate::vmc::{
    avatar::Pose,
    bone::{Bone, Limb},
    device::{ForwardPose, Wheel},
    math::FloatExt,
};

#[derive(Debug)]
pub struct Technique {
    cross_start: f32,
    cross_grip: f32,
    cross_out: f32,
    cross_retract: f32,
    cross_end: f32,

    turn_start: f32,
    turn_grip: f32,
    turn_lift: f32,
    turn_out: f32,
    turn_end: f32,

    rotation_base: f32,
    rotation_offset: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct TechniqueConfig {
    #[serde(default = "cross_start_default", deserialize_with = "parse_degrees")]
    cross_start: f32,

    #[serde(default = "cross_grip_default", deserialize_with = "parse_degrees")]
    cross_grip: f32,

    #[serde(default = "cross_out_default")]
    cross_out: f32,

    #[serde(default = "cross_retract_default")]
    cross_retract: f32,

    #[serde(default = "cross_end_default", deserialize_with = "parse_degrees")]
    cross_end: f32,

    #[serde(default = "turn_start_default", deserialize_with = "parse_degrees")]
    turn_start: f32,

    #[serde(default = "turn_grip_default", deserialize_with = "parse_degrees")]
    turn_grip: f32,

    #[serde(default = "turn_lift_default")]
    turn_lift: f32,

    #[serde(default = "turn_out_default")]
    turn_out: f32,

    #[serde(default = "turn_end_default", deserialize_with = "parse_degrees")]
    turn_end: f32,
}

fn cross_start_default() -> f32 {
    110.0f32.to_radians()
}

fn cross_grip_default() -> f32 {
    5.0f32.to_radians()
}

fn cross_out_default() -> f32 {
    0.25
}

fn cross_retract_default() -> f32 {
    1.3
}

fn cross_end_default() -> f32 {
    250.0f32.to_radians()
}

fn turn_start_default() -> f32 {
    270.0f32.to_radians()
}

fn turn_grip_default() -> f32 {
    5.0f32.to_radians()
}

fn turn_lift_default() -> f32 {
    0.6
}

fn turn_out_default() -> f32 {
    0.25
}

fn turn_end_default() -> f32 {
    290.0f32.to_radians()
}

impl Technique {
    pub fn pose_forward(&self, _: &Wheel, mut f: impl FnMut(Bone, f32, ForwardPose)) {
        let left_bones = [
            (Bone::LeftIndexProximal, Quat::from_rotation_z(0.1 * TAU)),
            (
                Bone::LeftIndexIntermediate,
                Quat::from_rotation_z(0.25 * TAU),
            ),
            (Bone::LeftIndexDistal, Quat::from_rotation_z(0.1 * TAU)),
            (Bone::LeftMiddleProximal, Quat::from_rotation_z(0.1 * TAU)),
            (
                Bone::LeftMiddleIntermediate,
                Quat::from_rotation_z(0.25 * TAU),
            ),
            (Bone::LeftMiddleDistal, Quat::from_rotation_z(0.1 * TAU)),
            (Bone::LeftRingProximal, Quat::from_rotation_z(0.1 * TAU)),
            (
                Bone::LeftRingIntermediate,
                Quat::from_rotation_z(0.25 * TAU),
            ),
            (Bone::LeftRingDistal, Quat::from_rotation_z(0.1 * TAU)),
            (Bone::LeftLittleProximal, Quat::from_rotation_z(0.1 * TAU)),
            (
                Bone::LeftLittleIntermediate,
                Quat::from_rotation_z(0.25 * TAU),
            ),
            (Bone::LeftLittleDistal, Quat::from_rotation_z(0.1 * TAU)),
            (
                Bone::LeftThumbProximal,
                Quat::from_euler(EulerRot::YZX, -0.02 * TAU, 0.0, 0.08 * TAU),
            ),
            (
                Bone::LeftThumbIntermediate,
                Quat::from_rotation_y(-0.08 * TAU),
            ),
            (Bone::LeftThumbDistal, Quat::from_rotation_y(-0.03 * TAU)),
        ];

        let right_bones = [
            (Bone::RightIndexProximal, Quat::from_rotation_z(-0.1 * TAU)),
            (
                Bone::RightIndexIntermediate,
                Quat::from_rotation_z(-0.25 * TAU),
            ),
            (Bone::RightIndexDistal, Quat::from_rotation_z(-0.1 * TAU)),
            (Bone::RightMiddleProximal, Quat::from_rotation_z(-0.1 * TAU)),
            (
                Bone::RightMiddleIntermediate,
                Quat::from_rotation_z(-0.25 * TAU),
            ),
            (Bone::RightMiddleDistal, Quat::from_rotation_z(-0.1 * TAU)),
            (Bone::RightRingProximal, Quat::from_rotation_z(-0.1 * TAU)),
            (
                Bone::RightRingIntermediate,
                Quat::from_rotation_z(-0.25 * TAU),
            ),
            (Bone::RightRingDistal, Quat::from_rotation_z(-0.1 * TAU)),
            (Bone::RightLittleProximal, Quat::from_rotation_z(-0.1 * TAU)),
            (
                Bone::RightLittleIntermediate,
                Quat::from_rotation_z(-0.25 * TAU),
            ),
            (Bone::RightLittleDistal, Quat::from_rotation_z(-0.1 * TAU)),
            (
                Bone::RightThumbProximal,
                Quat::from_euler(EulerRot::YZX, 0.02 * TAU, 0.0, 0.08 * TAU),
            ),
            (
                Bone::RightThumbIntermediate,
                Quat::from_rotation_y(0.08 * TAU),
            ),
            (Bone::RightThumbDistal, Quat::from_rotation_y(0.03 * TAU)),
        ];

        let left_open = self.pose_forward_single(-self.rotation_offset);
        for (bone, rot) in left_bones {
            f(bone, 1.0, ForwardPose::Local(Quat::IDENTITY.slerp(rot, left_open)));
        }

        let right_open = self.pose_forward_single(self.rotation_offset);
        for (bone, rot) in right_bones {
            f(bone, 1.0, ForwardPose::Local(Quat::IDENTITY.slerp(rot, right_open)));
        }
    }

    pub fn pose_inverse(
        &self,
        pose: &Pose,
        wheel: &Wheel,
        mut f: impl FnMut(Limb, f32, Vec3A, Quat),
    ) {
        let limbs = [
            (
                Limb::LeftHand,
                Bone::LeftUpperArm,
                Bone::LeftMiddleDistal,
                self.pose_inverse_single(-self.rotation_offset),
                0.5 * TAU,
                0.25 * TAU,
                1.0,
            ),
            (
                Limb::RightHand,
                Bone::RightUpperArm,
                Bone::RightMiddleDistal,
                self.pose_inverse_single(self.rotation_offset),
                0.0 * TAU,
                -0.25 * TAU,
                -1.0,
            ),
        ];

        for (limb, shoulder, finger, (pos_offset, rot_offset, lift, retract), angle, yaw, scale) in
            limbs
        {
            let (pos_offset, rot_offset) = (scale * pos_offset, scale * rot_offset);

            let mut local_pos =
                Vec3A::from((wheel.radius * Vec2::from_angle(pos_offset + angle), 0.0));

            if lift > 0.0 {
                let mut length = 0.0;

                let root = limb.end_bone();
                let mut end = finger;

                while end != root {
                    length += pose.local_transform(end).0.length();
                    end = end.parent().expect("Finger is not attached to limb!");
                }

                local_pos *= 1.0 + self.turn_out * lift;
                local_pos.z = -length * lift;
            }

            if retract > 0.0 {
                let out = retract
                    .inv_lerp_checked(0.0, 0.25)
                    .unwrap_or_else(|| retract.inv_lerp(1.0, 0.25))
                    .ease(-1.5);
                local_pos *= 1.0 + self.cross_out * out;
            }

            let mut global_pos = wheel.pos + wheel.base_rot * local_pos;
            let mut global_rot = wheel.base_rot
                * Quat::from_euler(
                    EulerRot::YXZ,
                    yaw,
                    yaw - yaw.signum() * (rot_offset + angle),
                    0.0,
                );

            if retract > 0.0 {
                let this_shoulder_pos = pose.global_transform(shoulder).0;
                let forward_dir = wheel.base_rot * Vec3A::Z;
                let reach_dir = Vec3A::normalize(global_pos - this_shoulder_pos);

                global_rot = Quat::IDENTITY.slerp(
                    Quat::from_rotation_arc(forward_dir.into(), reach_dir.into()),
                    retract,
                ) * global_rot;

                let other_hand_pos = wheel.pos + wheel.rot * Vec3A::from((wheel.radius * Vec2::from_angle(angle + 0.5 * TAU), 0.0));
                let other_shoulder_pos = pose.global_transform(shoulder.mirror()).0;

                let mut retract_point = other_shoulder_pos + (global_pos - other_shoulder_pos).project_onto(other_hand_pos - other_shoulder_pos);
                retract_point = retract_point.lerp(other_hand_pos, 0.5);
                retract_point.y = global_pos.y;

                global_pos = global_pos.lerp(retract_point, retract * self.cross_retract);
            }

            f(limb, 1.0, global_pos, global_rot);
        }
    }

    pub fn set_rotation(&mut self, angle: f32) {
        let mut base = self.rotation_base;
        let mut offset = angle.to_radians() - base;
        let wrap = f32::max(self.cross_end + self.cross_grip, self.turn_end + self.turn_grip);

        while offset < -wrap {
            base -= TAU;
            offset += TAU;
        }

        while offset > wrap {
            base += TAU;
            offset -= TAU;
        }

        self.rotation_base = base;
        self.rotation_offset = offset;
    }

    pub fn update(&mut self, _: f64, _: &Pose) {}

    fn pose_forward_single(&self, offset: f32) -> f32 {
        match offset.partial_cmp(&0.0) {
            Some(Ordering::Greater) => {
                // Hand opening to cross
                if let Some(t) = offset.inv_lerp_checked(self.cross_start - self.cross_grip, self.cross_start + self.cross_grip) {
                    return 1.0 - t.ease(-2.0);
                }

                // Hand crossing
                if let Some(t) = offset.inv_lerp_checked(self.cross_start + self.cross_grip, self.cross_end - self.cross_grip) {
                    return 0.5 * t.ping_pong(0.5).ease(-3.0);
                }

                // Hand closing after cross
                if let Some(t) = offset.inv_lerp_checked(self.cross_end - self.cross_grip, self.cross_end + self.cross_grip) {
                    return t.ease(-2.0);
                }
            }

            Some(Ordering::Less) => {
                let pos_offset = -offset;

                // Hand opening to turn
                if let Some(t) = pos_offset.inv_lerp_checked(self.turn_start - self.turn_grip, self.turn_start + self.turn_grip) {
                    return 1.0 - t.ease(-2.0);
                }

                // Hand turning
                if let Some(t) = pos_offset.inv_lerp_checked(self.turn_start + self.turn_grip, self.turn_end - self.turn_grip) {
                    return 0.5 * t.ping_pong(0.5).ease(-3.0);
                }

                // Hand closing after turn
                if let Some(t) = pos_offset.inv_lerp_checked(self.turn_end - self.turn_grip, self.turn_end + self.turn_grip) {
                    return t.ease(-2.0);
                }
            }

            _ => {}
        }

        1.0
    }

    fn pose_inverse_single(&self, offset: f32) -> (f32, f32, f32, f32) {
        match offset.partial_cmp(&0.0) {
            Some(Ordering::Greater) => {
                // Hand crossing
                if let Some(t) = offset.inv_lerp_checked(self.cross_start, self.cross_end) {
                    return (
                        t.lerp(t, t.ease(0.5)).lerp(self.cross_start, self.cross_end),
                        t.ease(-2.0).lerp(self.cross_start, self.cross_end - TAU),
                        0.0,
                        self.cross_retract * t.ping_pong(0.5).ease(-3.0),
                    );
                }
            }

            Some(Ordering::Less) => {
                let pos_offset = -offset;

                // Hand turning
                if let Some(t) = pos_offset.inv_lerp_checked(self.turn_start, self.turn_end) {
                    return (
                        offset,
                        -t.ease(-3.0).lerp(self.turn_start, self.turn_end - TAU),
                        self.turn_lift * t.ping_pong(0.5).ease(-2.0),
                        0.0,
                    );
                }
            }

            _ => {}
        }

        (offset, offset, 0.0, 0.0)
    }
}

impl TryFrom<TechniqueConfig> for Technique {
    type Error = AnyError;

    fn try_from(config: TechniqueConfig) -> AnyResult<Self> {
        ensure!(config.cross_grip >= 0.0, "cross-grip must be at least 0 degrees!");
        ensure!(config.cross_retract >= 0.0, "cross-retract must be at least than 0!");
        ensure!(config.turn_grip >= 0.0, "turn-grip must be at least 0 degrees!");
        ensure!(config.turn_lift >= 0.0, "turn-lift must be greater than 0!");

        ensure!(config.cross_start > 0.0, "cross-start must be greater than 0 degrees!");
        ensure!(config.cross_end >= config.cross_start, "cross-end must be greater than cross-start!");
        ensure!(config.cross_end < TAU, "cross-close must be less than 360 degrees!");

        ensure!(config.turn_start > 0.0, "turn-start must be greater than 0 degrees!");
        ensure!(config.turn_end >= config.turn_start, "turn-end must be greater than turn-start!");
        ensure!(config.turn_end < TAU, "turn-end must be less than 360 degrees!");

        Ok(Technique {
            cross_start: config.cross_start,
            cross_grip: config.cross_grip,
            cross_out: config.cross_out,
            cross_retract: config.cross_retract,
            cross_end: config.cross_end,

            turn_start: config.turn_start,
            turn_grip: config.turn_grip,
            turn_lift: config.turn_lift,
            turn_out: config.turn_out,
            turn_end: config.turn_end,

            rotation_base: 0.0,
            rotation_offset: 0.0,
        })
    }
}

impl Default for TechniqueConfig {
    fn default() -> Self {
        TechniqueConfig {
            cross_start: cross_start_default(),
            cross_grip: cross_grip_default(),
            cross_out: cross_out_default(),
            cross_retract: cross_retract_default(),
            cross_end: cross_end_default(),

            turn_start: turn_start_default(),
            turn_grip: turn_grip_default(),
            turn_lift: turn_lift_default(),
            turn_out: turn_out_default(),
            turn_end: turn_end_default(),
        }
    }
}

fn parse_degrees<'de, D: Deserializer<'de>>(de: D) -> Result<f32, D::Error> {
    f32::deserialize(de).map(f32::to_radians)
}
