use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "method", content="params")]
pub enum Methods {
    Hello,
    SendPacket{channel: u8, packet: Vec<u8>},
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
#[serde(untagged)]
pub enum Result {
    Hello(String),
    SendPacket{acked: bool, payload: Vec<u8>},
}

#[derive(Serialize, Deserialize)]
pub enum ResponseBody {
    Result(Result),
    Error {
        code: i64,
        message: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(flatten)]
    pub body: ResponseBody,
    pub id: Option<Id>
}
