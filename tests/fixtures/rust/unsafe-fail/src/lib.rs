/// Convert a raw pointer to a reference.
/// Missing SAFETY comment - should fail.
pub unsafe fn deref_ptr<T>(ptr: *const T) -> &'static T {
    unsafe { &*ptr }
}
