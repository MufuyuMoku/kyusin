//! Cartridge bawaan (SPEC §5.1, §6).
//!
//! Setiap game adalah satu modul dengan `manifest.toml`-nya sendiri dan
//! tutorial di `tutorials/<id>.toml`. Mendaftarkan game baru cukup dengan
//! menambah satu baris di [`builtin`]; menu tidak disentuh.

use kyusin_core::{Registry, RegistryError};

pub mod catur;
pub mod fixture;
pub mod reversi;

/// Game katalog yang dikirim bersama aplikasi.
pub fn builtin() -> Result<Registry, RegistryError> {
    let mut registry = Registry::new();
    registry.register(catur::cartridge()?)?;
    registry.register(reversi::cartridge()?)?;
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
