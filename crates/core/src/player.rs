//! Abstraksi pemain (SPEC §5.3): Human (UI lokal), Bot (per game, dengan
//! level), Remote (WebSocket, M9). Pemain LAN dan Nor-4 sama-sama Remote.

use serde::{Deserialize, Serialize};

use crate::game::{PlayerId, Session};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeatKind {
    Human,
    Bot { level: u8 },
    Remote,
}

pub trait Player: Send {
    fn kind(&self) -> SeatKind;

    /// Perintah teks yang dipilih bila giliran pemain ini. `None` berarti
    /// keputusan datang dari luar (UI lokal atau jaringan).
    fn decide(&mut self, session: &dyn Session, seat: PlayerId) -> Option<String>;
}

/// Pemain manusia di UI lokal: aksinya dikirim lewat `Match::act`.
pub struct Human;

impl Player for Human {
    fn kind(&self) -> SeatKind {
        SeatKind::Human
    }

    fn decide(&mut self, _: &dyn Session, _: PlayerId) -> Option<String> {
        None
    }
}
