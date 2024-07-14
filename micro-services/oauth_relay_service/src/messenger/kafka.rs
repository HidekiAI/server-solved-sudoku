pub mod data_kafka;
pub mod main_kafka;

use crate::{config::*, data::*};
use data_kafka::*;
use main_kafka::*;
use rdkafka::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type TMQConnection_kafka = Client; // Currently using Kafka as the MQ
pub type TMQConnectionLock_kafka = Arc<Mutex<TMQConnection_kafka>>;

pub(crate) fn open_mq_connection_kafka(
    host_address: std::net::IpAddr,
    host_port: u16,
) -> Arc<Mutex<Client>> {
    todo!()
}

pub(crate) fn get_token_kafka<TFn>(
    mq_lock: &TMQConnectionLock_kafka,
    lhs_state_token: &str,
    fn_equ_op: TFn,
) -> Option<TokenData>
where
    TFn: Fn(&str /*lhs_state_token*/, TokenData /*rhs*/) -> bool, // compare lhs_state_token == rhs:TokenData.state_token
{
    todo!()
}

pub(crate) async fn post_new_login_kafka(
    db_connection: &TMQConnectionLock_kafka,
    token_data: &TokenData,
) -> Result<(), anyhow::Error> {
    todo!()
}
