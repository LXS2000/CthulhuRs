#[macro_export]
macro_rules! auto_result {
    ($expr:expr, $var:ident => $err:block) => {
        match $expr {
            Ok(val) => val,
            Err($var) => $err,
        }
    };
    ($expr:expr, $var:ident => $err:expr) => {
        match $expr {
            Ok(val) => val,
            Err($var) => return $err,
        }
    };
    ($expr:expr, $var:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_e) => {
                // tracing::error!("{_e}");
                // eprintln!("{_e}");
                return $var;
            }
        }
    };
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_err) => return _err,
        }
    };
}

#[macro_export]
macro_rules! auto_option {
    ($expr:expr, $var:block) => {
        match $expr {
            Some(val) => val,
            None => $var,
        }
    };

    ($expr:expr, $var:expr) => {
        match $expr {
            Some(val) => val,
            None => return $var,
        }
    };
    ($expr:expr,$val:ident => $exp:expr, $blk:block) => {
        match $expr {
            Some($val) => {
                if $exp {
                    $blk
                } else {
                    $val
                }
            }
            None => $blk,
        }
    };
}

#[macro_export]
macro_rules! auto_batch_result {
    ($iter:expr, $var:ident => $err:block) => {{
        let mut _vec_ = vec![];
        for _ele_ in $iter {
            match _ele_ {
                Ok(val) => {
                    _vec_.push(val);
                }
                Err($var) => $err,
            }
        }
        _vec_
    }};
}

#[macro_export]
macro_rules! auto_batch_option {
    ($iter:expr, $var:block) => {
        let mut _vec_ = vec![];
        for _ele_ in $iter {
            match _ele_ {
                Some(val) => {
                    _vec_.push(val);
                }
                None => $var,
            }
        }
        _vec_
    };
}
