//! Which game package reads the game running, by its title id.

use crate::games::catalog::{Running, for_title};
use crate::games::gen6::game::Game;
use crate::games::sm::SunMoon;
use crate::games::{oras, sm, xy};

fn is_gen6(running: Running, game: &'static Game) -> bool {
    matches!(running, Running::Gen6(found) if std::ptr::eq(found, game))
}

fn is_sun(running: Running, game: &'static SunMoon) -> bool {
    matches!(running, Running::SunMoon(found) if std::ptr::eq(found, game))
}

#[test]
fn test_x_and_y_are_read_with_x_s_numbers_and_anything_else_with_oras_s() {
    assert!(is_gen6(for_title(xy::X), xy::game()));
    assert!(is_gen6(for_title(xy::Y), xy::game()));
    assert!(is_gen6(for_title(oras::OMEGA_RUBY), oras::game()));
    assert!(is_gen6(for_title(0x0004000000999900), oras::game()));
    assert!(is_sun(for_title(sm::ULTRA_SUN), sm::ultra()));
}

#[test]
fn test_sun_and_moon_are_read_by_their_own_profile() {
    assert!(is_sun(for_title(sm::SUN), sm::game()));
    assert!(is_sun(for_title(sm::MOON), sm::game()));
}
