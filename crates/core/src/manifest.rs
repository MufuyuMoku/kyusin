//! Manifest cartridge (SPEC §5.2).
//!
//! Satu manifest per game, ditulis sebagai TOML di samping modul game-nya.
//! Nama kunci mengikuti SPEC. Menu, `help`, `man`, dan autocomplete
//! dibangkitkan dari sini, tidak ditulis tangan.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    /// Nama tampilan.
    #[serde(rename = "nama")]
    pub name: String,
    #[serde(rename = "kategori")]
    pub category: Category,
    #[serde(rename = "pemain_min")]
    pub min_players: u8,
    #[serde(rename = "pemain_maks")]
    pub max_players: u8,
    #[serde(rename = "jenis")]
    pub kind: Kind,
    #[serde(rename = "lawan")]
    pub opponent: Opponent,
    /// Dapat rating dan wajib 3 level bot (SPEC §8).
    #[serde(rename = "kompetitif")]
    pub competitive: bool,
    pub lan: bool,
    #[serde(rename = "agen")]
    pub agent: bool,
    /// RTP dalam persen untuk casino yang melawan rumah; kosong (`null`)
    /// untuk game antar-pemain dan non-casino (SPEC §5.2, D-014).
    #[serde(default)]
    pub rtp: Option<f64>,
    /// Path tutorial relatif dari akar repo.
    pub tutorial: String,
    #[serde(rename = "perintah")]
    pub commands: Vec<CommandDoc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandDoc {
    /// Bentuk perintah, misalnya `take <n>`.
    #[serde(rename = "pola")]
    pub usage: String,
    /// Deskripsi singkat.
    #[serde(rename = "ringkas")]
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// §6.1
    Papan,
    /// §6.2
    Kartu,
    /// §6.3
    CasinoMeja,
    /// §6.4
    CasinoDadu,
    /// §6.5
    CasinoLotere,
    /// §6.6
    CasinoArcade,
    /// Game fixture khusus tes (SPEC §9 M0); bukan bagian katalog.
    Uji,
}

impl Category {
    pub const ALL: [Category; 7] = [
        Category::Papan,
        Category::Kartu,
        Category::CasinoMeja,
        Category::CasinoDadu,
        Category::CasinoLotere,
        Category::CasinoArcade,
        Category::Uji,
    ];

    pub fn is_casino(self) -> bool {
        matches!(
            self,
            Category::CasinoMeja
                | Category::CasinoDadu
                | Category::CasinoLotere
                | Category::CasinoArcade
        )
    }

    /// Kunci kategori seperti tertulis di manifest.
    pub fn key(self) -> &'static str {
        match self {
            Category::Papan => "papan",
            Category::Kartu => "kartu",
            Category::CasinoMeja => "casino-meja",
            Category::CasinoDadu => "casino-dadu",
            Category::CasinoLotere => "casino-lotere",
            Category::CasinoArcade => "casino-arcade",
            Category::Uji => "uji",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Papan => "Papan",
            Category::Kartu => "Kartu & domino",
            Category::CasinoMeja => "Casino: meja kartu",
            Category::CasinoDadu => "Casino: dadu, roda, ubin",
            Category::CasinoLotere => "Casino: lotere & instan",
            Category::CasinoArcade => "Casino: arcade",
            Category::Uji => "Uji (fixture)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// `TurnGame`
    Giliran,
    /// `TickGame`
    RealTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Opponent {
    Bandar,
    Bot,
    TidakAda,
}

impl Manifest {
    pub fn from_toml(src: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(src)
    }

    /// Apakah game ini casino yang melawan rumah (`rtp` wajib angka).
    pub fn is_against_house(&self) -> bool {
        self.category.is_casino() && self.opponent != Opponent::Bot
    }

    /// Memeriksa aturan manifest dari SPEC. Mengembalikan semua pelanggaran.
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        let id_ok = !self.id.is_empty()
            && self.id.as_bytes()[0].is_ascii_alphanumeric()
            && self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !id_ok {
            errs.push(format!(
                "id `{}` harus huruf kecil, angka, atau `-`",
                self.id
            ));
        }
        if self.name.trim().is_empty() {
            errs.push("nama kosong".into());
        }
        if self.min_players == 0 || self.min_players > self.max_players {
            errs.push(format!(
                "pemain_min/pemain_maks tidak masuk akal: {}/{}",
                self.min_players, self.max_players
            ));
        }
        match (self.is_against_house(), self.rtp) {
            (true, None) => errs.push("casino melawan rumah wajib punya `rtp`".into()),
            (true, Some(r)) if !(r > 0.0 && r < 100.0) => {
                errs.push(format!("rtp {r} harus di antara 0 dan 100 (persen)"))
            }
            (false, Some(_)) => {
                errs.push("`rtp` hanya untuk casino melawan rumah; hapus kuncinya".into())
            }
            _ => {}
        }
        if self.competitive && self.opponent != Opponent::Bot {
            errs.push("game kompetitif harus ber-lawan bot (SPEC §8)".into());
        }
        if (self.lan || self.agent) && self.kind != Kind::Giliran {
            errs.push("LAN dan agen hanya untuk game giliran (SPEC §6.6)".into());
        }
        if self.lan && self.max_players < 2 {
            errs.push("game LAN harus bisa dimainkan minimal 2 orang".into());
        }
        if self.category != Category::Uji {
            let expected = format!("tutorials/{}.toml", self.id);
            if self.tutorial != expected {
                errs.push(format!("tutorial harus di `{expected}` (SPEC §7.6)"));
            }
        }
        if self.commands.is_empty() {
            errs.push("daftar perintah kosong".into());
        }
        for c in &self.commands {
            if c.usage.trim().is_empty() || c.summary.trim().is_empty() {
                errs.push("setiap perintah wajib punya `pola` dan `ringkas`".into());
            }
        }
        errs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r#"
        id = "contoh"
        nama = "Contoh"
        kategori = "papan"
        pemain_min = 2
        pemain_maks = 2
        jenis = "giliran"
        lawan = "bot"
        kompetitif = true
        lan = true
        agen = true
        tutorial = "tutorials/contoh.toml"
        [[perintah]]
        pola = "move <dari> <ke>"
        ringkas = "Pindahkan bidak"
    "#;

    fn with(extra: &str, replace: (&str, &str)) -> Manifest {
        let src = BASE.replace(replace.0, replace.1);
        Manifest::from_toml(&format!("{extra}\n{src}")).unwrap()
    }

    #[test]
    fn valid_board_game() {
        let m = Manifest::from_toml(BASE).unwrap();
        assert_eq!(m.validate(), Vec::<String>::new());
        assert_eq!(m.rtp, None);
    }

    #[test]
    fn unknown_key_is_rejected() {
        assert!(Manifest::from_toml(&format!("warna = 1\n{BASE}")).is_err());
    }

    #[test]
    fn house_casino_requires_rtp() {
        let m = with(
            "",
            (
                "kategori = \"papan\"\n        pemain_min = 2",
                "kategori = \"casino-lotere\"\n        pemain_min = 1",
            ),
        );
        let m = Manifest {
            opponent: Opponent::TidakAda,
            competitive: false,
            ..m
        };
        assert!(m.validate().iter().any(|e| e.contains("wajib punya `rtp`")));
        let ok = Manifest {
            rtp: Some(92.5),
            ..m.clone()
        };
        assert_eq!(ok.validate(), Vec::<String>::new());
    }

    #[test]
    fn player_vs_player_casino_must_not_have_rtp() {
        let m = with(
            "rtp = 99.0",
            ("kategori = \"papan\"", "kategori = \"casino-meja\""),
        );
        assert!(m.validate().iter().any(|e| e.contains("hapus kuncinya")));
    }

    #[test]
    fn realtime_cannot_be_lan() {
        let m = with("", ("jenis = \"giliran\"", "jenis = \"real-time\""));
        assert!(m.validate().iter().any(|e| e.contains("SPEC §6.6")));
    }

    #[test]
    fn tutorial_path_follows_id() {
        let m = with("", ("tutorials/contoh.toml", "tutorials/lain.toml"));
        assert!(m.validate().iter().any(|e| e.contains("SPEC §7.6")));
    }
}
