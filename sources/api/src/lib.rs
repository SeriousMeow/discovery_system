use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub id: String,
}

pub mod register {
    #[allow(dead_code)]
    pub const ENDPOINT: &str = "/register/";

    #[derive(Debug, crate::Serialize, crate::Deserialize)]
    pub struct Request {
        pub credentials: crate::Credentials,
    }

    #[derive(Debug, crate::Serialize, crate::Deserialize)]
    pub struct Response {
        pub reserved_until: crate::DateTime<crate::Utc>,
    }
}

pub mod queue {
    pub type DataItem = String;

    pub mod post {
        #[allow(dead_code)]
        pub const ENDPOINT: &str = "/queue/post";

        #[derive(Debug, crate::Serialize, crate::Deserialize)]
        pub struct Request {
            pub credentials: crate::Credentials,
            pub recipient: String,
            pub data: crate::queue::DataItem,
        }
    }

    pub mod get {
        #[allow(dead_code)]
        pub const ENDPOINT: &str = "/queue/get";

        #[derive(Debug, crate::Serialize, crate::Deserialize)]
        pub struct Request {
            pub credentials: crate::Credentials,
        }

        #[derive(Debug, crate::Serialize, crate::Deserialize)]
        pub struct Response {
            pub data: Vec<crate::queue::DataItem>,
        }
    }
}
