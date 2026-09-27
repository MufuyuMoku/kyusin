//! Satu pertandingan: sesi game + pemain per kursi + catatan langkah dan
//! provably fair. Hasilnya bisa diekspor sebagai [`Replay`] (SPEC §2.5).

use crate::fair::FairRecord;
use crate::game::{GameError, PlayerId, Session};
use crate::player::{Player, SeatKind};
use crate::registry::Cartridge;
use crate::replay::{Move, REPLAY_FORMAT, Replay};

pub struct Match {
    game: String,
    config: serde_json::Value,
    session: Box<dyn Session>,
    players: Vec<Box<dyn Player>>,
    moves: Vec<Move>,
    fair: FairRecord,
}

impl Match {
    /// Memulai pertandingan dengan seed ronde dari catatan provably fair.
    pub fn new(
        cartridge: &Cartridge,
        config: serde_json::Value,
        fair: FairRecord,
        players: Vec<Box<dyn Player>>,
    ) -> Result<Self, GameError> {
        let seed = fair
            .round_seed_bytes()
            .map_err(|e| GameError::Config(e.to_string()))?;
        let session = (cartridge.create)(&config, seed)?;
        if players.len() != session.seats() as usize {
            return Err(GameError::Config(format!(
                "butuh {} pemain, diberi {}",
                session.seats(),
                players.len()
            )));
        }
        Ok(Match {
            game: cartridge.manifest.id.clone(),
            config,
            session,
            players,
            moves: Vec::new(),
            fair,
        })
    }

    pub fn session(&self) -> &dyn Session {
        self.session.as_ref()
    }

    pub fn fair(&self) -> &FairRecord {
        &self.fair
    }

    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    pub fn seat_kind(&self, seat: PlayerId) -> Option<SeatKind> {
        self.players.get(seat as usize).map(|p| p.kind())
    }

    /// Aksi dari luar (UI lokal atau jaringan).
    pub fn act(&mut self, seat: PlayerId, command: &str) -> Result<(), GameError> {
        self.session.act(seat, command)?;
        self.moves.push(Move {
            seat,
            command: crate::action::normalize(command),
        });
        Ok(())
    }

    /// Menjalankan satu langkah pemain yang memutuskan sendiri (bot) bila
    /// ada yang sedang ditunggu. Mengembalikan langkahnya, atau `None` bila
    /// yang ditunggu hanya pemain dari luar atau permainan selesai.
    pub fn step_auto(&mut self) -> Result<Option<Move>, GameError> {
        if self.session.is_over() {
            return Ok(None);
        }
        for seat in self.session.pending_players() {
            let Some(player) = self.players.get_mut(seat as usize) else {
                continue;
            };
            if let Some(command) = player.decide(self.session.as_ref(), seat) {
                self.act(seat, &command)?;
                return Ok(self.moves.last().cloned());
            }
        }
        Ok(None)
    }

    pub fn replay(&self) -> Replay {
        Replay {
            format: REPLAY_FORMAT,
            game: self.game.clone(),
            config: self.config.clone(),
            seats: self.players.iter().map(|p| p.kind()).collect(),
            fair: self.fair.clone(),
            moves: self.moves.clone(),
            result: self.session.result(),
            state_hash: self.session.state_hash(),
        }
    }
}
