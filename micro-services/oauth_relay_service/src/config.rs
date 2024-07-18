use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::{self, Read},
    net::IpAddr,
    path::Path,
    str::FromStr,
};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum HostType {
    HostAsIP(IpAddr),
    HostAsName(String),
}
impl std::fmt::Debug for HostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostType::HostAsIP(ip) => write!(f, "HostAsIP({})", ip),
            HostType::HostAsName(name) => write!(f, "HostAsName({})", name),
        }
    }
}
impl std::fmt::Display for HostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostType::HostAsIP(ip) => write!(f, "{}", ip),
            HostType::HostAsName(name) => write!(f, "{}", name),
        }
    }
}
// for as_str()
//impl AsRef<str> for HostType {
//    fn as_ref(&self) -> &str {
//        let s = match self {
//            HostType::HostAsIP(ip) => ip.clone().to_string(),
//            HostType::HostAsName(name) => name.clone(),
//        };
//        s.as_str()
//    }
//}
// from &str (Into<>) in which if IPAddres is any() or loopback, assum string passed is hostname
impl Into<HostType> for &str {
    fn into(self) -> HostType {
        // see if we can parse the string as IpAddr
        match self.parse::<IpAddr>() {
            Ok(ip) => {
                // if ip is loopback and/or local host and/or unspecified, assume it's a hostname
                if ip.is_loopback() || ip.is_unspecified() {
                    HostType::HostAsName(self.to_string())
                } else {
                    HostType::HostAsIP(ip)
                }
            }
            Err(_) => HostType::HostAsName(self.to_string()),
        }
    }
}
impl FromStr for HostType {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<IpAddr>() {
            Ok(ip) => {
                // if ip is loopback and/or local host and/or unspecified, assume it's a hostname
                if ip.is_loopback() || ip.is_unspecified() {
                    Ok(HostType::HostAsName(s.to_string()))
                } else {
                    Ok(HostType::HostAsIP(ip))
                }
            }
            Err(_) => Ok(HostType::HostAsName(s.to_string())),
        }
    }
}
// Making sure HostType::from<IpAddr> and HostType::from<String> works
impl From<IpAddr> for HostType {
    fn from(ip: IpAddr) -> Self {
        HostType::HostAsIP(ip)
    }
}
impl From<String> for HostType {
    fn from(name: String) -> Self {
        match name.parse::<IpAddr>() {
            Ok(ip) => {
                // if ip is loopback and/or local host and/or unspecified, assume it's a hostname
                if ip.is_loopback() || ip.is_unspecified() {
                    HostType::HostAsName(name)
                } else {
                    HostType::HostAsIP(ip)
                }
            }
            Err(_) => HostType::HostAsName(name),
        }
    }
}
//std::convert::From<&IpAddr>
impl From<&IpAddr> for HostType {
    fn from(ip: &IpAddr) -> Self {
        HostType::HostAsIP(*ip)
    }
}

#[derive(Clone)]
pub enum DBType {
    SQLite {
        db_path: String,
    },
    PostgresSQL {
        host_as_name_or_address: HostType,
        host_port: u16,
    },
}

#[derive(Clone)]
pub enum MQType {
    Kafka {
        host_as_name_or_address: HostType,
        host_port: u16,
    },
    RabbitMQ {
        host_as_name_or_address: HostType,
        host_port: u16,
    },
    Redis {
        host_as_name_or_address: HostType,
        host_port: u16,
    },
    MongoDB {
        host_as_name_or_address: HostType,
        host_port: u16,
    },
}

#[derive(Clone)]
pub struct Config {
    pub rest_port: u16,

    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,

    pub db_connection: DBType,

    pub mq_connection: MQType,
}

impl Config {
    fn parse_to_ip_address(ip_address_or_hostname: &str) -> HostType {
        // if the str is not an ip address, do a name lookup, and translate it into an ip address
        let ret_resolved = match ip_address_or_hostname.parse() {
            Ok(ip) => ip,
            Err(_) => {
                // name lookup via (local, i.e. Docker NAT) DNS (note that dns-lookup DOES NOT work on Docker)
                match dns_lookup::lookup_host(ip_address_or_hostname) {
                    Ok(ips) => {
                        // bias towards the IPv4 over IPv6 when possible
                        let ipv4s: Vec<IpAddr> =
                            ips.clone().into_iter().filter(|ip| ip.is_ipv4()).collect();
                        let ipv6s: Vec<IpAddr> =
                            ips.clone().into_iter().filter(|ip| ip.is_ipv6()).collect();

                        // if top/head IPv4 does not exist, pick the first IPv6
                        match ipv4s.first() {
                            Some(ip) => HostType::from(ip),
                            None => match ipv6s.first() {
                                Some(ip) => HostType::from(ip),
                                None => {
                                    println!(
                                        "No IP address found for '{}'",
                                        ip_address_or_hostname
                                    );
                                    HostType::HostAsName(ip_address_or_hostname.to_string())
                                }
                            },
                        }
                    }
                    Err(e) => {
                        println!("Error looking up host {}: '{}'", ip_address_or_hostname, e);
                        HostType::HostAsName(ip_address_or_hostname.to_string())
                    }
                }
            }
        };
        println!(
            "Resolved '{}' to '{:?}'",
            ip_address_or_hostname, ret_resolved
        );
        ret_resolved
    }

    fn make_db_type(db_type: &str) -> DBType {
        match db_type {
            "sqlite" => DBType::SQLite {
                db_path: env::var("DB_STORAGE_PATH").expect("DB_STORAGE_PATH must be set"),
            },
            "postgres" => DBType::PostgresSQL {
                host_as_name_or_address: Self::parse_to_ip_address(
                    env::var("DB_HOST").expect("DB_HOST must be set").as_str(),
                ),
                host_port: env::var("DB_PORT")
                    .expect("DB_PORT must be set")
                    .parse()
                    .expect("DB_PORT must be a valid port number"),
            },
            _ => panic!("DB_CONNECTION must be either 'sqlite' or 'postgres'"),
        }
    }

    fn make_mq_type(mq_type: &str) -> MQType {
        match mq_type {
            "kafka" => MQType::Kafka {
                host_as_name_or_address: Self::parse_to_ip_address(
                    env::var("BROKER_HOST")
                        .expect("BROKER_HOST must be set")
                        .as_str(),
                ),
                host_port: env::var("BROKER_PORT")
                    .expect("BROKER_PORT must be set")
                    .parse()
                    .expect("BROKER_PORT must be a valid port number"),
            },
            "rabbitmq" => MQType::RabbitMQ {
                host_as_name_or_address: Self::parse_to_ip_address(
                    env::var("BROKER_HOST")
                        .expect("BROKER_HOST must be set")
                        .as_str(),
                ),
                host_port: env::var("BROKER_PORT")
                    .expect("BROKER_PORT must be set")
                    .parse()
                    .expect("BROKER_PORT must be a valid port number"),
            },
            "redis" => MQType::Redis {
                host_as_name_or_address: Self::parse_to_ip_address(
                    env::var("BROKER_HOST")
                        .expect("BROKER_HOST must be set")
                        .as_str(),
                ),
                host_port: env::var("BROKER_PORT")
                    .expect("BROKER_PORT must be set")
                    .parse()
                    .expect("BROKER_PORT must be a valid port number"),
            },
            "mongodb" => MQType::MongoDB {
                host_as_name_or_address: Self::parse_to_ip_address(
                    env::var("BROKER_HOST")
                        .expect("BROKER_HOST must be set")
                        .as_str(),
                ),
                host_port: env::var("BROKER_PORT")
                    .expect("BROKER_PORT must be set")
                    .parse()
                    .expect("BROKER_PORT must be a valid port number"),
            },
            _ => panic!("MQ_CONNECTION must be either 'kafka', 'rabbitmq', 'redis', or 'mongodb'"),
        }
    }

    // ASSUMES that '.env' file is in the same directory as the executable and/or the
    // calling host already have set the dir where .evn file resides
    pub fn from_local_env_file() -> Self {
        // dump current dir so we know which .env it loaded:
        println!("Loading '.env' file from {:?}", env::current_dir());

        //dotenv::from_path(absolute_path_with_filename).ok();
        //dotenv::from_filename(".env").ok();
        dotenvy::dotenv().unwrap(); // assumes file is ".env" on current directory
        Config {
            rest_port: env::var("REST_PORT")
                .expect("REST_PORT must be set")
                .parse()
                .expect("REST_PORT must be a valid port number"),

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
