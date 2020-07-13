use serde::{Deserialize, Serialize};
use crazyradio::Channel;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "method", content = "params")]
pub enum Methods {
    GetVersion,
    Scan {
        start: Channel,
        stop: Channel,
        #[serde(default = "default_address")]
        address: [u8; 5],
        payload: Vec<u8>,
    },
    SendPacket {
        channel: Channel,
        #[serde(default = "default_address")]
        address: [u8; 5],
        payload: Vec<u8>,
    },
    Connect {
        channel: Channel,
        #[serde(default = "default_address")]
        address: [u8; 5],
    },
    GetConnectionStatus {
        channel: Channel,
        #[serde(default = "default_address")]
        address: [u8; 5],
    },
    Disconnect {
        channel: Channel,
        #[serde(default = "default_address")]
        address: [u8; 5],
    },
}

fn default_address() -> [u8; 5] {
    [0xe7; 5]
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Results {
    GetVersion(String),
    Scan {
        found: Vec<Channel>,
    },
    SendPacket {
        acked: bool,
        payload: Vec<u8>,
    },
    Connect {
        connected: bool,
        status: String,
        push: u16,
        pull: u16,
    },
    GetConnectionStatus {
        connected: bool,
        status: String,
    },
    Disconnect,
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
