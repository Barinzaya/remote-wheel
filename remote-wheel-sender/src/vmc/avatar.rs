use std::cell::Cell;

use enumset::EnumSet;
use glam::{Quat, Vec3A, Vec4};
use hashbrown::HashMap;
use string_cache::DefaultAtom;

use crate::vmc::device::ForwardPose;

use super::bone::{Bone, Limb};
use super::device::Device;
use super::ik::{solve_tri, AngularConstraint, Chain, Link, TriSettings};

pub(super) struct AvatarState;

impl AvatarState {
    pub fn new() -> AvatarState {
        AvatarState
    }

    pub fn apply_to(&self, devices: &HashMap<DefaultAtom, Device>, pose: &mut Pose) {
        let mut limbs = [(Vec4::ZERO, Quat::IDENTITY); Limb::NUM];
        let mut touched_limbs = EnumSet::empty();

        for limb in EnumSet::<Limb>::all() {
            let (pos, rot) = pose.global_transform(limb.end_bone());
            limbs[limb as u8 as usize] = (Vec4::from((pos, 1.0)), rot);
        }

        for device in devices.values() {
            device.pose_inverse(pose, |limb, weight, new_pos, new_rot| {
                let data = &mut limbs[limb as u8 as usize];

                let effective_weight = weight * data.0.w;
                if effective_weight > 0.0 {
                    let remaining_weight = data.0.w - effective_weight;

                    let new_pos = Vec3A::from(data.0).lerp(new_pos, effective_weight);
                    let new_rot = data.1.slerp(new_rot, effective_weight);
                    *data = (Vec4::from((new_pos, remaining_weight)), new_rot);

                    touched_limbs.insert(limb);
                }
            });
        }

        for limb in touched_limbs {
            let data = &limbs[limb as u8 as usize];
            let _ = solve_tri(
                &TriSettings {
                    elbow_axis: limb.elbow_axis(),
                    max_iterations: 10,
                    rot_tolerance: 0.001,
                },
                &mut TrackingChain {
                    bones: limb.bones(),
                    constraints: limb.angular_constraints(),
                    data: pose,
                },
                data.0.into(),
                data.1,
            );
        }

        let mut touched_bones = EnumSet::empty();

        for device in devices.values() {
            device.pose_forward(|bone, weight, target| {
                debug_assert!((0.0..=1.0).contains(&weight));
                if weight == 0.0 {
                    return;
                }

                let effective_weight = {
                    let data = &mut pose.bones[bone as u8 as usize];

                    let initial_weight = if touched_bones.insert(bone) {
                        1.0
                    } else {
                        data.local_pos.w
                    };

                    if initial_weight == 0.0 {
                        return;
                    }

                    let effective_weight = weight * initial_weight;
                    data.local_pos.w = initial_weight - effective_weight;
                    effective_weight
                };

                if effective_weight < 1.0 {
                    match target {
                        ForwardPose::Global(rot) => {
                            let old_rot = pose.global_transform(bone).1;
                            let new_rot = old_rot.slerp(rot, effective_weight);
                            pose.set_global_rot(bone, new_rot);
                        }

                        ForwardPose::Local(rot) => {
                            let old_rot = pose.local_transform(bone).1;
                            let new_rot = old_rot.slerp(rot, effective_weight);
                            pose.set_local_rot(bone, new_rot);
                        }
                    }
                } else {
                    match target {
                        ForwardPose::Global(rot) => pose.set_global_rot(bone, rot),
                        ForwardPose::Local(rot) => pose.set_local_rot(bone, rot),
                    }
                }
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct Pose {
    root_pos: Vec3A,
    root_rot: Quat,

    bones: Vec<PoseBone>,
    globalized: Cell<EnumSet<Bone>>,
}

#[derive(Clone, Debug)]
pub struct PoseBone {
    local_pos: Vec4,
    local_rot: Quat,

    global_pos: Cell<Vec3A>,
    global_rot: Cell<Quat>,
}

impl Pose {
    pub fn new() -> Self {
        Pose {
            root_pos: Vec3A::ZERO,
            root_rot: Quat::IDENTITY,

            bones: vec![PoseBone::new(); Bone::NUM],
            globalized: Cell::new(EnumSet::all()),
        }
    }

    pub fn global_transform(&self, bone: Bone) -> (Vec3A, Quat) {
        let mut global = self.globalized.get();
        if global.insert(bone) {
            self.globalized.set(global);

            let (parent_pos, parent_rot) = bone
                .parent()
                .map(|b| self.global_transform(b))
                .unwrap_or((self.root_pos, self.root_rot));

            let pose_bone = &self.bones[bone as u8 as usize];

            let new_pos = parent_pos + parent_rot * Vec3A::from(pose_bone.local_pos);
            let new_rot = parent_rot * pose_bone.local_rot;

            pose_bone.global_pos.set(new_pos);
            pose_bone.global_rot.set(new_rot);

            (new_pos, new_rot)
        } else {
            let pose_bone = &self.bones[bone as u8 as usize];
            (pose_bone.global_pos.get(), pose_bone.global_rot.get())
        }
    }

    pub fn local_transform(&self, bone: Bone) -> (Vec3A, Quat) {
        let pose_bone = &self.bones[bone as u8 as usize];
        (pose_bone.local_pos.into(), pose_bone.local_rot)
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
        pose_bone
            .global_pos
            .set(parent_pos + parent_rot * Vec3A::from(pose_bone.local_pos));
        pose_bone.global_rot.set(new_rot);
        pose_bone.local_rot = parent_rot.inverse() * new_rot;

        let global = self.globalized.get_mut();
        global.insert(bone);
        global.remove_all(bone.descendants());
    }

    pub fn set_local_rot(&mut self, bone: Bone, new_rot: Quat) {
        self.bones[bone as u8 as usize].local_rot = new_rot;
        self.globalized.get_mut().remove_all(bone.affected());
    }

    pub fn set_local_transform(&mut self, bone: Bone, new_pos: Vec3A, new_rot: Quat) {
        let pose_bone = &mut self.bones[bone as u8 as usize];
        pose_bone.local_pos = (new_pos, pose_bone.local_pos.w).into();
        pose_bone.local_rot = new_rot;

        self.globalized.get_mut().remove_all(bone.affected());
    }

    pub fn set_root_transform(&mut self, new_pos: Vec3A, new_rot: Quat) {
        self.root_pos = new_pos;
        self.root_rot = new_rot;

        self.globalized.get_mut().clear();
    }
}

impl PoseBone {
    pub const fn new() -> Self {
        PoseBone {
            local_pos: Vec4::ZERO,
            local_rot: Quat::IDENTITY,

            global_pos: Cell::new(Vec3A::ZERO),
            global_rot: Cell::new(Quat::IDENTITY),
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
