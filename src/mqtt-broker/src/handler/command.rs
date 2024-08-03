use super::mqtt::MqttService;
use crate::handler::response::response_packet_mqtt_distinct_by_reason;
use crate::handler::{cache_manager::CacheManager, response::response_packet_mqtt_connect_fail};
use crate::server::connection::NetworkConnection;
use crate::server::connection_manager::ConnectionManager;
use crate::subscribe::subscribe_manager::SubscribeManager;
use clients::poll::ClientPool;
use common_base::log::info;
use protocol::mqtt::common::{
    is_mqtt3, is_mqtt4, is_mqtt5, ConnectReturnCode, DisconnectReasonCode, MQTTPacket,
    MQTTProtocol, QoS,
};
use std::net::SocketAddr;
use std::sync::Arc;
use storage_adapter::storage::StorageAdapter;

// S: message storage adapter
#[derive(Clone)]
pub struct Command<S> {
    mqtt3_service: MqttService<S>,
    mqtt4_service: MqttService<S>,
    mqtt5_service: MqttService<S>,
    metadata_cache: Arc<CacheManager>,
}

impl<S> Command<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        cache_manager: Arc<CacheManager>,
        message_storage_adapter: Arc<S>,
        sucscribe_manager: Arc<SubscribeManager>,
        client_poll: Arc<ClientPool>,
        connnection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let mqtt3_service = MqttService::new(
            MQTTProtocol::MQTT3,
            cache_manager.clone(),
            connnection_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
        );
        let mqtt4_service = MqttService::new(
            MQTTProtocol::MQTT4,
            cache_manager.clone(),
            connnection_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
        );
        let mqtt5_service = MqttService::new(
            MQTTProtocol::MQTT5,
            cache_manager.clone(),
            connnection_manager.clone(),
            message_storage_adapter.clone(),
            sucscribe_manager.clone(),
            client_poll.clone(),
        );
        return Command {
            mqtt3_service,
            mqtt4_service,
            mqtt5_service,
            metadata_cache: cache_manager,
        };
    }

    pub async fn apply(
        &mut self,
        connect_manager: Arc<ConnectionManager>,
        tcp_connection: NetworkConnection,
        addr: SocketAddr,
        packet: MQTTPacket,
    ) -> Option<MQTTPacket> {
        if !self.check_login_status(tcp_connection.connection_id).await {
            return Some(response_packet_mqtt_distinct_by_reason(
                &MQTTProtocol::MQTT5,
                Some(DisconnectReasonCode::NotAuthorized),
            ));
        }

        match packet {
            MQTTPacket::Connect(
                protocol_version,
                connect,
                properties,
                last_will,
                last_will_peoperties,
                login,
            ) => {
                connect_manager
                    .set_connect_protocol(tcp_connection.connection_id, protocol_version);

                let resp_pkg = if is_mqtt3(protocol_version) {
                    Some(
                        self.mqtt3_service
                            .connect(
                                tcp_connection.connection_id,
                                connect,
                                properties,
                                last_will,
                                last_will_peoperties,
                                login,
                                addr,
                            )
                            .await,
                    )
                } else if is_mqtt4(protocol_version) {
                    Some(
                        self.mqtt4_service
                            .connect(
                                tcp_connection.connection_id,
                                connect,
                                properties,
                                last_will,
                                last_will_peoperties,
                                login,
                                addr,
                            )
                            .await,
                    )
                } else if is_mqtt5(protocol_version) {
                    Some(
                        self.mqtt5_service
                            .connect(
                                tcp_connection.connection_id,
                                connect,
                                properties,
                                last_will,
                                last_will_peoperties,
                                login,
                                addr,
                            )
                            .await,
                    )
                } else {
                    return Some(response_packet_mqtt_connect_fail(
                        &MQTTProtocol::MQTT4,
                        ConnectReturnCode::UnsupportedProtocolVersion,
                        &None,
                        None,
                    ));
                };

                let ack_pkg = resp_pkg.unwrap();
                if let MQTTPacket::ConnAck(conn_ack, _) = ack_pkg.clone() {
                    if conn_ack.code == ConnectReturnCode::Success {
                        self.metadata_cache
                            .login_success(tcp_connection.connection_id);
                        info(format!(
                            "connect [{}] login success",
                            tcp_connection.connection_id
                        ));
                    }
                }
                return Some(ack_pkg);
            }

            MQTTPacket::Publish(publish, publish_properties) => {
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish(tcp_connection.connection_id, publish, publish_properties)
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish(tcp_connection.connection_id, publish, publish_properties)
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish(tcp_connection.connection_id, publish, publish_properties)
                        .await;
                }
            }

            MQTTPacket::PubRec(pub_rec, pub_rec_properties) => {
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                        .await;
                }
                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                        .await;
                }
                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish_rec(tcp_connection.connection_id, pub_rec, pub_rec_properties)
                        .await;
                }
            }

            MQTTPacket::PubComp(pub_comp, pub_comp_properties) => {
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish_comp(tcp_connection.connection_id, pub_comp, pub_comp_properties)
                        .await;
                }
            }

            MQTTPacket::PubRel(pub_rel, pub_rel_properties) => {
                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                            .await,
                    );
                }
                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .publish_rel(tcp_connection.connection_id, pub_rel, pub_rel_properties)
                            .await,
                    );
                }
            }

            MQTTPacket::PubAck(pub_ack, pub_ack_properties) => {
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .publish_ack(tcp_connection.connection_id, pub_ack, pub_ack_properties)
                        .await;
                }
                return None;
            }

            MQTTPacket::Subscribe(subscribe, subscribe_properties) => {
                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .subscribe(
                                tcp_connection.connection_id,
                                subscribe,
                                subscribe_properties,
                            )
                            .await,
                    );
                }
                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .subscribe(
                                tcp_connection.connection_id,
                                subscribe,
                                subscribe_properties,
                            )
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .subscribe(
                                tcp_connection.connection_id,
                                subscribe,
                                subscribe_properties,
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::PingReq(ping) => {
                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .ping(tcp_connection.connection_id, ping)
                            .await,
                    );
                }

                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .ping(tcp_connection.connection_id, ping)
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .ping(tcp_connection.connection_id, ping)
                            .await,
                    );
                }
            }

            MQTTPacket::Unsubscribe(unsubscribe, unsubscribe_properties) => {
                if tcp_connection.is_mqtt3() {
                    return Some(
                        self.mqtt3_service
                            .un_subscribe(
                                tcp_connection.connection_id,
                                unsubscribe,
                                unsubscribe_properties,
                            )
                            .await,
                    );
                }

                if tcp_connection.is_mqtt4() {
                    return Some(
                        self.mqtt4_service
                            .un_subscribe(
                                tcp_connection.connection_id,
                                unsubscribe,
                                unsubscribe_properties,
                            )
                            .await,
                    );
                }

                if tcp_connection.is_mqtt5() {
                    return Some(
                        self.mqtt5_service
                            .un_subscribe(
                                tcp_connection.connection_id,
                                unsubscribe,
                                unsubscribe_properties,
                            )
                            .await,
                    );
                }
            }

            MQTTPacket::Disconnect(disconnect, disconnect_properties) => {
                if tcp_connection.is_mqtt3() {
                    return self
                        .mqtt3_service
                        .disconnect(
                            tcp_connection.connection_id,
                            disconnect,
                            disconnect_properties,
                        )
                        .await;
                }

                if tcp_connection.is_mqtt4() {
                    return self
                        .mqtt4_service
                        .disconnect(
                            tcp_connection.connection_id,
                            disconnect,
                            disconnect_properties,
                        )
                        .await;
                }

                if tcp_connection.is_mqtt5() {
                    return self
                        .mqtt5_service
                        .disconnect(
                            tcp_connection.connection_id,
                            disconnect,
                            disconnect_properties,
                        )
                        .await;
                }
            }

            _ => {
                return Some(response_packet_mqtt_connect_fail(
                    &MQTTProtocol::MQTT5,
                    ConnectReturnCode::MalformedPacket,
                    &None,
                    None,
                ));
            }
        }
        return Some(response_packet_mqtt_connect_fail(
            &MQTTProtocol::MQTT5,
            ConnectReturnCode::UnsupportedProtocolVersion,
            &None,
            None,
        ));
    }

    pub async fn check_login_status(&self, connection_id: u64) -> bool {
        return self.metadata_cache.is_login(connection_id);
    }
}
