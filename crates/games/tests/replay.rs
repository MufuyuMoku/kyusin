//! Pertandingan → replay → verify (SPEC §2.5, §5.4) dengan fixture.

use kyusin_core::fair::{FairRound, commitment};
use kyusin_core::replay::VerifyStep;
use kyusin_core::{Cartridge, Human, Match, Player, PlayerId, SeatKind, Session};

fn fixture() -> Cartridge {
    kyusin_games::fixture::cartridge().unwrap()
}

/// Pemain otomatis sederhana: selalu mengambil 1 batang.
struct TakeOne;

impl Player for TakeOne {
    fn kind(&self) -> SeatKind {
        SeatKind::Bot { level: 1 }
    }
    fn decide(&mut self, _: &dyn Session, _: PlayerId) -> Option<String> {
        Some("take 1".into())
    }
}

fn fair_record() -> kyusin_core::fair::FairRecord {
    let mut r = FairRound::new("host", [1; 32], vec!["pemain".into()]);
    r.commit("pemain", commitment(&[2; 32])).unwrap();
    r.close_commits().unwrap();
    r.reveal("pemain", [2; 32]).unwrap();
    r.close_reveals().unwrap();
    r.record().unwrap()
}

fn played() -> kyusin_core::Replay {
    let c = fixture();
    let players: Vec<Box<dyn Player>> = vec![Box::new(Human), Box::new(TakeOne)];
    let mut m = Match::new(
        &c,
        serde_json::json!({ "batang": 5 }),
        fair_record(),
        players,
    )
    .unwrap();
    // Manusia belum bergerak: bot tidak boleh jalan duluan.
    assert_eq!(m.step_auto().unwrap(), None);
    m.act(0, "take 2").unwrap();
    assert_eq!(m.step_auto().unwrap().unwrap().command, "take 1");
    m.act(0, "take 2").unwrap();
    assert!(m.session().is_over());
    assert_eq!(m.step_auto().unwrap(), None);
    m.replay()
}

#[test]
fn replay_verifies_and_frames_match_moves() {
    let c = fixture();
    let replay = played();
    assert_eq!(replay.moves.len(), 3);
    assert_eq!(replay.result.as_ref().unwrap().winners, vec![0]);
    let report = replay.verify(&c);
    assert!(report.ok, "{report:?}");
    assert_eq!(report.checks.len(), 5);
    let frames = replay.frames(&c, 0).unwrap();
    assert_eq!(frames.len(), 4);
    assert_eq!(frames[0].view_data["batang"], 5);
    assert_eq!(frames[3].view_data["batang"], 0);
    assert_eq!(frames[2].last.as_ref().unwrap().seat, 1);
}

#[test]
fn replay_survives_json_round_trip() {
    let c = fixture();
    let json = serde_json::to_string(&played()).unwrap();
    let back: kyusin_core::Replay = serde_json::from_str(&json).unwrap();
    assert!(back.verify(&c).ok);
}

fn failed_step(r: &kyusin_core::Replay) -> Option<VerifyStep> {
    let report = r.verify(&fixture());
    assert!(!report.ok);
    assert!(report.error.is_some());
    report.checks.iter().find(|c| !c.ok).map(|c| c.step)
}

#[test]
fn tampering_is_detected() {
    let good = played();

    let mut moved = good.clone();
    moved.moves[0].command = "take 3".into();
    // Langkah tetap sah tetapi hasil dan keadaan akhir berubah.
    assert!(matches!(
        failed_step(&moved),
        Some(VerifyStep::Moves | VerifyStep::Result | VerifyStep::StateHash)
    ));

    let mut illegal = good.clone();
    illegal.moves[1].seat = 0;
    assert_eq!(failed_step(&illegal), Some(VerifyStep::Moves));

    let mut hash = good.clone();
    hash.state_hash = "00".repeat(32);
    assert_eq!(failed_step(&hash), Some(VerifyStep::StateHash));

    let mut seed = good.clone();
    seed.fair.seeds.insert("pemain".into(), "11".repeat(32));
    assert_eq!(failed_step(&seed), Some(VerifyStep::Commitments));

    let mut round = good;
    round.fair.round_seed = "22".repeat(32);
    assert_eq!(failed_step(&round), Some(VerifyStep::RoundSeed));
}

#[test]
fn match_rejects_wrong_player_count() {
    let players: Vec<Box<dyn Player>> = vec![Box::new(Human)];
    assert!(Match::new(&fixture(), serde_json::Value::Null, fair_record(), players).is_err());
}

/// Menunda lalu melanjutkan (SPEC §4 Rev. 9): pertandingan dibangun ulang
/// dari seed ronde + langkah tersimpan, keadaannya sama persis, dan setelah
/// selesai replay-nya tetap lolos verify.
#[test]
fn restored_match_continues_from_the_same_state() {
    let c = fixture();
    let config = serde_json::json!({ "batang": 5 });
    let players = || -> Vec<Box<dyn Player>> { vec![Box::new(Human), Box::new(TakeOne)] };
    let mut m = Match::new(&c, config.clone(), fair_record(), players()).unwrap();
    m.act(0, "take 2").unwrap();
    m.step_auto().unwrap();
    let saved = m.replay();
    assert!(saved.result.is_none());

    let mut back = Match::restore(
        &c,
        saved.config.clone(),
        saved.fair.clone(),
        players(),
        saved.moves.clone(),
    )
    .unwrap();
    assert_eq!(back.session().state_hash(), m.session().state_hash());
    assert_eq!(back.moves(), m.moves());
    back.act(0, "take 2").unwrap();
    assert!(back.session().is_over());
    let replay = back.replay();
    assert_eq!(replay.moves.len(), 3);
    assert!(replay.verify(&c).ok);

    // Langkah tersimpan yang tidak sah ditolak.
    let mut bad = saved.moves.clone();
    bad[0].command = "take 9".into();
    assert!(Match::restore(&c, saved.config, saved.fair, players(), bad).is_err());
}
