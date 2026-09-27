//! Game fixture minimal khusus tes (SPEC §9 M0, D-005): Nim sederhana.
//!
//! Dua pemain bergantian mengambil 1–3 batang; yang mengambil batang
//! terakhir menang. Dipakai untuk menguji registry, `ActionSpec` templat, dan
//! runner tutorial sebelum kontrak final ada di M1.

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::{
    ActionSpec, Cartridge, GameError, Param, ParamKind, PlayerId, RegistryError, Seed, TurnGame,
};
use std::sync::OnceLock;

use kyusin_core::i18n::{Catalog, Lang};
use serde::{Deserialize, Serialize};

pub const ID: &str = "fixture";

/// Teks tampilan fixture dalam dua bahasa.
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG
        .get_or_init(|| Catalog::from_toml(include_str!("i18n.toml")).expect("fixture/i18n.toml"))
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("tutorial.toml"),
        create_session::<Batang>,
    )
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub batang: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config { batang: 7 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Take(pub u32);

#[derive(Debug, Clone, Serialize)]
pub struct View {
    pub batang: u32,
    pub giliran: PlayerId,
    pub kamu: PlayerId,
    pub pemenang: Option<PlayerId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Batang {
    left: u32,
    turn: PlayerId,
    winner: Option<PlayerId>,
}

impl TurnGame for Batang {
    type Config = Config;
    type Action = Take;
    type View = View;

    fn new(config: Config, _seed: Seed) -> Result<Self, GameError> {
        Ok(Batang {
            left: config.batang,
            turn: 0,
            winner: None,
        })
    }

    fn seats(&self) -> u8 {
        2
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.winner.is_some() {
            Vec::new()
        } else {
            vec![self.turn]
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if self.winner.is_some() || player != self.turn || self.left == 0 {
            return Vec::new();
        }
        vec![ActionSpec::template(
            "take",
            vec![Param {
                name: "n".into(),
                kind: ParamKind::Int {
                    min: 1,
                    max: i64::from(self.left.min(3)),
                    step: 1,
                },
            }],
        )]
    }

    fn apply(&mut self, player: PlayerId, Take(n): Take) -> Result<(), GameError> {
        if player != self.turn || n == 0 || n > 3 || n > self.left {
            return Err(GameError::Illegal(format!("take {n}")));
        }
        self.left -= n;
        if self.left == 0 {
            self.winner = Some(player);
        } else {
            self.turn = 1 - self.turn;
        }
        Ok(())
    }

    fn view_for(&self, player: PlayerId) -> View {
        View {
            batang: self.left,
            giliran: self.turn,
            kamu: player,
            pemenang: self.winner,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let sticks = "│ ".repeat(view.batang as usize);
        let p = |id: PlayerId| (id + 1).to_string();
        let status = match view.pemenang {
            Some(w) if w == view.kamu => c.text(lang, "you_win", &[]),
            Some(w) => c.text(lang, "player_wins", &[("p", &p(w))]),
            None if view.giliran == view.kamu => c.text(lang, "your_turn", &[]),
            None => c.text(lang, "player_turn", &[("p", &p(view.giliran))]),
        };
        format!(
            "{}\n\n  {}\n\n{status}",
            c.text(lang, "left", &[("n", &view.batang.to_string())]),
            sticks.trim_end()
        )
    }

    fn is_over(&self) -> bool {
        self.winner.is_some()
    }

    fn result(&self) -> Option<GameResult> {
        self.winner.map(|w| GameResult {
            winners: vec![w],
            scores: Vec::new(),
            summary: catalog().localized("summary", &[("p", &(w + 1).to_string())]),
        })
    }

    fn parse_command(&self, command: &str) -> Result<Take, GameError> {
        let mut parts = command.split_whitespace();
        match (parts.next(), parts.next(), parts.next()) {
            (Some("take"), Some(n), None) => n
                .parse()
                .map(Take)
                .map_err(|_| GameError::Parse(command.into())),
            _ => Err(GameError::Parse(command.into())),
        }
    }

    fn format_action(&self, Take(n): &Take) -> String {
        format!("take {n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyusin_core::Session;

    fn game(n: u32) -> Batang {
        Batang::new(Config { batang: n }, [0; 32]).unwrap()
    }

    #[test]
    fn last_stick_wins() {
        let mut g = game(4);
        Session::act(&mut g, 0, "take 3").unwrap();
        assert_eq!(TurnGame::pending_players(&g), vec![1]);
        Session::act(&mut g, 1, "take 1").unwrap();
        assert_eq!(TurnGame::result(&g).unwrap().winners, vec![1]);
        assert_eq!(Session::act(&mut g, 0, "take 1"), Err(GameError::Over));
    }

    #[test]
    fn cannot_take_more_than_left_or_out_of_turn() {
        let mut g = game(2);
        assert_eq!(
            Session::act(&mut g, 0, "take 3"),
            Err(GameError::Illegal("take 3".into()))
        );
        assert_eq!(
            Session::act(&mut g, 1, "take 1"),
            Err(GameError::NotPending(1))
        );
    }

    #[test]
    fn catalog_has_both_languages_and_all_keys_resolve() {
        assert_eq!(catalog().missing_keys(), Vec::<String>::new());
        let mut g = game(2);
        for lang in Lang::ALL {
            let text = Session::view_text(&g, 0, lang);
            // Kunci yang tidak ditemukan dikembalikan apa adanya.
            assert!(
                !text.contains("your_turn") && !text.starts_with("left"),
                "{text}"
            );
        }
        Session::act(&mut g, 0, "take 2").unwrap();
        assert!(Session::view_text(&g, 0, Lang::En).contains("You win."));
        assert!(Session::view_text(&g, 1, Lang::Id).contains("Pemain 1 menang."));
    }

    #[test]
    fn command_round_trip() {
        let g = game(7);
        for n in 1..=3 {
            let text = g.format_action(&Take(n));
            assert_eq!(g.parse_command(&text).unwrap(), Take(n));
        }
    }
}
