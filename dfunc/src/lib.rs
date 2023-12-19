#![no_std]

pub fn add(left: f32, right: f32) -> f32 {
    left + right
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
        assert_eq!(result, 4.0);
    }
}
