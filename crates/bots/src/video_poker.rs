//! Video Poker: pemain otomatis memakai strategi optimal tepat
//! ([`Solver`], nilai harapan terbesar untuk setiap tangan; D-072). Dipakai
//! untuk memverifikasi RTP (SPEC §7) dan sebagai strategi yang
//! didokumentasikan.

use std::sync::OnceLock;

use kyusin_core::{Seed, TurnGame};
use kyusin_games::meja::MejaView;
use kyusin_games::video_poker::optimal::Solver;
use kyusin_games::video_poker::{self, Kind, Variant, VideoPoker, VpCard};

use crate::meja::{Rtp, Wager, simulate};

/// Solver satu varian, dibangun sekali (menilai semua tangan lima kartu).
pub fn solver(variant: Variant) -> &'static Solver {
    static S: [OnceLock<Solver>; 3] = [OnceLock::new(), OnceLock::new(), OnceLock::new()];
    let i = Variant::ALL
        .iter()
        .position(|v| *v == variant)
        .expect("varian");
    S[i].get_or_init(|| Solver::new(variant))
}

/// Perintah strategi optimal untuk tangan di view.
pub fn choice(variant: Variant, view: &video_poker::View) -> String {
    if view.meja().fase != "tahan" {
        return "bet 10".into();
    }
    let cards: Vec<VpCard> = view
        .kartu
        .iter()
        .map(|c| c.parse().expect("kartu"))
        .collect();
    let hand: [VpCard; 5] = cards.try_into().expect("lima kartu");
    let mask = solver(variant).best(&hand);
    let pos: Vec<String> = (0..5)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| (i + 1).to_string())
        .collect();
    if pos.is_empty() {
        "draw".into()
    } else {
        format!("draw {}", pos.join(" "))
    }
}

fn run<V: Kind>(rounds: u64, seed: &Seed) -> Rtp {
    simulate(
        rounds,
        seed,
        10,
        |s| VideoPoker::<V>::new(Default::default(), s).expect("dek"),
        |_, v| choice(V::VARIANT, v),
    )
}

pub fn wagers() -> Vec<Wager> {
    vec![
        Wager {
            game: video_poker::JacksOrBetterGame::ID,
            rtp: video_poker::RTP_JACKS[0],
            run: run::<video_poker::Jacks>,
        },
        Wager {
            game: video_poker::DeucesWildGame::ID,
            rtp: video_poker::RTP_DEUCES[0],
            run: run::<video_poker::Deuces>,
        },
        Wager {
            game: video_poker::JokerPokerGame::ID,
            rtp: video_poker::RTP_JOKER[0],
            run: run::<video_poker::Joker>,
        },
    ]
}
