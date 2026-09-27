//! Inti KyuSin: kontrak cartridge, manifest, registry, dan runner tutorial
//! (SPEC §5).
//!
//! Status M0: kontrak [`game::TurnGame`] di sini **sementara**. Bentuk
//! finalnya, bersama RNG provably fair dan replay, dibuat di M1 (SPEC §9,
//! D-005). Registry dan runner tutorial sudah final dan diuji dengan game
//! fixture di `kyusin-games`.

pub mod action;
pub mod game;
pub mod help;
pub mod manifest;
pub mod registry;
pub mod tutorial;

pub use action::{ActionSpec, Param, ParamKind};
pub use game::{GameError, PlayerId, Seed, Session, TurnGame};
pub use manifest::{Category, Kind, Manifest, Opponent};
pub use registry::{Cartridge, Registry, RegistryError};
pub use tutorial::{Tutorial, TutorialError, TutorialRun};
