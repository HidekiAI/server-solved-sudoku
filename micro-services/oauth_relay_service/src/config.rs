use dotenv::dotenv;
use std::{env, net::IpAddr, path::Path};

#[derive(Clone)]
pub enum DBType {
    SQLite {
        db_path: String,
    },
    PostgresSQL {
        host_address: IpAddr,
        host_port: u16,
    },
}

#[derive(Clone)]
pub enum MQType {
    Kafka {
        host_address: IpAddr,
        host_port: u16,
    },
    RabbitMQ {
        host_address: IpAddr,
        host_port: u16,
    },
    Redis {
        host_address: IpAddr,
        host_port: u16,
    },
    MongoDB {
        host_address: IpAddr,
        host_port: u16,
    },
}

#[derive(Clone)]
pub struct Config {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,

    pub db_connection: DBType,

    pub mq_connection: MQType,
}

impl Config {
    fn parse_to_ip_address(ip_address_or_hostname: &str) -> IpAddr {
        // if the str is not an ip address, do a name lookup, and translate it into an ip address
        match ip_address_or_hostname.parse() {
            Ok(ip) => ip,
            Err(_) => {
                // name lookup via (local, i.e. Docker NAT) DNS
                match dns_lookup::lookup_host(ip_address_or_hostname) {
                    Ok(ips) => {
                        // bias towards the IPv4 over IPv6 when possible
                        let ipv4s: Vec<IpAddr> =
                            ips.clone().into_iter().filter(|ip| ip.is_ipv4()).collect();
                        let ipv6s: Vec<IpAddr> =
                            ips.clone().into_iter().filter(|ip| ip.is_ipv6()).collect();
                        // if top/head IPv4 does not exist, pick the first IPv6
                        match ipv4s.first() {
                            Some(ip) => ip.clone(),
                            None => match ipv6s.first() {
                                Some(ip) => ip.clone(),
                                None => {
                                    panic!("No IP address found for '{}'", ip_address_or_hostname)
                                }
                            },
                        }
                    }
                    Err(e) => panic!("Error looking up host {}: '{}'", ip_address_or_hostname, e),
                }
            }
        }
    }

    fn make_db_type(db_type: &str) -> DBType {
        match db_type {
            "sqlite" => DBType::SQLite {
                db_path: env::var("DB_PATH").expect("DB_PATH must be set"),
            },
            "postgres" => DBType::PostgresSQL {
                host_address: Self::parse_to_ip_address(
                    env::var("DB_HOST_ADDRESS")
                        .expect("DB_HOST_ADDRESS must be set")
                        .as_str(),
                ),
                host_port: env::var("DB_HOST_PORT")
                    .expect("DB_HOST_PORT must be set")
                    .parse()
                    .expect("DB_HOST_PORT must be a valid port number"),
            },
            _ => panic!("DB_CONNECTION must be either 'sqlite' or 'postgres'"),
        }
    }

    fn make_mq_type(mq_type: &str) -> MQType {
        match mq_type {
            "kafka" => MQType::Kafka {
                host_address: Self::parse_to_ip_address(
                    env::var("MQ_HOST_ADDRESS")
                        .expect("MQ_HOST_ADDRESS must be set")
                        .as_str(),
                ),
                host_port: env::var("MQ_HOST_PORT")
                    .expect("MQ_HOST_PORT must be set")
                    .parse()
                    .expect("MQ_HOST_PORT must be a valid port number"),
            },
            "rabbitmq" => MQType::RabbitMQ {
                host_address: Self::parse_to_ip_address(
                    env::var("MQ_HOST_ADDRESS")
                        .expect("MQ_HOST_ADDRESS must be set")
                        .as_str(),
                ),
                host_port: env::var("MQ_HOST_PORT")
                    .expect("MQ_HOST_PORT must be set")
                    .parse()
                    .expect("MQ_HOST_PORT must be a valid port number"),
            },
            "redis" => MQType::Redis {
                host_address: Self::parse_to_ip_address(
                    env::var("MQ_HOST_ADDRESS")
                        .expect("MQ_HOST_ADDRESS must be set")
                        .as_str(),
                ),
                host_port: env::var("MQ_HOST_PORT")
                    .expect("MQ_HOST_PORT must be set")
                    .parse()
                    .expect("MQ_HOST_PORT must be a valid port number"),
            },
            "mongodb" => MQType::MongoDB {
                host_address: Self::parse_to_ip_address(
                    env::var("MQ_HOST_ADDRESS")
                        .expect("MQ_HOST_ADDRESS must be set")
                        .as_str(),
                ),
                host_port: env::var("MQ_HOST_PORT")
                    .expect("MQ_HOST_PORT must be set")
                    .parse()
                    .expect("MQ_HOST_PORT must be a valid port number"),
            },
            _ => panic!("MQ_CONNECTION must be either 'kafka', 'rabbitmq', 'redis', or 'mongodb'"),
        }
    }

    pub fn from_env_paths(absolute_path_with_filename: &Path) -> Self {
        // problem with source'ing the .env file is that if it is nested souce'ing, that file
        // has to be on the same directory or the path has to be absolute.
        // hence we'll push (remember) current path, change directory to the path where the .env file is
        // and then source it, then popd back to the original path
        let current_path = env::current_dir().expect("Cannot get the current directory");
        let env_file_path = absolute_path_with_filename.parent().unwrap();
        env::set_current_dir(env_file_path).expect("Cannot change directory to the .env file path");
        dotenv::from_path(absolute_path_with_filename).ok();
        env::set_current_dir(current_path)
            .expect("Cannot change directory back to the original path");
        Config {
            google_client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
                .expect("GOOGLE_CLIENT_SECRET must be set"),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI")
                .expect("GOOGLE_REDIRECT_URI must be set"),

            db_connection: Self::make_db_type(
                env::var("DB_CONNECTION")
                    .expect("DB_CONNECTION must be set")
                    .as_str(),
            ),

            mq_connection: Self::make_mq_type(
                env::var("MQ_CONNECTION")
                    .expect("MQ_CONNECTION must be set")
                    .as_str(),
            ),
        }
    }
}
