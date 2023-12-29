use dfunc;

#[test]
fn test_rust() {
    // Check the Rust interface
    let result = dfunc::swirl(0.0, 2.0);
    assert_eq!(8.0, result);
    let dresult = dfunc::d_swirl(1.0, 2.0);
    assert_eq!(12.5, dresult);
}

#[test]
fn test_extern_c() {
    // Check the extern "C" interface
    let result = dfunc::dfunc_swirl(0.0, 2.0);
    assert_eq!(8.0, result);
    let dresult = dfunc::dfunc_d_swirl(1.0, 2.0);
    assert_eq!(12.5, dresult);
}
