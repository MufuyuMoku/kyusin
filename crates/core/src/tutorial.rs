//! Tutorial interaktif dan runner-nya (SPEC §7.6, §7.7).
//!
//! Satu berkas TOML per game. Keadaan awal = konfigurasi + seed + perintah
//! `sebelum` tiap langkah (misalnya langkah lawan), jadi tutorial selalu
//! dijalankan terhadap mesin aturan asli, bukan tiruan. Runner yang sama
//! dipakai UI dan tes CI.

use serde::{Deserialize, Serialize};

use crate::action::{find_match, normalize};
use crate::game::{GameError, PlayerId, Seed, Session};
use crate::registry::Cartridge;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tutorial {
    /// Id game yang diajarkan; harus sama dengan manifest.
    pub game: String,
    #[serde(rename = "judul")]
    pub title: String,
    /// Seed hex 64 karakter; kosong = semua nol.
    #[serde(default)]
    pub seed: Option<String>,
    #[serde(default)]
    pub config: Option<toml::Table>,
    /// Kursi pemain yang sedang belajar.
    #[serde(rename = "pemain", default)]
    pub learner: PlayerId,
    pub man: ManDoc,
    #[serde(rename = "langkah")]
    pub steps: Vec<Step>,
}

/// Bagian halaman `man <id>` yang tidak bisa diambil dari manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManDoc {
    pub aturan: String,
    pub kontrol: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// Teks penjelasan.
    #[serde(rename = "teks")]
    pub text: String,
    /// Perintah yang diterapkan sebelum langkah ini, masing-masing oleh
    /// satu-satunya pemain yang sedang ditunggu (biasanya lawan).
    #[serde(rename = "sebelum", default)]
    pub before: Vec<String>,
    /// Aksi yang diharapkan dari pemain; kosong = langkah bacaan saja.
    #[serde(rename = "aksi", default)]
    pub action: Option<String>,
    /// Elemen visual yang disorot, misalnya `aksi:take 2`.
    #[serde(rename = "sorot", default)]
    pub highlight: Vec<String>,
    /// Petunjuk bila pemain melakukan aksi lain.
    #[serde(rename = "petunjuk", default)]
    pub hint: Option<String>,
}

/// Prefiks target sorotan untuk tombol aksi generik.
pub const HIGHLIGHT_ACTION: &str = "aksi:";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TutorialError {
    #[error("tutorial tidak bisa dibaca: {0}")]
    Parse(String),
    #[error("tutorial untuk `{found}`, tetapi dipasang di game `{expected}`")]
    WrongGame { expected: String, found: String },
    #[error("seed harus hex 64 karakter")]
    BadSeed,
    #[error("tutorial tidak punya langkah")]
    NoSteps,
    #[error("halaman man belum lengkap (aturan/kontrol kosong)")]
    EmptyMan,
    #[error("langkah {step}: {message}")]
    Step { step: usize, message: String },
    #[error("langkah {step}: {source}")]
    Game { step: usize, source: GameError },
    #[error("tutorial sudah selesai")]
    Finished,
}

impl Tutorial {
    pub fn from_toml(src: &str) -> Result<Self, TutorialError> {
        toml::from_str(src).map_err(|e| TutorialError::Parse(e.to_string()))
    }

    pub fn seed(&self) -> Result<Seed, TutorialError> {
        let Some(hex) = &self.seed else {
            return Ok([0; 32]);
        };
        if hex.len() != 64 {
            return Err(TutorialError::BadSeed);
        }
        let mut seed = [0u8; 32];
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| TutorialError::BadSeed)?;
        }
        Ok(seed)
    }

    fn config_json(&self) -> serde_json::Value {
        self.config
            .as_ref()
            .and_then(|t| serde_json::to_value(t).ok())
            .unwrap_or(serde_json::Value::Null)
    }
}

/// Tanggapan runner atas satu aksi pemain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Feedback {
    /// Aksi sesuai; langkah maju.
    Correct,
    /// Aksi lain dari yang diminta; keadaan tidak berubah.
    Wrong { hint: String },
}

pub struct TutorialRun {
    tutorial: Tutorial,
    session: Box<dyn Session>,
    index: usize,
}

impl TutorialRun {
    /// Memulai tutorial dan menerapkan `sebelum` langkah pertama.
    pub fn start(tutorial: Tutorial, cartridge: &Cartridge) -> Result<Self, TutorialError> {
        if tutorial.game != cartridge.manifest.id {
            return Err(TutorialError::WrongGame {
                expected: cartridge.manifest.id.clone(),
                found: tutorial.game.clone(),
            });
        }
        if tutorial.steps.is_empty() {
            return Err(TutorialError::NoSteps);
        }
        let session = (cartridge.create)(&tutorial.config_json(), tutorial.seed()?)
            .map_err(|source| TutorialError::Game { step: 0, source })?;
        let mut run = TutorialRun {
            tutorial,
            session,
            index: 0,
        };
        run.prepare_step()?;
        Ok(run)
    }

    pub fn tutorial(&self) -> &Tutorial {
        &self.tutorial
    }

    pub fn session(&self) -> &dyn Session {
        self.session.as_ref()
    }

    pub fn learner(&self) -> PlayerId {
        self.tutorial.learner
    }

    /// Indeks langkah saat ini (0-based); sama dengan jumlah langkah bila
    /// tutorial selesai.
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.tutorial.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tutorial.steps.is_empty()
    }

    pub fn is_finished(&self) -> bool {
        self.index >= self.tutorial.steps.len()
    }

    pub fn step(&self) -> Option<&Step> {
        self.tutorial.steps.get(self.index)
    }

    /// Menerima satu perintah dari pemain (dari kontrol visual atau mode
    /// perintah; keduanya sama di sini).
    pub fn submit(&mut self, command: &str) -> Result<Feedback, TutorialError> {
        let step_no = self.index + 1;
        let step = self.step().ok_or(TutorialError::Finished)?;
        let Some(expected) = &step.action else {
            return Ok(Feedback::Wrong {
                hint: "Langkah ini hanya bacaan; pilih [ LANJUT ].".into(),
            });
        };
        if normalize(command) != normalize(expected) {
            let hint = step
                .hint
                .clone()
                .unwrap_or_else(|| format!("Coba `{expected}`."));
            return Ok(Feedback::Wrong { hint });
        }
        let learner = self.tutorial.learner;
        self.session
            .act(learner, command)
            .map_err(|source| TutorialError::Game {
                step: step_no,
                source,
            })?;
        self.index += 1;
        self.prepare_step()?;
        Ok(Feedback::Correct)
    }

    /// Maju dari langkah bacaan (tanpa `aksi`).
    pub fn advance(&mut self) -> Result<(), TutorialError> {
        let step = self.step().ok_or(TutorialError::Finished)?;
        if step.action.is_some() {
            return Err(TutorialError::Step {
                step: self.index + 1,
                message: "langkah ini menunggu aksi pemain".into(),
            });
        }
        self.index += 1;
        self.prepare_step()
    }

    fn prepare_step(&mut self) -> Result<(), TutorialError> {
        let step_no = self.index + 1;
        let Some(step) = self.tutorial.steps.get(self.index) else {
            return Ok(());
        };
        for command in step.before.clone() {
            let pending = self.session.pending_players();
            let [player] = pending[..] else {
                return Err(TutorialError::Step {
                    step: step_no,
                    message: format!(
                        "`sebelum` = `{command}` butuh tepat satu pemain yang ditunggu, ada {}",
                        pending.len()
                    ),
                });
            };
            self.session
                .act(player, &command)
                .map_err(|source| TutorialError::Game {
                    step: step_no,
                    source,
                })?;
        }
        Ok(())
    }
}

/// Memutar seluruh tutorial terhadap mesin aturan asli dan melaporkan
/// langkah pertama yang tidak valid (SPEC §7.7).
pub fn validate(src: &str, cartridge: &Cartridge) -> Result<(), TutorialError> {
    let tutorial = Tutorial::from_toml(src)?;
    if tutorial.man.aturan.trim().is_empty() || tutorial.man.kontrol.trim().is_empty() {
        return Err(TutorialError::EmptyMan);
    }
    let mut run = TutorialRun::start(tutorial, cartridge)?;
    while let Some(step) = run.step().cloned() {
        let step_no = run.index() + 1;
        let fail = |message: String| TutorialError::Step {
            step: step_no,
            message,
        };
        if step.text.trim().is_empty() {
            return Err(fail("teks kosong".into()));
        }
        let Some(expected) = &step.action else {
            run.advance()?;
            continue;
        };
        let learner = run.learner();
        if !run.session().pending_players().contains(&learner) {
            return Err(fail(format!(
                "aksi `{expected}` diminta, tetapi bukan giliran pemain {learner}"
            )));
        }
        let specs = run.session().legal_actions(learner);
        if find_match(&specs, expected).is_none() {
            return Err(fail(format!(
                "aksi `{expected}` tidak ada di legal_actions"
            )));
        }
        if step.hint.as_deref().is_none_or(|h| h.trim().is_empty()) {
            return Err(fail("langkah beraksi wajib punya `petunjuk`".into()));
        }
        if step.highlight.is_empty() {
            return Err(fail(
                "langkah beraksi wajib menyorot kontrol visual (`sorot`)".into(),
            ));
        }
        for target in &step.highlight {
            if let Some(cmd) = target.strip_prefix(HIGHLIGHT_ACTION)
                && find_match(&specs, cmd).is_none()
            {
                return Err(fail(format!("sorotan `{target}` bukan aksi yang sah")));
            }
        }
        match run.submit(expected)? {
            Feedback::Correct => {}
            Feedback::Wrong { .. } => unreachable!("aksi yang diharapkan selalu benar"),
        }
    }
    Ok(())
}
