use crate::topic_filter::TopicFilter;

mod decoding;
mod encoding;
pub use encoding::ConnectBuilder;
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod tests;

#[doc(hidden)]
const PROTOCOL_LEVEL_3_1_1: u8 = 0x04;
#[doc(hidden)]
const USER_NAME_PRESENT: u8 = 0x80;
#[doc(hidden)]
const PASSWORD_PRESENT: u8 = 0x40;
#[doc(hidden)]
const LAST_WILL_PRESENT: u8 = 0x04;
#[doc(hidden)]
const WILL_RETAIN: u8 = 0x20;
#[doc(hidden)]
const WILL_QOS_SHIFT: u8 = 3;
#[doc(hidden)]
const CLEAN_SESSION: u8 = 0x02;
#[doc(hidden)]
const RESERVED_BITS: u8 = 0x0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastWill {
    pub retain_flag: bool,
    pub topic: TopicFilter,
    pub topic_message: String,
}

impl LastWill {
    pub fn new(topic: TopicFilter, topic_message: String, retain_flag: bool) -> LastWill {
        LastWill {
            retain_flag,
            topic,
            topic_message,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connect {
    client_id: String,
    clean_session: bool,
    user_name: Option<String>,
    password: Option<String>,
    last_will: Option<LastWill>,
    keep_alive: u16,
}

impl Connect {
    /// Get a reference to the connect's client id.
    pub fn client_id(&self) -> &str {
        self.client_id.as_str()
    }

    /// Get a reference to the connect's clean session.
    pub fn clean_session(&self) -> &bool {
        &self.clean_session
    }

    /// Get a reference to the connect's user name.
    pub fn user_name(&self) -> Option<&String> {
        self.user_name.as_ref()
    }

    /// Get a reference to the connect's password.
    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    /// Get a reference to the connect's last will.
    pub fn last_will(&self) -> Option<&LastWill> {
        self.last_will.as_ref()
    }

    /// Take the [LastWill] packet, replacing it with
    /// None. If the packet was already None, return None
    pub fn take_last_will(&mut self) -> Option<LastWill> {
        self.last_will.take()
    }

    /// Get a reference to the connect's keep alive.
    pub fn keep_alive(&self) -> u16 {
        self.keep_alive
    }

    /// Set the client Id if it is None
    /// If not None, it silently does nothing
    pub fn set_id(&mut self, id: String) {
        if self.client_id.is_empty() {
            self.client_id = id;
        }
    }
}
