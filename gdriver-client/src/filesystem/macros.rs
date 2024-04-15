mod reply {
    #[macro_export]
    macro_rules! reply_error_o {
        ($option_in:expr, $reply:ident, $error_code:expr, $error_msg:expr) => {
            reply_error_o!($option_in, $reply, $error_code, $error_msg,)
        };
        ($option_in:expr, $reply:ident, $error_code:expr, $error_msg:expr, $($arg:tt)*) => {{
            match $option_in {
                None=>{
                    ::tracing::error!($error_msg, $($arg)*);
                    $reply.error($error_code);
                    return;
                },
                Some(x) => x,
            }
        }};
    }
    #[macro_export]
    macro_rules! reply_error_e {
        ($result_in:expr, $reply:ident, $error_code:expr, $error_msg:expr) => {
            reply_error_e!($result_in, $reply, $error_code, $error_msg,)
        };
        ($result:expr, $reply:ident, $error_code:expr, $error_msg:expr, $($arg:tt)*) => {{
            match $result {
                Ok(x) => x,
                Err(e) => {
                    error!("{}; e:{}",format!($error_msg, $($arg)*), e);
                    $reply.error($error_code);
                    return;
                }
            }
        }};
    }
}

mod send_requests {
    #[macro_export]
    macro_rules! send_request {
        ($func:expr ) => {{
            let handle = ::tokio::runtime::Handle::current();
            handle.block_on($func)
        }};
    }
    #[macro_export]
    macro_rules! send_request_handled_internal {
        ($func:expr ,$reply:ident, $error_code:expr, $error_msg:expr, $($arg:tt)*) => {{
            let x = send_request!($func);
            reply_error_e!(x, $reply, $error_code, $error_msg, $($arg)*)
        }};
    }
    #[macro_export]
    macro_rules! send_request_handled {
        ($func:expr ,$reply:ident) => {
            send_request_handled!($func, $reply, "")
        };
        ($func:expr ,$reply:ident, $error_msg:expr) => {
            send_request_handled_internal!(
                $func,
                $reply,
                ::libc::EREMOTEIO,
                "Failed send request: {}",
                $error_msg
            )
        };
    }

    #[macro_export]
    macro_rules! send_request_handled_consuming {
        ($func:expr ,$reply:ident) => {
            send_request_handled_consuming!($func, $reply, "");
        };
        ($func:expr ,$reply:ident, $error_msg:expr) => {
            let _ = send_request_handled!($func, $reply, $error_msg);
        };
    }
}
