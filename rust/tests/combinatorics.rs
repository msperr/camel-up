use camel_up::combinatorics::{Permutations, Product};

#[test]
fn test_permutations_small() {
    let p = Permutations::new(vec![1, 2, 3]);
    let mut got: Vec<Vec<u8>> = p.collect();
    got.sort(); // canonicalize order for assertion
    let mut expected = vec![
        vec![1, 2, 3],
        vec![1, 3, 2],
        vec![2, 1, 3],
        vec![2, 3, 1],
        vec![3, 1, 2],
        vec![3, 2, 1],
    ];
    expected.sort();
    assert_eq!(got, expected);
}

#[test]
fn test_product_repeat_2() {
    let p = Product::new(vec![1, 2], 2);
    let mut got: Vec<Vec<u8>> = p.collect();
    got.sort();
    let mut expected = vec![vec![1, 1], vec![1, 2], vec![2, 1], vec![2, 2]];
    expected.sort();
    assert_eq!(got, expected);
}
