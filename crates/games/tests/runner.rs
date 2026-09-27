//! Runner tutorial terhadap fixture: tanggapan benar/salah dan penolakan
//! tutorial yang rusak.

use kyusin_core::tutorial::{Feedback, TutorialError, TutorialRun, validate};
use kyusin_core::{Cartridge, Lang, Tutorial};

fn fixture() -> Cartridge {
    kyusin_games::fixture::cartridge().unwrap()
}

const HEAD: &str = r#"
game = "fixture"
judul = { id = "uji", en = "test" }
[config]
batang = 5
[man]
aturan = { id = "a", en = "a" }
kontrol = { id = "k", en = "c" }
"#;

fn tutorial(steps: &str) -> String {
    format!("{HEAD}\n{steps}")
}

#[test]
fn wrong_action_gives_hint_and_keeps_state() {
    let c = fixture();
    let t = Tutorial::from_toml(c.tutorial_src).unwrap();
    let mut run = TutorialRun::start(t, &c).unwrap();
    run.advance().unwrap();
    let before = run.session().view_text(0, Lang::Id);
    match run.submit("take 1").unwrap() {
        Feedback::Wrong { hint } => {
            assert!(hint.id.contains("tiga"));
            assert!(hint.en.contains("three"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(run.session().view_text(0, Lang::Id), before);
    assert_eq!(run.submit("take 3").unwrap(), Feedback::Correct);
    assert_eq!(run.index(), 2);
}

#[test]
fn full_fixture_tutorial_finishes() {
    let c = fixture();
    let mut run = TutorialRun::start(Tutorial::from_toml(c.tutorial_src).unwrap(), &c).unwrap();
    run.advance().unwrap();
    run.submit("take 3").unwrap();
    run.submit("take 3").unwrap();
    run.advance().unwrap();
    assert!(run.is_finished());
    assert!(run.session().is_over());
}

#[test]
fn illegal_expected_action_fails() {
    let src = tutorial(
        r#"
[[langkah]]
teks = { id = "x", en = "x" }
aksi = "take 4"
sorot = ["aksi:take 4"]
petunjuk = { id = "p", en = "h" }
"#,
    );
    let err = validate(&src, &fixture()).unwrap_err();
    assert!(matches!(err, TutorialError::Step { step: 1, .. }), "{err}");
}

#[test]
fn illegal_before_command_fails() {
    let src = tutorial(
        r#"
[[langkah]]
teks = { id = "x", en = "x" }
sebelum = ["take 9"]
"#,
    );
    let err = validate(&src, &fixture()).unwrap_err();
    assert!(matches!(err, TutorialError::Game { step: 1, .. }), "{err}");
}

#[test]
fn action_step_needs_hint_and_highlight() {
    let no_hint = tutorial(
        r#"
[[langkah]]
teks = { id = "x", en = "x" }
aksi = "take 1"
sorot = ["aksi:take 1"]
"#,
    );
    assert!(validate(&no_hint, &fixture()).is_err());
    let no_highlight = tutorial(
        r#"
[[langkah]]
teks = { id = "x", en = "x" }
aksi = "take 1"
petunjuk = { id = "p", en = "h" }
"#,
    );
    assert!(validate(&no_highlight, &fixture()).is_err());
}

#[test]
fn highlight_must_point_at_a_legal_action() {
    let src = tutorial(
        r#"
[[langkah]]
teks = { id = "x", en = "x" }
aksi = "take 1"
sorot = ["aksi:take 7"]
petunjuk = { id = "p", en = "h" }
"#,
    );
    assert!(validate(&src, &fixture()).is_err());
}

#[test]
fn tutorial_for_another_game_is_rejected() {
    let src = tutorial("[[langkah]]\nteks = { id = \"x\", en = \"x\" }")
        .replace("\"fixture\"", "\"catur\"");
    assert!(matches!(
        validate(&src, &fixture()),
        Err(TutorialError::WrongGame { .. })
    ));
}

#[test]
fn action_when_not_learners_turn_fails() {
    let src = tutorial(
        r#"
[[langkah]]
teks = { id = "x", en = "x" }
aksi = "take 1"
sorot = ["aksi:take 1"]
petunjuk = { id = "p", en = "h" }

[[langkah]]
teks = { id = "y", en = "y" }
aksi = "take 1"
sorot = ["aksi:take 1"]
petunjuk = { id = "p", en = "h" }
"#,
    );
    let err = validate(&src, &fixture()).unwrap_err();
    assert!(matches!(err, TutorialError::Step { step: 2, .. }), "{err}");
}

#[test]
fn missing_english_text_fails() {
    let src = tutorial(
        r#"
[[langkah]]
teks = { id = "hanya indonesia" }
"#,
    );
    assert!(matches!(
        validate(&src, &fixture()),
        Err(TutorialError::Parse(_))
    ));
}

#[test]
fn blank_translation_fails() {
    let src = tutorial(
        r#"
[[langkah]]
teks = { id = "ada", en = "  " }
"#,
    );
    assert!(validate(&src, &fixture()).is_err());
}
