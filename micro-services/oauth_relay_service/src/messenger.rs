//#include
pub mod kafka;

use anyhow::Result as AnyResult;
use crate::{config::*, data::TokenData};

pub type TMQProducer = kafka::TMQProducerKafka; // Currently using Kafka as the MQ
pub type TMQProducerLock = kafka::TMQProducerLockKafka;
pub type TMQConsumer = kafka::TMQConsumerKafka; // Currently using Kafka as the MQ
pub type TMQConsumerLock = kafka::TMQConsumerLockKafka;

pub async fn open_mq_connections_from_config(config: &Config) -> (TMQProducerLock, TMQConsumerLock) {
    match &config.mq_connection {
        MQType::Kafka {
            host_as_name_or_address,
            host_port,
        } => kafka::open_mq_connections_kafka(&host_as_name_or_address, host_port),
        MQType::MongoDB {
            host_as_name_or_address: _,
            host_port: _,
        } => panic!("MongoDB not yet implemented"),
        MQType::RabbitMQ {
            host_as_name_or_address: _,
            host_port: _,
        } => panic!("RabbitMQ not yet implemented"),
        MQType::Redis {
            host_as_name_or_address: _,
            host_port: _,
        } => panic!("Redis not yet implemented"),
    }
}

pub(crate) fn get_mq_type(config: &Config) -> String {
    match &config.mq_connection {
        MQType::Kafka {
            host_as_name_or_address: _,
            host_port: _,
        } => "kafka".to_string(),
        MQType::MongoDB {
            host_as_name_or_address: _,
            host_port: _,
        } => "mongodb".to_string(),
        MQType::RabbitMQ {
            host_as_name_or_address: _,
            host_port: _,
        } => "rabbitmq".to_string(),
        MQType::Redis {
            host_as_name_or_address: _,
            host_port: _,
        } => "redis".to_string(),
    }
}

pub(crate) fn get_mq_address(config: &Config) -> Option<HostType> {
    match &config.mq_connection {
        MQType::Kafka {
            host_as_name_or_address,
            host_port: _,
        } => Some(host_as_name_or_address.clone()),
        MQType::MongoDB {
            host_as_name_or_address,
            host_port: _,
        } => Some(host_as_name_or_address.clone()),
        MQType::RabbitMQ {
            host_as_name_or_address,
            host_port: _,
        } => Some(host_as_name_or_address.clone()),
        MQType::Redis {
            host_as_name_or_address,
            host_port: _,
        } => Some(host_as_name_or_address.clone()),
    }
}

pub(crate) fn get_mq_port(config: &Config) -> Option<u16> {
    match &config.mq_connection {
        MQType::Kafka {
            host_as_name_or_address: _,
            host_port,
        } => Some(host_port.clone()),
        MQType::MongoDB {
            host_as_name_or_address: _,
            host_port,
        } => Some(host_port.clone()),
        MQType::RabbitMQ {
            host_as_name_or_address: _,
            host_port,
        } => Some(host_port.clone()),
        MQType::Redis {
            host_as_name_or_address: _,
            host_port,
        } => Some(host_port.clone()),
    }
}

pub(crate) fn get_token<TFn>(
    config: &Config,
    mq_lock: &TMQProducerLock,
    lhs_state_token: &str,
    fn_equ_op: TFn,
) -> Option<TokenData>
where
    TFn: Fn(&str /*lhs_state_token*/, TokenData /*rhs*/) -> bool, // compare lhs_state_token == rhs:TokenData.state_token
{
    match &config.mq_connection {
        MQType::Kafka {
            host_as_name_or_address: _,
            host_port: _,
        } => kafka::get_token_kafka(mq_lock, lhs_state_token, fn_equ_op),
        MQType::MongoDB {
            host_as_name_or_address: _,
            host_port: _,
        } => None,
        MQType::RabbitMQ {
            host_as_name_or_address: _,
            host_port: _,
        } => None,
        MQType::Redis {
            host_as_name_or_address: _,
            host_port: _,
        } => None,
    }
}

pub(crate) async fn post_new_login(
    config: &Config,
    db_connection: &TMQProducerLock,
    token_data: &TokenData,
) -> AnyResult<()> {
    match &config.mq_connection {
        MQType::Kafka {
            host_as_name_or_address: _,
            host_port: _,
        } => kafka::post_new_login_kafka(db_connection, token_data).await,
        MQType::MongoDB {
            host_as_name_or_address: _,
            host_port: _,
        } => panic!("MongoDB not yet implemented"),
        MQType::RabbitMQ {
            host_as_name_or_address: _,
            host_port: _,
        } => panic!("RabbitMQ not yet implemented"),
        MQType::Redis {
            host_as_name_or_address: _,
            host_port: _,
        } => panic!("Redis not yet implemented"),
    }
}
