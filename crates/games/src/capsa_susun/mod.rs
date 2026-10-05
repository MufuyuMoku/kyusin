//! Capsa Susun (SPEC §6.3; M5b-2): 13 kartu per pemain disusun menjadi tiga
//! baris (depan 3, tengah 5, belakang 5) lalu dibandingkan dengan setiap
//! lawan. Aturan lokal dipilih klien (D-065).
//!
//! Satu pertandingan = satu sesi meja seperti poker KyuSin (D-063): buy-in
//! dari saldo saat duduk, tumpukan kembali saat berdiri, kartu tiap tangan
//! dikocok dari seed sesi dan nomor tangan. Semua kursi menyusun serentak;
//! susunan yang tidak sah (depan ≤ tengah ≤ belakang dilanggar) ditolak.
//! Baris depan hanya mengenal kartu tinggi, pair, dan three of a kind;
//! A-2-3-4-5 straight terendah.
//!
//! Poin per pasangan pemain: 1 per baris (seri persis 0); menang ketiga
//! baris (sapu bersih) = poin baris dikali dua; royalti untuk baris yang
//! menang. Kartu istimewa diumumkan otomatis dan langsung menang tanpa
//! disusun. 1 poin = [`POINT`] chip; yang kalah membayar paling banyak
//! tumpukannya, dan kekurangannya mengurangi bagian para pemenang secara
//! proporsional.

use std::cmp::Ordering;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, Umum};
use crate::poker::{Category, Value, eval5, rank_value};

pub const ID: &str = "capsa-susun";
/// Chip per poin.
pub const POINT: i64 = 10;
pub const DEFAULT_SEATS: u8 = 4;
pub const MAX_SEATS: u8 = 4;
pub const DEFAULT_STACK: i64 = 2000;
pub const CARDS: usize = 13;

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("capsa_susun/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/capsa-susun.toml"),
        create_session::<Capsa>,
    )
}

/// Nilai baris depan (tiga kartu): kartu tinggi, pair, atau three of a
/// kind, sebanding dengan nilai baris lima kartu.
pub fn front_value(cards: &[Card]) -> Value {
    assert_eq!(cards.len(), 3, "baris depan tiga kartu");
    let mut r: Vec<u8> = cards.iter().map(|c| rank_value(c.rank)).collect();
    r.sort_unstable_by(|a, b| b.cmp(a));
    if r[0] == r[2] {
        Value::from_parts(Category::Trips, &[r[0]])
    } else if r[0] == r[1] {
        Value::from_parts(Category::Pair, &[r[0], r[2]])
    } else if r[1] == r[2] {
        Value::from_parts(Category::Pair, &[r[1], r[0]])
    } else {
        Value::from_parts(Category::HighCard, &r)
    }
}

/// Nilai baris tengah atau belakang (lima kartu).
pub fn row_value(cards: &[Card]) -> Value {
    eval5(cards)
}

/// Susunan sah: depan ≤ tengah ≤ belakang.
pub fn valid(front: &[Card], mid: &[Card], back: &[Card]) -> bool {
    front.len() == 3
        && mid.len() == 5
        && back.len() == 5
        && front_value(front) <= row_value(mid)
        && row_value(mid) <= row_value(back)
}

/// Kartu istimewa, dari terendah.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Special {
    TigaStraight,
    TigaFlush,
    EnamPasang,
    Naga,
}

impl Special {
    pub fn points(self) -> i64 {
        match self {
            Special::Naga => 13,
            _ => 3,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Special::TigaStraight => "tiga_straight",
            Special::TigaFlush => "tiga_flush",
            Special::EnamPasang => "enam_pasang",
            Special::Naga => "naga",
        }
    }
}

/// Jumlah kartu per peringkat; indeks 1 = as rendah (sama dengan 14).
fn rank_counts(cards: &[Card]) -> [u8; 15] {
    let mut c = [0u8; 15];
    for card in cards {
        c[rank_value(card.rank) as usize] += 1;
    }
    c[1] = c[14];
    c
}

/// Bisakah `counts` dipecah menjadi straight dengan panjang `sizes`.
fn straights(counts: &mut [u8; 15], sizes: &[usize]) -> bool {
    let Some((&len, rest)) = sizes.split_first() else {
        return counts[2..].iter().all(|&n| n == 0);
    };
    for low in 1..=(15 - len) {
        let ranks: Vec<usize> = (low..low + len)
            .map(|r| if r == 1 { 14 } else { r })
            .collect();
        if ranks.iter().any(|&r| r > 14) || ranks.iter().any(|&r| counts[r] == 0) {
            continue;
        }
        for &r in &ranks {
            counts[r] -= 1;
        }
        counts[1] = counts[14];
        let ok = straights(counts, rest);
        for &r in &ranks {
            counts[r] += 1;
        }
        counts[1] = counts[14];
        if ok {
            return true;
        }
    }
    false
}

/// Kartu istimewa tertinggi dari 13 kartu, bila ada.
pub fn special(cards: &[Card]) -> Option<Special> {
    let counts = rank_counts(cards);
    if counts[2..].iter().all(|&n| n == 1) {
        return Some(Special::Naga);
    }
    if counts[2..].iter().map(|&n| n / 2).sum::<u8>() >= 6 {
        return Some(Special::EnamPasang);
    }
    let mut suits = [0usize; 4];
    for c in cards {
        suits[c.suit as usize] += 1;
    }
    let sizes = [3usize, 5, 5];
    let flush = (0..64usize).any(|code| {
        let mut need = [0usize; 4];
        for (i, &size) in sizes.iter().enumerate() {
            need[(code >> (2 * i)) & 3] += size;
        }
        need == suits
    });
    if flush {
        return Some(Special::TigaFlush);
    }
    let mut counts = counts;
    if straights(&mut counts, &[5, 5, 3]) {
        return Some(Special::TigaStraight);
    }
    None
}

/// Tangan yang dibandingkan: susunan tiga baris atau kartu istimewa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hand {
    Normal {
        front: Vec<Card>,
        mid: Vec<Card>,
        back: Vec<Card>,
    },
    Special(Special),
}

/// Royalti baris `row` (0 depan, 1 tengah, 2 belakang) bernilai `v`.
pub fn royalty(row: usize, v: Value) -> i64 {
    match (row, v.category()) {
        (0, Category::Trips) => 3,
        (1, Category::FullHouse) => 2,
        (1, Category::Quads) => 8,
        (1, Category::StraightFlush) => 10,
        (2, Category::Quads) => 4,
        (2, Category::StraightFlush) => 5,
        _ => 0,
    }
}

fn values(front: &[Card], mid: &[Card], back: &[Card]) -> [Value; 3] {
    [front_value(front), row_value(mid), row_value(back)]
}

/// Poin yang didapat `a` dari `b` (negatif = `a` membayar).
pub fn versus(a: &Hand, b: &Hand) -> i64 {
    match (a, b) {
        (Hand::Special(x), Hand::Special(y)) => match x.cmp(y) {
            Ordering::Greater => x.points(),
            Ordering::Less => -y.points(),
            Ordering::Equal => 0,
        },
        (Hand::Special(x), _) => x.points(),
        (_, Hand::Special(y)) => -y.points(),
        (
            Hand::Normal {
                front: af,
                mid: am,
                back: ab,
            },
            Hand::Normal {
                front: bf,
                mid: bm,
                back: bb,
            },
        ) => {
            let (va, vb) = (values(af, am, ab), values(bf, bm, bb));
            let (mut wins, mut losses, mut bonus) = (0i64, 0i64, 0i64);
            for row in 0..3 {
                match va[row].cmp(&vb[row]) {
                    Ordering::Greater => {
                        wins += 1;
                        bonus += royalty(row, va[row]);
                    }
                    Ordering::Less => {
                        losses += 1;
                        bonus -= royalty(row, vb[row]);
                    }
                    Ordering::Equal => {}
                }
            }
            let base = if wins == 3 || losses == 3 {
                2 * (wins - losses)
            } else {
                wins - losses
            };
            base + bonus
        }
    }
}

/// Perubahan tumpukan dari poin antar-pasangan `p[i][j]` (poin yang didapat
/// `i` dari `j`). Yang kalah bersih membayar paling banyak tumpukannya;
/// bila ada kekurangan, para pemenang menerima bagian proporsional dengan
/// klaimnya (bulat ke bawah, sisanya satu per satu ke klaim terbesar lalu
/// kursi terkecil). Jumlahnya selalu nol.
pub fn settle(p: &[Vec<i64>], stacks: &[i64]) -> Vec<i64> {
    let net: Vec<i64> = p
        .iter()
        .map(|row| row.iter().sum::<i64>() * POINT)
        .collect();
    let mut out = vec![0i64; net.len()];
    let mut paid = 0i64;
    for (i, &n) in net.iter().enumerate() {
        if n < 0 {
            let pay = (-n).min(stacks[i]);
            out[i] = -pay;
            paid += pay;
        }
    }
    let claims: i64 = net.iter().filter(|&&n| n > 0).sum();
    if claims == 0 {
        return out;
    }
    let mut given = 0i64;
    for (i, &n) in net.iter().enumerate() {
        if n > 0 {
            let share = if paid >= claims { n } else { paid * n / claims };
            out[i] = share;
            given += share;
        }
    }
    let mut order: Vec<usize> = (0..net.len()).filter(|&i| net[i] > 0).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(net[i]), i));
    let mut left = paid.min(claims) - given;
    while left > 0 {
        for &i in &order {
            if left == 0 {
                break;
            }
            out[i] += 1;
            left -= 1;
        }
    }
    out
}

fn by_rank(cards: &mut [Card]) {
    cards.sort_by(|a, b| {
        rank_value(b.rank)
            .cmp(&rank_value(a.rank))
            .then(a.suit.cmp(&b.suit))
    });
}

/// Semua kombinasi `k` indeks dari `0..n`.
fn combos(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        let Some(i) = (0..k).rev().find(|&i| idx[i] != i + n - k) else {
            return out;
        };
        idx[i] += 1;
        for j in i + 1..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

/// Susunan saran (sederhana dan selalu sah): belakang = lima kartu
/// terbaik, tengah = lima terbaik dari sisanya yang tetap sah, depan =
/// sisanya. Urutan hasil: depan 3, tengah 5, belakang 5.
pub fn suggest(cards: &[Card]) -> Vec<Card> {
    assert_eq!(cards.len(), CARDS, "13 kartu");
    let pick = |from: &[Card], idx: &[usize]| -> (Vec<Card>, Vec<Card>) {
        let chosen: Vec<Card> = idx.iter().map(|&i| from[i]).collect();
        let rest: Vec<Card> = (0..from.len())
            .filter(|i| !idx.contains(i))
            .map(|i| from[i])
            .collect();
        (chosen, rest)
    };
    let mut backs: Vec<(Value, Vec<Card>, Vec<Card>)> = combos(13, 5)
        .iter()
        .map(|idx| {
            let (b, rest) = pick(cards, idx);
            (row_value(&b), b, rest)
        })
        .collect();
    backs.sort_by_key(|b| std::cmp::Reverse(b.0));
    for (bv, back, rest) in backs {
        let mut best: Option<(Value, Vec<Card>, Vec<Card>)> = None;
        for idx in combos(8, 5) {
            let (mid, front) = pick(&rest, &idx);
            let mv = row_value(&mid);
            if mv <= bv && front_value(&front) <= mv && best.as_ref().is_none_or(|b| mv > b.0) {
                best = Some((mv, mid, front));
            }
        }
        if let Some((_, mut mid, mut front)) = best {
            let mut back = back;
            by_rank(&mut front);
            by_rank(&mut mid);
            by_rank(&mut back);
            return front.into_iter().chain(mid).chain(back).collect();
        }
    }
    unreachable!("selalu ada susunan sah")
}

/// Konfigurasi meja. Semua opsional; host mengisi `tumpukan` (buy-in
/// manusia dari saldo, bot 2.000) dan `manusia`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Jumlah kursi (2–4; bawaan 4).
    #[serde(default)]
    pub kursi: Option<u8>,
    #[serde(default)]
    pub tumpukan: Option<Vec<i64>>,
    #[serde(default)]
    pub manusia: Option<Vec<u8>>,
    #[serde(default)]
    pub dealer: Option<u8>,
    #[serde(default)]
    pub tangan_maks: Option<u32>,
    /// Urutan kartu tetap untuk tangan-tangan pertama (tes dan tutorial).
    #[serde(default)]
    pub dek: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Tiga belas kartu: depan 3, tengah 5, belakang 5.
    Arrange(Vec<Card>),
    Auto,
    Next,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Arrange,
    Between,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Seat {
    start: i64,
    stack: i64,
    cards: Vec<Card>,
    /// Susunan yang dikirim (depan, tengah, belakang berurutan).
    rows: Option<Vec<Card>>,
    special: Option<Special>,
    in_hand: bool,
    out: bool,
    left: bool,
    ready: bool,
    revealed: bool,
    points: i64,
    won: i64,
}

impl Seat {
    fn new(stack: i64) -> Seat {
        Seat {
            start: stack,
            stack,
            cards: Vec::new(),
            rows: None,
            special: None,
            in_hand: false,
            out: stack <= 0,
            left: false,
            ready: false,
            revealed: false,
            points: 0,
            won: 0,
        }
    }

    fn seated(&self) -> bool {
        !self.out && !self.left
    }

    fn arranging(&self) -> bool {
        self.in_hand && self.rows.is_none() && self.special.is_none()
    }

    fn hand(&self) -> Hand {
        match (self.special, &self.rows) {
            (Some(s), _) => Hand::Special(s),
            (None, Some(r)) => Hand::Normal {
                front: r[..3].to_vec(),
                mid: r[3..8].to_vec(),
                back: r[8..].to_vec(),
            },
            (None, None) => unreachable!("kursi belum menyusun"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Capsa {
    seed: Seed,
    seats: Vec<Seat>,
    humans: Vec<u8>,
    max_hands: Option<u32>,
    fixed: Vec<Vec<Card>>,
    phase: Phase,
    hand: u32,
    dealer: u8,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    pub tumpukan: i64,
    pub awal: i64,
    /// `menyusun`, `siap`, `duduk`, `habis`, atau `berdiri`.
    pub status: String,
    /// Kartu sendiri (terurut), kartu yang dibuka (depan, tengah,
    /// belakang), atau `??`.
    pub kartu: Vec<String>,
    /// Tiga baris yang dibuka.
    pub baris: Option<Vec<Vec<String>>>,
    /// Jenis tangan tiap baris yang dibuka.
    pub nama_baris: Option<Vec<String>>,
    pub istimewa: Option<String>,
    pub siap: bool,
    /// Poin tangan terakhir (jumlah dari semua lawan).
    pub poin: i64,
    /// Perubahan tumpukan tangan terakhir.
    pub menang: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    pub kamu: u8,
    pub kursi: Vec<SeatView>,
    pub dealer: u8,
    pub tangan_ke: u32,
    pub poin_chip: i64,
    /// Susunan saran untuk kursi yang melihat, selama ia menyusun.
    pub saran: Option<Vec<String>>,
}

impl meja::MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl Capsa {
    fn n(&self) -> u8 {
        self.seats.len() as u8
    }

    fn next_seat(&self, from: u8, ok: impl Fn(&Seat) -> bool) -> Option<u8> {
        let n = self.n();
        (1..=n)
            .map(|i| (from + i) % n)
            .find(|&s| ok(&self.seats[s as usize]))
    }

    fn hand_deck(&self) -> Shoe {
        match self.fixed.get((self.hand - 1) as usize) {
            Some(cards) => Shoe::from_cards(cards.clone()),
            None => Shoe::shuffled(
                1,
                &mut GameRng::from_seed(derive(&self.seed, &format!("tangan:{}", self.hand))),
            ),
        }
    }

    fn start_hand(&mut self) -> Result<(), GameError> {
        let players = self
            .seats
            .iter()
            .filter(|s| s.seated() && s.stack > 0)
            .count();
        if players < 2 {
            self.finish_session("meja_bubar");
            return Ok(());
        }
        self.hand += 1;
        if self.hand > 1 || !self.seats[self.dealer as usize].seated() {
            self.dealer = self
                .next_seat(self.dealer, |s| s.seated() && s.stack > 0)
                .expect("ada pemain");
        }
        for s in &mut self.seats {
            s.cards.clear();
            s.rows = None;
            s.special = None;
            s.in_hand = s.seated() && s.stack > 0;
            s.ready = false;
            s.revealed = false;
            s.points = 0;
            s.won = 0;
        }
        let mut deck = self.hand_deck();
        let mut s = self.dealer;
        for _ in 0..CARDS {
            for _ in 0..players {
                s = self.next_seat(s, |x| x.in_hand).expect("pemain");
                let c = meja::draw(&mut deck)?;
                self.seats[s as usize].cards.push(c);
            }
        }
        for seat in &mut self.seats {
            by_rank(&mut seat.cards);
            if seat.in_hand {
                seat.special = special(&seat.cards);
            }
        }
        self.phase = Phase::Arrange;
        self.resolve_if_ready();
        Ok(())
    }

    fn resolve_if_ready(&mut self) {
        if self.phase != Phase::Arrange || self.seats.iter().any(Seat::arranging) {
            return;
        }
        let playing: Vec<usize> = (0..self.seats.len())
            .filter(|&i| self.seats[i].in_hand)
            .collect();
        let hands: Vec<Option<Hand>> = self
            .seats
            .iter()
            .map(|s| s.in_hand.then(|| s.hand()))
            .collect();
        let n = self.seats.len();
        let mut p = vec![vec![0i64; n]; n];
        for &i in &playing {
            for &j in &playing {
                if i != j {
                    p[i][j] = versus(
                        hands[i].as_ref().expect("bermain"),
                        hands[j].as_ref().expect("bermain"),
                    );
                }
            }
        }
        let stacks: Vec<i64> = self.seats.iter().map(|s| s.stack).collect();
        let delta = settle(&p, &stacks);
        for (i, s) in self.seats.iter_mut().enumerate() {
            s.points = p[i].iter().sum();
            s.won = delta[i];
            s.stack += delta[i];
            s.revealed = s.in_hand;
            if s.stack == 0 {
                s.out = true;
            }
        }
        self.phase = Phase::Between;
        self.check_session();
    }

    fn check_session(&mut self) {
        let humans_gone = self
            .humans
            .iter()
            .all(|&h| !self.seats.get(h as usize).is_some_and(Seat::seated));
        let with_chips = self.seats.iter().filter(|s| s.seated()).count();
        if humans_gone {
            self.finish_session("berdiri");
        } else if with_chips < 2 {
            self.finish_session("meja_bubar");
        } else if self.max_hands.is_some_and(|m| self.hand >= m) {
            self.finish_session("batas_tangan");
        }
    }

    fn finish_session(&mut self, reason: &'static str) {
        self.phase = Phase::Over;
        self.ended = Some(reason);
    }

    fn arrange(&mut self, player: u8, rows: Vec<Card>) -> Result<(), GameError> {
        let seat = &self.seats[player as usize];
        let text = || {
            GameError::Illegal(format!(
                "arrange {}",
                rows.iter()
                    .map(Card::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        };
        let mut a = rows.clone();
        let mut b = seat.cards.clone();
        a.sort();
        b.sort();
        if a != b || !valid(&rows[..3], &rows[3..8], &rows[8..]) {
            return Err(text());
        }
        self.seats[player as usize].rows = Some(rows);
        Ok(())
    }
}

impl TurnGame for Capsa {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let n = config.kursi.unwrap_or(DEFAULT_SEATS);
        if !(2..=MAX_SEATS).contains(&n) {
            return Err(GameError::Config(format!("kursi {n}")));
        }
        let stacks = config
            .tumpukan
            .clone()
            .unwrap_or_else(|| vec![DEFAULT_STACK; n as usize]);
        if stacks.len() != n as usize || stacks.iter().any(|&s| s < 0) {
            return Err(GameError::Config("tumpukan".into()));
        }
        let humans = config.manusia.clone().unwrap_or_else(|| vec![0]);
        if humans.iter().any(|&h| h >= n) {
            return Err(GameError::Config("manusia".into()));
        }
        let fixed = config
            .dek
            .as_ref()
            .map(|decks| decks.iter().map(|d| meja::parse_cards(d)).collect())
            .transpose()?
            .unwrap_or_default();
        let dealer = match config.dealer {
            Some(d) if d < n => d,
            Some(d) => return Err(GameError::Config(format!("dealer {d}"))),
            None => GameRng::from_seed(derive(&seed, "dealer")).below(u32::from(n)) as u8,
        };
        let mut table = Capsa {
            seed,
            seats: stacks.into_iter().map(Seat::new).collect(),
            humans,
            max_hands: config.tangan_maks,
            fixed,
            phase: Phase::Between,
            hand: 0,
            dealer,
            ended: None,
        };
        table.start_hand()?;
        Ok(table)
    }

    fn seats(&self) -> u8 {
        self.n()
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        match self.phase {
            Phase::Arrange => (0..self.n())
                .filter(|&s| self.seats[s as usize].arranging())
                .collect(),
            Phase::Between => (0..self.n())
                .filter(|&s| self.seats[s as usize].seated() && !self.seats[s as usize].ready)
                .collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if !self.pending_players().contains(&player) {
            return Vec::new();
        }
        match self.phase {
            Phase::Arrange => {
                let options: Vec<String> = self.seats[player as usize]
                    .cards
                    .iter()
                    .map(Card::to_string)
                    .collect();
                let card = || Param {
                    name: "kartu".into(),
                    kind: ParamKind::Choice {
                        options: options.clone(),
                    },
                };
                vec![
                    ActionSpec::template("arrange", (0..CARDS).map(|_| card()).collect()),
                    ActionSpec::fixed("auto"),
                ]
            }
            Phase::Between => vec![ActionSpec::fixed("next"), ActionSpec::fixed("leave")],
            Phase::Over => Vec::new(),
        }
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::Over {
            return Err(GameError::Over);
        }
        if !self.pending_players().contains(&player) {
            return Err(GameError::NotPending(player));
        }
        match (self.phase, action) {
            (Phase::Arrange, Action::Arrange(rows)) if rows.len() == CARDS => {
                self.arrange(player, rows)?;
                self.resolve_if_ready();
            }
            (Phase::Arrange, Action::Auto) => {
                let rows = suggest(&self.seats[player as usize].cards);
                self.arrange(player, rows)?;
                self.resolve_if_ready();
            }
            (Phase::Between, Action::Next) => {
                self.seats[player as usize].ready = true;
                if self.pending_players().is_empty() {
                    self.start_hand()?;
                }
            }
            (Phase::Between, Action::Leave) => {
                self.seats[player as usize].left = true;
                self.check_session();
                if self.phase == Phase::Between && self.pending_players().is_empty() {
                    self.start_hand()?;
                }
            }
            (_, a) => return Err(GameError::Illegal(self.format_action(&a))),
        }
        Ok(())
    }

    fn view_for(&self, player: PlayerId) -> View {
        let me = self.seats.get(player as usize);
        let over = self.phase == Phase::Over;
        let fase = match self.phase {
            Phase::Arrange => "susun",
            Phase::Between => "antara",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(
            fase,
            meja::Limits {
                min: POINT,
                max: me.map(|s| s.stack).unwrap_or(0),
                step: POINT,
            },
        );
        let gone = over || me.is_some_and(|s| s.left);
        meja.taruhan_meja = if gone {
            0
        } else {
            me.map(|s| s.start).unwrap_or(0)
        };
        meja.bersih = if gone {
            me.map(|s| s.stack - s.start).unwrap_or(0)
        } else {
            0
        };
        meja.ronde = self.hand;
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        let pending = self.pending_players().contains(&player);
        if pending {
            meja.netral = Some(match self.phase {
                Phase::Arrange => "auto".into(),
                _ => "leave".into(),
            });
        }
        let strings = |cards: &[Card]| cards.iter().map(Card::to_string).collect::<Vec<_>>();
        View {
            meja,
            kamu: player,
            kursi: self
                .seats
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let rows = s
                        .rows
                        .as_ref()
                        .filter(|_| s.revealed && s.special.is_none());
                    let mine = i as u8 == player;
                    SeatView {
                        tumpukan: s.stack,
                        awal: s.start,
                        status: match self.phase {
                            Phase::Arrange if s.arranging() => "menyusun",
                            Phase::Arrange if s.in_hand => "siap",
                            _ if s.left => "berdiri",
                            _ if s.out => "habis",
                            _ => "duduk",
                        }
                        .into(),
                        kartu: if !s.in_hand {
                            Vec::new()
                        } else if let Some(r) = rows {
                            strings(r)
                        } else if mine || s.revealed {
                            strings(&s.cards)
                        } else {
                            vec!["??".into(); s.cards.len()]
                        },
                        baris: rows
                            .map(|r| vec![strings(&r[..3]), strings(&r[3..8]), strings(&r[8..])]),
                        nama_baris: rows.map(|r| {
                            values(&r[..3], &r[3..8], &r[8..])
                                .iter()
                                .map(|v| v.key().to_string())
                                .collect()
                        }),
                        istimewa: s.special.map(|x| x.key().to_string()),
                        siap: match self.phase {
                            Phase::Arrange => s.in_hand && !s.arranging(),
                            _ => s.ready,
                        },
                        poin: s.points,
                        menang: s.won,
                    }
                })
                .collect(),
            dealer: self.dealer,
            tangan_ke: self.hand,
            poin_chip: POINT,
            saran: (pending && self.phase == Phase::Arrange)
                .then(|| strings(&suggest(&self.seats[player as usize].cards))),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = c.text(lang, "hand", &[("n", &view.tangan_ke.to_string())]);
        out.push('\n');
        for (i, s) in view.kursi.iter().enumerate() {
            let who = if i as u8 == view.kamu {
                c.text(lang, "you", &[])
            } else {
                c.text(lang, "seat", &[("n", &(i + 1).to_string())])
            };
            let cards = match (&s.baris, &s.istimewa) {
                (Some(rows), _) => rows
                    .iter()
                    .map(|r| r.join(" "))
                    .collect::<Vec<_>>()
                    .join(" | "),
                (None, Some(k)) => format!(
                    "{} ({})",
                    s.kartu.join(" "),
                    c.text(lang, &format!("special.{k}"), &[])
                ),
                _ => s.kartu.join(" "),
            };
            let points = if view.meja.fase == "susun" {
                String::new()
            } else {
                format!(" · {} {:+}", c.text(lang, "points", &[]), s.poin)
            };
            out.push_str(&format!(
                "  {who}: {} · {}{points} · {cards}\n",
                s.tumpukan,
                c.text(lang, &format!("status.{}", s.status), &[]),
            ));
        }
        out.push_str(&match view.meja.fase.as_str() {
            "susun" if view.saran.is_some() => c.text(lang, "your_turn", &[]),
            "susun" => c.text(lang, "waiting", &[]),
            "antara" => c.text(lang, "between", &[]),
            _ => c.text(
                lang,
                &format!("over.{}", view.meja.alasan.as_deref().unwrap_or("berdiri")),
                &[],
            ),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        if self.phase != Phase::Over {
            return None;
        }
        let scores: Vec<i64> = self.seats.iter().map(|s| s.stack - s.start).collect();
        let best = scores.iter().copied().max().unwrap_or(0);
        let winners: Vec<u8> = if scores.iter().all(|&x| x == best) {
            Vec::new()
        } else {
            (0..self.n())
                .filter(|&s| scores[s as usize] == best)
                .collect()
        };
        let hands = self.hand;
        let human = self.humans.first().copied().unwrap_or(0) as usize;
        let net = scores.get(human).copied().unwrap_or(0);
        Some(GameResult {
            winners,
            scores,
            summary: Localized::build(|lang| {
                catalog().text(
                    lang,
                    "summary",
                    &[("hands", &hands.to_string()), ("net", &format!("{net:+}"))],
                )
            }),
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["arrange", cards @ ..] => Ok(Action::Arrange(
                cards
                    .iter()
                    .map(|x| x.parse::<Card>().map_err(|_| err()))
                    .collect::<Result<_, _>>()?,
            )),
            ["auto"] => Ok(Action::Auto),
            ["next"] => Ok(Action::Next),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Arrange(cards) => format!(
                "arrange {}",
                cards
                    .iter()
                    .map(Card::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Action::Auto => "auto".into(),
            Action::Next => "next".into(),
            Action::Leave => "leave".into(),
        }
    }
}
