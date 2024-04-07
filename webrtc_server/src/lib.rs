pub mod sfu;

use std::{collections::HashMap, sync::Arc};

use uuid::Uuid;
use webrtc::peer_connection::RTCPeerConnection;

struct Room {
    last_activity: std::time::Instant,
    participants: HashMap<Uuid, Arc<RTCPeerConnection>>,
    password: String,
}
