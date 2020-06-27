use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "method", content = "params")]
pub enum Methods {
    GetVersion,
    Scan {
        start: u8,
        stop: u8,
        payload: Vec<u8>,
    },
    SendPacket {
        channel: u8,
        payload: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Results {
    GetVersion(String),
    Scan { found: Vec<u8> },
    SendPacket { acked: bool, payload: Vec<u8> },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub jsonrpc: String,
    #[serde(flatten)]
    pub method: Methods,
    pub id: Option<Id>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseBody {
    Result(Results),
    Error { code: i64, message: String },
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(flatten)]
    pub body: ResponseBody,
    pub id: Option<Id>,
}
