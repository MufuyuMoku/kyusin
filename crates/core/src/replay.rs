//! Replay dan `verify` (SPEC §2.5, §5.4).
//!
//! Setiap pertandingan = konfigurasi + catatan provably fair + urutan
//! perintah teks. Memutar ulang perintah itu terhadap mesin aturan asli
//! menghasilkan keadaan yang persis sama, jadi replay sekaligus bukti:
//! `verify` memeriksa komitmen seed, menghitung ulang seed ronde, memutar
//! ulang semua langkah, lalu membandingkan hasil dan hash keadaan akhir.

use serde::{Deserialize, Serialize};

use crate::fair::{FairError, FairRecord};
use crate::game::{GameError, GameResult, PlayerId, Session};
use crate::i18n::{Lang, Localized};
use crate::player::SeatKind;
use crate::registry::Cartridge;

pub const REPLAY_FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub seat: PlayerId,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replay {
    pub format: u32,
    pub game: String,
    pub config: serde_json::Value,
    pub seats: Vec<SeatKind>,
    pub fair: FairRecord,
    pub moves: Vec<Move>,
    pub result: Option<GameResult>,
    /// Hash keadaan akhir (hex SHA-256).
    pub state_hash: String,
}

/// Satu keadaan dalam pemutaran ulang: sebelum langkah pertama (`index` 0)
/// dan setelah tiap langkah.
#[derive(Debug, Clone, Serialize)]
pub struct Frame {
    pub index: usize,
    /// Langkah yang menghasilkan keadaan ini.
    pub last: Option<Move>,
    pub view_data: serde_json::Value,
    pub view_text: Localized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    UnsupportedFormat(u32),
    WrongGame {
        expected: String,
        found: String,
    },
    Fair(FairError),
    /// Langkah ke-`index` (mulai 1) tidak sah.
    Move {
        index: usize,
        source: GameError,
    },
    Config(GameError),
}

impl ReplayError {
    pub fn message(&self) -> Localized {
        let c = crate::i18n::core();
        match self {
            ReplayError::Move { index, source } => Localized::build(|lang| {
                c.text(
                    lang,
                    "error.replay_move",
                    &[
                        ("index", &index.to_string()),
                        ("detail", source.message().get(lang)),
                    ],
                )
            }),
            ReplayError::Config(e) => e.message(),
            ReplayError::Fair(e) => c.localized("error.replay_fair", &[("detail", &e.to_string())]),
            ReplayError::UnsupportedFormat(v) => {
                c.localized("error.replay_format", &[("version", &v.to_string())])
            }
            ReplayError::WrongGame { expected, found } => c.localized(
                "error.tutorial_wrong_game",
                &[("expected", expected), ("found", found)],
            ),
        }
    }
}

/// Langkah pemeriksaan `verify`, berurutan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifyStep {
    /// Setiap seed yang dibuka cocok dengan komitmennya.
    Commitments,
    /// Seed ronde sama dengan hasil hitung ulang.
    RoundSeed,
    /// Semua langkah sah bila diputar ulang.
    Moves,
    /// Hasil akhir sama dengan yang dicatat.
    Result,
    /// Keadaan akhir sama persis (hash).
    StateHash,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyCheck {
    pub step: VerifyStep,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub checks: Vec<VerifyCheck>,
    /// Penjelasan kegagalan pertama, bila ada.
    pub error: Option<Localized>,
}

impl Replay {
    fn check_header(&self, cartridge: &Cartridge) -> Result<(), ReplayError> {
        if self.format != REPLAY_FORMAT {
            return Err(ReplayError::UnsupportedFormat(self.format));
        }
        if self.game != cartridge.manifest.id {
            return Err(ReplayError::WrongGame {
                expected: cartridge.manifest.id.clone(),
                found: self.game.clone(),
            });
        }
        Ok(())
    }

    fn start(&self, cartridge: &Cartridge) -> Result<Box<dyn Session>, ReplayError> {
        let seed = self.fair.round_seed_bytes().map_err(ReplayError::Fair)?;
        (cartridge.create)(&self.config, seed).map_err(ReplayError::Config)
    }

    fn frame(session: &dyn Session, seat: PlayerId, index: usize, last: Option<Move>) -> Frame {
        Frame {
            index,
            last,
            view_data: session.view_data(seat),
            view_text: Localized::build(|lang: Lang| session.view_text(seat, lang)),
        }
    }

    /// Semua keadaan dari sudut pandang `seat`, untuk penampil replay.
    pub fn frames(&self, cartridge: &Cartridge, seat: PlayerId) -> Result<Vec<Frame>, ReplayError> {
        self.check_header(cartridge)?;
        let mut session = self.start(cartridge)?;
        let mut frames = vec![Self::frame(session.as_ref(), seat, 0, None)];
        for (i, mv) in self.moves.iter().enumerate() {
            session
                .act(mv.seat, &mv.command)
                .map_err(|source| ReplayError::Move {
                    index: i + 1,
                    source,
                })?;
            frames.push(Self::frame(session.as_ref(), seat, i + 1, Some(mv.clone())));
        }
        Ok(frames)
    }

    /// Pemeriksaan lengkap (SPEC §5.4 langkah 4).
    pub fn verify(&self, cartridge: &Cartridge) -> VerifyReport {
        let mut checks = Vec::new();
        let fail = |checks: Vec<VerifyCheck>, error: Localized| VerifyReport {
            ok: false,
            checks,
            error: Some(error),
        };
        if let Err(e) = self.check_header(cartridge) {
            return fail(checks, e.message());
        }
        match self.fair.verify() {
            Ok(_) => {
                checks.push(VerifyCheck {
                    step: VerifyStep::Commitments,
                    ok: true,
                });
                checks.push(VerifyCheck {
                    step: VerifyStep::RoundSeed,
                    ok: true,
                });
            }
            Err(e) => {
                let step = if e == FairError::RoundSeedMismatch {
                    checks.push(VerifyCheck {
                        step: VerifyStep::Commitments,
                        ok: true,
                    });
                    VerifyStep::RoundSeed
                } else {
                    VerifyStep::Commitments
                };
                checks.push(VerifyCheck { step, ok: false });
                return fail(checks, ReplayError::Fair(e).message());
            }
        }
        let mut session = match self.start(cartridge) {
            Ok(s) => s,
            Err(e) => {
                checks.push(VerifyCheck {
                    step: VerifyStep::Moves,
                    ok: false,
                });
                return fail(checks, e.message());
            }
        };
        for (i, mv) in self.moves.iter().enumerate() {
            if let Err(source) = session.act(mv.seat, &mv.command) {
                checks.push(VerifyCheck {
                    step: VerifyStep::Moves,
                    ok: false,
                });
                return fail(
                    checks,
                    ReplayError::Move {
                        index: i + 1,
                        source,
                    }
                    .message(),
                );
            }
        }
        checks.push(VerifyCheck {
            step: VerifyStep::Moves,
            ok: true,
        });
        let result_ok = session.result() == self.result;
        checks.push(VerifyCheck {
            step: VerifyStep::Result,
            ok: result_ok,
        });
        let hash_ok = session.state_hash() == self.state_hash;
        checks.push(VerifyCheck {
            step: VerifyStep::StateHash,
            ok: hash_ok,
        });
        let ok = result_ok && hash_ok;
        VerifyReport {
            ok,
            checks,
            error: (!ok).then(|| crate::i18n::core().localized("error.replay_mismatch", &[])),
        }
    }
}
