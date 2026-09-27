//! Kontrak game giliran (SPEC §5.2).
//!
//! **Sementara (M0).** Bentuk ini cukup untuk registry dan runner tutorial.
//! Kontrak final, termasuk RNG yang disuntikkan dan replay, ditetapkan di M1
//! bersama Reversi; fixture ikut disesuaikan (D-005).

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::action::{ActionSpec, find_match};

/// Nomor kursi pemain dalam satu permainan, mulai dari 0.
pub type PlayerId = u8;

/// Seed 32 byte (SPEC §5.4).
pub type Seed = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GameError {
    #[error("bukan giliran pemain {0}")]
    NotPending(PlayerId),
    #[error("`{0}` bukan aksi yang sah saat ini")]
    Illegal(String),
    #[error("perintah tidak dikenal: `{0}`")]
    Parse(String),
    #[error("konfigurasi tidak sah: {0}")]
    Config(String),
    #[error("permainan sudah selesai")]
    Over,
}

/// Hasil akhir sementara; bentuk final (termasuk skor dan chip) di M1/M4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameResult {
    pub winners: Vec<PlayerId>,
    pub summary: String,
}

/// Mesin keadaan murni: tanpa IO, tanpa jam dinding, tanpa RNG global.
pub trait TurnGame: Sized {
    type Config: DeserializeOwned + Default;
    type Action;
    type View: Serialize;

    fn new(config: Self::Config, seed: Seed) -> Self;
    /// Pemain yang sedang ditunggu aksinya; bisa lebih dari satu pada fase
    /// serentak.
    fn pending_players(&self) -> Vec<PlayerId>;
    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec>;
    /// Dipanggil hanya dengan aksi yang cocok dengan `legal_actions(player)`.
    fn apply(&mut self, player: PlayerId, action: Self::Action) -> Result<(), GameError>;
    fn view_for(&self, player: PlayerId) -> Self::View;
    fn to_text(view: &Self::View) -> String;
    fn is_over(&self) -> bool;
    fn result(&self) -> Option<GameResult>;
    fn parse_command(&self, command: &str) -> Result<Self::Action, GameError>;
    fn format_action(&self, action: &Self::Action) -> String;
}

/// Permainan yang sedang berjalan, tanpa tipe konkret. Semua aksi masuk
/// sebagai perintah teks, sama seperti dari LAN dan agen (SPEC §2.1).
pub trait Session: Send {
    fn pending_players(&self) -> Vec<PlayerId>;
    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec>;
    /// Memeriksa giliran dan `legal_actions`, lalu menerapkan perintah.
    fn act(&mut self, player: PlayerId, command: &str) -> Result<(), GameError>;
    fn view_data(&self, player: PlayerId) -> serde_json::Value;
    fn view_text(&self, player: PlayerId) -> String;
    fn is_over(&self) -> bool;
    fn result(&self) -> Option<GameResult>;
}

impl<G: TurnGame + Send> Session for G {
    fn pending_players(&self) -> Vec<PlayerId> {
        TurnGame::pending_players(self)
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        TurnGame::legal_actions(self, player)
    }

    fn act(&mut self, player: PlayerId, command: &str) -> Result<(), GameError> {
        if TurnGame::is_over(self) {
            return Err(GameError::Over);
        }
        if !TurnGame::pending_players(self).contains(&player) {
            return Err(GameError::NotPending(player));
        }
        let specs = TurnGame::legal_actions(self, player);
        if find_match(&specs, command).is_none() {
            return Err(GameError::Illegal(crate::action::normalize(command)));
        }
        let action = self.parse_command(command)?;
        self.apply(player, action)
    }

    fn view_data(&self, player: PlayerId) -> serde_json::Value {
        serde_json::to_value(self.view_for(player)).unwrap_or(serde_json::Value::Null)
    }

    fn view_text(&self, player: PlayerId) -> String {
        G::to_text(&self.view_for(player))
    }

    fn is_over(&self) -> bool {
        TurnGame::is_over(self)
    }

    fn result(&self) -> Option<GameResult> {
        TurnGame::result(self)
    }
}

/// Membuat sesi baru dari konfigurasi JSON (`null` = konfigurasi bawaan).
pub fn create_session<G: TurnGame + Send + 'static>(
    config: &serde_json::Value,
    seed: Seed,
) -> Result<Box<dyn Session>, GameError> {
    let config = if config.is_null() {
        G::Config::default()
    } else {
        serde_json::from_value(config.clone()).map_err(|e| GameError::Config(e.to_string()))?
    };
    Ok(Box::new(G::new(config, seed)))
}
