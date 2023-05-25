// check for error. if no error, do nothing. if there is an error, print it and return a nonzero value.
#[macro_export]
macro_rules! check_err {
    ($ptr:expr, $str:expr) => {
        match ($ptr) {
            Ok(ref _v) => (),
            Err(e) => {
                eprint!($str);
                eprintln!(": {e}");
                return 1;
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
                eprint!($str);
                return 1;
            },
        }
    };
}

#[macro_export]
macro_rules! err {
    ($str:expr) => {
        eprintln!($str);
        return 1;
    };
}
