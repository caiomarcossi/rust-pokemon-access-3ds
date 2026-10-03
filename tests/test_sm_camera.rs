//! Sun's camera: which way up walks, and the compass turned to it (spike 119).

use std::collections::HashSet;
use std::f64::consts::PI;

use crate::games::sm::camera::{quarter, turned, turned_box, unturned};

/// The pad translator's angle over sun_5.ctm: pi in the living room, -1.934 on the porch,
/// where up walked the player west-north-west to the door; about -2.9 on Route 1.
#[test]
fn test_up_walks_north_in_the_living_room_and_west_on_the_porch() {
    assert_eq!(quarter(PI, None).unwrap(), 0);
    assert_eq!(quarter(-1.934, None).unwrap(), 3);
    assert_eq!(quarter((-166.0_f64).to_radians(), None).unwrap(), 0);
    assert_eq!(quarter(PI / 2.0, None).unwrap(), 1);
    assert_eq!(quarter(0.0, None).unwrap(), 2);
    assert_eq!(quarter(f64::NAN, None).unwrap(), 0);
}

/// Route 1's camera swings about 20 degrees; one swinging about a quarter's edge would turn
/// the whole compass with it. The angle for a bearing b is pi - b.
#[test]
fn test_the_compass_keeps_its_quarter_until_the_camera_looks_well_past_the_edge() {
    fn looking(bearing: f64) -> f64 {
        PI - bearing.to_radians()
    }

    assert_eq!(quarter(looking(50.0), None).unwrap(), 1);
    assert_eq!(quarter(looking(50.0), Some(0)).unwrap(), 0);
    assert_eq!(quarter(looking(56.0), Some(0)).unwrap(), 1);
    assert_eq!(quarter(looking(-50.0), Some(0)).unwrap(), 0);
    assert_eq!(quarter(looking(120.0), Some(0)).unwrap(), 1);
    assert_eq!(quarter(f64::NAN, Some(3)).unwrap(), 3);
}

/// Facing west, the cell west of the player is the one north of them on the pad.
#[test]
fn test_a_turned_cell_keeps_its_neighbours_and_turns_back() {
    for turn in 0..4 {
        for cell in [(0, 0), (5, -3), (-7, 12)] {
            assert_eq!(unturned(turned(cell, turn), turn), cell, "turn {turn}");
        }
    }
    assert_eq!(
        turned((9, 4), 3),
        (turned((10, 4), 3).0, turned((10, 4), 3).1 - 1)
    );
    assert_eq!(
        turned((10, 3), 3),
        (turned((10, 4), 3).0 + 1, turned((10, 4), 3).1)
    );
}

#[test]
fn test_a_turned_box_starts_at_its_north_west_cell_again() {
    let (first, size) = turned_box((10, 20), (3, 1), 3);
    assert_eq!(size, (1, 3));
    let turned_cells: HashSet<(i64, i64)> = (0..3).map(|dx| turned((10 + dx, 20), 3)).collect();
    let expected: HashSet<(i64, i64)> = (0..3).map(|dz| (first.0, first.1 + dz)).collect();
    assert_eq!(turned_cells, expected);
}
