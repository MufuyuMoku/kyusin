//! Cartridge bawaan (SPEC §5.1, §6).
//!
//! Setiap game adalah satu modul dengan `manifest.toml`-nya sendiri dan
//! tutorial di `tutorials/<id>.toml`. Mendaftarkan game baru cukup dengan
//! menambah satu baris di [`builtin`]; menu tidak disentuh.

use kyusin_core::{Registry, RegistryError};

pub mod andar_bahar;
pub mod baccarat;
pub mod big_six;
pub mod blackjack;
pub mod capsa_susun;
pub mod cards;
pub mod caribbean_stud;
pub mod casino_holdem;
pub mod casino_war;
pub mod catur;
pub mod chuck_a_luck;
pub mod dadu;
pub mod domino;
pub mod domino_qiuqiu;
pub mod dragon_tiger;
pub mod fan_tan;
pub mod fixture;
pub mod let_it_ride;
pub mod meja;
pub mod omaha;
pub mod pai_gow;
pub mod papan_taruhan;
pub mod poker;
pub mod poker_meja;
pub mod pot;
pub mod red_dog;
pub mod reversi;
pub mod roulette;
pub mod sic_bo;
pub mod teen_patti;
pub mod texas_holdem;
pub mod three_card_poker;

/// Game katalog yang dikirim bersama aplikasi.
pub fn builtin() -> Result<Registry, RegistryError> {
    let mut registry = Registry::new();
    registry.register(catur::cartridge()?)?;
    registry.register(reversi::cartridge()?)?;
    registry.register(blackjack::cartridge()?)?;
    registry.register(dragon_tiger::cartridge()?)?;
    registry.register(casino_war::cartridge()?)?;
    registry.register(andar_bahar::cartridge()?)?;
    registry.register(baccarat::cartridge()?)?;
    registry.register(red_dog::cartridge()?)?;
    registry.register(three_card_poker::cartridge()?)?;
    registry.register(caribbean_stud::cartridge()?)?;
    registry.register(casino_holdem::cartridge()?)?;
    registry.register(let_it_ride::cartridge()?)?;
    registry.register(pai_gow::cartridge()?)?;
    registry.register(texas_holdem::cartridge()?)?;
    registry.register(omaha::cartridge()?)?;
    registry.register(teen_patti::cartridge()?)?;
    registry.register(capsa_susun::cartridge()?)?;
    registry.register(domino_qiuqiu::cartridge()?)?;
    registry.register(roulette::cartridge_eropa()?)?;
    registry.register(roulette::cartridge_amerika()?)?;
    registry.register(sic_bo::cartridge()?)?;
    registry.register(big_six::cartridge()?)?;
    registry.register(fan_tan::cartridge()?)?;
    registry.register(chuck_a_luck::cartridge()?)?;
    #[cfg(feature = "fixture")]
    registry.register(fixture::cartridge()?)?;
    Ok(registry)
}

/// Registry katalog ditambah fixture; dipakai tes (SPEC §9 M0).
pub fn with_fixture() -> Result<Registry, RegistryError> {
    let mut registry = builtin()?;
    if registry.get(fixture::ID).is_none() {
        registry.register(fixture::cartridge()?)?;
    }
    Ok(registry)
}
