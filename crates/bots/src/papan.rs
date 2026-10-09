//! Papan taruhan M6a (Roulette, Sic Bo, Chuck-a-luck, Big Six, Fan-Tan)
//! tidak punya keputusan: pemain otomatis memasang satu taruhan terkecil
//! di satu tempat lalu memutar/melempar, untuk memverifikasi RTP tiap
//! taruhan yang diumumkan (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::meja::WagerRtp;
use kyusin_games::papan_taruhan::{Board, BoardRules};
use kyusin_games::{big_six, chuck_a_luck, fan_tan, roulette, sic_bo};

use crate::meja::{Rtp, Wager, simulate};

/// Game papan beserta daftar RTP-nya.
trait Papan: BoardRules {
    const ID: &'static str;
    const RTP: &'static [WagerRtp];
}

impl Papan for roulette::Eropa {
    const ID: &'static str = roulette::ID_EROPA;
    const RTP: &'static [WagerRtp] = roulette::RTP_EROPA;
}

impl Papan for roulette::Amerika {
    const ID: &'static str = roulette::ID_AMERIKA;
    const RTP: &'static [WagerRtp] = roulette::RTP_AMERIKA;
}

impl Papan for sic_bo::SicBoRules {
    const ID: &'static str = sic_bo::ID;
    const RTP: &'static [WagerRtp] = sic_bo::RTP;
}

impl Papan for chuck_a_luck::Chuck {
    const ID: &'static str = chuck_a_luck::ID;
    const RTP: &'static [WagerRtp] = chuck_a_luck::RTP;
}

impl Papan for big_six::Wheel {
    const ID: &'static str = big_six::ID;
    const RTP: &'static [WagerRtp] = big_six::RTP;
}

impl Papan for fan_tan::FanTanRules {
    const ID: &'static str = fan_tan::ID;
    const RTP: &'static [WagerRtp] = fan_tan::RTP;
}

/// Simulasi taruhan ke-`I` di daftar RTP game `R`.
fn run<R: Papan, const I: usize>(rounds: u64, seed: &Seed) -> Rtp {
    let spot = R::RTP[I].wager;
    let stake = R::limits(spot).min;
    simulate(
        rounds,
        seed,
        stake,
        |s| Board::<R>::new(Default::default(), s).expect("papan"),
        |_, v| {
            if v.taruhan.is_empty() {
                format!("bet {spot} {stake}")
            } else {
                R::verb().into()
            }
        },
    )
}

fn wagers_of<R: Papan>() -> Vec<Wager> {
    let runs: [fn(u64, &Seed) -> Rtp; 12] = [
        run::<R, 0>,
        run::<R, 1>,
        run::<R, 2>,
        run::<R, 3>,
        run::<R, 4>,
        run::<R, 5>,
        run::<R, 6>,
        run::<R, 7>,
        run::<R, 8>,
        run::<R, 9>,
        run::<R, 10>,
        run::<R, 11>,
    ];
    assert!(
        R::RTP.len() <= runs.len(),
        "{}: daftar RTP terlalu panjang",
        R::ID
    );
    R::RTP
        .iter()
        .zip(runs)
        .map(|(&rtp, run)| Wager {
            game: R::ID,
            rtp,
            run,
        })
        .collect()
}

pub fn wagers() -> Vec<Wager> {
    let mut out = wagers_of::<roulette::Eropa>();
    out.extend(wagers_of::<roulette::Amerika>());
    out.extend(wagers_of::<sic_bo::SicBoRules>());
    out.extend(wagers_of::<chuck_a_luck::Chuck>());
    out.extend(wagers_of::<big_six::Wheel>());
    out.extend(wagers_of::<fan_tan::FanTanRules>());
    out
}
