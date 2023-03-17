use std::f32::consts::TAU;

use glam::{EulerRot, Quat, Vec3, Vec3A};
use hashbrown::HashMap;
use string_cache::DefaultAtom;

use super::bone::Bone;
use super::device::Device;
use super::ik::{solve_tri, AngularConstraint, Chain, Link, TriSettings};
use super::TrackingData;

pub(super) struct AvatarState {
    left_hand_pose: Option<(Vec3, Quat)>,
    right_hand_pose: Option<(Vec3, Quat)>,

    poses: Vec<(Bone, Quat)>,
}

impl AvatarState {
    pub fn new() -> AvatarState {
        AvatarState {
            left_hand_pose: None,
            right_hand_pose: None,

            poses: vec![
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
                    Bone::LeftThumbProximal,
                    Quat::from_euler(EulerRot::YZX, -0.02 * TAU, 0.0, 0.08 * TAU),
                ),
                (
                    Bone::LeftThumbIntermediate,
                    Quat::from_rotation_y(-0.08 * TAU),
                ),
                (Bone::LeftThumbDistal, Quat::from_rotation_y(-0.03 * TAU)),
                (
                    Bone::RightThumbProximal,
                    Quat::from_euler(EulerRot::YZX, 0.02 * TAU, 0.0, 0.08 * TAU),
                ),
                (
                    Bone::RightThumbIntermediate,
                    Quat::from_rotation_y(0.08 * TAU),
                ),
                (Bone::RightThumbDistal, Quat::from_rotation_y(0.03 * TAU)),
            ],
        }
    }

    pub fn apply_to(&self, tracking: &mut TrackingData) {
        if let Some((pos, rot)) = self.left_hand_pose {
            let _ = solve_tri(
                &TriSettings {
                    elbow_axis: Vec3A::Y,
                    max_iterations: 10,
                    rot_tolerance: 0.001,
                },
                &mut TrackingChain {
                    bones: &[
                        Bone::LeftShoulder,
                        Bone::LeftUpperArm,
                        Bone::LeftLowerArm,
                        Bone::LeftHand,
                    ],
                    constraints: &[
                        AngularConstraint::None,
                        AngularConstraint::Euler(
                            EulerRot::YZX,
                            (-60.0, 135.0), // Yaw
                            (-75.0, 90.0),  // Pitch
                            (-45.0, 45.0),  // Roll
                        )
                        .to_radians(),
                        AngularConstraint::Hinge(Vec3::Y, (0.0, 150.0)).to_radians(),
                        AngularConstraint::None,
                    ],
                    data: tracking,
                },
                pos.into(),
                rot,
            );
        }

        if let Some((pos, rot)) = self.right_hand_pose {
            let _ = solve_tri(
                &TriSettings {
                    elbow_axis: Vec3A::NEG_Y,
                    max_iterations: 10,
                    rot_tolerance: 0.001,
                },
                &mut TrackingChain {
                    bones: &[
                        Bone::RightShoulder,
                        Bone::RightUpperArm,
                        Bone::RightLowerArm,
                        Bone::RightHand,
                    ],
                    constraints: &[
                        AngularConstraint::None,
                        AngularConstraint::Euler(
                            EulerRot::YZX,
                            (-135.0, 60.0), // Yaw
                            (-75.0, 90.0),  // Pitch
                            (-45.0, 45.0),  // Roll
                        )
                        .to_radians(),
                        AngularConstraint::Hinge(Vec3::NEG_Y, (0.0, 150.0)).to_radians(),
                        AngularConstraint::None,
                    ],
                    data: tracking,
                },
                pos.into(),
                rot,
            );
        }

        for &(bone, rot) in &self.poses {
            tracking.set_local_bone_rot(bone, rot);
        }
    }

    pub fn update(&mut self, _: f64, devices: &HashMap<DefaultAtom, Device>) {
        let mut left_hand_pose = None;
        let mut right_hand_pose = None;

        for device in devices.values() {
            left_hand_pose = left_hand_pose.or_else(|| device.pose(Bone::LeftHand));
            right_hand_pose = right_hand_pose.or_else(|| device.pose(Bone::RightHand));
        }

        self.left_hand_pose = left_hand_pose;
        self.right_hand_pose = right_hand_pose;
    }
}

struct TrackingChain<'d> {
    bones: &'d [Bone],
    constraints: &'d [AngularConstraint],
    data: &'d mut TrackingData,
}

impl<'d> Chain for TrackingChain<'d> {
    type Link<'l> = TrackingLink<'l> where Self: 'l;

    fn num_links(&self) -> usize {
        self.bones.len()
    }

    fn link(&mut self, index: usize) -> Self::Link<'_> {
        let bone = self.bones[index];
        TrackingLink {
            bone,
            constraint: &self.constraints[index],
            data: self.data,
        }
    }
}

struct TrackingLink<'d> {
    bone: Bone,
    constraint: &'d AngularConstraint,
    data: &'d mut TrackingData,
}

impl<'d> Link for TrackingLink<'d> {
    fn angular_constraint(&self) -> AngularConstraint {
        *self.constraint
    }

    fn pos(&mut self) -> glam::Vec3A {
        self.data.global_bone(self.bone).pos
    }

    fn rot(&mut self) -> glam::Quat {
        self.data.global_bone(self.bone).rot
    }

    fn set_rot(&mut self, new_rot: glam::Quat) {
        self.data.set_global_bone_rot(self.bone, new_rot);
    }
}
