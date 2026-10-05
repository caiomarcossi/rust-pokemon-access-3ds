//! The Bag's party is found by its shape: six PokemonParam pointers, then how many are filled.

use super::fake_memory::{CODE, FakeClasses, FakeMemory};
use crate::core::rtti::ClassNames;
use crate::games::common::pokemon;
use crate::games::gen6::partyicons;

const PARTY: i64 = 0x08CF7200;
const PARAMS: i64 = 0x08CF723C;

#[test]
fn test_the_party_is_six_params_and_a_count() {
    let memory = FakeMemory::new();
    let param = FakeClasses::new(memory.clone()).add(pokemon::PARAM_CLASS);
    for i in 0..6 {
        memory.u32(PARAMS + 0x1E4 * i, &[param]);
        memory.u32(PARTY + 4 * i, &[PARAMS + 0x1E4 * i]);
    }
    memory.u32(PARTY + partyicons::COUNT, &[5]);
    let names = ClassNames::new(memory.clone(), CODE);
    // A word holding a param on its own is not the party; the list is.
    assert_eq!(
        partyicons::party_of([PARTY + 4, PARTY], &*memory, &names).unwrap(),
        Some(PARTY)
    );
    memory.u32(PARTY + partyicons::COUNT, &[7]);
    assert_eq!(
        partyicons::party_of([PARTY], &*memory, &names).unwrap(),
        None
    );
}
