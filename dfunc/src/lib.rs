#![no_std]

#[cfg(not(test))]
use num_traits::float::FloatCore;
//use num_traits::Float;

pub fn add(left: f32, right: f32) -> f32 {
    left + right.powi(3)
}

#[no_mangle]
extern "C" fn dfunc_add(left: f32, right: f32) -> f32 {
    add(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2.0, 2.0);
        assert_eq!(result, 10.0);
    }
}
