/// Get the first element or panic.
/// This uses .unwrap() in source code, which should fail.
pub fn first(items: &[i32]) -> i32 {
    items.first().unwrap().to_owned()
}
