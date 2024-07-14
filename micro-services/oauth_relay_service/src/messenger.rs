//#include
pub mod kafka;

use anyhow::Result as AnyResult;
use rdkafka::client::Client;
use std::{net::IpAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::{config::*, data::TokenData};

pub type TMQConnection = kafka::TMQConnection_kafka; // Currently using Kafka as the MQ
pub type TMQConnectionLock = kafka::TMQConnectionLock_kafka;

pub async fn open_mq_connection_from_config(config: Config) -> TMQConnectionLock {
    match config.mq_connection {
        MQType::Kafka {
            host_address,
            host_port,
        } => kafka::open_mq_connection_kafka(host_address, host_port),
        MQType::MongoDB {
            host_address: _,
            host_port: _,
        } => panic!("MongoDB not yet implemented"),
        MQType::RabbitMQ {
            host_address: _,
            host_port: _,
        } => panic!("RabbitMQ not yet implemented"),
        MQType::Redis {
            host_address: _,
            host_port: _,
        } => panic!("Redis not yet implemented"),
    }
}

pub(crate) fn get_mq_type(config: &Config) -> String {
    match config.mq_connection {
        MQType::Kafka {
            host_address: _,
            host_port: _,
        } => "kafka".to_string(),
        MQType::MongoDB {
            host_address: _,
            host_port: _,
        } => "mongodb".to_string(),
        MQType::RabbitMQ {
            host_address: _,
            host_port: _,
        } => "rabbitmq".to_string(),
        MQType::Redis {
            host_address: _,
            host_port: _,
        } => "redis".to_string(),
    }
}

pub(crate) fn get_mq_address(config: &Config) -> Option<IpAddr> {
    match config.mq_connection {
        MQType::Kafka {
            host_address,
            host_port: _,
        } => Some(host_address),
        MQType::MongoDB {
            host_address,
            host_port: _,
        } => Some(host_address),
        MQType::RabbitMQ {
            host_address,
            host_port: _,
        } => Some(host_address),
        MQType::Redis {
            host_address,
            host_port: _,
        } => Some(host_address),
    }
}

pub(crate) fn get_mq_port(config: &Config) -> Option<u16> {
    match config.mq_connection {
        MQType::Kafka {
            host_address: _,
            host_port,
        } => Some(host_port),
        MQType::MongoDB {
            host_address: _,
            host_port,
        } => Some(host_port),
        MQType::RabbitMQ {
            host_address: _,
            host_port,
        } => Some(host_port),
        MQType::Redis {
            host_address: _,
            host_port,
        } => Some(host_port),
    }
}

pub(crate) fn get_token<TFn>(
    config: &Config,
    mq_lock: &TMQConnectionLock,
    lhs_state_token: &str,
    fn_equ_op: TFn,
) -> Option<TokenData>
where
    TFn: Fn(&str /*lhs_state_token*/, TokenData /*rhs*/) -> bool, // compare lhs_state_token == rhs:TokenData.state_token
{
    match config.mq_connection {
        MQType::Kafka {
            host_address: _,
            host_port: _,
        } => kafka::get_token_kafka(mq_lock, lhs_state_token, fn_equ_op),
        MQType::MongoDB {
            host_address: _,
            host_port: _,
        } => None,
        MQType::RabbitMQ {
            host_address: _,
            host_port: _,
        } => None,
        MQType::Redis {
            host_address: _,
            host_port: _,
        } => None,
    }
}

pub(crate) async fn post_new_login(
    config: &Config,
    db_connection: &TMQConnectionLock,
    token_data: &TokenData,
) -> AnyResult<()> {
    match config.mq_connection {
        MQType::Kafka {
            host_address: _,
            host_port: _,
        } => kafka::post_new_login_kafka(db_connection, token_data).await,
        MQType::MongoDB {
            host_address: _,
            host_port: _,
        } => panic!("MongoDB not yet implemented"),
        MQType::RabbitMQ {
            host_address: _,
            host_port: _,
        } => panic!("RabbitMQ not yet implemented"),
        MQType::Redis {
            host_address: _,
            host_port: _,
        } => panic!("Redis not yet implemented"),
    }
}
