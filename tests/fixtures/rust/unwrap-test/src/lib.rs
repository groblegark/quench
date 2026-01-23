/// Get the first element or return None.
pub fn first(items: &[i32]) -> Option<i32> {
    items.first().copied()
}
