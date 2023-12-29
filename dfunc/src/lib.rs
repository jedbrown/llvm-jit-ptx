#![cfg_attr(not(test), no_std)]
#![feature(linkage)]

#[allow(unused_imports)]
use num_traits::float::FloatCore;

#[cfg(target_arch = "nvptx64")]
extern "C" {
    #[linkage = "internal"]
    #[link_name = "__nv_log1pf"]
    fn log1pf(x: f32) -> f32;
}

#[cfg(not(target_arch = "nvptx64"))]
extern "C" {
    fn log1pf(x: f32) -> f32;
}

#[cfg(feature = "enzyme")]
mod swirl {
    use super::log1pf;
    #[cfg(not(test))]
    use super::FloatCore;
    use autodiff::autodiff;
    #[autodiff(d_swirl_impl, Reverse, Active)]
    fn swirl_impl(#[dup] a: &[f32; 2]) -> f32 {
        (unsafe { log1pf(a[0]) }) + a[1].powi(3)
    }
    pub fn swirl(left: f32, right: f32) -> f32 {
        swirl_impl(&[left, right])
    }
    pub fn d_swirl(left: f32, right: f32) -> f32 {
        let mut x_ = [0.0; 2];
        d_swirl_impl(&[left, right], &mut x_, 1.0);
        x_[0] + x_[1]
    }
}

#[cfg(not(feature = "enzyme"))]
mod swirl {
    use super::{log1pf, FloatCore};
    pub fn swirl(left: f32, right: f32) -> f32 {
        (unsafe { log1pf(left) }) + right.powi(3)
    }

    pub fn d_swirl(left: f32, right: f32) -> f32 {
        let f_ = 1.0;
        let left_ = f_ / (1.0 + left);
        let right_ = f_ * 3.0 * right.powi(2);
        left_ + right_
    }
}

pub use swirl::{d_swirl, swirl};

#[no_mangle]
pub extern "C" fn dfunc_swirl(left: f32, right: f32) -> f32 {
    swirl(left, right)
}

#[no_mangle]
pub extern "C" fn dfunc_d_swirl(left: f32, right: f32) -> f32 {
    d_swirl(left, right)
}

#[cfg(test)]
mod tests {
    #[test]
    fn swirl() {
        let result = super::swirl(0.0, 2.0);
        assert_eq!(8.0, result);
    }

    #[test]
    fn d_swirl() {
        let result = super::d_swirl(1.0, 2.0);
        assert_eq!(12.5, result);
    }

    #[test]
    fn extern_c() {
        let result = super::dfunc_swirl(0.0, 2.0);
        assert_eq!(8.0, result);
        let dresult = super::dfunc_d_swirl(1.0, 2.0);
        assert_eq!(12.5, dresult);
    }
}
