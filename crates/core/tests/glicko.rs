//! Tes Glicko-2 (SPEC §8), ditulis sebelum implementasinya.
//!
//! Rujukan: Mark E. Glickman, "Example of the Glicko-2 system" (2013).
//! Contoh baku: pemain 1500 / RD 200 / volatilitas 0,06, τ = 0,5, melawan
//! 1400/30 (menang), 1550/100 (kalah), 1700/300 (kalah) dalam satu periode
//! → 1464,06 / RD 151,52 / volatilitas 0,05999.
//!
//! Di KyuSin satu pertandingan = satu periode rating; lawan bot punya
//! rating dan RD tetap dari kalibrasi (rating lokal).

use kyusin_core::rating::{Outcome, Rating, TAU};

fn close(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

fn opp(rating: f64, rd: f64) -> Rating {
    Rating {
        rating,
        rd,
        vol: 0.06,
    }
}

#[test]
fn glickman_reference_example() {
    let player = Rating {
        rating: 1500.0,
        rd: 200.0,
        vol: 0.06,
    };
    let next = player.update(
        &[
            (opp(1400.0, 30.0), Outcome::Win),
            (opp(1550.0, 100.0), Outcome::Loss),
            (opp(1700.0, 300.0), Outcome::Loss),
        ],
        TAU,
    );
    assert!(close(next.rating, 1464.06, 0.01), "{next:?}");
    assert!(close(next.rd, 151.52, 0.01), "{next:?}");
    assert!(close(next.vol, 0.05999, 0.00001), "{next:?}");
}

#[test]
fn new_player_defaults() {
    let p = Rating::new_player();
    assert_eq!((p.rating, p.rd, p.vol), (1500.0, 350.0, 0.06));
}

#[test]
fn win_raises_loss_lowers_and_rd_shrinks() {
    let p = Rating::new_player();
    let o = opp(1500.0, 50.0);
    let win = p.update(&[(o, Outcome::Win)], TAU);
    let loss = p.update(&[(o, Outcome::Loss)], TAU);
    let draw = p.update(&[(o, Outcome::Draw)], TAU);
    assert!(win.rating > p.rating && loss.rating < p.rating);
    assert!(close(draw.rating, p.rating, 0.001), "{draw:?}");
    for r in [win, loss, draw] {
        assert!(r.rd < p.rd, "{r:?}");
    }
    // Simetris terhadap lawan yang setara.
    assert!(close(win.rating - p.rating, p.rating - loss.rating, 0.001));
}

#[test]
fn beating_a_stronger_opponent_gains_more() {
    let p = Rating {
        rating: 1500.0,
        rd: 100.0,
        vol: 0.06,
    };
    let weak = p.update(&[(opp(1200.0, 50.0), Outcome::Win)], TAU);
    let strong = p.update(&[(opp(1800.0, 50.0), Outcome::Win)], TAU);
    assert!(strong.rating - p.rating > weak.rating - p.rating);
    // Kalah dari yang jauh lebih lemah menurunkan lebih banyak.
    let upset = p.update(&[(opp(1200.0, 50.0), Outcome::Loss)], TAU);
    let expected = p.update(&[(opp(1800.0, 50.0), Outcome::Loss)], TAU);
    assert!(p.rating - upset.rating > p.rating - expected.rating);
}

#[test]
fn uncertain_ratings_move_more() {
    let settled = Rating {
        rating: 1500.0,
        rd: 60.0,
        vol: 0.06,
    };
    let fresh = Rating::new_player();
    let o = opp(1500.0, 50.0);
    let a = settled.update(&[(o, Outcome::Win)], TAU);
    let b = fresh.update(&[(o, Outcome::Win)], TAU);
    assert!(b.rating - fresh.rating > a.rating - settled.rating);
}

#[test]
fn outcome_scores() {
    assert_eq!(Outcome::Win.score(), 1.0);
    assert_eq!(Outcome::Draw.score(), 0.5);
    assert_eq!(Outcome::Loss.score(), 0.0);
}

#[test]
fn long_series_stays_finite_and_converges() {
    // Pemain yang selalu seri dengan lawan 1600 mendekati 1600, RD menyusut.
    let mut p = Rating::new_player();
    for _ in 0..200 {
        p = p.update(&[(opp(1600.0, 40.0), Outcome::Draw)], TAU);
        assert!(p.rating.is_finite() && p.rd.is_finite() && p.vol.is_finite());
    }
    assert!(close(p.rating, 1600.0, 15.0), "{p:?}");
    assert!(p.rd < 80.0, "{p:?}");
}
