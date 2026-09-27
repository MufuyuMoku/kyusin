//! Registry cartridge (SPEC §5.2).
//!
//! Menambah game = mendaftarkan satu [`Cartridge`] (modul + manifest +
//! tutorial). Menu, `help`, `man`, dan autocomplete membaca registry ini.

use crate::game::{GameError, Seed, Session};
use crate::manifest::{Category, Manifest};

pub type CreateFn = fn(&serde_json::Value, Seed) -> Result<Box<dyn Session>, GameError>;

pub struct Cartridge {
    pub manifest: Manifest,
    /// Isi berkas tutorial, ditanam saat kompilasi supaya aplikasi tetap
    /// luring. Tes memastikan isinya sama dengan berkas di `manifest.tutorial`.
    pub tutorial_src: &'static str,
    pub create: CreateFn,
}

impl Cartridge {
    pub fn new(
        manifest_src: &str,
        tutorial_src: &'static str,
        create: CreateFn,
    ) -> Result<Self, RegistryError> {
        let manifest = Manifest::from_toml(manifest_src)
            .map_err(|e| RegistryError::ManifestParse(e.to_string()))?;
        Ok(Cartridge {
            manifest,
            tutorial_src,
            create,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("manifest tidak bisa dibaca: {0}")]
    ManifestParse(String),
    #[error("manifest `{id}` tidak sah: {}", errors.join("; "))]
    Invalid { id: String, errors: Vec<String> },
    #[error("id `{0}` sudah terdaftar")]
    Duplicate(String),
}

#[derive(Default)]
pub struct Registry {
    cartridges: Vec<Cartridge>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, cartridge: Cartridge) -> Result<(), RegistryError> {
        let errors = cartridge.manifest.validate();
        if !errors.is_empty() {
            return Err(RegistryError::Invalid {
                id: cartridge.manifest.id.clone(),
                errors,
            });
        }
        if self.get(&cartridge.manifest.id).is_some() {
            return Err(RegistryError::Duplicate(cartridge.manifest.id.clone()));
        }
        self.cartridges.push(cartridge);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Cartridge> {
        self.cartridges.iter().find(|c| c.manifest.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cartridge> {
        self.cartridges.iter()
    }

    pub fn manifests(&self) -> impl Iterator<Item = &Manifest> {
        self.cartridges.iter().map(|c| &c.manifest)
    }

    pub fn len(&self) -> usize {
        self.cartridges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cartridges.is_empty()
    }

    /// Game per kategori, urut menurut kategori lalu id (nama tampilan bergantung bahasa).
    pub fn by_category(&self) -> Vec<(Category, Vec<&Manifest>)> {
        Category::ALL
            .iter()
            .filter_map(|cat| {
                let mut games: Vec<&Manifest> =
                    self.manifests().filter(|m| m.category == *cat).collect();
                games.sort_by(|a, b| a.id.cmp(&b.id));
                (!games.is_empty()).then_some((*cat, games))
            })
            .collect()
    }
}
