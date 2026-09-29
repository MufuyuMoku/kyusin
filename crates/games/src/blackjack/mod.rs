//! Blackjack (SPEC §6.3; D-056): satu pemain melawan bandar otomatis.
//!
//! Satu sesi = satu shoe (6 dek). Provably fair per shoe: seed dikomit saat
//! shoe dikocok dan dibuka saat shoe habis atau pemain berhenti, jadi
//! `verify` memeriksa semua ronde dalam shoe (penerapan §5.4 untuk game
//! ber-shoe; membuka seed tiap ronde akan membocorkan sisa shoe).
//!
//! Aturan: taruhan 10–2.000 (kelipatan 10); blackjack 3:2; bandar
//! mengintip bila kartu terbuka as atau bernilai 10; insurance saat as
//! terbuka (setengah taruhan, 2:1); bandar berdiri di semua 17 (S17);
//! double di dua kartu pertama mana pun, termasuk setelah split; split
//! pasangan peringkat sama sampai 4 tangan; as yang di-split mendapat satu
//! kartu masing-masing dan tidak di-split lagi; 21 setelah split bukan
//! blackjack; late surrender sebagai aksi pertama (bukan setelah split);
//! tangan yang mencapai 21 otomatis berdiri. Shoe selesai di akhir ronde
//! bila kartu terpakai mencapai titik potong (75%).
//!
//! Game ini murni: ia tidak tahu saldo chip. Host memeriksa saldo sebelum
//! aksi yang memakai chip (`bet`, `double`, `split`, `insure`) lewat
//! `biaya` dan `taruhan_meja` di view, dan mencatat `bersih` ke profil.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Rank, Shoe};

pub const ID: &str = "blackjack";
pub const MIN_BET: i64 = 10;
pub const MAX_BET: i64 = 2000;
pub const BET_STEP: i64 = 10;
pub const DECKS: u8 = 6;
/// Titik potong: shoe selesai setelah 75% kartu terpakai.
pub const PENETRATION: f64 = 0.75;
pub const MAX_HANDS: usize = 4;

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("blackjack/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/blackjack.toml"),
        create_session::<Blackjack>,
    )
}

/// Konfigurasi; `shoe` dan `potong` untuk tes dan tutorial.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Jumlah dek (bawaan 6).
    #[serde(default)]
    pub dek: Option<u8>,
    /// Urutan kartu tetap, kartu pertama dibagi lebih dulu.
    #[serde(default)]
    pub shoe: Option<Vec<String>>,
    /// Jumlah kartu terpakai yang mengakhiri shoe (bawaan 75%).
    #[serde(default)]
    pub potong: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Hit,
    Stand,
    Double,
    Split,
    Surrender,
    Insure,
    Decline,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Insurance,
    Player,
    Over,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Outcome {
    Win,
    Lose,
    Push,
    Surrender,
}

impl Outcome {
    fn key(self) -> &'static str {
        match self {
            Outcome::Win => "menang",
            Outcome::Lose => "kalah",
            Outcome::Push => "seri",
            Outcome::Surrender => "menyerah",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Hand {
    cards: Vec<Card>,
    bet: i64,
    doubled: bool,
    from_split: bool,
    split_aces: bool,
    done: bool,
    surrendered: bool,
    outcome: Option<Outcome>,
    payout: Option<i64>,
}

impl Hand {
    fn new(bet: i64) -> Self {
        Hand {
            cards: Vec::new(),
            bet,
            doubled: false,
            from_split: false,
            split_aces: false,
            done: false,
            surrendered: false,
            outcome: None,
            payout: None,
        }
    }

    fn natural(&self) -> bool {
        !self.from_split && self.cards.len() == 2 && value(&self.cards).0 == 21
    }
}

/// Nilai terbaik kartu dan apakah lunak (as dihitung 11).
pub fn value(cards: &[Card]) -> (u8, bool) {
    let hard: u8 = cards.iter().map(|c| c.rank.blackjack_value()).sum();
    let ace = cards.iter().any(|c| c.rank == Rank::Ace);
    if ace && hard + 10 <= 21 {
        (hard + 10, true)
    } else {
        (hard, false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Blackjack {
    shoe: Shoe,
    cut: usize,
    phase: Phase,
    hands: Vec<Hand>,
    active: usize,
    dealer: Vec<Card>,
    revealed: bool,
    insurance: Option<i64>,
    insurance_payout: Option<i64>,
    net: i64,
    rounds: u32,
    ended: Option<&'static str>,
}

/// Tampilan meja untuk pemain (hole card bandar tersembunyi sampai dibuka).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    /// `taruhan`, `asuransi`, `giliran`, atau `selesai`.
    pub fase: String,
    /// Kartu bandar; `??` untuk hole card yang belum dibuka.
    pub bandar: Vec<String>,
    pub bandar_nilai: Option<u8>,
    pub tangan: Vec<HandView>,
    /// Indeks tangan yang sedang dimainkan.
    pub aktif: Option<usize>,
    pub asuransi: Option<i64>,
    pub asuransi_bayar: Option<i64>,
    /// Hasil bersih semua ronde yang sudah selesai dalam shoe ini.
    pub bersih: i64,
    /// Chip yang sedang dipertaruhkan di ronde berjalan.
    pub taruhan_meja: i64,
    /// Biaya chip aksi yang tersedia (`double`, `split`, `insure`).
    pub biaya: BTreeMap<String, i64>,
    pub ronde: u32,
    pub kartu_terpakai: usize,
    pub sisa: usize,
    pub potong: usize,
    pub min_taruhan: i64,
    pub maks_taruhan: i64,
    pub selesai: bool,
    /// `shoe_habis` atau `berhenti`.
    pub alasan: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandView {
    pub kartu: Vec<String>,
    pub taruhan: i64,
    pub nilai: u8,
    pub lunak: bool,
    pub ganda: bool,
    pub blackjack: bool,
    pub selesai: bool,
    /// `menang`, `kalah`, `seri`, atau `menyerah` setelah ronde selesai.
    pub hasil: Option<String>,
    pub bayar: Option<i64>,
}

impl Blackjack {
    fn draw(&mut self) -> Result<Card, GameError> {
        self.shoe
            .draw()
            .ok_or_else(|| GameError::Illegal("shoe kosong".into()))
    }

    fn dealer_up(&self) -> Option<Card> {
        self.dealer.first().copied()
    }

    fn dealer_blackjack(&self) -> bool {
        self.dealer.len() == 2 && value(&self.dealer).0 == 21
    }

    fn start_round(&mut self, bet: i64) -> Result<(), GameError> {
        self.hands = vec![Hand::new(bet)];
        self.active = 0;
        self.dealer.clear();
        self.revealed = false;
        self.insurance = None;
        self.insurance_payout = None;
        self.rounds += 1;
        let p1 = self.draw()?;
        let d1 = self.draw()?;
        let p2 = self.draw()?;
        let d2 = self.draw()?;
        self.hands[0].cards = vec![p1, p2];
        self.dealer = vec![d1, d2];
        if d1.rank == Rank::Ace {
            self.phase = Phase::Insurance;
            Ok(())
        } else {
            self.after_insurance()
        }
    }

    /// Setelah keputusan insurance (atau bila tidak ditawarkan): bandar
    /// mengintip, lalu blackjack pemain dibayar atau giliran pemain dimulai.
    fn after_insurance(&mut self) -> Result<(), GameError> {
        let up = self
            .dealer_up()
            .map(|c| c.rank.blackjack_value())
            .unwrap_or(0);
        let peeks = up == 1 || up == 10;
        let dealer_bj = self.dealer_blackjack();
        if let Some(ins) = self.insurance {
            self.insurance_payout = Some(if dealer_bj { 2 * ins } else { -ins });
        }
        if peeks && dealer_bj {
            self.revealed = true;
            let hand = &mut self.hands[0];
            if hand.natural() {
                hand.outcome = Some(Outcome::Push);
                hand.payout = Some(0);
            } else {
                hand.outcome = Some(Outcome::Lose);
                hand.payout = Some(-hand.bet);
            }
            hand.done = true;
            self.end_round();
            return Ok(());
        }
        if self.hands[0].natural() {
            self.revealed = true;
            let hand = &mut self.hands[0];
            hand.outcome = Some(Outcome::Win);
            hand.payout = Some(hand.bet * 3 / 2);
            hand.done = true;
            self.end_round();
            return Ok(());
        }
        self.phase = Phase::Player;
        self.active = 0;
        self.advance()
    }

    /// Memajukan ke tangan berikutnya yang belum selesai (membagi kartu
    /// kedua untuk tangan hasil split), atau ke giliran bandar.
    fn advance(&mut self) -> Result<(), GameError> {
        while self.active < self.hands.len() {
            if self.hands[self.active].cards.len() == 1 {
                let c = self.draw()?;
                self.hands[self.active].cards.push(c);
            }
            let hand = &mut self.hands[self.active];
            if !hand.done && value(&hand.cards).0 >= 21 {
                hand.done = true;
            }
            if !hand.done {
                return Ok(());
            }
            self.active += 1;
        }
        self.dealer_turn()
    }

    fn dealer_turn(&mut self) -> Result<(), GameError> {
        self.revealed = true;
        let live = self
            .hands
            .iter()
            .any(|h| !h.surrendered && value(&h.cards).0 <= 21);
        if live {
            // S17: berdiri di semua 17, termasuk soft 17.
            while value(&self.dealer).0 < 17 {
                let c = self.draw()?;
                self.dealer.push(c);
            }
        }
        let dealer = value(&self.dealer).0;
        for hand in &mut self.hands {
            let mine = value(&hand.cards).0;
            let (outcome, payout) = if hand.surrendered {
                (Outcome::Surrender, -hand.bet / 2)
            } else if mine > 21 {
                (Outcome::Lose, -hand.bet)
            } else if dealer > 21 || mine > dealer {
                (Outcome::Win, hand.bet)
            } else if mine == dealer {
                (Outcome::Push, 0)
            } else {
                (Outcome::Lose, -hand.bet)
            };
            hand.outcome = Some(outcome);
            hand.payout = Some(payout);
        }
        self.end_round();
        Ok(())
    }

    fn end_round(&mut self) {
        let hands: i64 = self.hands.iter().filter_map(|h| h.payout).sum();
        self.net += hands + self.insurance_payout.unwrap_or(0);
        self.phase = if self.shoe.dealt() >= self.cut {
            self.ended = Some("shoe_habis");
            Phase::Over
        } else {
            Phase::Betting
        };
    }

    fn active_hand(&self) -> Option<&Hand> {
        (self.phase == Phase::Player)
            .then(|| self.hands.get(self.active))
            .flatten()
    }

    fn can_double(&self) -> bool {
        self.active_hand()
            .is_some_and(|h| h.cards.len() == 2 && !h.split_aces)
    }

    fn can_split(&self) -> bool {
        self.active_hand().is_some_and(|h| {
            h.cards.len() == 2
                && h.cards[0].rank == h.cards[1].rank
                && !h.split_aces
                && self.hands.len() < MAX_HANDS
        })
    }

    fn can_surrender(&self) -> bool {
        self.hands.len() == 1
            && self
                .active_hand()
                .is_some_and(|h| h.cards.len() == 2 && !h.from_split)
    }

    fn table_stake(&self) -> i64 {
        match self.phase {
            Phase::Insurance | Phase::Player => {
                self.hands.iter().map(|h| h.bet).sum::<i64>() + self.insurance.unwrap_or(0)
            }
            _ => 0,
        }
    }
}

fn bet_spec() -> ActionSpec {
    ActionSpec::template(
        "bet",
        vec![Param {
            name: "jumlah".into(),
            kind: ParamKind::Int {
                min: MIN_BET,
                max: MAX_BET,
                step: BET_STEP,
            },
        }],
    )
}

fn card_text(c: &Card) -> String {
    c.to_string()
}

impl TurnGame for Blackjack {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let shoe = match &config.shoe {
            Some(cards) => Shoe::from_cards(
                cards
                    .iter()
                    .map(|c| {
                        c.parse::<Card>()
                            .map_err(|_| GameError::Config(format!("kartu `{c}`")))
                    })
                    .collect::<Result<_, _>>()?,
            ),
            None => {
                let decks = config.dek.unwrap_or(DECKS);
                if decks == 0 || decks > 8 {
                    return Err(GameError::Config(format!("dek {decks}")));
                }
                let mut rng = GameRng::from_seed(seed);
                Shoe::shuffled(decks, &mut rng)
            }
        };
        let cut = config
            .potong
            .unwrap_or_else(|| (shoe.len() as f64 * PENETRATION).round() as usize);
        Ok(Blackjack {
            shoe,
            cut,
            phase: Phase::Betting,
            hands: Vec::new(),
            active: 0,
            dealer: Vec::new(),
            revealed: false,
            insurance: None,
            insurance_payout: None,
            net: 0,
            rounds: 0,
            ended: None,
        })
    }

    fn seats(&self) -> u8 {
        1
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.phase == Phase::Over {
            Vec::new()
        } else {
            vec![0]
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if player != 0 {
            return Vec::new();
        }
        match self.phase {
            Phase::Betting => vec![bet_spec(), ActionSpec::fixed("leave")],
            Phase::Insurance => vec![ActionSpec::fixed("insure"), ActionSpec::fixed("decline")],
            Phase::Player => {
                let mut out = vec![ActionSpec::fixed("hit"), ActionSpec::fixed("stand")];
                if self.can_double() {
                    out.push(ActionSpec::fixed("double"));
                }
                if self.can_split() {
                    out.push(ActionSpec::fixed("split"));
                }
                if self.can_surrender() {
                    out.push(ActionSpec::fixed("surrender"));
                }
                out
            }
            Phase::Over => Vec::new(),
        }
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::Over {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        let illegal = |a: &Action, g: &Blackjack| GameError::Illegal(g.format_action(a));
        match (self.phase, action) {
            (Phase::Betting, Action::Bet(n)) => {
                if !(MIN_BET..=MAX_BET).contains(&n) || n % BET_STEP != 0 {
                    return Err(illegal(&action, self));
                }
                self.start_round(n)
            }
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Insurance, Action::Insure) => {
                self.insurance = Some(self.hands[0].bet / 2);
                self.after_insurance()
            }
            (Phase::Insurance, Action::Decline) => self.after_insurance(),
            (Phase::Player, Action::Hit) => {
                let c = self.draw()?;
                let hand = &mut self.hands[self.active];
                hand.cards.push(c);
                if value(&hand.cards).0 >= 21 {
                    hand.done = true;
                }
                self.advance()
            }
            (Phase::Player, Action::Stand) => {
                self.hands[self.active].done = true;
                self.advance()
            }
            (Phase::Player, Action::Double) if self.can_double() => {
                let c = self.draw()?;
                let hand = &mut self.hands[self.active];
                hand.bet *= 2;
                hand.doubled = true;
                hand.cards.push(c);
                hand.done = true;
                self.advance()
            }
            (Phase::Player, Action::Split) if self.can_split() => {
                let i = self.active;
                let aces = self.hands[i].cards[0].rank == Rank::Ace;
                let second = self.hands[i].cards.pop().expect("dua kartu");
                let mut new = Hand::new(self.hands[i].bet);
                new.cards = vec![second];
                new.from_split = true;
                new.split_aces = aces;
                self.hands[i].from_split = true;
                self.hands[i].split_aces = aces;
                self.hands.insert(i + 1, new);
                let c = self.draw()?;
                self.hands[i].cards.push(c);
                if aces {
                    // As yang di-split: satu kartu masing-masing, lalu selesai.
                    let c = self.draw()?;
                    self.hands[i + 1].cards.push(c);
                    self.hands[i].done = true;
                    self.hands[i + 1].done = true;
                } else if value(&self.hands[i].cards).0 == 21 {
                    self.hands[i].done = true;
                }
                self.advance()
            }
            (Phase::Player, Action::Surrender) if self.can_surrender() => {
                let hand = &mut self.hands[self.active];
                hand.surrendered = true;
                hand.done = true;
                self.advance()
            }
            _ => Err(illegal(&action, self)),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Insurance => "asuransi",
            Phase::Player => "giliran",
            Phase::Over => "selesai",
        };
        let bandar = self
            .dealer
            .iter()
            .enumerate()
            .map(|(i, c)| {
                if i == 1 && !self.revealed {
                    "??".to_string()
                } else {
                    card_text(c)
                }
            })
            .collect();
        let mut biaya = BTreeMap::new();
        match self.phase {
            Phase::Insurance => {
                biaya.insert("insure".to_string(), self.hands[0].bet / 2);
            }
            Phase::Player => {
                let bet = self.hands[self.active].bet;
                if self.can_double() {
                    biaya.insert("double".to_string(), bet);
                }
                if self.can_split() {
                    biaya.insert("split".to_string(), bet);
                }
            }
            _ => {}
        }
        View {
            fase: fase.into(),
            bandar,
            bandar_nilai: self.revealed.then(|| value(&self.dealer).0),
            tangan: self
                .hands
                .iter()
                .map(|h| {
                    let (nilai, lunak) = value(&h.cards);
                    HandView {
                        kartu: h.cards.iter().map(card_text).collect(),
                        taruhan: h.bet,
                        nilai,
                        lunak,
                        ganda: h.doubled,
                        blackjack: h.natural(),
                        selesai: h.done,
                        hasil: h.outcome.map(|o| o.key().to_string()),
                        bayar: h.payout,
                    }
                })
                .collect(),
            aktif: (self.phase == Phase::Player).then_some(self.active),
            asuransi: self.insurance,
            asuransi_bayar: self.insurance_payout,
            bersih: self.net,
            taruhan_meja: self.table_stake(),
            biaya,
            ronde: self.rounds,
            kartu_terpakai: self.shoe.dealt(),
            sisa: self.shoe.remaining(),
            potong: self.cut,
            min_taruhan: MIN_BET,
            maks_taruhan: MAX_BET,
            selesai: self.phase == Phase::Over,
            alasan: self.ended.map(str::to_string),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        let dealer = if view.bandar.is_empty() {
            "-".to_string()
        } else {
            view.bandar.join(" ")
        };
        let dealer_value = view
            .bandar_nilai
            .map(|v| format!(" = {v}"))
            .unwrap_or_default();
        out.push_str(&c.text(
            lang,
            "dealer",
            &[("cards", &format!("{dealer}{dealer_value}"))],
        ));
        out.push('\n');
        for (i, h) in view.tangan.iter().enumerate() {
            let marker = if view.aktif == Some(i) { ">" } else { " " };
            let mut line = format!(
                "{marker} {} {} = {}{} · {}",
                c.text(lang, "hand", &[("n", &(i + 1).to_string())]),
                h.kartu.join(" "),
                if h.lunak { "soft " } else { "" },
                h.nilai,
                c.text(lang, "bet", &[("n", &h.taruhan.to_string())]),
            );
            if let (Some(hasil), Some(bayar)) = (&h.hasil, h.bayar) {
                line.push_str(&format!(
                    " · {} {bayar:+}",
                    c.text(lang, &format!("result.{hasil}"), &[])
                ));
            }
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str(&c.text(
            lang,
            "net",
            &[
                ("net", &format!("{:+}", view.bersih)),
                ("left", &view.sisa.to_string()),
            ],
        ));
        out.push('\n');
        let status = match view.fase.as_str() {
            "taruhan" => c.text(
                lang,
                "place_bet",
                &[
                    ("min", &view.min_taruhan.to_string()),
                    ("max", &view.maks_taruhan.to_string()),
                ],
            ),
            "asuransi" => c.text(lang, "insurance", &[]),
            "giliran" => c.text(lang, "your_turn", &[]),
            _ => c.text(
                lang,
                &format!("over.{}", view.alasan.as_deref().unwrap_or("berhenti")),
                &[],
            ),
        };
        out.push_str(&status);
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        if self.phase != Phase::Over {
            return None;
        }
        let net = self.net;
        let rounds = self.rounds;
        Some(GameResult {
            winners: if net > 0 { vec![0] } else { Vec::new() },
            scores: vec![net],
            summary: Localized::build(|lang| {
                catalog().text(
                    lang,
                    "summary",
                    &[
                        ("rounds", &rounds.to_string()),
                        ("net", &format!("{net:+}")),
                    ],
                )
            }),
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let mut parts = c.split(' ');
        let verb = parts.next().unwrap_or("");
        let arg = parts.next();
        if parts.next().is_some() {
            return Err(GameError::Parse(c.into()));
        }
        let action = match (verb, arg) {
            ("bet", Some(n)) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => {
                Action::Bet(n.parse().map_err(|_| GameError::Parse(c.into()))?)
            }
            ("hit", None) => Action::Hit,
            ("stand", None) => Action::Stand,
            ("double", None) => Action::Double,
            ("split", None) => Action::Split,
            ("surrender", None) => Action::Surrender,
            ("insure", None) => Action::Insure,
            ("decline", None) => Action::Decline,
            ("leave", None) => Action::Leave,
            _ => return Err(GameError::Parse(c.into())),
        };
        Ok(action)
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Hit => "hit".into(),
            Action::Stand => "stand".into(),
            Action::Double => "double".into(),
            Action::Split => "split".into(),
            Action::Surrender => "surrender".into(),
            Action::Insure => "insure".into(),
            Action::Decline => "decline".into(),
            Action::Leave => "leave".into(),
        }
    }
}
