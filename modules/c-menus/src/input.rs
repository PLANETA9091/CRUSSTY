//! Packet-level input: decode the serverbound play packets that drive menus,
//! map connections to players, and queue events for the main thread.
//!
//! Targets Paper 26.3 (protocol 777):
//!   0x2E punch (swing, no fields)      0x01 attack (entity, VarInt)
//!   0x43 use item (RMB air)            0x42 use item on (RMB block)
//!   0x1A interact (RMB entity)         0x29 player action (status 6 = swap F)
//!   0x2B player input (flags u8)       0x36 set held item (slot Short)
//!   0x07 chat command (String)         0x32 resource pack response (in)
//! Clientbound play login = 0x32 (carries the player UUID).

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

// serverbound play ids (protocol 777)
pub const SB_ATTACK: i32 = 0x01;
pub const SB_CHAT_COMMAND: i32 = 0x07;
pub const SB_INTERACT: i32 = 0x1A;
pub const SB_PLAYER_ACTION: i32 = 0x29;
pub const SB_PLAYER_INPUT: i32 = 0x2B;
pub const SB_SET_HELD_ITEM: i32 = 0x36;
pub const SB_PUNCH: i32 = 0x2E;
pub const SB_USE_ITEM_ON: i32 = 0x42;
pub const SB_USE_ITEM: i32 = 0x43;
pub const SB_PACK_RESPONSE: i32 = 0x32;

// clientbound play id
pub const CB_LOGIN: i32 = 0x32;

pub const PLAY_STATE: u8 = 3;

// pack response results (1.20.3+)
pub const PACK_ACCEPTED: i32 = 3;
pub const PACK_DOWNLOADED: i32 = 4;
#[allow(dead_code)]
pub const PACK_DECLINED: i32 = 1;

#[derive(Clone, Debug, PartialEq)]
pub enum Btn {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub enum InputEvent {
    /// Player left/right clicked; yaw/pitch from the packet when present.
    Click {
        uuid: String,
        btn: Btn,
        yaw: Option<f32>,
        pitch: Option<f32>,
    },
    /// F key (offhand swap) pressed.
    Back {
        uuid: String,
    },
    Sneak {
        uuid: String,
        down: bool,
    },
    /// Hotbar slot changed.
    Scroll {
        uuid: String,
        slot: i8,
        dir: i8,
    },
    /// /cmenus ... command.
    Command {
        uuid: String,
        line: String,
    },
    /// Resource pack status changed.
    PackStatus {
        uuid: String,
        status: i32,
    },
}

struct ConnState {
    uuid: String,
    last_slot: i8,
}

pub struct InputRouter {
    conns: Mutex<HashMap<u64, ConnState>>,
    queue: Mutex<VecDeque<InputEvent>>,
    dropped: AtomicU64,
}

impl InputRouter {
    pub fn new() -> Self {
        InputRouter {
            conns: Mutex::new(HashMap::new()),
            queue: Mutex::new(VecDeque::new()),
            dropped: AtomicU64::new(0),
        }
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Outbound play login (0x32): bind conn_id -> player uuid.
    pub fn on_outbound_login(&self, conn: u64, payload: &[u8]) {
        if let Some(uuid) = decode_login_uuid(payload) {
            let mut m = self.conns.lock().unwrap();
            let st = m.entry(conn).or_insert_with(|| ConnState {
                uuid: uuid.clone(),
                last_slot: 0,
            });
            st.uuid = uuid;
        }
    }

    pub fn on_disconnect(&self, conn: u64) {
        self.conns.lock().unwrap().remove(&conn);
    }

    pub fn uuid_of(&self, conn: u64) -> Option<String> {
        self.conns.lock().unwrap().get(&conn).map(|c| c.uuid.clone())
    }

    pub fn uuids(&self) -> Vec<String> {
        self.conns.lock().unwrap().values().map(|c| c.uuid.clone()).collect()
    }

    /// Push an event for the main thread.
    pub fn push(&self, ev: InputEvent) {
        self.queue.lock().unwrap().push_back(ev);
    }

    /// Drain all queued events (main thread).
    pub fn drain(&self) -> Vec<InputEvent> {
        let mut q = self.queue.lock().unwrap();
        let out: Vec<_> = q.drain(..).collect();
        out
    }

    /// Inbound play packet while `session_open` decides the verdict.
    /// Returns NET verdict: 0 pass, 1 drop.
    pub fn on_inbound(
        &self,
        conn: u64,
        packet_id: i32,
        payload: &[u8],
        session_open: bool,
    ) -> i32 {
        let Some(uuid) = self.uuid_of(conn) else {
            return 0;
        };
        match packet_id {
            SB_PUNCH => {
                if session_open {
                    self.push(InputEvent::Click { uuid, btn: Btn::Left, yaw: None, pitch: None });
                    self.note_drop();
                    1
                } else {
                    0
                }
            }
            SB_ATTACK => {
                if session_open {
                    self.push(InputEvent::Click { uuid, btn: Btn::Left, yaw: None, pitch: None });
                    self.note_drop();
                    1
                } else {
                    0
                }
            }
            SB_USE_ITEM => {
                if session_open {
                    let (yaw, pitch) = decode_use_item(payload);
                    self.push(InputEvent::Click { uuid, btn: Btn::Right, yaw, pitch });
                    self.note_drop();
                    1
                } else {
                    0
                }
            }
            SB_USE_ITEM_ON | SB_INTERACT => {
                if session_open {
                    self.push(InputEvent::Click { uuid, btn: Btn::Right, yaw: None, pitch: None });
                    self.note_drop();
                    1
                } else {
                    0
                }
            }
            SB_PLAYER_ACTION => {
                // status 6 = swap item with offhand (F key)
                if session_open && decode_varint(payload).map(|(v, _)| v) == Some(6) {
                    self.push(InputEvent::Back { uuid });
                    self.note_drop();
                    1
                } else {
                    0
                }
            }
            SB_PLAYER_INPUT => {
                if let Some(&flags) = payload.first() {
                    let down = flags & 0x20 != 0;
                    self.push(InputEvent::Sneak { uuid, down });
                }
                0
            }
            SB_SET_HELD_ITEM => {
                if payload.len() >= 2 {
                    let slot = payload[1] as i8;
                    let dir = {
                        let mut m = self.conns.lock().unwrap();
                        let st = m.entry(conn).or_insert_with(|| ConnState { uuid: uuid.clone(), last_slot: slot });
                        let d = if slot > st.last_slot { 1 } else if slot < st.last_slot { -1 } else { 0 };
                        st.last_slot = slot;
                        d
                    };
                    if session_open && dir != 0 {
                        self.push(InputEvent::Scroll { uuid, slot, dir });
                        // pass through: the client already switched its hotbar
                    }
                }
                0
            }
            SB_CHAT_COMMAND => {
                if let Some(cmd) = decode_string(payload) {
                    let root = crate::command_root();
                    if cmd.len() == root.len() || cmd.starts_with(&format!("{root} ")) {
                        self.push(InputEvent::Command { uuid, line: cmd });
                        self.note_drop();
                        return 1;
                    }
                }
                0
            }
            SB_PACK_RESPONSE => {
                if let Some((uuid_raw, status)) = decode_pack_response(payload) {
                    let u = format_uuid(uuid_raw);
                    self.push(InputEvent::PackStatus { uuid: u, status });
                }
                0
            }
            _ => 0,
        }
    }

    fn note_drop(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }
}

// ---------------------------------------------------------------------------
// Wire decoding
// ---------------------------------------------------------------------------

pub fn decode_varint(buf: &[u8]) -> Option<(i32, usize)> {
    let mut value: i32 = 0;
    for i in 0..5 {
        let b = *buf.get(i)?;
        value |= ((b & 0x7F) as i32) << (i * 7);
        if b & 0x80 == 0 {
            return Some((value, i + 1));
        }
    }
    None
}

pub fn decode_login_uuid(payload: &[u8]) -> Option<String> {
    let (_, n) = decode_varint(payload)?;
    if payload.len() < n + 16 {
        return None;
    }
    let raw: [u8; 16] = payload[n..n + 16].try_into().ok()?;
    Some(format_uuid(raw))
}

pub fn format_uuid(raw: [u8; 16]) -> String {
    let hex = |b: u8| format!("{b:02x}");
    let s: String = raw.iter().map(|b| hex(*b)).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &s[0..8],
        &s[8..12],
        &s[12..16],
        &s[16..20],
        &s[20..32]
    )
}

fn f32_be(buf: &[u8], off: usize) -> Option<f32> {
    if buf.len() < off + 4 {
        return None;
    }
    Some(f32::from_be_bytes(buf[off..off + 4].try_into().ok()?))
}

/// use_item: hand VarInt, sequence VarInt, yaw Float, pitch Float.
fn decode_use_item(payload: &[u8]) -> (Option<f32>, Option<f32>) {
    let (_, n1) = match decode_varint(payload) {
        Some(x) => x,
        None => return (None, None),
    };
    let (_, n2) = match decode_varint(&payload[n1..]) {
        Some(x) => x,
        None => return (None, None),
    };
    let base = n1 + n2;
    let yaw = f32_be(payload, base);
    let pitch = f32_be(payload, base + 4);
    (yaw, pitch)
}

/// First VarInt-prefixed string in the payload.
pub fn decode_string(payload: &[u8]) -> Option<String> {
    let (len, n) = decode_varint(payload)?;
    if len < 0 {
        return None;
    }
    let end = n + len as usize;
    if payload.len() < end {
        return None;
    }
    String::from_utf8(payload[n..end].to_vec()).ok()
}

/// pack response: UUID(16) + status VarInt.
fn decode_pack_response(payload: &[u8]) -> Option<([u8; 16], i32)> {
    if payload.len() < 16 {
        return None;
    }
    let raw: [u8; 16] = payload[..16].try_into().ok()?;
    let (status, _) = decode_varint(&payload[16..])?;
    Some((raw, status))
}

/// Extract packet id (first VarInt) and remaining payload.
pub fn split_packet(frame: &[u8]) -> Option<(i32, &[u8])> {
    let (id, n) = decode_varint(frame)?;
    Some((id, &frame[n..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint() {
        assert_eq!(decode_varint(&[0x00]), Some((0, 1)));
        assert_eq!(decode_varint(&[0x7F]), Some((127, 1)));
        assert_eq!(decode_varint(&[0x80, 0x01]), Some((128, 2)));
        assert_eq!(decode_varint(&[0xFF, 0x01]), Some((255, 2)));
        assert_eq!(decode_varint(&[0xDD, 0xC7, 0x01]), Some((25565, 3)));
        assert_eq!(decode_varint(&[]), None);
    }

    #[test]
    fn login_uuid() {
        // entity id 42 (0x2A), then uuid 0..16
        let mut p = vec![0x2A];
        p.extend_from_slice(&[0u8; 16]);
        let u = decode_login_uuid(&p).expect("uuid");
        assert_eq!(u, "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn uuid_format() {
        let raw = [
            0x06, 0x9a, 0xec, 0xdc, 0xae, 0x40, 0x43, 0x9d, 0xb9, 0x91, 0x65, 0xc4, 0x3e, 0x39,
            0x9f, 0xac,
        ];
        assert_eq!(format_uuid(raw), "069aecdc-ae40-439d-b991-65c43e399fac");
    }

    #[test]
    fn router_click_flow() {
        let r = InputRouter::new();
        // login binds conn -> uuid
        let mut login = vec![0x2A];
        login.extend_from_slice(&[1u8; 16]);
        r.on_outbound_login(7, &login);
        assert_eq!(r.uuid_of(7).as_deref(), Some("01010101-0101-0101-0101-010101010101"));

        // no session -> pass
        assert_eq!(r.on_inbound(7, SB_PUNCH, &[], false), 0);
        // session open -> drop + event
        assert_eq!(r.on_inbound(7, SB_PUNCH, &[], true), 1);
        assert_eq!(r.on_inbound(7, SB_USE_ITEM, &[0x00, 0x00, 0x3f, 0x80, 0, 0, 0, 0, 0, 0], true), 1);
        // F key swap
        assert_eq!(r.on_inbound(7, SB_PLAYER_ACTION, &[6, 0, 0, 0, 0, 0, 0, 0, 0, 1], true), 1);
        let evs = r.drain();
        assert_eq!(evs.len(), 3);
        match &evs[0] {
            InputEvent::Click { uuid, btn, .. } => {
                assert_eq!(uuid, "01010101-0101-0101-0101-010101010101");
                assert!(matches!(btn, Btn::Left));
            }
            _ => panic!("click expected"),
        }
        match &evs[1] {
            InputEvent::Click { btn, yaw, .. } => {
                assert!(matches!(btn, Btn::Right));
                assert_eq!(*yaw, Some(1.0));
            }
            _ => panic!("click expected"),
        }
        match &evs[2] {
            InputEvent::Back { .. } => {}
            _ => panic!("back expected"),
        }
        assert_eq!(r.dropped_count(), 3);
    }

    #[test]
    fn router_scroll_slots() {
        let r = InputRouter::new();
        let mut login = vec![0x2A];
        login.extend_from_slice(&[2u8; 16]);
        r.on_outbound_login(9, &login);
        // slot change: short 0x0003
        assert_eq!(r.on_inbound(9, SB_SET_HELD_ITEM, &[0x00, 3], true), 0);
        assert_eq!(r.on_inbound(9, SB_SET_HELD_ITEM, &[0x00, 4], true), 0);
        let evs = r.drain();
        match &evs[1] {
            InputEvent::Scroll { dir, slot, .. } => {
                assert_eq!(*dir, 1);
                assert_eq!(*slot, 4);
            }
            _ => panic!("scroll expected"),
        }
    }

    #[test]
    fn router_command() {
        let r = InputRouter::new();
        let mut login = vec![0x2A];
        login.extend_from_slice(&[3u8; 16]);
        r.on_outbound_login(11, &login);
        crate::set_command_root("cmenus");
        let mut p = vec![];
        let s = b"cmenus open item_view";
        p.extend_from_slice(&[s.len() as u8]);
        p.extend_from_slice(s);
        assert_eq!(r.on_inbound(11, SB_CHAT_COMMAND, &p, false), 1);
        assert!(matches!(r.drain()[0], InputEvent::Command { .. }));
        // other commands pass through
        let mut p = vec![];
        p.push(4);
        p.extend_from_slice(b"help");
        assert_eq!(r.on_inbound(11, SB_CHAT_COMMAND, &p, false), 0);
    }
}
