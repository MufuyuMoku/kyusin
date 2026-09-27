//! Dua bahasa: Indonesia dan Inggris (SPEC §4, D-027).
//!
//! Teks yang dilihat pemain tidak ditulis di kode Rust. Teks inti ada di
//! `crates/core/i18n.toml`; teks per game di katalog milik game itu; teks
//! manifest dan tutorial berupa tabel `{ id, en }` yang keduanya wajib.
//! Kata perintah (`take`, `help`) tetap Inggris di kedua bahasa.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Id,
    /// Bawaan protokol (SPEC §10).
    #[default]
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Id, Lang::En];
}

/// Teks dalam dua bahasa. Keduanya wajib, jadi berkas yang kehilangan salah
/// satu bahasa gagal dibaca (dan CI gagal).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Localized {
    pub id: String,
    pub en: String,
}

impl Localized {
    pub fn get(&self, lang: Lang) -> &str {
        match lang {
            Lang::Id => &self.id,
            Lang::En => &self.en,
        }
    }

    /// Membangun teks per bahasa dengan fungsi yang sama.
    pub fn build(f: impl Fn(Lang) -> String) -> Self {
        Localized {
            id: f(Lang::Id),
            en: f(Lang::En),
        }
    }
}

/// Katalog terjemahan: satu berkas TOML dengan tabel `[id]` dan `[en]`
/// berisi kunci yang sama.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    id: BTreeMap<String, String>,
    en: BTreeMap<String, String>,
}

impl Catalog {
    pub fn from_toml(src: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(src)
    }

    fn table(&self, lang: Lang) -> &BTreeMap<String, String> {
        match lang {
            Lang::Id => &self.id,
            Lang::En => &self.en,
        }
    }

    /// Kunci yang ada di satu bahasa tetapi tidak di bahasa lain.
    pub fn missing_keys(&self) -> Vec<String> {
        let only_id = self.id.keys().filter(|k| !self.en.contains_key(*k));
        let only_en = self.en.keys().filter(|k| !self.id.contains_key(*k));
        only_id
            .map(|k| format!("en: {k}"))
            .chain(only_en.map(|k| format!("id: {k}")))
            .collect()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.id.keys()
    }

    /// Teks untuk `key` dengan `{nama}` diganti nilai `args`. Kunci yang
    /// hilang dikembalikan apa adanya supaya mudah ketahuan di tes.
    pub fn text(&self, lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self
            .table(lang)
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string());
        for (name, value) in args {
            s = s.replace(&format!("{{{name}}}"), value);
        }
        s
    }

    pub fn localized(&self, key: &str, args: &[(&str, &str)]) -> Localized {
        Localized::build(|lang| self.text(lang, key, args))
    }
}

/// Katalog teks inti (man, kategori, kesalahan, runner tutorial).
pub fn core() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("../i18n.toml")).expect("crates/core/i18n.toml tidak sah")
    })
}

pub fn tr(lang: Lang, key: &str) -> String {
    core().text(lang, key, &[])
}

pub fn trf(lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
    core().text(lang, key, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_catalog_has_both_languages() {
        assert_eq!(core().missing_keys(), Vec::<String>::new());
    }

    #[test]
    fn placeholders_and_missing_keys() {
        let c = Catalog::from_toml(
            r#"
            [id]
            sapa = "Halo {nama}"
            [en]
            sapa = "Hello {nama}"
            "#,
        )
        .unwrap();
        assert_eq!(c.text(Lang::En, "sapa", &[("nama", "A")]), "Hello A");
        assert_eq!(c.text(Lang::Id, "tidak.ada", &[]), "tidak.ada");
    }

    #[test]
    fn detects_key_only_in_one_language() {
        let c = Catalog::from_toml("[id]\na = \"x\"\nb = \"y\"\n[en]\na = \"x\"\n").unwrap();
        assert_eq!(c.missing_keys(), vec!["en: b".to_string()]);
    }

    #[test]
    fn localized_requires_both() {
        #[derive(Deserialize)]
        struct T {
            #[allow(dead_code)]
            x: Localized,
        }
        assert!(toml::from_str::<T>("x = { id = \"a\" }").is_err());
        assert!(toml::from_str::<T>("x = { id = \"a\", en = \"b\" }").is_ok());
    }

    #[test]
    fn lang_serializes_lowercase() {
        assert_eq!(serde_json::to_value(Lang::Id).unwrap(), "id");
    }
}
