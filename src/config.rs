use std::env;

const ENV_SOCKET_ADDR: &str = "SOCKET_ADDR";
const ENV_INDEX_NAME: &str = "INDEX";
const ENV_STORAGE_NAME: &str = "STORAGE";
const ENV_CLUSTER_ADDRS: &str = "CLUSTER_ADDRS";

const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:6669";
const DEFAULT_INDEX_NAME: &str = "nonsense";
const DEFAULT_STORAGE_NAME: &str = "in_memory";

pub(crate) struct Config {
    pub(crate) socket_addr: String,
    pub(crate) index_name: String,
    pub(crate) storage_name: String,
    pub(crate) cluster_addrs: String,
}

impl Config {
    pub(super) fn new() -> Self {
        let socket_addr =
            env::var(ENV_SOCKET_ADDR).unwrap_or_else(|_| DEFAULT_SOCKET_ADDR.to_string());
        let index_name =
            env::var(ENV_INDEX_NAME).unwrap_or_else(|_| DEFAULT_INDEX_NAME.to_string());
        let storage_name =
            env::var(ENV_STORAGE_NAME).unwrap_or_else(|_| DEFAULT_STORAGE_NAME.to_string());
        let cluster_addrs = env::var(ENV_CLUSTER_ADDRS).unwrap_or_default();
        Self {
            socket_addr,
            index_name,
            storage_name,
            cluster_addrs,
        }
    }
}
