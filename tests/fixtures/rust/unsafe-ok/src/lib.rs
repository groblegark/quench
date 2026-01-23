/// Convert a raw pointer to a reference.
///
/// # Safety
///
/// The pointer must be valid and properly aligned.
pub unsafe fn deref_ptr<T>(ptr: *const T) -> &'static T {
    // SAFETY: Caller guarantees the pointer is valid and properly aligned.
    unsafe { &*ptr }
}
