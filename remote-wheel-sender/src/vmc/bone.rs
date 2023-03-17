use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(
    Clone, Copy, Debug, Eq, Hash, IntoPrimitive, Ord, PartialEq, PartialOrd, TryFromPrimitive,
)]
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

static BONES: phf::OrderedMap<&str, Bone> = phf::phf_ordered_map! {
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

    pub const fn affected(&self) -> BoneMask {
        self.descendants().with(*self)
    }

    pub const fn children(&self) -> BoneMask {
        BoneMask(CHILDREN[*self as u8 as usize])
    }

    pub const fn descendants(&self) -> BoneMask {
        BoneMask(DESCENDANTS[*self as u8 as usize])
    }

    pub fn iter(
    ) -> impl 'static + Iterator<Item = Self> + Clone + DoubleEndedIterator + ExactSizeIterator
    {
        BONES.values().copied()
    }

    pub const fn mask(&self) -> BoneMask {
        BoneMask::empty().with(*self)
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoneMask(u64);

impl BoneMask {
    pub const fn all() -> BoneMask {
        BoneMask((1u64 << Bone::NUM).wrapping_sub(1))
    }

    pub const fn empty() -> BoneMask {
        BoneMask(0)
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub const fn contains(&self, bone: &Bone) -> bool {
        let bit = 1 << *bone as u8;
        (self.0 & bit) == 0
    }

    pub fn insert(&mut self, bone: Bone) -> bool {
        let bit = 1 << bone as u8;

        let old = self.0;
        self.0 |= bit;

        (old & bit) == 0
    }

    pub const fn into_raw(self) -> u64 {
        self.0
    }

    pub const fn is_disjoint(&self, other: &BoneMask) -> bool {
        (self.0 & other.0) == 0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_subset(&self, other: &BoneMask) -> bool {
        (self.0 & other.0) == self.0
    }

    pub const fn is_superset(&self, other: &BoneMask) -> bool {
        (self.0 | other.0) == self.0
    }

    pub const fn iter(&self) -> BoneMaskIter {
        BoneMaskIter(self.0)
    }

    pub const fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    pub fn remove(&mut self, bone: &Bone) -> bool {
        let bit = 1 << *bone as u8;

        let old = self.0;
        self.0 |= bit;

        (old & bit) == 0
    }

    #[must_use]
    pub const fn difference(&self, other: &BoneMask) -> BoneMask {
        BoneMask(self.0 & !other.0)
    }

    #[must_use]
    pub const fn intersection(&self, other: &BoneMask) -> BoneMask {
        BoneMask(self.0 & other.0)
    }

    #[must_use]
    pub const fn symmetric_difference(&self, other: &BoneMask) -> BoneMask {
        BoneMask(self.0 ^ other.0)
    }

    #[must_use]
    pub const fn union(&self, other: &BoneMask) -> BoneMask {
        BoneMask(self.0 | other.0)
    }

    #[must_use]
    pub const fn with(self, bone: Bone) -> BoneMask {
        let bit = 1 << bone as u8;
        BoneMask(self.0 | bit)
    }

    #[must_use]
    pub const fn without(self, bone: Bone) -> BoneMask {
        let bit = 1 << bone as u8;
        BoneMask(self.0 & !bit)
    }
}

impl Extend<Bone> for BoneMask {
    fn extend<I: IntoIterator<Item = Bone>>(&mut self, iter: I) {
        for bone in iter {
            self.insert(bone);
        }
    }
}

impl<'b> Extend<&'b Bone> for BoneMask {
    fn extend<I: IntoIterator<Item = &'b Bone>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied())
    }
}

impl From<Bone> for BoneMask {
    fn from(value: Bone) -> Self {
        BoneMask(1 << value as u8)
    }
}

impl<const N: usize> From<[Bone; N]> for BoneMask {
    fn from(value: [Bone; N]) -> Self {
        Self::from_iter(value)
    }
}

impl FromIterator<Bone> for BoneMask {
    fn from_iter<I: IntoIterator<Item = Bone>>(iter: I) -> Self {
        let mut mask = BoneMask::empty();
        mask.extend(iter);
        mask
    }
}

impl IntoIterator for BoneMask {
    type IntoIter = BoneMaskIter;
    type Item = Bone;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoneMaskIter(u64);

impl Iterator for BoneMaskIter {
    type Item = Bone;

    fn next(&mut self) -> Option<Self::Item> {
        (self.0 != 0).then(|| {
            let bit = self.0.trailing_zeros();
            self.0 &= !(1 << bit);
            Bone::try_from(bit as u8).expect("BoneMask contains an invalid bone!")
        })
    }
}

impl DoubleEndedIterator for BoneMaskIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        (self.0 != 0).then(|| {
            let bit = 63 - self.0.leading_zeros();
            self.0 &= !(1 << bit);
            Bone::try_from(bit as u8).expect("BoneMask contains an invalid bone!")
        })
    }
}

impl ExactSizeIterator for BoneMaskIter {
    fn len(&self) -> usize {
        self.0.count_ones() as usize
    }
}
