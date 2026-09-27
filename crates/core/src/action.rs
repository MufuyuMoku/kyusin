//! `ActionSpec`: bentuk aksi yang sah bagi seorang pemain (SPEC §5.2, §10).
//!
//! Sebuah spesifikasi berupa **aksi tetap** (`hit`) atau **templat
//! berparameter** dengan batas (`bet <jumlah>` dengan min/maks/kelipatan).
//! Keduanya punya bentuk perintah teks, karena perintah teks adalah protokol
//! LAN/agen dan juga yang dijalankan kontrol visual di balik layar.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionSpec {
    /// Satu perintah persis, misalnya `hit` atau `done`.
    Fixed { command: String },
    /// Kata kerja diikuti parameter berbatas, misalnya `take <n>`.
    Template { verb: String, params: Vec<Param> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    #[serde(flatten)]
    pub kind: ParamKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParamKind {
    /// Bilangan bulat `min..=max` dengan kelipatan `step` dihitung dari `min`.
    Int { min: i64, max: i64, step: i64 },
    /// Salah satu kata dari daftar.
    Choice { options: Vec<String> },
}

impl ParamKind {
    fn accepts(&self, token: &str) -> bool {
        match self {
            ParamKind::Int { min, max, step } => match token.parse::<i64>() {
                Ok(v) => v >= *min && v <= *max && *step > 0 && (v - min) % step == 0,
                Err(_) => false,
            },
            ParamKind::Choice { options } => options.iter().any(|o| o == token),
        }
    }

    /// Semua nilai yang sah, bila jumlahnya tidak melebihi `limit`.
    fn values(&self, limit: usize) -> Option<Vec<String>> {
        match self {
            ParamKind::Int { min, max, step } => {
                if *step <= 0 || max < min {
                    return Some(Vec::new());
                }
                let count = ((max - min) / step + 1) as usize;
                (count <= limit).then(|| {
                    (0..count as i64)
                        .map(|i| (min + i * step).to_string())
                        .collect()
                })
            }
            ParamKind::Choice { options } => (options.len() <= limit).then(|| options.clone()),
        }
    }
}

/// Merapikan spasi sebuah perintah. Huruf besar/kecil dipertahankan karena
/// ada notasi yang membedakannya (SAN catur: `Bxc3` bukan `bxc3`).
pub fn normalize(command: &str) -> String {
    command.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl ActionSpec {
    pub fn fixed(command: impl Into<String>) -> Self {
        ActionSpec::Fixed {
            command: command.into(),
        }
    }

    pub fn template(verb: impl Into<String>, params: Vec<Param>) -> Self {
        ActionSpec::Template {
            verb: verb.into(),
            params,
        }
    }

    /// Kata pertama perintah.
    pub fn verb(&self) -> &str {
        match self {
            ActionSpec::Fixed { command } => command.split_whitespace().next().unwrap_or(""),
            ActionSpec::Template { verb, .. } => verb,
        }
    }

    /// Bentuk perintah teks, misalnya `take <n>`.
    pub fn usage(&self) -> String {
        match self {
            ActionSpec::Fixed { command } => command.clone(),
            ActionSpec::Template { verb, params } => {
                let mut s = verb.clone();
                for p in params {
                    s.push_str(&format!(" <{}>", p.name));
                }
                s
            }
        }
    }

    /// Apakah perintah teks ini cocok dengan spesifikasi ini.
    pub fn matches(&self, command: &str) -> bool {
        let command = normalize(command);
        match self {
            ActionSpec::Fixed { command: c } => normalize(c) == command,
            ActionSpec::Template { verb, params } => {
                let mut tokens = command.split(' ');
                if tokens.next() != Some(verb.as_str()) {
                    return false;
                }
                let rest: Vec<&str> = tokens.collect();
                rest.len() == params.len()
                    && params.iter().zip(&rest).all(|(p, t)| p.kind.accepts(t))
            }
        }
    }

    /// Semua perintah konkret yang dicakup spesifikasi ini, bila jumlahnya
    /// tidak melebihi `limit`. Dipakai untuk tombol visual dan autocomplete.
    pub fn concrete(&self, limit: usize) -> Option<Vec<String>> {
        match self {
            ActionSpec::Fixed { command } => Some(vec![normalize(command)]),
            ActionSpec::Template { verb, params } => {
                let mut out = vec![verb.clone()];
                for p in params {
                    let values = p.kind.values(limit)?;
                    out = out
                        .iter()
                        .flat_map(|prefix| values.iter().map(move |v| format!("{prefix} {v}")))
                        .collect();
                    if out.len() > limit {
                        return None;
                    }
                }
                Some(out)
            }
        }
    }
}

/// Indeks spesifikasi pertama yang cocok dengan perintah.
pub fn find_match(specs: &[ActionSpec], command: &str) -> Option<usize> {
    specs.iter().position(|s| s.matches(command))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bet() -> ActionSpec {
        ActionSpec::template(
            "bet",
            vec![Param {
                name: "jumlah".into(),
                kind: ParamKind::Int {
                    min: 10,
                    max: 100,
                    step: 10,
                },
            }],
        )
    }

    #[test]
    fn fixed_matches_exactly_after_whitespace_normalization() {
        let s = ActionSpec::fixed("hit");
        assert!(s.matches("hit"));
        assert!(s.matches("  hit "));
        assert!(!s.matches("hit 1"));
        assert!(!s.matches("HIT"));
    }

    #[test]
    fn template_checks_bounds_and_step() {
        let s = bet();
        assert!(s.matches("bet 10"));
        assert!(s.matches("bet  100"));
        assert!(!s.matches("bet 15"));
        assert!(!s.matches("bet 0"));
        assert!(!s.matches("bet 110"));
        assert!(!s.matches("bet"));
        assert!(!s.matches("bet 10 20"));
        assert!(!s.matches("bet x"));
    }

    #[test]
    fn choice_param() {
        let s = ActionSpec::template(
            "place",
            vec![Param {
                name: "jenis".into(),
                kind: ParamKind::Choice {
                    options: vec!["merah".into(), "hitam".into()],
                },
            }],
        );
        assert!(s.matches("place merah"));
        assert!(!s.matches("place hijau"));
        assert_eq!(s.usage(), "place <jenis>");
    }

    #[test]
    fn concrete_expands_small_templates_only() {
        assert_eq!(bet().concrete(10).unwrap().len(), 10);
        assert_eq!(bet().concrete(10).unwrap()[0], "bet 10");
        assert!(bet().concrete(9).is_none());
    }

    #[test]
    fn serializes_with_kind_tag() {
        let json = serde_json::to_value(bet()).unwrap();
        assert_eq!(json["kind"], "template");
        assert_eq!(json["params"][0]["type"], "int");
        assert_eq!(json["params"][0]["min"], 10);
    }
}
