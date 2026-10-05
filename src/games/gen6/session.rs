//! A gen 6 game — ORAS, X or Y — put together for the runtime: its readers, and its heaps watched
//! for what they allocate (spike 110).

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::discovery::{Found, HEAP_SPAN};
use crate::core::heaps::{Block, Heaps};
use crate::core::memory::{Cache, PageCache};
use crate::core::names::Names;
use crate::core::rpc::Rpc;
use crate::core::rtti::ClassNames;
use crate::core::speech::Speech;
use crate::core::window;
use crate::games::common::session::{Profile, Session, Setup};
use crate::games::gen6::archives::{GameText, RomLayout};
use crate::games::gen6::game::Game;
use crate::games::gen6::profile::{Gen6Profile, Options, REQUIRED};
use crate::games::gen6::{buttons, gflheap, textures};
use crate::py::Result;

/// `game` read through a page cache, every poll in a phase of its own: one pipelined fetch of
/// the pages it read last time, and every read inside a page answered from it (spike 62).
/// `image` is the executable as read from `code`, where the game keeps tables no ROM file has.
pub fn start(
    game: &'static Game,
    rpc: Rc<dyn Rpc>,
    speech: Rc<Speech>,
    code: (i64, i64),
    setup: Setup,
    image: &[u8],
) -> Result<Session> {
    let memory = Rc::new(PageCache::new(rpc));
    // The game's own words come from the game; these are for what it never names at all.
    let title = window::game_title(window::EMULATOR);
    // `Names.load(game.data, ...)`, as `Game::sprites` reads the table the binary carries in
    // place of a folder.
    let named = {
        let fingerprint = &setup.process.name;
        let shipped = game.sprites(&setup.language);
        let mine = setup.names_home.as_ref().map(|home| {
            home.join("names")
                .join(&setup.language)
                .join(format!("{fingerprint}.toml"))
        });
        let header = IndexMap::from([
            (
                "game".to_string(),
                if title.is_empty() {
                    fingerprint
                } else {
                    &title
                }
                .to_string(),
            ),
            ("rom".to_string(), fingerprint.to_string()),
            ("language".to_string(), setup.language.clone()),
        ]);
        Rc::new(Names::new(Some(&shipped), mine.as_deref(), Some(&header))?)
    };
    let text = game_text(&setup, &game.rom)?.map(Rc::new);
    let mut pictures = None;
    if !game.rom.textures.is_empty()
        && let Some(home) = &setup.names_home
    {
        let cache = home
            .join("cache")
            .join("textures")
            .join(format!("{}.toml", setup.process.name));
        pictures = textures::start(
            &cache,
            setup.rom_path()?.as_deref(),
            &game.rom.textures,
            &title,
            &setup.process.name,
            text.clone(),
            &*setup.log,
        )?
        .map(Rc::new);
    }
    let profile = Rc::new(Gen6Profile::new(
        memory.clone(),
        Rc::new(ClassNames::new(memory.clone(), code)),
        speech,
        setup.log.clone(),
        game,
        Options {
            named: Some(named),
            ui: Some(setup.ui.clone()),
            navigation: Some(setup.navigation.clone()),
            keys: Some(setup.keys.clone()),
            text,
            pictures,
            image: image.to_vec(),
            tap: setup.tap.clone(),
            tones: setup.tones.clone(),
        },
    ));
    // What the game allocates is found where it was allocated, the look after (spike 110).
    let read_each = {
        let memory = memory.clone();
        Rc::new(move |reads: &[(i64, i64)]| memory.read_each(reads))
    };
    let heaps = Rc::new(RefCell::new(Heaps::with(
        memory.clone(),
        gflheap::HEAD,
        HEAP_SPAN,
        Some(gflheap::COUNT),
        None,
        Rc::new(|_heap| false),
        Some(read_each),
    )));
    let adopting = heaps.clone();
    Ok(Session {
        classes: profile.classes.clone(),
        profile: profile as Rc<dyn Profile>,
        required: REQUIRED.iter().map(|class| class.to_string()).collect(),
        heaps,
        watched: memory.clone() as Rc<dyn Cache>,
        adopt: Rc::new(move |found: &Found| {
            adopting
                .borrow_mut()
                .adopt(&found.of(&[gflheap::CLASS]), false)?;
            Ok(())
        }),
        to_sweep: Rc::new(|block: Block| Some(gflheap::to_sweep(block))),
        allocated: Rc::new(|_blocks: &[Block]| {}),
        walk: None,
        bases: vec![buttons::BAR_CLASS.to_string()],
        pages: Some(memory),
    })
}

/// The game's own words out of the player's ROM, or None; the log says which, and why.
///
/// Not having them costs a place's name said a moment early and the rival's name, both of which
/// have a fallback — AreaNav's words and "Rival" — so a ROM that cannot be read is logged, not
/// spoken at every start.
pub fn game_text(setup: &Setup, layout: &RomLayout) -> Result<Option<GameText>> {
    setup.words(
        |image, language| GameText::load(image, language, layout),
        "the game's own words for places and the rival are off",
    )
}
