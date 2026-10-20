//! Which way the pad walks the player where the camera is turned (`field.pad`).

use crate::core::pathfind::{self, Leg};

/// Where up walks west, the world's north is right's way, said "east", and its west up's.
#[test]
fn test_a_way_of_the_compass_is_said_as_the_key_that_walks_it() {
    pathfind::walked(3);
    // Python's `finally`: the compass is turned back before anything is asserted.
    let said = (
        pathfind::spoken(&[Leg::new("north", 39, 39), Leg::new("west", 6, 45)]),
        pathfind::way("south-west"),
    );
    pathfind::walked(0);
    assert_eq!(said.0, "39 east, 6 north");
    assert_eq!(said.1, "north-west");
    assert_eq!(pathfind::way("north"), "north");
}
