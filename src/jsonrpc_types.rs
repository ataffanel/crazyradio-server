use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "method", content = "params")]
pub enum Methods {
    GetVersion,
    Scan {
        #[serde(default = "default_address")]
        address: [u8; 5],
    },
    Connect {
        uri: String,
    },
    GetConnectionStatus {
        uri: String,
    },
    Disconnect {
        uri: String,
    },
}

fn default_address() -> [u8; 5] {
    [0xe7; 5]
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Results {
    GetVersion(String),
    Scan {
        found: Vec<String>,
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub jsonrpc: String,
    #[serde(flatten)]
    pub method: Methods,
    pub id: Option<Id>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ResponseBody {
    Result(Results),
    Error { code: i64, message: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub jsonrpc: String,
    #[serde(flatten)]
    pub body: ResponseBody,
    pub id: Option<Id>,
}
