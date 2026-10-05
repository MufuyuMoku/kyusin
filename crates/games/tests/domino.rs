//! Tes mesin domino bersama (SPEC §6.3; M5b-2), ditulis sebelum mesinnya.
//! Dipakai Domino QiuQiu sekarang dan Gaple nanti: satu set double-six
//! berisi 28 kartu, ditulis `besar-kecil` (misalnya `6-4`, `0-0`).

use std::collections::HashSet;

use kyusin_core::GameRng;
use kyusin_games::domino::{Boneyard, Tile};

#[test]
fn a_double_six_set_has_28_unique_tiles() {
    let set = Tile::set();
    assert_eq!(set.len(), 28);
    let unique: HashSet<Tile> = set.iter().copied().collect();
    assert_eq!(unique.len(), 28);
    assert_eq!(set.iter().filter(|t| t.is_double()).count(), 7);
    assert_eq!(set.iter().map(|t| u32::from(t.pips())).sum::<u32>(), 168);
    assert!(set.iter().all(|t| t.hi >= t.lo && t.hi <= 6));
}

#[test]
fn tiles_parse_in_either_order_and_print_high_first() {
    let t: Tile = "4-6".parse().unwrap();
    assert_eq!(t, Tile::new(6, 4));
    assert_eq!(t.to_string(), "6-4");
    assert_eq!("0-0".parse::<Tile>().unwrap().to_string(), "0-0");
    assert_eq!(Tile::new(3, 3).pips(), 6);
    assert!(Tile::new(3, 3).is_double());
    assert!(!Tile::new(6, 0).is_double());
    for bad in ["7-1", "6", "a-b", "6-4-1", ""] {
        assert!(bad.parse::<Tile>().is_err(), "{bad}");
    }
    let json = serde_json::to_string(&Tile::new(5, 2)).unwrap();
    assert_eq!(json, "\"5-2\"");
    assert_eq!(serde_json::from_str::<Tile>(&json).unwrap(), Tile::new(5, 2));
}

#[test]
fn a_shuffled_boneyard_is_a_seeded_permutation() {
    let a = Boneyard::shuffled(&mut GameRng::from_seed([7; 32]));
    let b = Boneyard::shuffled(&mut GameRng::from_seed([7; 32]));
    let c = Boneyard::shuffled(&mut GameRng::from_seed([8; 32]));
    assert_eq!(a.tiles(), b.tiles(), "seed sama = urutan sama");
    assert_ne!(a.tiles(), c.tiles());
    let mut sorted = a.tiles().to_vec();
    sorted.sort();
    let mut set = Tile::set();
    set.sort();
    assert_eq!(sorted, set);

    let mut y = Boneyard::from_tiles(vec![Tile::new(6, 6), Tile::new(1, 0)]);
    assert_eq!(y.len(), 2);
    assert_eq!(y.draw(), Some(Tile::new(6, 6)));
    assert_eq!(y.draw(), Some(Tile::new(1, 0)));
    assert_eq!(y.draw(), None);
    assert!(y.is_empty());
}
