//! Strategi optimal Video Poker dengan nilai harapan tepat (D-072).
//!
//! Untuk setiap himpunan kartu H (0–5 kartu), `sum[H]` = jumlah bayaran
//! semua tangan akhir lima kartu yang memuat H. Nilai harapan menahan H dari
//! tangan D lalu mengganti sisanya: tangan akhir harus memuat H dan tidak
//! memuat kartu yang dibuang R = D \ H, jadi dengan inklusi–eksklusi
//!
//!   jumlah = Σ_{T ⊆ R} (−1)^|T| · sum[H ∪ T],
//!
//! dibagi banyaknya kemungkinan pengganti C(n − 5, 5 − |H|). Membangun tabel
//! = satu kali menilai semua C(n, 5) tangan; satu keputusan = 243 pencarian.
//! RTP tepat = rata-rata EV terbaik atas semua tangan awal ([`Solver::exact`],
//! berat; hanya di GitHub Actions).

use super::{Variant, VpCard, payout};

/// Koefisien binomial kecil.
fn binom(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    (0..k).fold(1, |acc, i| acc * (n - i) / (i + 1))
}

fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

pub struct Solver {
    variant: Variant,
    n: usize,
    /// `choose[c][k]` = C(c, k) untuk indeks kombinasi.
    choose: Vec<[u64; 6]>,
    /// `sum[k][indeks]` untuk himpunan berukuran k.
    sum: [Vec<u64>; 6],
}

impl Solver {
    pub fn new(variant: Variant) -> Solver {
        let n = variant.deck().len();
        let choose: Vec<[u64; 6]> = (0..=n as u64)
            .map(|c| std::array::from_fn(|k| binom(c, k as u64)))
            .collect();
        let mut sum: [Vec<u64>; 6] =
            std::array::from_fn(|k| vec![0u64; binom(n as u64, k as u64) as usize]);
        let n8 = n as u8;
        for a in 0..n8 {
            for b in a + 1..n8 {
                for c in b + 1..n8 {
                    for d in c + 1..n8 {
                        for e in d + 1..n8 {
                            let cards = [a, b, c, d, e];
                            let hand = cards.map(VpCard);
                            let pay = u64::from(payout(variant, &hand));
                            if pay == 0 {
                                continue;
                            }
                            for mask in 0u8..32 {
                                let (k, idx) = index(&choose, &cards, mask);
                                sum[k][idx] += pay;
                            }
                        }
                    }
                }
            }
        }
        Solver {
            variant,
            n,
            choose,
            sum,
        }
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    /// Jumlah bayaran dan pembagi untuk menahan `mask` (bit i = posisi i).
    fn raw(&self, subsets: &[(usize, usize); 32], mask: u8) -> (u64, u64) {
        let discarded = !mask & 0b11111;
        let mut total: i64 = 0;
        // Semua T ⊆ discarded: U = mask | T.
        let mut t = discarded;
        loop {
            let (k, idx) = subsets[(mask | t) as usize];
            let s = self.sum[k][idx] as i64;
            if t.count_ones().is_multiple_of(2) {
                total += s;
            } else {
                total -= s;
            }
            if t == 0 {
                break;
            }
            t = (t - 1) & discarded;
        }
        let held = mask.count_ones() as u64;
        (total as u64, binom(self.n as u64 - 5, 5 - held))
    }

    /// Indeks semua 32 himpunan bagian posisi tangan ini.
    fn subsets(&self, hand: &[VpCard; 5]) -> [(usize, usize); 32] {
        // Urutkan kartu; bit posisi dipetakan ke urutan terurut.
        let mut order: [usize; 5] = [0, 1, 2, 3, 4];
        order.sort_by_key(|&i| hand[i].0);
        let sorted: [u8; 5] = order.map(|i| hand[i].0);
        std::array::from_fn(|m| {
            let mut sm = 0u8;
            for (j, &pos) in order.iter().enumerate() {
                if m & (1 << pos) != 0 {
                    sm |= 1 << j;
                }
            }
            index(&self.choose, &sorted, sm)
        })
    }

    /// Nilai harapan tepat menahan `mask` (bit i = kartu ke-i), sebagai
    /// pecahan tersederhana (pembilang, penyebut) per 1 taruhan.
    pub fn ev(&self, hand: &[VpCard; 5], mask: u8) -> (u64, u64) {
        let subs = self.subsets(hand);
        let (num, den) = self.raw(&subs, mask);
        let g = gcd(u128::from(num), u128::from(den)).max(1) as u64;
        (num / g, den / g)
    }

    /// Pilihan tahan dengan nilai harapan terbesar (seri: menahan lebih
    /// banyak kartu, lalu bitmask terbesar).
    pub fn best(&self, hand: &[VpCard; 5]) -> u8 {
        self.best_ev(hand).0
    }

    fn best_ev(&self, hand: &[VpCard; 5]) -> (u8, u64, u64) {
        let subs = self.subsets(hand);
        let mut best = (31u8, 0u64, 1u64);
        let mut first = true;
        let mut masks: Vec<u8> = (0..32).collect();
        masks.sort_by_key(|m| std::cmp::Reverse((m.count_ones(), *m)));
        for m in masks {
            let (num, den) = self.raw(&subs, m);
            if first || u128::from(num) * u128::from(best.2) > u128::from(best.1) * u128::from(den)
            {
                best = (m, num, den);
                first = false;
            }
        }
        best
    }

    /// RTP tepat strategi optimal: (persen, pembilang, penyebut) dengan
    /// penyebut = KPK pembagi × C(n, 5). Berat (semua tangan awal); dibagi
    /// ke `threads` utas.
    pub fn exact(&self, threads: usize) -> (f64, u128, u128) {
        let n = self.n as u64;
        let lcm = (0..=5u64).fold(1u128, |l, k| {
            let d = u128::from(binom(n - 5, 5 - k));
            l / gcd(l, d) * d
        });
        let threads = threads.max(1);
        let n8 = self.n as u8;
        let total: u128 = std::thread::scope(|scope| {
            let parts: Vec<_> = (0..threads)
                .map(|t| {
                    scope.spawn(move || {
                        let mut acc: u128 = 0;
                        for a in (0..n8).skip(t).step_by(threads) {
                            for b in a + 1..n8 {
                                for c in b + 1..n8 {
                                    for d in c + 1..n8 {
                                        for e in d + 1..n8 {
                                            let hand = [a, b, c, d, e].map(VpCard);
                                            let (_, num, den) = self.best_ev(&hand);
                                            acc += u128::from(num) * (lcm / u128::from(den));
                                        }
                                    }
                                }
                            }
                        }
                        acc
                    })
                })
                .collect();
            parts.into_iter().map(|h| h.join().expect("utas RTP")).sum()
        });
        let den = lcm * u128::from(binom(n, 5));
        let g = gcd(total, den);
        (100.0 * total as f64 / den as f64, total / g, den / g)
    }
}

/// (ukuran, indeks kombinasi) himpunan kartu `cards` (terurut naik) yang
/// dipilih `mask`.
fn index(choose: &[[u64; 6]], cards: &[u8; 5], mask: u8) -> (usize, usize) {
    let mut k = 0usize;
    let mut idx = 0u64;
    for (i, c) in cards.iter().enumerate() {
        if mask & (1 << i) != 0 {
            k += 1;
            idx += choose[*c as usize][k];
        }
    }
    (k, idx as usize)
}
