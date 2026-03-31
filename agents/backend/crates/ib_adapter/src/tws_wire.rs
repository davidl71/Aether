//! Minimal TWS protobuf wire spike: message IDs and frame layout.
//!
//! Documents/prototypes the TWS API protobuf wire format for a future Rust implementation.
//! No I/O or protobuf codegen; see `docs/platform/TWS_PROTOBUF_WIRE_SPIKE.md` and
//! `docs/platform/TWS_API_PROTOBUF_LEARNINGS_FROM_SIBLING_REPO.md`.

/// Offset added to base message IDs when sending protobuf (from tws-api).
/// Wire msg_id = base_id + PROTOBUF_MSG_ID_OFFSET.
pub const PROTOBUF_MSG_ID_OFFSET: u32 = 1000;

// --- Minimum server versions for protobuf (tws-api server_versions.py) ---
/// Base: protobuf allowed at all.
pub const MIN_SERVER_VER_PROTOBUF: i32 = 201;
/// IBKR wire threshold for place/cancel order messages (TWS `MIN_SERVER_VER` constant; not an Aether execution feature).
pub const MIN_SERVER_VER_PLACE_ORDER: i32 = 203;
/// Contract details.
pub const MIN_SERVER_VER_CONTRACT_DATA: i32 = 205;
/// Streaming market data.
pub const MIN_SERVER_VER_MKT_DATA: i32 = 206;
/// Positions request.
pub const MIN_SERVER_VER_POSITIONS: i32 = 207;
/// Historical bars.
pub const MIN_SERVER_VER_HISTORICAL: i32 = 208;
/// Scanner.
pub const MIN_SERVER_VER_SCANNER: i32 = 210;

/// One TWS protobuf wire frame: length (BE u32) + msg_id (BE u32) + payload.
/// On the wire: `[length: 4 bytes BE][msg_id: 4 bytes BE][payload]`;
/// length = 4 + payload.len().
#[derive(Debug, Clone)]
pub struct TwsProtoFrame {
    pub msg_id: u32,
    pub payload: Vec<u8>,
}

impl TwsProtoFrame {
    /// Wire length (4 + payload.len()) as used in the length prefix.
    #[inline]
    pub fn wire_payload_len(&self) -> u32 {
        4u32.saturating_add(self.payload.len() as u32)
    }

    /// Encode to the TWS wire format: 4-byte big-endian length, then `msg_id`, then `payload` bytes.
    pub fn encode(&self) -> Vec<u8> {
        let len = self.wire_payload_len();
        let mut out = Vec::with_capacity(4 + 4 + self.payload.len());
        out.extend_from_slice(&len.to_be_bytes());
        out.extend_from_slice(&self.msg_id.to_be_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    /// Decode from the TWS wire format. Caller must provide the full message (length prefix + payload).
    /// Returns (msg_id, payload slice) or None if buffer too short.
    pub fn decode(buf: &[u8]) -> Option<(u32, &[u8])> {
        if buf.len() < 8 {
            return None;
        }
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let msg_id = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let payload_len = len.saturating_sub(4) as usize;
        if buf.len() < 8 + payload_len {
            return None;
        }
        Some((msg_id, &buf[8..8 + payload_len]))
    }
}
