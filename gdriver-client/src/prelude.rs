pub(crate) use gdriver_common::prelude::result::*;
pub(crate) use gdriver_common::prelude::*;
pub(crate) mod macros {
    pub(crate) use crate::{
        reply_error_e, reply_error_o, send_request, send_request_handled_internal, send_request_handled,
        send_request_handled_consuming,
    };
}
