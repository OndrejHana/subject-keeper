use std::sync::Mutex;

use crate::{config::Config, data_handler::DataHandler};

pub struct SKStateInner {
    pub data: Option<DataHandler>,
    pub c: Option<Config>,
}

pub type SKState = Mutex<SKStateInner>;
