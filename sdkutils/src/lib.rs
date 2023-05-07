// check for error. if no error, do nothing. if there is an error, print it and return a null ptr.
#[macro_export]
macro_rules! check_err {
    ($ptr:expr, $str:expr) => {
        match ($ptr) {
            Ok(ref _v) => (),
            Err(e) => {
                // TODO: return error message rather than printing it
                eprint!($str);
                eprintln!(": {e}");
                return std::ptr::null_mut();
            },
        }
    };
}

#[macro_export]
macro_rules! check_none {
    ($ptr:expr, $str:expr) => {
        match ($ptr) {
            Some(ref _v) => (),
            None => {
                // TODO: return error message rather than printing it
                eprint!($str);
                return std::ptr::null_mut();
            },
        }
    };
}

#[macro_export]
macro_rules! err {
    ($str:expr) => {
        eprintln!($str);
        return std::ptr::null_mut();
    };
}

/// free the memory allocated and returned by extern functions by transferring ownership back to
/// Rust. must be called on each pointer returned by the functions precisely once to ensure safety.
#[no_mangle]
pub extern "C" fn freeptr(ptr: *mut u8) {
    unsafe {
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr);
        }
    }
}
