// Tests are allowed to use .unwrap()

#[test]
fn test_first() {
    let items = [1, 2, 3];
    let result = unwrap_test::first(&items).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn test_first_empty() {
    let items: [i32; 0] = [];
    assert!(unwrap_test::first(&items).is_none());
}
