use std::sync::Mutex;

use crate::{config::Config, data_handler::DataHandler};

#[derive(Debug)]
pub struct SKStateInner {
    pub data: Option<DataHandler>,
    pub c: Option<Config>,
}

pub type SKState = Mutex<SKStateInner>;
