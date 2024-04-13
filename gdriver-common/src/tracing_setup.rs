use crate::prelude::*;
pub fn init_tracing(){
    tracing_subscriber::fmt().init();
    info!("tracing initialized");
}
