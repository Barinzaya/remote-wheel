use std::f32::consts::TAU;

use glam::{EulerRot, Quat, Vec3, Vec3A};
use hashbrown::HashMap;
use string_cache::DefaultAtom;

use super::bone::{Bone, BoneMask};
use super::device::Device;
use super::ik::{solve_tri, AngularConstraint, Chain, Link, TriSettings};

pub(super) struct AvatarState {
    left_hand_pose: Option<(Vec3A, Quat)>,
    right_hand_pose: Option<(Vec3A, Quat)>,

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

    pub fn apply_to(&self, tracking: &mut Pose) {
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
                pos,
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
                pos,
                rot,
            );
        }

        for &(bone, rot) in &self.poses {
            tracking.set_local_rot(bone, rot);
        }
    }

    pub fn update(&mut self, _: f64, devices: &HashMap<DefaultAtom, Device>) {
        self.left_hand_pose = devices
            .values()
            .filter_map(|d| d.pose(Bone::LeftHand))
            .next();
        self.right_hand_pose = devices
            .values()
            .filter_map(|d| d.pose(Bone::RightHand))
            .next();
    }
}

#[derive(Clone, Debug)]
pub struct Pose {
    root_pos: Vec3A,
    root_rot: Quat,

    bones: Vec<PoseBone>,
    globalized: BoneMask,
}

#[derive(Clone, Debug)]
pub struct PoseBone {
    local_pos: Vec3A,
    local_rot: Quat,

    global_pos: Vec3,
    global_rot: Quat,
}

impl Pose {
    pub fn new() -> Self {
        Pose {
            root_pos: Vec3A::ZERO,
            root_rot: Quat::IDENTITY,

            bones: vec![PoseBone::new(); Bone::NUM],
            globalized: BoneMask::all(),
        }
    }

    pub fn global_transform(&mut self, bone: Bone) -> (Vec3A, Quat) {
        if self.globalized.insert(bone) {
            let (parent_pos, parent_rot) = bone
                .parent()
                .map(|b| self.global_transform(b))
                .unwrap_or((self.root_pos, self.root_rot));

            let pose_bone = &mut self.bones[bone as u8 as usize];

            let new_pos = parent_pos + parent_rot * pose_bone.local_pos;
            let new_rot = parent_rot * pose_bone.local_rot;

            pose_bone.global_pos = new_pos.into();
            pose_bone.global_rot = new_rot;

            (new_pos, new_rot)
        } else {
            let pose_bone = &self.bones[bone as u8 as usize];
            (pose_bone.global_pos.into(), pose_bone.global_rot)
        }
    }

    pub fn local_transform(&self, bone: Bone) -> (Vec3A, Quat) {
        let pose_bone = &self.bones[bone as u8 as usize];
        (pose_bone.local_pos, pose_bone.local_rot)
    }

    pub fn root_transform(&self) -> (Vec3A, Quat) {
        (self.root_pos, self.root_rot)
    }

    pub fn set_global_rot(&mut self, bone: Bone, new_rot: Quat) {
        let (parent_pos, parent_rot) = bone
            .parent()
            .map(|b| self.global_transform(b))
            .unwrap_or((self.root_pos, self.root_rot));

        let pose_bone = &mut self.bones[bone as u8 as usize];
        pose_bone.global_pos = Vec3::from(parent_pos + parent_rot * pose_bone.local_pos);
        pose_bone.global_rot = new_rot;
        pose_bone.local_rot = parent_rot.inverse() * new_rot;

        self.globalized = self
            .globalized
            .union(&bone.mask())
            .difference(&bone.descendants());
    }

    pub fn set_local_rot(&mut self, bone: Bone, new_rot: Quat) {
        self.bones[bone as u8 as usize].local_rot = new_rot;
        self.globalized = self.globalized.difference(&bone.affected());
    }

    pub fn set_local_transform(&mut self, bone: Bone, new_pos: Vec3A, new_rot: Quat) {
        let pose_bone = &mut self.bones[bone as u8 as usize];
        pose_bone.local_pos = new_pos;
        pose_bone.local_rot = new_rot;

        self.globalized = self.globalized.difference(&bone.affected());
    }

    pub fn set_root_transform(&mut self, new_pos: Vec3A, new_rot: Quat) {
        self.root_pos = new_pos;
        self.root_rot = new_rot;

        self.globalized.clear();
    }
}

impl PoseBone {
    pub const fn new() -> Self {
        PoseBone {
            local_pos: Vec3A::ZERO,
            local_rot: Quat::IDENTITY,

            global_pos: Vec3::ZERO,
            global_rot: Quat::IDENTITY,
        }
    }
}

struct TrackingChain<'d> {
    bones: &'d [Bone],
    constraints: &'d [AngularConstraint],
    data: &'d mut Pose,
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
    data: &'d mut Pose,
}

impl<'d> Link for TrackingLink<'d> {
    fn angular_constraint(&self) -> AngularConstraint {
        *self.constraint
    }

    fn pos(&mut self) -> Vec3A {
        self.state().0
    }

    fn rot(&mut self) -> Quat {
        self.state().1
    }

    fn set_rot(&mut self, new_rot: Quat) {
        self.data.set_global_rot(self.bone, new_rot);
    }

    fn state(&mut self) -> (Vec3A, Quat) {
        self.data.global_transform(self.bone)
    }
}
