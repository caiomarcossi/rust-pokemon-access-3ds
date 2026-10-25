//! The Battle Maison's partners are named by the portrait each button's picture draws.

use indexmap::IndexMap;

use super::fake_memory::{FakeClasses, FakeMemory};
use crate::games::common::lyt;
use crate::games::gen6::screens::partners;
use crate::py::Result;

const VIEW: i64 = 0x08603D9C;
const TEXTURES: i64 = 0x08640BF0;
const PICTURE: i64 = 0x15948360;

/// May, Wally, Maxie and Steven, as or_multi_battle_partners shows them; the third
/// portrait is Maxie's.
#[test]
fn test_a_portrait_is_named_by_its_partner_and_nothing_once_the_view_is_freed() -> Result<()> {
    let memory = FakeMemory::new();
    memory.u32(
        VIEW,
        &[FakeClasses::new(memory.clone()).add(partners::CLASS)],
    );
    memory.u32(VIEW + partners::COUNT, &[4]);
    for (i, name) in ["May", "Wally", "Maxie", "Steven"].iter().enumerate() {
        let i = i as i64;
        let texture = TEXTURES + 0x14C * i;
        memory.u32(VIEW + partners::TEXTURES + 4 * i, &[texture]);
        memory.u32(texture + partners::IMAGE, &[0x219CA600 + 0x2100 * i]);
        memory.u32(VIEW + partners::ENTRIES + partners::ENTRY * i, &[2 + i]);
        let mut raw: Vec<u8> = name.encode_utf16().flat_map(u16::to_le_bytes).collect();
        raw.extend([0, 0]);
        memory.write(
            VIEW + partners::ENTRIES + partners::ENTRY * i + partners::NAME,
            &raw,
        );
    }
    memory.u32(PICTURE + lyt::PICTURE_MATERIAL, &[PICTURE + 0x1D8]);
    memory.u32(
        PICTURE + 0x1D8 + lyt::MATERIAL_TEXTURE,
        &[0x219CA600 + 0x2100 * 2],
    );
    let mut named = partners::Partners::new(memory.clone());
    assert_eq!(named.name(PICTURE)?, None);
    named.track(
        &IndexMap::from([(VIEW, partners::CLASS.to_string())]),
        false,
    )?;
    assert_eq!(named.name(PICTURE)?.as_deref(), Some("Maxie"));
    memory.u32(VIEW, &[0]);
    named.track(&IndexMap::new(), false)?;
    assert_eq!(named.name(PICTURE)?, None);
    Ok(())
}
