use std::{collections::HashMap, sync::Arc};
use tokio::{sync::Mutex, time::Duration};
use uuid::Uuid;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine,
        setting_engine::SettingEngine, APIBuilder, API,
    },
    ice_transport::{ice_candidate::RTCIceCandidateInit, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
    },
    Error,
};

use crate::Room;

pub struct SFU {
    rooms: Mutex<HashMap<Uuid, Room>>,
    api: API,
    config: RTCConfiguration,
}

impl SFU {
    pub fn default() -> Result<Self, Error> {
        let setting_engine = SettingEngine::default();

        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs().unwrap();
        let mut registry = Registry::new();

        registry = register_default_interceptors(registry, &mut media_engine).unwrap();

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .with_interceptor_registry(registry)
            .build();

        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        Ok(SFU {
            rooms: Mutex::new(HashMap::new()),
            api,
            config,
        })
    }

    pub fn new(api: API, config: RTCConfiguration) -> Result<SFU, Error> {
        Ok(SFU {
            rooms: Mutex::new(HashMap::new()),
            api,
            config,
        })
    }

    pub async fn clean_empty_rooms(&self, idle_timeout: Duration) {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            let mut rooms = self.rooms.lock().await;
            let mut empty_rooms = Vec::new();

            for (room_name, room) in &*rooms {
                if room.participants.is_empty() && room.last_activity.elapsed() >= idle_timeout {
                    empty_rooms.push(room_name.clone());
                }
            }

            for room_name in empty_rooms {
                rooms.remove(&room_name);
            }
        }
    }

    pub async fn handle_offer(
        &self,
        offer: RTCSessionDescription,
        ice_candidates: Vec<RTCIceCandidateInit>,
    ) -> Result<RTCSessionDescription, Error> {
        let peer_connection = Arc::new(self.api.new_peer_connection(self.config.clone()).await?);

        for ice_candidate in ice_candidates {
            peer_connection
                .add_ice_candidate(ice_candidate)
                .await
                .unwrap();
        }

        peer_connection.set_remote_description(offer).await.unwrap();
        let answer = peer_connection.create_answer(None).await.unwrap();
        peer_connection
            .set_local_description(answer.clone())
            .await
            .unwrap();

        Ok(answer)
    }
}
