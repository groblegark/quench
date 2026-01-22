// For future escapes check testing
fn main() {
    unsafe {
        // Missing SAFETY comment
        std::ptr::null::<i32>().read();
    }
}
