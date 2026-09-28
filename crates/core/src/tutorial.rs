//! Tutorial interaktif dan runner-nya (SPEC §7.6, §7.7).
//!
//! Satu berkas TOML per game. Keadaan awal = konfigurasi + seed + perintah
//! `sebelum` tiap langkah (misalnya langkah lawan), jadi tutorial selalu
//! dijalankan terhadap mesin aturan asli, bukan tiruan. Runner yang sama
//! dipakai UI dan tes CI.
//!
//! Semua teks untuk pemain berupa tabel `{ id, en }`; bahasa yang hilang
//! membuat berkas gagal dibaca, jadi CI gagal (SPEC §4, §7.6).

use serde::{Deserialize, Serialize};

use crate::action::find_match;
use crate::game::{GameError, PlayerId, Seed, Session};
use crate::i18n::{Lang, Localized, core};
use crate::registry::Cartridge;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tutorial {
    /// Id game yang diajarkan; harus sama dengan manifest.
    pub game: String,
    #[serde(rename = "judul")]
    pub title: Localized,
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
    pub aturan: Localized,
    pub kontrol: Localized,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// Teks penjelasan.
    #[serde(rename = "teks")]
    pub text: Localized,
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
    pub hint: Option<Localized>,
}

/// Prefiks target sorotan untuk tombol aksi generik.
pub const HIGHLIGHT_ACTION: &str = "aksi:";

/// Kesalahan tutorial. `Display` (bahasa Indonesia) untuk log dan CI;
/// pemain melihat [`TutorialError::message`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialError {
    Parse(String),
    WrongGame {
        expected: String,
        found: String,
    },
    BadSeed,
    NoSteps,
    EmptyMan,
    /// Tutorial tidak valid; pesannya untuk penulis tutorial (CI).
    Step {
        step: usize,
        message: String,
    },
    Game {
        step: usize,
        source: GameError,
    },
    Finished,
}

impl TutorialError {
    pub fn message(&self) -> Localized {
        let c = core();
        match self {
            TutorialError::Parse(d) => c.localized("error.tutorial_parse", &[("detail", d)]),
            TutorialError::WrongGame { expected, found } => c.localized(
                "error.tutorial_wrong_game",
                &[("expected", expected), ("found", found)],
            ),
            TutorialError::BadSeed => c.localized("error.tutorial_seed", &[]),
            TutorialError::NoSteps => c.localized("error.tutorial_no_steps", &[]),
            TutorialError::EmptyMan => c.localized("error.tutorial_empty_man", &[]),
            TutorialError::Step { step, message } => c.localized(
                "error.tutorial_step",
                &[("step", &step.to_string()), ("detail", message)],
            ),
            TutorialError::Game { step, source } => Localized::build(|lang| {
                c.text(
                    lang,
                    "error.tutorial_step",
                    &[
                        ("step", &step.to_string()),
                        ("detail", source.message().get(lang)),
                    ],
                )
            }),
            TutorialError::Finished => c.localized("error.tutorial_finished", &[]),
        }
    }
}

impl std::fmt::Display for TutorialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message().get(Lang::Id))
    }
}

impl std::error::Error for TutorialError {}

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
    Wrong { hint: Localized },
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
                hint: core().localized("tutorial.readonly", &[]),
            });
        };
        // Alias (misalnya `e2e4` untuk `e4`) juga diterima.
        if self.session.canonical(command) != self.session.canonical(expected) {
            let hint = step
                .hint
                .clone()
                .unwrap_or_else(|| core().localized("tutorial.try", &[("command", expected)]));
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

/// Kosong di salah satu bahasa.
fn blank(t: &Localized) -> bool {
    t.id.trim().is_empty() || t.en.trim().is_empty()
}

/// Memutar seluruh tutorial terhadap mesin aturan asli dan melaporkan
/// langkah pertama yang tidak valid (SPEC §7.7).
pub fn validate(src: &str, cartridge: &Cartridge) -> Result<(), TutorialError> {
    let tutorial = Tutorial::from_toml(src)?;
    if blank(&tutorial.title) || blank(&tutorial.man.aturan) || blank(&tutorial.man.kontrol) {
        return Err(TutorialError::EmptyMan);
    }
    let mut run = TutorialRun::start(tutorial, cartridge)?;
    while let Some(step) = run.step().cloned() {
        let step_no = run.index() + 1;
        let fail = |message: String| TutorialError::Step {
            step: step_no,
            message,
        };
        if blank(&step.text) {
            return Err(fail("teks kosong di salah satu bahasa".into()));
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
        if step.hint.as_ref().is_none_or(blank) {
            return Err(fail(
                "langkah beraksi wajib punya `petunjuk` dalam dua bahasa".into(),
            ));
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
