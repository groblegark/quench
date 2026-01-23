use source_test::*;

#[test]
fn test_add() {
    let result = add(1, 2);
    assert_eq!(result, 3);
}

#[test]
fn test_subtract() { assert_eq!(subtract(3, 1), 2); }
