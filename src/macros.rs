#[macro_export]
macro_rules! assert_enum_variant {
    ($target: expr, $pat: path) => {{
        if let $pat = $target {
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat)); // #2
        }
    }};
}

#[macro_export]
macro_rules! extract_enum_value {
    ($value:expr, $pattern:pat => $extracted_value:expr) => {
        match $value {
            $pattern => $extracted_value,
            _ => panic!("Pattern doesn't match!"),
        }
    };
}

#[macro_export]
#[cfg(test)]
macro_rules! now {
    () => {
        SystemTime::UNIX_EPOCH
    };
}

#[macro_export]
#[cfg(not(test))]
macro_rules! now {
    () => {
        SystemTime::now()
    };
}
