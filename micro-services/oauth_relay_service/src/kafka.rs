//#include
mod data_kafka;
mod main_kafka;

use anyhow::Result as AnyResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use rdkafka::client::Client;

use crate::config::*;
use data_kafka::*;
use main_kafka::*;

pub type TMQConnection = Arc<Mutex<Client>>;

pub fn open_mq_connection_from_config(config: Config) -> TMQConnection {
    match config.mq_connection {
        MQType::Kafka {
            host_address,
            host_port,
        } => open_mq_connection(host_address, host_port),
        _ => panic!("Unsupported MQType"),
    }
}
