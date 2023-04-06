pub trait FloatExt: Sized {
    fn inv_lerp(self, a: Self, b: Self) -> Self;
    fn inv_lerp_checked(self, a: Self, b: Self) -> Option<Self>;
    fn lerp(self, a: Self, b: Self) -> Self;
    fn ping_pong(self, w: Self) -> Self;

    fn ease(self, shape: Self) -> Self;
}

macro_rules! impl_float_ext{
    ($t:ty) => {
        impl FloatExt for $t {
            #[inline]
            fn inv_lerp(self, a: $t, b: $t) -> $t {
                debug_assert_ne!(a, b);
                (self - a) / (b - a)
            }

            #[inline]
            fn inv_lerp_checked(self, a: $t, b: $t) -> Option<$t> {
                (a != b).then(|| (self - a) / (b - a)).filter(|t| (0.0..=1.0).contains(t))
            }

            #[inline]
            fn lerp(self, a: $t, b: $t) -> $t {
                a + self * (b - a)
            }

            #[inline]
            fn ping_pong(self, w: $t) -> $t {
                debug_assert!(w > 0.0);
                let t = (self / w).rem_euclid(2.0);
                if t <= 1.0 { t } else { 2.0 - t }
            }

            fn ease(self, shape: $t) -> $t {
                let t = self.clamp(0.0, 1.0);
                if shape > 0.0 {
                    if shape >= 1.0 {
                        t.powf(shape)
                    } else {
                        1.0 - (1.0 - t).powf(1.0 / shape)
                    }
                } else if shape < 0.0 {
                    if t <= 0.5 {
                        let u = 2.0 * t;
                        0.5 * u.powf(-shape)
                    } else {
                        let u = 2.0 - 2.0 * t;
                        1.0 - 0.5 * u.powf(-shape)
                    }
                } else {
                    0.0
                }
            }
        }
    };

    ($t:ty, $($u:ty),+) => {
        impl_float_ext!($t);
        impl_float_ext!($($u),+);
    };
}

impl_float_ext!(f32, f64);
