pub struct SKStateInner {
    pub data: Option<DataHandler>,
    pub c: Option<Config>,
}

pub type SKState = Mutex<SKStateInner>;
