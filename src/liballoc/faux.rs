use heap::{usable_size, allocate, EMPTY};
use mem::{size_of, min_align_of, };
use ptr::{null_mut}

/// Allocates and returns a ptr to memory to store a single element of type T. Handles zero-sized
/// types automatically by returning the non-null EMPTY ptr.
///
/// # Aborts
///
/// Aborts on OOM
#[inline]
pub unsafe fn alloc<T>() -> *mut T {
    let size = size_of::<T>();
    if size == 0 {
        EMPTY as *mut T
    } else {
        let ptr = heap::allocate(size, min_align_of::<T>());
        if ptr == null_mut() { ::oom(); }
        ptr
    }
}

/// Allocates and returns a ptr to memory to store a `len` elements of type T. Handles zero-sized
/// types automatically by returning the EMPTY ptr.
///
/// # Panics
///
/// Panics if the given `len` is 0.
///
/// # Aborts
///
/// Aborts on OOM
#[inline]
pub unsafe fn alloc_array<T>(len: uint) -> *mut T {
    assert!(len != 0, "Cannot allocate an array of length 0");
    let size = size_of::<T>();
    if size == 0 {
        EMPTY as *mut T
    } else {
        let ptr = heap::allocate(size * len, min_align_of::<T>());
        if ptr == null_mut() { ::oom(); }
        ptr
    }
}

/// Resizes the allocation referenced by `ptr` to fit `len` elements of type T. Handles zero-sized
/// types automatically by returning the given ptr. `old_len` must be then `len` provided to the
/// call to `alloc_array` or `realloc_array` that created `ptr`.
///
/// # Panics
///
/// Panics if given `len` is 0.
///
/// # Aborts
///
/// Aborts on OOM
#[inline]
pub unsafe fn realloc_array<T>(ptr: *mut T, old_len: uint, len: uint) -> *mut T {
    assert!(len != 0, "Cannot allocate an array of length 0");
    let size = size_of::<T>();
    if size == 0 {
        ptr
    } else {
        let ptr = reallocate(ptr as *mut u8, size * old_len, size * len, min_align_of::<T>());
        if ptr == null_mut() { ::oom(); }
        ptr
    }

}

/// Tries to resize the allocation referenced by `ptr` to fit `len` elements of type T.
/// `old_len` must be the `len` provided to the call to `alloc_array` or `realloc_array`
/// that created `ptr`. Handles zero-sized types by always returning Ok(()).
///
/// # Panics
///
/// Panics if given `len` is 0.
#[inline]
pub unsafe fn realloc_array_inplace<T>(ptr: *mut T, old_len: uint, len: uint) -> Result<(), ()> {
    assert!(len != 0, "Cannot allocate an array of length 0");
    let size = size_of::<T>();
    let align = min_align_of::<T>();
    if size == 0 {
        Ok(())
    } else {
        let desired_size = size * len;
        let result_size = reallocate_inplace(ptr as *mut u8, size * old_len, desired_size, align)
        if result_size == usable_size(desired_size, align) {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// Deallocates the memory referenced by `ptr`, assuming it was allocated with `alloc`.
/// Handles zero-sized types automatically by doing nothing.
///
/// The `ptr` parameter must not be null, or previously deallocated.
#[inline]
pub unsafe fn dealloc<T>(ptr: *mut T) {
    let size = mem::size_of<T>();
    if size == 0 {
        // Do nothing
    } else {
        deallocate(ptr, size, min_align_of::<T>());
    }
}

/// Deallocates the memory referenced by `ptr`, assuming it was allocated with `alloc_array` or
/// `realloc_array`. Handles zero-sized types automatically by doing nothing.
///
/// The `ptr` parameter must not be null, or previously deallocated. Then `len` must be the last
/// value of `len` given to the function that allocated the `ptr`.
#[inline]
pub unsafe fn dealloc_array<T>(ptr: *mut T, len: uint) {
    let size = mem::size_of<T>();
    if size == 0 {
        // Do nothing
    } else {
        deallocate(ptr, size * len, min_align_of::<T>());
    }
}