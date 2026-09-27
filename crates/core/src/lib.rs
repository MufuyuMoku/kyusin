//! Inti KyuSin: kontrak cartridge, manifest, registry, dan runner tutorial
//! (SPEC §5).
//!
//! Kontrak [`game::TurnGame`] final sejak M1 (D-034), bersama RNG yang
//! disuntikkan ([`rng`]), provably fair ([`fair`]), pemain ([`player`]),
//! pertandingan ([`game_match`]), dan replay + `verify` ([`replay`]).

pub mod action;
pub mod fair;
pub mod game;
pub mod game_match;
pub mod hash;
pub mod help;
pub mod i18n;
pub mod manifest;
pub mod player;
pub mod registry;
pub mod replay;
pub mod rng;
pub mod tutorial;

pub use action::{ActionSpec, Param, ParamKind};
pub use game::{GameError, GameResult, PlayerId, Seed, Session, TurnGame};
pub use game_match::Match;
pub use i18n::{Lang, Localized};
pub use manifest::{Category, Kind, Manifest, Opponent};
pub use player::{Human, Player, SeatKind};
pub use registry::{Cartridge, Registry, RegistryError};
pub use replay::{Replay, VerifyReport};
pub use rng::GameRng;
pub use tutorial::{Tutorial, TutorialError, TutorialRun};
