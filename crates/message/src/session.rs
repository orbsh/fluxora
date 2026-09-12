use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fmt::Display, ops::Deref};

pub type SessionId = String;
pub type SessionCount = u128;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash)]
pub struct Session(pub SessionId);

impl Deref for Session {
    type Target = SessionId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for Session {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<Session> for String {
    fn from(value: Session) -> Self {
        value.0
    }
}

impl<'a> From<&'a Session> for &'a str {
    fn from(value: &'a Session) -> Self {
        &value.0
    }
}

impl From<Session> for Value {
    fn from(value: Session) -> Self {
        value.0.into()
    }
}

impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<SessionCount> for Session {
    fn from(value: SessionCount) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash)]
pub struct SessionInfo {
    #[serde(rename = "session_id")]
    pub id: Session,
    pub info: Map<String, Value>,
}

impl From<SessionInfo> for Map<String, Value> {
    fn from(session: SessionInfo) -> Self {
        let mut m = Map::new();
        m.insert("session_id".into(), session.id.into());
        m.insert("info".into(), session.info.into());
        m
    }
}
