
use std::fmt::{self, Debug, Formatter};

#[derive(Clone)]
pub struct PreDebug {
    pub debug: String,
    pub debug_alt: String,
}

impl PreDebug {
    pub fn new<F>(f: F) -> Self 
    where
        F: Debug,
    {
        PreDebug {
            debug: format!("{:?}", f),
            debug_alt: format!("{:#?}", f),
        }
    }
}

impl Debug for PreDebug {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.debug_alt)
        } else {
            f.write_str(&self.debug)
        }
    }
}
