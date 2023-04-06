use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use enumset::{EnumSet, EnumSetIter, EnumSetType};
use glam::{EulerRot, Vec3, Vec3A};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::ik::AngularConstraint;

#[derive(Debug, EnumSetType, IntoPrimitive, Ord, PartialOrd, TryFromPrimitive)]
#[repr(u8)]
pub enum Bone {
    Hips,
    LeftUpperLeg,
    RightUpperLeg,
    LeftLowerLeg,
    RightLowerLeg,
    LeftFoot,
    RightFoot,
    Spine,
    Chest,
    UpperChest,
    Neck,
    Head,
    LeftShoulder,
    RightShoulder,
    LeftUpperArm,
    RightUpperArm,
    LeftLowerArm,
    RightLowerArm,
    LeftHand,
    RightHand,
    LeftToes,
    RightToes,
    LeftEye,
    RightEye,
    Jaw,
    LeftThumbProximal,
    LeftThumbIntermediate,
    LeftThumbDistal,
    LeftIndexProximal,
    LeftIndexIntermediate,
    LeftIndexDistal,
    LeftMiddleProximal,
    LeftMiddleIntermediate,
    LeftMiddleDistal,
    LeftRingProximal,
    LeftRingIntermediate,
    LeftRingDistal,
    LeftLittleProximal,
    LeftLittleIntermediate,
    LeftLittleDistal,
    RightThumbProximal,
    RightThumbIntermediate,
    RightThumbDistal,
    RightIndexProximal,
    RightIndexIntermediate,
    RightIndexDistal,
    RightMiddleProximal,
    RightMiddleIntermediate,
    RightMiddleDistal,
    RightRingProximal,
    RightRingIntermediate,
    RightRingDistal,
    RightLittleProximal,
    RightLittleIntermediate,
    RightLittleDistal,
}

static BONES: phf::Map<&str, Bone> = phf::phf_map! {
    "Hips" => Bone::Hips,
    "LeftUpperLeg" => Bone::LeftUpperLeg,
    "RightUpperLeg" => Bone::RightUpperLeg,
    "LeftLowerLeg" => Bone::LeftLowerLeg,
    "RightLowerLeg" => Bone::RightLowerLeg,
    "LeftFoot" => Bone::LeftFoot,
    "RightFoot" => Bone::RightFoot,
    "Spine" => Bone::Spine,
    "Chest" => Bone::Chest,
    "UpperChest" => Bone::UpperChest,
    "Neck" => Bone::Neck,
    "Head" => Bone::Head,
    "LeftShoulder" => Bone::LeftShoulder,
    "RightShoulder" => Bone::RightShoulder,
    "LeftUpperArm" => Bone::LeftUpperArm,
    "RightUpperArm" => Bone::RightUpperArm,
    "LeftLowerArm" => Bone::LeftLowerArm,
    "RightLowerArm" => Bone::RightLowerArm,
    "LeftHand" => Bone::LeftHand,
    "RightHand" => Bone::RightHand,
    "LeftToes" => Bone::LeftToes,
    "RightToes" => Bone::RightToes,
    "LeftEye" => Bone::LeftEye,
    "RightEye" => Bone::RightEye,
    "Jaw" => Bone::Jaw,
    "LeftThumbProximal" => Bone::LeftThumbProximal,
    "LeftThumbIntermediate" => Bone::LeftThumbIntermediate,
    "LeftThumbDistal" => Bone::LeftThumbDistal,
    "LeftIndexProximal" => Bone::LeftIndexProximal,
    "LeftIndexIntermediate" => Bone::LeftIndexIntermediate,
    "LeftIndexDistal" => Bone::LeftIndexDistal,
    "LeftMiddleProximal" => Bone::LeftMiddleProximal,
    "LeftMiddleIntermediate" => Bone::LeftMiddleIntermediate,
    "LeftMiddleDistal" => Bone::LeftMiddleDistal,
    "LeftRingProximal" => Bone::LeftRingProximal,
    "LeftRingIntermediate" => Bone::LeftRingIntermediate,
    "LeftRingDistal" => Bone::LeftRingDistal,
    "LeftLittleProximal" => Bone::LeftLittleProximal,
    "LeftLittleIntermediate" => Bone::LeftLittleIntermediate,
    "LeftLittleDistal" => Bone::LeftLittleDistal,
    "RightThumbProximal" => Bone::RightThumbProximal,
    "RightThumbIntermediate" => Bone::RightThumbIntermediate,
    "RightThumbDistal" => Bone::RightThumbDistal,
    "RightIndexProximal" => Bone::RightIndexProximal,
    "RightIndexIntermediate" => Bone::RightIndexIntermediate,
    "RightIndexDistal" => Bone::RightIndexDistal,
    "RightMiddleProximal" => Bone::RightMiddleProximal,
    "RightMiddleIntermediate" => Bone::RightMiddleIntermediate,
    "RightMiddleDistal" => Bone::RightMiddleDistal,
    "RightRingProximal" => Bone::RightRingProximal,
    "RightRingIntermediate" => Bone::RightRingIntermediate,
    "RightRingDistal" => Bone::RightRingDistal,
    "RightLittleProximal" => Bone::RightLittleProximal,
    "RightLittleIntermediate" => Bone::RightLittleIntermediate,
    "RightLittleDistal" => Bone::RightLittleDistal,
};

const CHILDREN: [u64; Bone::NUM] = [
    0x0000000000000086,
    0x0000000000000008,
    0x0000000000000010,
    0x0000000000000020,
    0x0000000000000040,
    0x0000000000100000,
    0x0000000000200000,
    0x0000000000000100,
    0x0000000000000200,
    0x0000000000003400,
    0x0000000000000800,
    0x0000000001c00000,
    0x0000000000004000,
    0x0000000000008000,
    0x0000000000010000,
    0x0000000000020000,
    0x0000000000040000,
    0x0000000000080000,
    0x0000002492000000,
    0x0012490000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000004000000,
    0x0000000008000000,
    0x0000000000000000,
    0x0000000020000000,
    0x0000000040000000,
    0x0000000000000000,
    0x0000000100000000,
    0x0000000200000000,
    0x0000000000000000,
    0x0000000800000000,
    0x0000001000000000,
    0x0000000000000000,
    0x0000004000000000,
    0x0000008000000000,
    0x0000000000000000,
    0x0000020000000000,
    0x0000040000000000,
    0x0000000000000000,
    0x0000100000000000,
    0x0000200000000000,
    0x0000000000000000,
    0x0000800000000000,
    0x0001000000000000,
    0x0000000000000000,
    0x0004000000000000,
    0x0008000000000000,
    0x0000000000000000,
    0x0020000000000000,
    0x0040000000000000,
    0x0000000000000000,
];

const DESCENDANTS: [u64; Bone::NUM] = [
    0x007ffffffffffffe,
    0x0000000000100028,
    0x0000000000200050,
    0x0000000000100020,
    0x0000000000200040,
    0x0000000000100000,
    0x0000000000200000,
    0x007fffffffcfff00,
    0x007fffffffcffe00,
    0x007fffffffcffc00,
    0x0000000001c00800,
    0x0000000001c00000,
    0x000000fffe054000,
    0x007fff00000a8000,
    0x000000fffe050000,
    0x007fff00000a0000,
    0x000000fffe040000,
    0x007fff0000080000,
    0x000000fffe000000,
    0x007fff0000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x0000000000000000,
    0x000000000c000000,
    0x0000000008000000,
    0x0000000000000000,
    0x0000000060000000,
    0x0000000040000000,
    0x0000000000000000,
    0x0000000300000000,
    0x0000000200000000,
    0x0000000000000000,
    0x0000001800000000,
    0x0000001000000000,
    0x0000000000000000,
    0x000000c000000000,
    0x0000008000000000,
    0x0000000000000000,
    0x0000060000000000,
    0x0000040000000000,
    0x0000000000000000,
    0x0000300000000000,
    0x0000200000000000,
    0x0000000000000000,
    0x0001800000000000,
    0x0001000000000000,
    0x0000000000000000,
    0x000c000000000000,
    0x0008000000000000,
    0x0000000000000000,
    0x0060000000000000,
    0x0040000000000000,
    0x0000000000000000,
];

impl Bone {
    pub const NUM: usize = Self::RightLittleDistal as u8 as usize + 1;

    pub fn affected(&self) -> EnumSet<Bone> {
        self.descendants() | *self
    }

    pub fn children(&self) -> EnumSet<Bone> {
        EnumSet::from_u64(CHILDREN[*self as u8 as usize])
    }

    pub fn descendants(&self) -> EnumSet<Bone> {
        EnumSet::from_u64(DESCENDANTS[*self as u8 as usize])
    }

    pub fn iter() -> EnumSetIter<Bone> {
        EnumSet::all().into_iter()
    }

    pub fn mask(&self) -> EnumSet<Bone> {
        EnumSet::from(*self)
    }

    pub const fn mirror(&self) -> Bone {
        match *self {
            Self::Hips => Self::Hips,
            Self::LeftUpperLeg => Self::RightUpperLeg,
            Self::RightUpperLeg => Self::LeftUpperLeg,
            Self::LeftLowerLeg => Self::RightLowerLeg,
            Self::RightLowerLeg => Self::LeftLowerLeg,
            Self::LeftFoot => Self::RightFoot,
            Self::RightFoot => Self::LeftFoot,
            Self::Spine => Self::Spine,
            Self::Chest => Self::Chest,
            Self::UpperChest => Self::UpperChest,
            Self::Neck => Self::Neck,
            Self::Head => Self::Head,
            Self::LeftShoulder => Self::RightShoulder,
            Self::RightShoulder => Self::LeftShoulder,
            Self::LeftUpperArm => Self::RightUpperArm,
            Self::RightUpperArm => Self::LeftUpperArm,
            Self::LeftLowerArm => Self::RightLowerArm,
            Self::RightLowerArm => Self::LeftLowerArm,
            Self::LeftHand => Self::RightHand,
            Self::RightHand => Self::LeftHand,
            Self::LeftToes => Self::RightToes,
            Self::RightToes => Self::LeftToes,
            Self::LeftEye => Self::RightEye,
            Self::RightEye => Self::LeftEye,
            Self::Jaw => Self::Jaw,
            Self::LeftThumbProximal => Self::RightThumbProximal,
            Self::LeftThumbIntermediate => Self::RightThumbIntermediate,
            Self::LeftThumbDistal => Self::RightThumbDistal,
            Self::LeftIndexProximal => Self::RightIndexProximal,
            Self::LeftIndexIntermediate => Self::RightIndexIntermediate,
            Self::LeftIndexDistal => Self::RightIndexDistal,
            Self::LeftMiddleProximal => Self::RightMiddleProximal,
            Self::LeftMiddleIntermediate => Self::RightMiddleIntermediate,
            Self::LeftMiddleDistal => Self::RightMiddleDistal,
            Self::LeftRingProximal => Self::RightRingProximal,
            Self::LeftRingIntermediate => Self::RightRingIntermediate,
            Self::LeftRingDistal => Self::RightRingDistal,
            Self::LeftLittleProximal => Self::RightLittleProximal,
            Self::LeftLittleIntermediate => Self::RightLittleIntermediate,
            Self::LeftLittleDistal => Self::RightLittleDistal,
            Self::RightThumbProximal => Self::LeftThumbProximal,
            Self::RightThumbIntermediate => Self::LeftThumbIntermediate,
            Self::RightThumbDistal => Self::LeftThumbDistal,
            Self::RightIndexProximal => Self::LeftIndexProximal,
            Self::RightIndexIntermediate => Self::LeftIndexIntermediate,
            Self::RightIndexDistal => Self::LeftIndexDistal,
            Self::RightMiddleProximal => Self::LeftMiddleProximal,
            Self::RightMiddleIntermediate => Self::LeftMiddleIntermediate,
            Self::RightMiddleDistal => Self::LeftMiddleDistal,
            Self::RightRingProximal => Self::LeftRingProximal,
            Self::RightRingIntermediate => Self::LeftRingIntermediate,
            Self::RightRingDistal => Self::LeftRingDistal,
            Self::RightLittleProximal => Self::LeftLittleProximal,
            Self::RightLittleIntermediate => Self::LeftLittleIntermediate,
            Self::RightLittleDistal => Self::LeftLittleDistal,
        }
    }

    pub const fn name(&self) -> &'static str {
        match *self {
            Self::Hips => "Hips",
            Self::LeftUpperLeg => "LeftUpperLeg",
            Self::RightUpperLeg => "RightUpperLeg",
            Self::LeftLowerLeg => "LeftLowerLeg",
            Self::RightLowerLeg => "RightLowerLeg",
            Self::LeftFoot => "LeftFoot",
            Self::RightFoot => "RightFoot",
            Self::Spine => "Spine",
            Self::Chest => "Chest",
            Self::UpperChest => "UpperChest",
            Self::Neck => "Neck",
            Self::Head => "Head",
            Self::LeftShoulder => "LeftShoulder",
            Self::RightShoulder => "RightShoulder",
            Self::LeftUpperArm => "LeftUpperArm",
            Self::RightUpperArm => "RightUpperArm",
            Self::LeftLowerArm => "LeftLowerArm",
            Self::RightLowerArm => "RightLowerArm",
            Self::LeftHand => "LeftHand",
            Self::RightHand => "RightHand",
            Self::LeftToes => "LeftToes",
            Self::RightToes => "RightToes",
            Self::LeftEye => "LeftdescendantsEye",
            Self::RightEye => "RightEye",
            Self::Jaw => "Jaw",
            Self::LeftThumbProximal => "LeftThumbProximal",
            Self::LeftThumbIntermediate => "LeftThumbIntermediate",
            Self::LeftThumbDistal => "LeftThumbDistal",
            Self::LeftIndexProximal => "LeftIndexProximal",
            Self::LeftIndexIntermediate => "LeftIndexIntermediate",
            Self::LeftIndexDistal => "LeftIndexDistal",
            Self::LeftMiddleProximal => "LeftMiddleProximal",
            Self::LeftMiddleIntermediate => "LeftMiddleIntermediate",
            Self::LeftMiddleDistal => "LeftMiddleDistal",
            Self::LeftRingProximal => "LeftRingProximal",
            Self::LeftRingIntermediate => "LeftRingIntermediate",
            Self::LeftRingDistal => "LeftRingDistal",
            Self::LeftLittleProximal => "LeftLittleProximal",
            Self::LeftLittleIntermediate => "LeftLittleIntermediate",
            Self::LeftLittleDistal => "LeftLittleDistal",
            Self::RightThumbProximal => "RightThumbProximal",
            Self::RightThumbIntermediate => "RightThumbIntermediate",
            Self::RightThumbDistal => "RightThumbDistal",
            Self::RightIndexProximal => "RightIndexProximal",
            Self::RightIndexIntermediate => "RightIndexIntermediate",
            Self::RightIndexDistal => "RightIndexDistal",
            Self::RightMiddleProximal => "RightMiddleProximal",
            Self::RightMiddleIntermediate => "RightMiddleIntermediate",
            Self::RightMiddleDistal => "RightMiddleDistal",
            Self::RightRingProximal => "RightRingProximal",
            Self::RightRingIntermediate => "RightRingIntermediate",
            Self::RightRingDistal => "RightRingDistal",
            Self::RightLittleProximal => "RightLittleProximal",
            Self::RightLittleIntermediate => "RightLittleIntermediate",
            Self::RightLittleDistal => "RightLittleDistal",
        }
    }

    pub const fn parent(&self) -> Option<Self> {
        match *self {
            Self::Hips => None,
            Self::LeftUpperLeg => Some(Self::Hips),
            Self::RightUpperLeg => Some(Self::Hips),
            Self::LeftLowerLeg => Some(Self::LeftUpperLeg),
            Self::RightLowerLeg => Some(Self::RightUpperLeg),
            Self::LeftFoot => Some(Self::LeftLowerLeg),
            Self::RightFoot => Some(Self::RightLowerLeg),
            Self::Spine => Some(Self::Hips),
            Self::Chest => Some(Self::Spine),
            Self::UpperChest => Some(Self::Chest),
            Self::Neck => Some(Self::UpperChest),
            Self::Head => Some(Self::Neck),
            Self::LeftShoulder => Some(Self::UpperChest),
            Self::RightShoulder => Some(Self::UpperChest),
            Self::LeftUpperArm => Some(Self::LeftShoulder),
            Self::RightUpperArm => Some(Self::RightShoulder),
            Self::LeftLowerArm => Some(Self::LeftUpperArm),
            Self::RightLowerArm => Some(Self::RightUpperArm),
            Self::LeftHand => Some(Self::LeftLowerArm),
            Self::RightHand => Some(Self::RightLowerArm),
            Self::LeftToes => Some(Self::LeftFoot),
            Self::RightToes => Some(Self::RightFoot),
            Self::LeftEye => Some(Self::Head),
            Self::RightEye => Some(Self::Head),
            Self::Jaw => Some(Self::Head),
            Self::LeftThumbProximal => Some(Self::LeftHand),
            Self::LeftThumbIntermediate => Some(Self::LeftThumbProximal),
            Self::LeftThumbDistal => Some(Self::LeftThumbIntermediate),
            Self::LeftIndexProximal => Some(Self::LeftHand),
            Self::LeftIndexIntermediate => Some(Self::LeftIndexProximal),
            Self::LeftIndexDistal => Some(Self::LeftIndexIntermediate),
            Self::LeftMiddleProximal => Some(Self::LeftHand),
            Self::LeftMiddleIntermediate => Some(Self::LeftMiddleProximal),
            Self::LeftMiddleDistal => Some(Self::LeftMiddleIntermediate),
            Self::LeftRingProximal => Some(Self::LeftHand),
            Self::LeftRingIntermediate => Some(Self::LeftRingProximal),
            Self::LeftRingDistal => Some(Self::LeftRingIntermediate),
            Self::LeftLittleProximal => Some(Self::LeftHand),
            Self::LeftLittleIntermediate => Some(Self::LeftLittleProximal),
            Self::LeftLittleDistal => Some(Self::LeftLittleIntermediate),
            Self::RightThumbProximal => Some(Self::RightHand),
            Self::RightThumbIntermediate => Some(Self::RightThumbProximal),
            Self::RightThumbDistal => Some(Self::RightThumbIntermediate),
            Self::RightIndexProximal => Some(Self::RightHand),
            Self::RightIndexIntermediate => Some(Self::RightIndexProximal),
            Self::RightIndexDistal => Some(Self::RightIndexIntermediate),
            Self::RightMiddleProximal => Some(Self::RightHand),
            Self::RightMiddleIntermediate => Some(Self::RightMiddleProximal),
            Self::RightMiddleDistal => Some(Self::RightMiddleIntermediate),
            Self::RightRingProximal => Some(Self::RightHand),
            Self::RightRingIntermediate => Some(Self::RightRingProximal),
            Self::RightRingDistal => Some(Self::RightRingIntermediate),
            Self::RightLittleProximal => Some(Self::RightHand),
            Self::RightLittleIntermediate => Some(Self::RightLittleProximal),
            Self::RightLittleDistal => Some(Self::RightLittleIntermediate),
        }
    }
}

impl FromStr for Bone {
    type Err = FromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(&bone) = BONES.get(s) {
            Ok(bone)
        } else {
            Err(FromStrErr)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FromStrErr;

impl Display for FromStrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bone name not recognized.")
    }
}

impl Error for FromStrErr {}

#[derive(Debug, EnumSetType, IntoPrimitive, Ord, PartialOrd, TryFromPrimitive)]
#[repr(u8)]
pub enum Limb {
    LeftHand,
    RightHand,
}

impl Limb {
    pub const NUM: usize = Self::RightHand as u8 as usize + 1;

    pub const fn angular_constraints(&self) -> &'static [AngularConstraint] {
        match *self {
            Limb::LeftHand => &[
                AngularConstraint::None,
                AngularConstraint::Euler(
                    EulerRot::YZX,
                    (-1.59, 2.62), // Yaw; -90 to 150 deg
                    (-1.40, 1.59), // Pitch; -80 to 90 deg
                    (-1.04, 1.04), // Roll; -60 to 60 deg
                ),
                AngularConstraint::Hinge(Vec3::Y, (0.0, 2.88)), // 0 to 165 deg
                AngularConstraint::None,
            ],

            Limb::RightHand => &[
                AngularConstraint::None,
                AngularConstraint::Euler(
                    EulerRot::YZX,
                    (-2.62, 1.59), // Yaw; -150 to 90 deg
                    (-1.40, 1.59), // Pitch; -80 to 90 deg
                    (-1.04, 1.04), // Roll; -60 to 60 deg
                ),
                AngularConstraint::None,
                AngularConstraint::Hinge(Vec3::NEG_Y, (0.0, 2.88)), // 0 to 165 deg
                AngularConstraint::None,
            ],
        }
    }

    pub const fn bones(&self) -> &'static [Bone] {
        match *self {
            Limb::LeftHand => &[
                Bone::LeftShoulder,
                Bone::LeftUpperArm,
                Bone::LeftLowerArm,
                Bone::LeftHand,
            ],

            Limb::RightHand => &[
                Bone::RightShoulder,
                Bone::RightUpperArm,
                Bone::RightLowerArm,
                Bone::RightHand,
            ],
        }
    }

    pub const fn elbow_axis(&self) -> Vec3A {
        match *self {
            Limb::LeftHand => Vec3A::Y,
            Limb::RightHand => Vec3A::NEG_Y,
        }
    }

    pub const fn end_bone(&self) -> Bone {
        match *self {
            Limb::LeftHand => Bone::LeftHand,
            Limb::RightHand => Bone::RightHand,
        }
    }

    pub fn iter() -> EnumSetIter<Limb> {
        EnumSet::all().into_iter()
    }

    pub fn mask(&self) -> EnumSet<Limb> {
        EnumSet::from(*self)
    }
}
