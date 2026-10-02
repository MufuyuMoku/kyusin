//! Tes pembagian pot termasuk side pot (SPEC §7 poin 1 untuk game
//! antar-pemain), ditulis sebelum modulnya.
//!
//! Kontribusi tiap kursi dalam satu tangan dipecah menjadi pot utama dan
//! side pot menurut lapisan kontribusi; kursi yang fold tetap menyumbang
//! tetapi tidak berhak. Setiap pot dimenangkan tangan terbaik di antara
//! kursi yang berhak; seri dibagi rata, sisa chip ganjil diberikan satu per
//! satu mulai dari kursi pertama searah jarum jam setelah dealer.

use kyusin_games::pot::{Pot, award, pots};

#[test]
fn single_pot_without_all_in() {
    let p = pots(&[100, 100, 100], &[false, false, true]);
    assert_eq!(
        p,
        vec![Pot {
            amount: 300,
            eligible: vec![0, 1]
        }]
    );
}

#[test]
fn side_pots_by_contribution_layers() {
    // Kursi 0 all-in 50, kursi 1 all-in 120, kursi 2 dan 3 masing-masing 200,
    // kursi 3 fold.
    let p = pots(&[50, 120, 200, 200], &[false, false, false, true]);
    assert_eq!(
        p,
        vec![
            Pot {
                amount: 200,
                eligible: vec![0, 1, 2]
            },
            Pot {
                amount: 210,
                eligible: vec![1, 2]
            },
            Pot {
                amount: 160,
                eligible: vec![2]
            },
        ]
    );
    assert_eq!(p.iter().map(|x| x.amount).sum::<i64>(), 570);
}

#[test]
fn folded_chips_above_everyone_still_go_to_the_last_pot() {
    // Kursi 2 fold setelah memasang 300; kursi 0 dan 1 hanya 100. Lapisan
    // tanpa kursi yang berhak digabung ke pot sebelumnya.
    let p = pots(&[100, 100, 300], &[false, false, true]);
    assert_eq!(
        p,
        vec![Pot {
            amount: 500,
            eligible: vec![0, 1]
        }]
    );
}

#[test]
fn award_best_hand_per_pot_with_splits_and_odd_chips() {
    // Nilai tangan: lebih besar lebih baik.
    let strength = [5, 9, 9, 1];
    let p = vec![
        Pot {
            amount: 301,
            eligible: vec![0, 1, 2],
        },
        Pot {
            amount: 100,
            eligible: vec![0, 3],
        },
    ];
    // Dealer di kursi 1: sisa ganjil mulai dari kursi 2.
    let won = award(&p, |s| strength[s as usize], 4, 1);
    assert_eq!(won, vec![100, 150, 151, 0]);
    assert_eq!(won.iter().sum::<i64>(), 401);
}

#[test]
fn odd_chip_order_wraps_around_the_table() {
    let p = vec![Pot {
        amount: 5,
        eligible: vec![0, 1],
    }];
    // Dealer di kursi 0: kursi 1 duluan.
    assert_eq!(award(&p, |_| 1, 2, 0), vec![2, 3]);
    // Dealer di kursi 1: kursi 0 duluan.
    assert_eq!(award(&p, |_| 1, 2, 1), vec![3, 2]);
}

/// Teen Patti: chaal tidak "disamakan" seperti taruhan poker, jadi semua chip
/// masuk satu pot; hanya pemain all-in yang dibatasi ke lapisan
/// kontribusinya.
#[test]
fn all_in_layers_only_for_unmatched_games() {
    use kyusin_games::pot::pots_all_in;
    // Tanpa all-in: satu pot untuk semua yang belum pack, walau kontribusi
    // berbeda.
    let p = pots_all_in(&[30, 20, 10], &[false, false, true], &[false, false, false]);
    assert_eq!(
        p,
        vec![Pot {
            amount: 60,
            eligible: vec![0, 1]
        }]
    );
    // Kursi 2 all-in 25: berhak atas lapisan sampai 25 dari setiap kursi.
    let p = pots_all_in(&[10, 50, 25], &[true, false, false], &[false, false, true]);
    assert_eq!(
        p,
        vec![
            Pot {
                amount: 60,
                eligible: vec![1, 2]
            },
            Pot {
                amount: 25,
                eligible: vec![1]
            },
        ]
    );
}
