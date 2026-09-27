//! Kontrak game giliran (SPEC §5.2), final sejak M1 (D-034).
//!
//! Game adalah mesin keadaan murni: tanpa IO, tanpa jam dinding, tanpa RNG
//! global. Semua acak lewat [`crate::rng::GameRng`] yang dibuat dari seed
//! ronde di `new`. Keadaan game bisa diserialisasi supaya bisa di-hash
//! untuk `verify` dan replay (SPEC §5.4, §2.5).

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::action::{ActionSpec, find_match};
use crate::hash::{hex, sha256};
use crate::i18n::{Lang, Localized, core};

/// Nomor kursi pemain dalam satu permainan, mulai dari 0.
pub type PlayerId = u8;

/// Seed 32 byte (SPEC §5.4).
pub type Seed = [u8; 32];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NotPending(PlayerId),
    Illegal(String),
    Parse(String),
    Config(String),
    Over,
}

impl GameError {
    /// Pesan untuk pemain dalam dua bahasa (SPEC §4, D-027).
    pub fn message(&self) -> Localized {
        let c = core();
        match self {
            GameError::NotPending(p) => {
                c.localized("error.not_pending", &[("player", &(p + 1).to_string())])
            }
            GameError::Illegal(cmd) => c.localized("error.illegal", &[("command", cmd)]),
            GameError::Parse(cmd) => c.localized("error.parse", &[("command", cmd)]),
            GameError::Config(d) => c.localized("error.config", &[("detail", d)]),
            GameError::Over => c.localized("error.over", &[]),
        }
    }
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message().get(Lang::Id))
    }
}

impl std::error::Error for GameError {}

/// Hasil akhir permainan. Chip dan pembayaran casino ditambahkan di M4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameResult {
    /// Kursi pemenang; kosong = seri.
    pub winners: Vec<PlayerId>,
    /// Skor per kursi (arti bergantung game, misalnya jumlah bidak).
    pub scores: Vec<i64>,
    pub summary: Localized,
}

/// Mesin keadaan murni: tanpa IO, tanpa jam dinding, tanpa RNG global.
pub trait TurnGame: Sized + Serialize {
    type Config: DeserializeOwned + Default;
    type Action;
    type View: Serialize;

    /// Membuat permainan dari konfigurasi dan seed ronde. Gagal bila
    /// konfigurasinya tidak sah (misalnya posisi awal kustom yang rusak).
    fn new(config: Self::Config, seed: Seed) -> Result<Self, GameError>;
    /// Jumlah kursi.
    fn seats(&self) -> u8;
    /// Pemain yang sedang ditunggu aksinya; bisa lebih dari satu pada fase
    /// serentak.
    fn pending_players(&self) -> Vec<PlayerId>;
    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec>;
    /// Dipanggil hanya dengan aksi yang cocok dengan `legal_actions(player)`.
    fn apply(&mut self, player: PlayerId, action: Self::Action) -> Result<(), GameError>;
    fn view_for(&self, player: PlayerId) -> Self::View;
    /// Bentuk teks untuk manusia (`state.text`) dalam bahasa yang diminta
    /// (SPEC §10, D-028).
    fn to_text(view: &Self::View, lang: Lang) -> String;
    fn is_over(&self) -> bool;
    fn result(&self) -> Option<GameResult>;
    fn parse_command(&self, command: &str) -> Result<Self::Action, GameError>;
    fn format_action(&self, action: &Self::Action) -> String;
}

/// Permainan yang sedang berjalan, tanpa tipe konkret. Semua aksi masuk
/// sebagai perintah teks, sama seperti dari LAN dan agen (SPEC §2.1).
pub trait Session: Send {
    fn seats(&self) -> u8;
    fn pending_players(&self) -> Vec<PlayerId>;
    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec>;
    /// Memeriksa giliran dan `legal_actions`, lalu menerapkan perintah.
    fn act(&mut self, player: PlayerId, command: &str) -> Result<(), GameError>;
    fn view_data(&self, player: PlayerId) -> serde_json::Value;
    fn view_text(&self, player: PlayerId, lang: Lang) -> String;
    fn is_over(&self) -> bool;
    fn result(&self) -> Option<GameResult>;
    /// SHA-256 (hex) dari seluruh keadaan game; dipakai `verify` dan replay.
    fn state_hash(&self) -> String;
}

impl<G: TurnGame + Send> Session for G {
    fn seats(&self) -> u8 {
        TurnGame::seats(self)
    }

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

    fn view_text(&self, player: PlayerId, lang: Lang) -> String {
        G::to_text(&self.view_for(player), lang)
    }

    fn is_over(&self) -> bool {
        TurnGame::is_over(self)
    }

    fn result(&self) -> Option<GameResult> {
        TurnGame::result(self)
    }

    fn state_hash(&self) -> String {
        let json = serde_json::to_vec(self).expect("keadaan game harus bisa diserialisasi");
        hex(&sha256(&json))
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
    Ok(Box::new(G::new(config, seed)?))
}
