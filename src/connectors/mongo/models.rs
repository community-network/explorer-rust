use bf_sparta::cookie::Cookie;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackendCookie {
    pub _id: String,
    pub sid: String,
    pub remid: String,
}

impl From<BackendCookie> for Cookie {
    fn from(cookie: BackendCookie) -> Self {
        Cookie {
            remid: cookie.remid,
            sid: cookie.sid,
        }
    }
}
