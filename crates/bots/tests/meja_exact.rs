//! RTP analitis/enumerasi tepat untuk meja tanpa keputusan pemain (SPEC §7:
//! "dihitung secara analitis atau enumerasi bila memungkinkan"). Angka yang
//! diumumkan di modul game harus sama dengan hitungan di sini (4 desimal);
//! simulasi di `meja_rtp.rs` dan `rtp.yml` memverifikasinya secara
//! independen.

use kyusin_games::meja::WagerRtp;

fn announced(list: &[WagerRtp], wager: &str) -> f64 {
    list.iter()
        .find(|w| w.wager == wager)
        .unwrap_or_else(|| panic!("taruhan {wager}"))
        .percent
}

fn close(computed: f64, list: &[WagerRtp], wager: &str) {
    let a = announced(list, wager);
    eprintln!("{wager}: hitung {computed:.6}%, diumumkan {a}%");
    assert!(
        (computed - a).abs() < 0.00006,
        "{wager}: hitung {computed:.6}% ≠ diumumkan {a}%"
    );
}

#[test]
fn dragon_tiger_from_eight_decks() {
    // Kartu kedua seperingkat dengan kartu pertama: 31 dari 415 sisa kartu.
    let tie = 31.0 / 415.0;
    let rtp = kyusin_games::dragon_tiger::RTP;
    close(100.0 * (1.0 - tie / 2.0), rtp, "dragon");
    close(100.0 * (1.0 - tie / 2.0), rtp, "tiger");
    close(100.0 * (9.0 * tie), rtp, "tie");
}

#[test]
fn casino_war_from_six_decks_always_war() {
    let n = 312.0;
    let tie = 23.0 / (n - 1.0);
    // Setelah seri pertama: 310 kartu (peringkat seri tinggal 22). Kartu
    // yang dibuang tidak mengubah peluang.
    let m = n - 2.0;
    let tie2 = (22.0 * 21.0 + 12.0 * 24.0 * 23.0) / (m * (m - 1.0));
    let war = (1.0 + tie2) / 2.0 - 2.0 * (1.0 - tie2) / 2.0;
    let rtp = kyusin_games::casino_war::RTP;
    close(100.0 * (1.0 + tie * war), rtp, "ante");
    close(100.0 * 11.0 * tie, rtp, "tie");
}

#[test]
fn andar_bahar_from_one_deck() {
    // Tiga kartu kembar di antara 51 kartu; kembar pertama di posisi k
    // (1 = Andar) dengan peluang C(51−k, 2) / C(51, 3).
    let pairs = |n: f64| n * (n - 1.0) / 2.0;
    let total = 51.0 * 50.0 * 49.0 / 6.0;
    let andar: f64 = (1..=49)
        .step_by(2)
        .map(|k| pairs(51.0 - k as f64) / total)
        .sum();
    let rtp = kyusin_games::andar_bahar::RTP;
    close(100.0 * andar * 1.9, rtp, "andar");
    close(100.0 * (1.0 - andar) * 2.0, rtp, "bahar");
}

/// Enumerasi semua urutan nilai kartu baccarat dari 8 dek penuh (nilai 0
/// untuk 10/J/Q/K: 128 kartu; nilai 1–9: masing-masing 32).
#[test]
fn baccarat_by_exact_enumeration() {
    use kyusin_games::baccarat::banker_draws;

    fn draw(counts: &mut [u32; 10], n: u32, prob: f64, drawn: &mut Vec<u8>, out: &mut [f64; 3]) {
        for v in 0..10u8 {
            let c = counts[v as usize];
            if c == 0 {
                continue;
            }
            counts[v as usize] -= 1;
            drawn.push(v);
            step(
                counts,
                n - 1,
                prob * f64::from(c) / f64::from(n),
                drawn,
                out,
            );
            drawn.pop();
            counts[v as usize] += 1;
        }
    }

    // `drawn` = p1 b1 p2 b2 [p3] [b3]; kartu ketiga pemain dicatat 10 bila
    // pemain berdiri tetapi bankir menarik.
    fn step(counts: &mut [u32; 10], n: u32, prob: f64, drawn: &mut Vec<u8>, out: &mut [f64; 3]) {
        if drawn.len() < 4 {
            return draw(counts, n, prob, drawn, out);
        }
        let p = (drawn[0] + drawn[2]) % 10;
        let b = (drawn[1] + drawn[3]) % 10;
        let finish = |p: u8, b: u8, out: &mut [f64; 3]| {
            out[if p > b {
                0
            } else if b > p {
                1
            } else {
                2
            }] += prob
        };
        if p >= 8 || b >= 8 {
            return finish(p, b, out);
        }
        match drawn.len() {
            4 if p <= 5 => draw(counts, n, prob, drawn, out),
            4 if banker_draws(b, None) => {
                drawn.push(10);
                draw(counts, n, prob, drawn, out);
                drawn.pop();
            }
            4 => finish(p, b, out),
            5 if banker_draws(b, Some(drawn[4])) => draw(counts, n, prob, drawn, out),
            5 => finish((p + drawn[4]) % 10, b, out),
            _ => {
                let p = if drawn[4] == 10 {
                    p
                } else {
                    (p + drawn[4]) % 10
                };
                finish(p, (b + drawn[5]) % 10, out)
            }
        }
    }

    let mut counts = [32u32; 10];
    counts[0] = 128;
    let mut out = [0.0; 3];
    draw(&mut counts, 416, 1.0, &mut Vec::new(), &mut out);
    let [p, b, t] = out;
    assert!((p + b + t - 1.0).abs() < 1e-9);
    let rtp = kyusin_games::baccarat::RTP;
    close(100.0 * (1.0 + p - b), rtp, "player");
    close(100.0 * (1.0 + 0.95 * b - p), rtp, "banker");
    close(100.0 * (9.0 * t), rtp, "tie");
}

#[test]
fn red_dog_from_eight_decks_with_the_raise_strategy() {
    use kyusin_bots::red_dog::RAISE_FROM;
    use kyusin_games::red_dog::{TRIPS_PAYS, spread_pays};
    let (n, c) = (416.0, 32.0);
    let mut ev = 0.0;
    for a in 2..=14u8 {
        for b in 2..=14u8 {
            let p = c / n * if a == b { c - 1.0 } else { c } / (n - 1.0);
            if a == b {
                ev += p * (c - 2.0) / (n - 2.0) * TRIPS_PAYS as f64;
                continue;
            }
            let spread = a.abs_diff(b).saturating_sub(1);
            if spread == 0 {
                continue;
            }
            let win = f64::from(spread) * c / (n - 2.0);
            let units = if spread >= RAISE_FROM { 2.0 } else { 1.0 };
            ev += p * units * (win * spread_pays(spread) as f64 - (1.0 - win));
        }
    }
    close(100.0 * (1.0 + ev), kyusin_games::red_dog::RTP, "bet");
}
