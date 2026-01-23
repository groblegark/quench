/// Get the first element.
pub fn first(items: &[i32]) -> Option<i32> {
    items.first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first() {
        // .unwrap() inside #[cfg(test)] should be allowed
        let result = first(&[1, 2, 3]).unwrap();
        assert_eq!(result, 1);
    }
}
