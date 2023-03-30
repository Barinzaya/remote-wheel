use glam::{EulerRot, Quat, Vec3, Vec3A};

mod tri;
pub use tri::{solve as solve_tri, Settings as TriSettings};

#[derive(Clone, Copy, Debug)]
pub enum AngularConstraint {
    None,
    Hinge(Vec3, (f32, f32)),
    Euler(EulerRot, (f32, f32), (f32, f32), (f32, f32)),
}

impl AngularConstraint {
    pub fn apply(&self, rot: Quat) -> Quat {
        match *self {
            AngularConstraint::None => rot,

            AngularConstraint::Euler(euler, (min_x, max_x), (min_y, max_y), (min_z, max_z)) => {
                let (angle_x, angle_y, angle_z) = rot.to_euler(euler);

                let angle_x = angle_x.clamp_angle(min_x, max_x).normalize_angle_pi();
                let angle_y = angle_y.clamp_angle(min_y, max_y).normalize_angle_pi();
                let angle_z = angle_z.clamp_angle(min_z, max_z).normalize_angle_pi();

                Quat::from_euler(euler, angle_x, angle_y, angle_z)
            }

            AngularConstraint::Hinge(hinge_axis, (min_angle, max_angle)) => {
                let angle = rot
                    .to_scaled_axis()
                    .dot(hinge_axis)
                    .clamp_angle(min_angle, max_angle)
                    .normalize_angle_pi();
                Quat::from_axis_angle(hinge_axis, angle)
            }
        }
    }

    pub fn to_radians(self) -> AngularConstraint {
        match self {
            AngularConstraint::None => AngularConstraint::None,
            AngularConstraint::Euler(euler, (min_x, max_x), (min_y, max_y), (min_z, max_z)) => {
                AngularConstraint::Euler(
                    euler,
                    (min_x.to_radians(), max_x.to_radians()),
                    (min_y.to_radians(), max_y.to_radians()),
                    (min_z.to_radians(), max_z.to_radians()),
                )
            }

            AngularConstraint::Hinge(hinge_axis, (min_angle, max_angle)) => {
                AngularConstraint::Hinge(
                    hinge_axis,
                    (min_angle.to_radians(), max_angle.to_radians()),
                )
            }
        }
    }
}

pub trait Chain {
    type Link<'l>: 'l + Link
    where
        Self: 'l;

    fn num_links(&self) -> usize;
    fn link(&mut self, index: usize) -> Self::Link<'_>;
}

pub trait Link {
    fn angular_constraint(&self) -> AngularConstraint;

    fn pos(&mut self) -> Vec3A;
    fn rot(&mut self) -> Quat;
    fn state(&mut self) -> (Vec3A, Quat) {
        (self.pos(), self.rot())
    }

    fn set_rot(&mut self, new_rot: Quat);
}

trait FloatExt {
    fn clamp_angle(self, min: Self, max: Self) -> Self;
    fn normalize_angle_pi(self) -> Self;
    fn normalize_angle_2pi(self) -> Self;
}

impl FloatExt for f32 {
    fn clamp_angle(self, min: Self, max: Self) -> Self {
        let half_span = 0.5 * (max - min);
        let center = min + half_span;
        let delta = (self - center).normalize_angle_pi();
        center + delta.clamp(-half_span, half_span)
    }

    fn normalize_angle_pi(self) -> Self {
        (self + std::f32::consts::PI).normalize_angle_2pi() - std::f32::consts::PI
    }

    fn normalize_angle_2pi(self) -> Self {
        self.rem_euclid(std::f32::consts::TAU)
    }
}
