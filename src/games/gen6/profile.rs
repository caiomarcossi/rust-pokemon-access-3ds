//! The gen 6 profile: which of the game's objects to find, the readers they go to, and how each
//! poll runs them.
//!
//! The runtime (`app`) knows when to sweep and when to poll; this knows what a sweep should look for,
//! which reader each object found belongs to, and in what order the readers are polled and their news
//! said. It reads ORAS and Pokémon X and Y, one engine, each game told apart by its `Game`: its
//! numbers, and what it alone has (spike 100).

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::binary::u32;
use crate::core::config::{Navigation, Ui};
use crate::core::discovery::{Discovery, Found, HEAP_SPAN, LINEAR_SPAN, within};
use crate::core::memory::{Cache, Memory, phase};
use crate::core::names::{GamePlace, GameSprite, Names};
use crate::core::rtti::{ClassNames, demangle};
use crate::core::speech::{Category, Speech};
use crate::games::common::announce::{Announcements, ScreenAnnouncer, Speaker};
use crate::games::common::cursor::CursorReader;
use crate::games::common::dialogue::{DialogueReader, Shown};
use crate::games::common::field::{
    Field, Ground, Interactable, WARP_SOURCE, bearing, describe, source_kind, source_number,
};
use crate::games::common::layout::Layouts;
use crate::games::common::layout_file::LayoutFiles;
use crate::games::common::navigate::{
    Bindings, FieldSource, LeadsTo, Navigator, Options as NavigatorOptions, Sides, Street,
};
use crate::games::common::screen::{Pictures, ScreenText, ScreenTextReader};
use crate::games::common::session::Profile;
use crate::games::common::{lyt, msgwin, strbuf};
use crate::games::gen6::apps::{BATTLE, FIELD, MANAGER_CLASS, RunningApp};
use crate::games::gen6::archives::GameText;
use crate::games::gen6::battle::{BattleReader, Battlers, GAUGE_CLASSES, Gauges};
use crate::games::gen6::choices::{BOX_DEVICE_MANAGER, ChoiceReader, DEVICE_MANAGER_CLASS};
use crate::games::gen6::dowsing::Lights;
use crate::games::gen6::field::{
    FieldReader, MOVE_MODEL_CLASS, Options as FieldOptions, PC_TILE, Streets,
};
use crate::games::gen6::game::Game;
use crate::games::gen6::habitat::HabitatReader;
use crate::games::gen6::lists::{CONTROLLER_CLASS, ListReader};
use crate::games::gen6::sky::{Place, SkySource};
use crate::games::gen6::speaker::SpeakerReader;
use crate::games::gen6::textures::Index;
use crate::games::gen6::{camera, gflheap, mapblock, sky, talk};
use crate::py::{self, Error, Log, PySet, Result};

/// What the heap is swept for: message windows, list controllers, the device manager a choice box
/// holds, layout works through their DrawInfo, the battlers' names and HP gauges, the map's move
/// models, the heaps themselves, whose blocks the runtime watches for what they allocate (`app`,
/// ADR-0034), the camera whose heading the D-pad walks by on Lumiose's rails, and the process manager
/// that says which app is running. A game's own readers add what they find their objects by
/// (`Gen6Profile.classes`): ORAS's DexNav, the player's work its hidden Pokémon's record is found
/// through.
///
/// Not TextBoxes or cursor windows: walking each layout's tree finds every one that can be placed on
/// screen (spike 62). And the linear heap is not swept: the battle builds its layouts there, and its
/// message window leads to their pool (spike 64).
pub const HEAP_CLASSES: [&str; 14] = [
    msgwin::CLASSES[0],
    msgwin::CLASSES[1],
    talk::CLASSES[0],
    CONTROLLER_CLASS,
    DEVICE_MANAGER_CLASS,
    lyt::DRAWINFO_CLASS,
    strbuf::CLASS,
    GAUGE_CLASSES[0],
    GAUGE_CLASSES[1],
    MOVE_MODEL_CLASS,
    gflheap::CLASS,
    mapblock::CLASS,
    camera::CLASS,
    MANAGER_CLASS,
];

/// Without a message window class there is nothing to read: not this game, or not this version.
pub const REQUIRED: [&str; 2] = msgwin::CLASSES;

/// `Gen6Profile.__init__`'s arguments after `memory`, `names`, `speech`, `log` and the keyword
/// `game`, each as the Python defaults it: set what differs with `..Options::default()`.
#[derive(Default)]
pub struct Options {
    /// None is `Names()`.
    pub named: Option<Rc<Names>>,
    /// None is `Ui()`.
    pub ui: Option<Ui>,
    pub navigation: Option<Navigation>,
    /// None is `DEFAULT_BINDINGS`.
    pub keys: Option<Bindings>,
    pub text: Option<Rc<GameText>>,
    pub pictures: Option<Rc<Index>>,
    /// The executable as read from its code, `b""` when there is none.
    pub image: Vec<u8>,
}

/// The gen 6 readers, and what each sweep and poll does with them, for ORAS or for X and Y.
///
/// A reader that Python mutates in place sits in a `RefCell`, and a reader that several others hold
/// is an `Rc`; none is borrowed across a call into another reader or into the discovery, which the
/// runtime's `between` can re-enter the profile from (a sweep's `between` polls dialogue and
/// answers keys).
pub struct Gen6Profile {
    memory: Rc<dyn Cache>,
    /// The game's own words out of the player's ROM, or None when it could not be read.
    text: Option<Rc<GameText>>,
    /// The place-name index of the zone the player was last polled in.
    place: Cell<Option<i64>>,
    pub layouts: Rc<Layouts>,
    /// Which screen the game is running, and the moment another starts (spike 99).
    ///
    /// Shared with the sky source's `soaring`, which the Python hands as a bound method.
    pub apps: Rc<RefCell<RunningApp>>,
    pub dialogue: RefCell<DialogueReader>,
    pub lists: RefCell<ListReader>,
    pub choices: RefCell<ChoiceReader>,
    pub screen: RefCell<ScreenTextReader>,
    /// Layout files name the textures a screen's pictures use, which is all the battle's
    /// command menu has to say what its buttons are (spike 50). They live in both heaps.
    pub files: Rc<LayoutFiles>,
    pub cursors: RefCell<CursorReader>,
    /// The battle's name plates: the StrBufs whose text ends in a gender sign, which exist only
    /// while a battle is up (spike 57). Its gauges say which Pokémon each HP panel shows.
    pub battlers: Rc<Battlers>,
    pub gauges: Rc<Gauges>,
    pub battle: RefCell<BattleReader>,
    /// The field says which model is showing a message; what to call it is pa3ds's own, since
    /// the game keeps no name for anyone on its maps (spikes 73 to 77).
    pub speaker: Rc<SpeakerReader>,
    /// The map: the player's cell, and the zone data and models around it (spike 78), listed
    /// in the log as zones are entered and walked through by the navigation keys.
    ///
    /// Python's attribute is assigned to (a test puts its own reader in); the navigator keeps the
    /// one it was made with, as it does there, and what is read through the profile is read
    /// through whichever this holds (`field`, `set_field`).
    field: Rc<RefCell<Rc<FieldReader>>>,
    /// What the heap is swept for in this game: `HEAP_CLASSES`, and what the game's own
    /// readers find their objects by.
    pub classes: Vec<String>,
    pub navigator: RefCell<Navigator>,
    log: Log,
    speech: Rc<Speech>,
    /// The Dowsing Machine's lights, in a game where they have been measured.
    pub dowsing: Option<RefCell<Lights>>,
    /// The zone last listed and what was in it (people by kind alone, since they walk), to
    /// list again only when something comes or goes.
    listed: RefCell<Option<(i64, Vec<(String, String, Option<(i64, i64)>)>)>>,
    /// Whether a sweep has been taken in since the last listing.
    relist: Cell<bool>,
    /// The chunks the last listing was narrowed by, as logged.
    chunks: RefCell<Vec<(i64, i64)>>,
    /// The hidden Pokémon last seen, species and cell, to list again when one comes or goes
    /// between sweeps.
    hiding: Cell<Option<(i64, (i64, i64))>>,
    /// The record's raw words as last logged, so a change is logged whether or not it
    /// passes as a Pokémon: the user heard a rustle the list did not have (2026-09-22).
    hidden_words: RefCell<Option<String>>,
    /// Whether the player was riding the bike at the last field poll, to log each change.
    on_bike: Cell<bool>,
    pub announcements: Rc<RefCell<Announcements>>,
    pub screen_announcer: RefCell<ScreenAnnouncer>,
    /// The Pokédex's habitat map draws the Pokémon's places and never names them (spike 152).
    pub habitat: RefCell<HabitatReader>,
    /// (work, root) of every layout whose files have been looked for.
    archived: RefCell<HashSet<(i64, i64)>>,
    /// The zone the player was last polled in.
    zone: Cell<Option<i64>>,
    /// Message windows the field has shown a page in that no talk object kept was showing,
    /// swept for once each, and again after the player enters another zone.
    unheard: RefCell<HashSet<i64>>,
    /// Whether a sweep is wanted for the talk object showing a page.
    speaker_wanted: Cell<bool>,
    /// Every device manager a sweep has found. It lives all session, so a choice box built
    /// later is found in the blocks just allocated by the manager it holds. Iterated by
    /// `Found.holders`, whose result follows its order, so a `PySet`.
    managers: RefCell<PySet>,
    /// The cell the Secret Base's decorating cursor was last said on, and what was there.
    decorating: RefCell<Option<((i64, i64), String)>>,
    /// The sky's places, where the game has any (`GameText.sky`).
    sky: Rc<Vec<Place>>,
    /// The heading the player soared with at the last poll, and the compass point last said
    /// for it: said again once a turn stops on another.
    heading: Cell<Option<f64>>,
    facing: RefCell<Option<String>>,
}

/// `FieldReader` is what a `Navigator` reads the map through in the gen 6 games.
impl FieldSource for FieldReader {
    fn visit(&self) -> Option<(i64, i64)> {
        FieldReader::visit(self)
    }

    fn poll(&self) -> Result<Option<Field>> {
        FieldReader::poll(self)
    }

    fn interactables(&self, r#where: &Field) -> Result<Field> {
        FieldReader::interactables(self, r#where)
    }

    fn ground(&self, r#where: &Field, height: Option<f64>) -> Result<Ground> {
        FieldReader::ground(self, r#where, height)
    }
}

impl Gen6Profile {
    pub fn new(
        memory: Rc<dyn Cache>,
        names: Rc<ClassNames>,
        speech: Rc<Speech>,
        log: Log,
        game: &Game,
        options: Options,
    ) -> Gen6Profile {
        let Options {
            named,
            ui,
            navigation,
            keys,
            text,
            pictures,
            image,
        } = options;
        let cache = memory;
        let memory: Rc<dyn Memory> = cache.clone();
        let named = named.unwrap_or_else(|| Rc::new(Names::empty()));
        let layouts = Rc::new(Layouts::new(memory.clone(), names.clone(), lyt::ORAS));
        let apps = Rc::new(RefCell::new(RunningApp::new(memory.clone(), names.clone())));
        let dialogue = DialogueReader::new(memory.clone(), names.clone(), Some(layouts.clone()));
        let lists = ListReader::new(memory.clone(), names.clone());
        let choices = ChoiceReader::new(memory.clone(), names.clone());
        // A picture whose texture the game swaps as it runs is named by the image it draws: the
        // Pokédex's language tabs and its type (spike 147).
        let named_by_image: Option<Pictures> = pictures.map(|pictures| {
            let memory = memory.clone();
            Rc::new(move |pane: i64| pictures.said(&*memory, pane)) as Pictures
        });
        let screen = ScreenTextReader::with(
            memory.clone(),
            names.clone(),
            layouts.clone(),
            msgwin::CLASSES
                .iter()
                .map(|class| class.to_string())
                .collect(),
            None,
            named_by_image.clone(),
        );
        let files = Rc::new(LayoutFiles::new(memory.clone()));
        let cursors = CursorReader::with(
            layouts.clone(),
            names.clone(),
            Some(files.clone()),
            false,
            named_by_image,
        );
        let battlers = Rc::new(Battlers::new(memory.clone(), names.clone()));
        let gauges = Rc::new(Gauges::new(memory.clone(), names.clone()));
        let battle = BattleReader::new(layouts.clone(), battlers.clone(), gauges.clone());
        let speaker = Rc::new(SpeakerReader::new(
            memory.clone(),
            named.clone(),
            game.drawn,
        ));
        let hidden = game.hidden.map(|make| make(memory.clone()));
        // Every spot a Secret Base can be made at is listed in the executable, not in any zone's
        // data, and which one is the player's is the save's (spike 154).
        let bases = match game.secret_bases {
            Some(make) if !image.is_empty() => Some(make(memory.clone(), &image)),
            _ => None,
        };
        // Route 113's ash piles are in a list its own module writes as it starts (spike 155).
        let ash = game.ash.map(|make| make(memory.clone()));
        // Fortree Gym's doors are built by the Gym's own module as it starts (spike 161).
        let doors = game.doors.map(|make| make(memory.clone()));
        // Sootopolis Gym's thin ice is kept by the Gym's own module, a byte a tile (spike 167).
        let ice = game.ice.map(|make| make(memory.clone()));
        let words = match text.as_ref().and_then(|text| text.pc.clone()) {
            Some(pc) if !pc.is_empty() => IndexMap::from([(PC_TILE, pc)]),
            _ => IndexMap::new(),
        };
        let streets: Option<Streets> = text
            .as_ref()
            .and_then(|text| text.streets.clone())
            .map(|streets| Rc::new(move |chunks: &[i64]| streets.network(chunks)) as Streets);
        let chunks_of = text.as_ref().and_then(|text| text.map_chunks.clone());
        let field = FieldReader::with(
            memory.clone(),
            FieldOptions {
                words,
                drawn: game.drawn,
                streets,
                outdoors: text
                    .as_ref()
                    .map(|text| text.outdoors.clone())
                    .unwrap_or_default(),
                street_records: text
                    .as_ref()
                    .map(|text| text.street_records.clone())
                    .unwrap_or_default(),
                hidden: hidden.clone(),
                belts: game.belts.clone(),
                far: chunks_of.clone().map(|chunks| {
                    Rc::new(move |file: i64| chunks.cells(file))
                        as crate::games::gen6::field::FarCells
                }),
                far_heights: chunks_of.clone().map(|chunks| {
                    Rc::new(move |file: i64| chunks.heights(file))
                        as crate::games::gen6::field::FarHeights
                }),
                far_floors: chunks_of.clone().map(|chunks| {
                    Rc::new(move |file: i64| chunks.floors(file))
                        as crate::games::gen6::field::FarFloors
                }),
                matrices: chunks_of.map(|chunks| {
                    Rc::new(move |number: i64| chunks.matrix(number))
                        as crate::games::gen6::field::FarMatrices
                }),
                clears: game.clears.clone(),
                berries: text
                    .as_ref()
                    .map(|text| text.berries.clone())
                    .unwrap_or_default(),
                barriers: game.barriers.clone(),
                triggers: game.triggers.clone(),
                furniture: game.furniture.clone(),
                secret_bases: bases.clone(),
                decorations: text
                    .as_ref()
                    .map(|text| text.decorations.clone())
                    .unwrap_or_default(),
                ash: ash.clone(),
                doors: doors.clone(),
                blocker: game.blocker,
                stairs: game.stairs.clone(),
                ice: ice.clone(),
                ..FieldOptions::default()
            },
        );
        let mut classes: IndexSet<String> =
            HEAP_CLASSES.iter().map(|class| class.to_string()).collect();
        if let Some(hidden) = &hidden {
            classes.extend(hidden.classes().into_iter().map(String::from));
        }
        if let Some(bases) = &bases {
            classes.extend(bases.classes().into_iter().map(String::from));
        }
        if let Some(ash) = &ash {
            classes.extend(ash.classes().into_iter().map(String::from));
        }
        if let Some(doors) = &doors {
            classes.extend(doors.classes().into_iter().map(String::from));
        }
        if let Some(ice) = &ice {
            classes.extend(ice.classes().into_iter().map(String::from));
        }
        // The navigator says what it finds by the name it has been given, and renames it on a
        // key: the player standing in front of a sign is the only one who can name it.
        let ui = ui.unwrap_or_default();
        let announcements = Rc::new(RefCell::new(Announcements::with(
            speech.clone(),
            log.clone(),
            Some(name_for(&speaker)),
            false,
        )));
        let field = Rc::new(RefCell::new(field));
        // Soaring on Latios, the keys walk the sky's places instead, which the field's stale
        // lists know nothing of (spike 166).
        let known: Rc<Vec<Place>> = Rc::new(
            text.as_ref()
                .map(|text| text.sky.clone())
                .unwrap_or_default(),
        );
        let soaring: Rc<dyn Fn() -> Option<i64>> = {
            let (apps, known) = (apps.clone(), known.clone());
            Rc::new(move || soaring_of(&apps, &known))
        };
        let navigator = Navigator::new(
            Rc::new(SkySource::new(
                memory.clone(),
                field.borrow().clone() as Rc<dyn FieldSource>,
                soaring,
                known.to_vec(),
            )),
            speech.clone(),
            log.clone(),
            NavigatorOptions {
                bindings: keys,
                names: Some(named.clone()),
                navigation,
                speaker: Some({
                    let speaker = speaker.clone();
                    Rc::new(move || speaker.last())
                }),
                leads_to: Some(leads_to_of(text.clone(), field.clone())),
                street: Some(street_of(text.clone())),
                moves: text.as_ref().map(|text| text.moves.clone()),
                bike: text.as_ref().and_then(|text| text.bike.clone()),
                acro: text.as_ref().and_then(|text| text.acro.clone()),
                sides: Some({
                    let announcements = announcements.clone();
                    Rc::new(move |player: bool| announcements.borrow().sides(player)) as Sides
                }),
                ..NavigatorOptions::default()
            },
        );
        let dowsing = game
            .dowsing
            .map(|offset| RefCell::new(Lights::new(memory.clone(), offset)));
        let screen_announcer = ScreenAnnouncer::with(speech.clone(), log.clone(), ui.pauses);
        let habitat = HabitatReader::new(
            memory.clone(),
            layouts.clone(),
            files.clone(),
            text.as_ref()
                .map(|text| text.map_places.clone())
                .unwrap_or_default(),
            speech.clone(),
            log.clone(),
            ui.pauses,
        );
        // The rival is a work object in RAM and a name in the ROM, and so are some doors' buildings:
        // the game's own word, behind whatever the player has called them and ahead of pa3ds's.
        let sprite: GameSprite = {
            let text = text.clone();
            Rc::new(move |character| game_word(text.as_deref(), character))
        };
        let place: GamePlace = {
            let text = text.clone();
            Rc::new(move |zone, slot| game_place(text.as_deref(), zone, slot))
        };
        named.use_game(Some(sprite), Some(place));
        Gen6Profile {
            memory: cache,
            text,
            place: Cell::new(None),
            layouts,
            apps,
            dialogue: RefCell::new(dialogue),
            lists: RefCell::new(lists),
            choices: RefCell::new(choices),
            screen: RefCell::new(screen),
            files,
            cursors: RefCell::new(cursors),
            battlers,
            gauges,
            battle: RefCell::new(battle),
            speaker,
            field,
            classes: classes.into_iter().collect(),
            navigator: RefCell::new(navigator),
            log,
            speech,
            dowsing,
            listed: RefCell::new(None),
            relist: Cell::new(false),
            chunks: RefCell::new(Vec::new()),
            hiding: Cell::new(None),
            hidden_words: RefCell::new(None),
            on_bike: Cell::new(false),
            announcements,
            screen_announcer: RefCell::new(screen_announcer),
            habitat: RefCell::new(habitat),
            archived: RefCell::new(HashSet::new()),
            zone: Cell::new(None),
            unheard: RefCell::new(HashSet::new()),
            speaker_wanted: Cell::new(false),
            managers: RefCell::new(PySet::new()),
            decorating: RefCell::new(None),
            sky: known,
            heading: Cell::new(None),
            facing: RefCell::new(None),
        }
    }

    /// `profile.field`: the reader the profile reads the map through now.
    pub fn field(&self) -> Rc<FieldReader> {
        self.field.borrow().clone()
    }

    /// `profile.field = field`.
    pub fn set_field(&self, field: Rc<FieldReader>) {
        *self.field.borrow_mut() = field;
    }

    /// Hand each reader what a sweep found of its classes.
    ///
    /// A sweep of the blocks the game has just allocated is partial: it says what is new and
    /// nothing of what is gone, so readers add what it found to what they know, and the layouts
    /// are looked over only when it brings new ones (spike 110).
    pub fn track(&self, found: &Found) -> Result<()> {
        let adding = found.partial;
        self.dialogue
            .borrow_mut()
            .track(&found.of(&msgwin::CLASSES), adding);
        self.apps.borrow_mut().track(found.of(&[MANAGER_CLASS]))?;
        // A talk object's class is a DLL's while a talk runs, so every one found is kept.
        self.speaker.track(found.of(&talk::CLASSES));
        self.lists.borrow_mut().track(found.of(&[CONTROLLER_CLASS]));
        // A choice box has no class of its own; the device manager it holds does.
        for manager in found.of(&[DEVICE_MANAGER_CLASS]) {
            self.managers.borrow_mut().add(manager);
        }
        let managers: Vec<i64> = self.managers.borrow().iter().collect();
        let holders = found.holders(&managers)?;
        let field = self.field();
        phase(&*self.memory, "tracking", || {
            // A layout's work has no class of its own; the DrawInfo embedded in it does.
            let works: Vec<i64> = found
                .of(&[lyt::DRAWINFO_CLASS])
                .into_iter()
                .map(|at| at - lyt::WORK_DRAWINFO)
                .collect();
            let watched = self.layouts.pools();
            if adding {
                // A screen's pool is made before its layouts are built into it.
                self.layouts.watch_pools_in(&found.pages)?;
            }
            if !works.is_empty() || !adding {
                self.layouts.track(works.iter().copied())?;
            }
            for (pool, capacity) in self.layouts.pools() {
                if !watched.contains_key(&pool) {
                    (self.log)(&format!(
                        "layouts: watching the pool of {capacity} at {pool:#010x}"
                    ));
                }
            }
            if !works.is_empty() || !adding {
                // Every TextBox and cursor window worth reading is under a layout's tree.
                self.screen.borrow_mut().track([]);
                self.cursors
                    .borrow_mut()
                    .track(self.layouts.panes(lyt::WINDOW_CLASS))?;
            }
            self.battlers.track(found.of(&[strbuf::CLASS]), adding)?;
            let [ours, theirs] = GAUGE_CLASSES;
            self.gauges
                .track(found.of(&[ours]), found.of(&[theirs]), adding)?;
            self.choices
                .borrow_mut()
                .track(holders.iter().map(|holder| holder - BOX_DEVICE_MANAGER))?;
            field.track(found)
        })?;
        // Listing the field reads every model in the pool: after a whole sweep, or when the
        // blocks just allocated held the field's own.
        if !adding || !found.of(&[MOVE_MODEL_CLASS, mapblock::CLASS]).is_empty() {
            self.relist.set(true);
        }
        self.read_archives()
    }

    /// Whether a reader needs something only a sweep finds, sooner than the schedule has it.
    pub fn sweep_wanted(&self) -> Result<bool> {
        let battle = self.battle.borrow_mut().sweep_wanted();
        let running = self.apps.borrow().running.clone();
        let field = self
            .field()
            .sweep_wanted(running.as_deref().is_none_or(|running| running == FIELD))?;
        if let Some(field) = field
            && !field.is_empty()
        {
            (self.log)(&format!("field: {field}"));
        }
        let app = self.apps.borrow_mut().sweep_wanted();
        if app {
            (self.log)("app: no process manager, sweeping for it");
        }
        let speaker = self.speaker_wanted.replace(false);
        if speaker {
            (self.log)("speaker: no talk object shows the page, sweeping for it");
        }
        Ok(battle || field.is_some() || speaker || app)
    }

    /// Whether another app has started since the last poll: the field, the Bag, the Pokédex, a
    /// battle. One read a poll, and the moment to sweep for the new screen's objects.
    pub fn poll_app(&self) -> Result<bool> {
        let started = phase(&*self.memory, "app", || self.apps.borrow_mut().poll())?;
        let Some(started) = started else {
            return Ok(false);
        };
        (self.log)(&format!("app: {}", demangle(&started)));
        self.dialogue.borrow_mut().forget_hosts();
        // A battle's sides are said again on a key only while it lasts.
        self.announcements.borrow_mut().forget_sides();
        if started == BATTLE {
            self.battle.borrow_mut().begin();
        }
        // Each flight says the way the player faces as it settles, the first included.
        self.heading.set(None);
        *self.facing.borrow_mut() = None;
        Ok(true)
    }

    /// Whether the app just started wants the whole heap swept as it builds: not the field
    /// coming back after a battle or a menu, which builds its map blocks again where they were
    /// and keeps the rest; those sweeps found nothing a reader asks for by class, 16 of 65 over
    /// movies 4 and 7 (spike 134).
    pub fn app_sweep_wanted(&self) -> bool {
        let apps = self.apps.borrow();
        !(apps.running.as_deref() == Some(FIELD) && apps.back)
    }

    /// Poll dialogue. Returns whether a page was said.
    pub fn poll_dialogue(&self) -> Result<bool> {
        let shown = phase(&*self.memory, "dialogue", || {
            self.dialogue.borrow_mut().poll()
        })?;
        let said = self.announcements.borrow_mut().pages(&shown)?;
        if said && self.apps.borrow().running.as_deref() == Some(FIELD) {
            self.look_for_speakers(&shown)?;
        }
        Ok(said)
    }

    /// Ask for a sweep when the field shows a page no talk object kept is showing. A door
    /// builds the next zone's talk windows into memory taken as the field loaded, a few seconds
    /// on, where no sweep of new blocks looks, and the first pages Mom said in the house had no
    /// name to them (movie1, 2026-09-26). Asked as the talk starts, the sweep finds it for the
    /// pages after; swept for at every door instead, the replays went their own way.
    fn look_for_speakers(&self, shown: &[Shown]) -> Result<()> {
        for page in shown {
            if let Shown::Page(page) = page
                && !self.unheard.borrow().contains(&page.window)
                && !self.speaker.shows(page.window)?
            {
                self.unheard.borrow_mut().insert(page.window);
                self.speaker_wanted.set(true);
            }
        }
        Ok(())
    }

    /// Poll lists and choice boxes. Returns whether screen text was polled along the way.
    pub fn poll_menus(&self, discovery: &mut Discovery) -> Result<bool> {
        let focuses = phase(&*self.memory, "lists", || self.lists.borrow_mut().poll())?;
        let mut screen_polled = false;
        if focuses.iter().any(|focus| focus.opened) {
            // The question comes before its prompt, but its window can be new and not yet swept:
            // at real speed "YES, 1 of 2" was heard 1.6 s before "So you're wee?". Sweep now and
            // say the question first.
            let swept = discovery.sweep_for(&msgwin::CLASSES)?;
            self.dialogue.borrow_mut().track(&swept, false);
            self.poll_dialogue()?;
            // A question the screen draws itself, such as "Would you like to make these changes?",
            // comes first too, without waiting for it to hold.
            self.poll_screen_with(false, false)?;
            screen_polled = true;
        }
        for focus in &focuses {
            // The save's YES was read by the cursor over it before a sweep found its list, and
            // heard again from the list a second later (movie1, 2026-09-26).
            let heard = focus.opened
                && self
                    .screen_announcer
                    .borrow_mut()
                    .said_by_cursor(&focus.label);
            self.announcements.borrow_mut().focus(focus, "list", heard);
        }
        // A choice box opens only after A on its question, so the question is already said.
        let chosen = phase(&*self.memory, "choices", || {
            self.choices.borrow_mut().poll()
        })?;
        for focus in &chosen {
            // A box that draws a cursor picture beside its words is the cursor reader's from the
            // poll it is shown, and this reader's only once a sweep has found it (spike 98).
            let heard = focus.opened
                && self
                    .screen_announcer
                    .borrow_mut()
                    .said_by_cursor(&focus.label);
            self.announcements
                .borrow_mut()
                .focus(focus, "choice", heard);
        }
        Ok(screen_polled)
    }

    /// `poll_screen()`: `hold` and `with_choices` both True, which is what the runtime asks.
    pub fn poll_screen(&self) -> Result<()> {
        self.poll_screen_with(true, true)
    }

    /// `poll_screen(hold, with_choices)`. Named apart from `poll_screen` because the runtime's
    /// `Profile::poll_screen` takes neither.
    pub fn poll_screen_with(&self, hold: bool, with_choices: bool) -> Result<()> {
        let (texts, on_screen, sides, items) = phase(&*self.memory, "screen", || {
            // The field's layouts come back where they were when a menu screen closes, and a
            // layout found before it was built is taken in once it is.
            let revived = self.layouts.revive()?;
            // The battle's layouts are built into a pool its message window leads to.
            let failed: RefCell<Option<Error>> = RefCell::new(None);
            self.layouts.watch_pools(self.linear_handles(&failed))?;
            if let Some(error) = failed.into_inner() {
                return Err(error);
            }
            if self.layouts.poll_pools()? || revived {
                self.cursors
                    .borrow_mut()
                    .track(self.layouts.panes(lyt::WINDOW_CLASS))?;
            }
            let exclude: HashSet<i64> = self.dialogue.borrow().line_objects().iter().collect();
            let texts = self.screen.borrow_mut().poll(&exclude, hold)?;
            // A message box's lines lead to a window typing into them before any sweep finds it.
            let lines: Vec<i64> = self.screen.borrow().message_lines().into_iter().collect();
            self.dialogue.borrow_mut().watch(&lines);
            let on_screen = self.screen.borrow().on_screen();
            // The HP box is the battle reader's to say, whole and once it holds still.
            let sides = self.battle.borrow_mut().poll(&on_screen)?;
            self.habitat.borrow_mut().poll(&on_screen)?;
            let items = self.cursors.borrow_mut().poll(&on_screen)?;
            Ok((texts, on_screen, sides, items))
        })?;
        let mut labels = self.lists.borrow().labels();
        labels.extend(self.announcements.borrow_mut().recent_labels());
        if with_choices {
            labels.extend(self.choices.borrow().labels());
        }
        let mut kept: Vec<ScreenText> = Vec::new();
        for text in &texts {
            if !self.battle.borrow().owns(text)? && !self.habitat.borrow().owns(text) {
                kept.push(text.clone());
            }
        }
        let under = self.cursors.borrow().under();
        // A layout built again where it was is the one that was there, and a menu stays a menu:
        // the battle's move list, forgotten while it was gone, was read out whole the moment it
        // showed.
        let mut roots = self.layouts.roots();
        roots.extend(self.layouts.buried());
        let settling = self.screen.borrow().settling();
        let left = self.screen.borrow().left();
        let listed = self.cursors.borrow().lists();
        let moving = self.cursors.borrow().moving();
        self.screen_announcer.borrow_mut().say(
            &kept,
            &items,
            &under,
            &on_screen,
            &labels,
            Some(&roots),
            &settling,
            &left,
            Some(&listed),
            &moving,
        )?;
        self.announcements.borrow_mut().battlers(&sides);
        self.announcements.borrow_mut().screen_polled();
        self.read_archives()
    }

    /// Answer a navigation key, by name.
    pub fn press(&self, key: &str) -> Result<()> {
        phase(&*self.memory, "keys", || {
            self.navigator.borrow_mut().press_key(key)
        })
    }

    /// Follow the player along a path they are walking, every poll while there is one: the
    /// field's own poll is every fifth of a second, and a turn said that late is a step the
    /// player has already taken past it. Nothing is read while there is none.
    pub fn poll_guidance(&self) -> Result<()> {
        if !self.navigator.borrow().guiding() {
            return Ok(());
        }
        phase(&*self.memory, "guidance", || {
            let at = self.field().poll()?;
            if let Some(at) = at {
                self.navigator.borrow_mut().follow(&at)?;
            }
            Ok(())
        })
    }

    /// Sample where each named cursor on screen stands, every tick while there is one, so the
    /// screen poll can tell one at rest from one gliding past (spike 129). One small read each.
    pub fn poll_pointers(&self) -> Result<()> {
        if !self.cursors.borrow().sampling() {
            return Ok(());
        }
        phase(&*self.memory, "pointers", || {
            self.cursors.borrow_mut().sample(&*self.memory)
        })
    }

    /// Keep any name the player has just given. Costs one empty queue read a poll, and the
    /// dialog it answers has been up on a thread of its own while the game kept running.
    pub fn poll_renames(&self) -> Result<()> {
        self.navigator.borrow_mut().poll_renames()
    }

    /// Poll where the player is, and log what is around them when it changes.
    ///
    /// The listing reads every model in the pool, so it is made only on a change of zone and
    /// after each sweep, not every poll.
    pub fn poll_field(&self) -> Result<()> {
        if self.soaring().is_some() {
            return self.poll_sky();
        }
        let field = self.field();
        // `None` is the Python's `return` from inside the `with`.
        let polled = phase(&*self.memory, "field", || -> Result<Option<Field>> {
            self.poll_dowsing()?;
            let Some(mut at) = field.poll()? else {
                return Ok(None);
            };
            if Some(at.zone) != self.zone.get() {
                self.zone.set(Some(at.zone));
                self.unheard.borrow_mut().clear();
            }
            if at.on_bike != self.on_bike.get() {
                self.on_bike.set(at.on_bike);
                (self.log)(&format!(
                    "field: {} the bike",
                    if at.on_bike { "on" } else { "off" }
                ));
            }
            self.arrive(at.zone)?;
            self.poll_decorating(&at)?;
            // A hidden Pokémon shows and goes between sweeps, so its record is read every poll.
            let hiding = field.hidden(at.zone)?;
            let key = hiding.map(|hiding| (hiding.code, hiding.cell));
            let words = field.hidden_words()?;
            if words != *self.hidden_words.borrow() {
                (self.log)(&format!(
                    "field hidden record: {}",
                    words.as_deref().unwrap_or("None")
                ));
                *self.hidden_words.borrow_mut() = words;
            }
            if key != self.hiding.get() {
                self.hiding.set(key);
                self.relist.set(true);
            }
            if !self.relist.get()
                && let Some(listed) = &*self.listed.borrow()
                && listed.0 == at.zone
            {
                return Ok(None);
            }
            at = field.interactables(&at)?;
            // Which chunks the edges were narrowed by, for the log: the blocks come and go
            // with the map, and a border listed whole is a border no chunk covers.
            let mut chunks: Vec<(i64, i64)> = field
                .chunks_loaded()?
                .iter()
                .map(|chunk| chunk.origin)
                .collect();
            chunks.sort();
            if chunks != *self.chunks.borrow() {
                let shown = if chunks.is_empty() {
                    "none".to_string()
                } else {
                    format!(
                        "[{}]",
                        chunks
                            .iter()
                            .map(|(x, z)| format!("({x}, {z})"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                *self.chunks.borrow_mut() = chunks;
                (self.log)(&format!("field chunks: {shown}"));
            }
            Ok(Some(at))
        })?;
        let Some(at) = polled else {
            return Ok(());
        };
        self.relist.set(false);
        let mut items: Vec<(String, String, Option<(i64, i64)>)> = at
            .interactables
            .iter()
            .map(|item| {
                (
                    item.category.clone(),
                    item.kind.clone(),
                    if item.category == "people" {
                        None
                    } else {
                        Some(item.cell)
                    },
                )
            })
            .collect();
        items.sort();
        let listed = (at.zone, items);
        if *self.listed.borrow() == Some(listed.clone()) {
            return Ok(());
        }
        *self.listed.borrow_mut() = Some(listed);
        (self.log)(&format!(
            "field: zone {:#x} at ({}, {})",
            at.zone, at.cell.0, at.cell.1
        ));
        let zone = at.zone;
        let called = |item: &Interactable| -> Result<Option<String>> {
            Ok(self.navigator.borrow().called(item, zone))
        };
        let leads = |item: &Interactable| -> Result<Option<String>> {
            self.navigator.borrow().destination(item, zone)
        };
        for line in describe(&at, Some(&called), Some(&leads)) {
            (self.log)(&format!("field {}", line?));
        }
        Ok(())
    }

    /// The sky's process while the player soars on Latios and its places are known, else None.
    fn soaring(&self) -> Option<i64> {
        soaring_of(&self.apps, &self.sky)
    }

    /// Say the compass point the player faces once a turn stops on another, as the bottom
    /// screen's arrow shows it: one small read a field poll while soaring.
    fn poll_sky(&self) -> Result<()> {
        let Some(process) = self.soaring() else {
            return Ok(());
        };
        let flight = phase(&*self.memory, "sky", || sky::read(&*self.memory, process))?;
        let Some(flight) = flight else {
            return Ok(());
        };
        let was = self.heading.replace(Some(flight.heading));
        let Some(was) = was else {
            return Ok(());
        };
        if (flight.heading - was).abs() > sky::STILL {
            return Ok(());
        }
        let word = sky::facing(flight.heading)?;
        if self.facing.borrow().as_deref() != Some(word) {
            *self.facing.borrow_mut() = Some(word.to_string());
            (self.log)(&format!(
                "sky: facing {word} at ({}, {}) height {}",
                py::fixed(flight.x, 0),
                py::fixed(flight.z, 0),
                py::fixed(flight.height, 0)
            ));
            self.speech
                .say(&format!("facing {word}"), Category::Guidance);
        }
        Ok(())
    }

    /// Say what the Dowsing Machine's lights show as it changes: three small reads a field
    /// poll, and nothing while no field runs.
    fn poll_dowsing(&self) -> Result<()> {
        let (process, running) = {
            let apps = self.apps.borrow();
            (apps.process, apps.running.clone())
        };
        let (Some(dowsing), Some(process)) = (&self.dowsing, process) else {
            return Ok(());
        };
        if running.as_deref() != Some(FIELD) {
            return Ok(());
        }
        let said = dowsing.borrow_mut().poll(process)?;
        if let Some(said) = said {
            (self.log)(&format!("field {said}"));
            self.speech.say(&said, Category::Screen);
        }
        Ok(())
    }

    /// Say what the Secret Base's decorating cursor is on and where, each time it moves or
    /// what is there changes: a desk set down under it is said as it lands.
    fn poll_decorating(&self, r#where: &Field) -> Result<()> {
        let on = self.field().decorating(r#where)?;
        if on == *self.decorating.borrow() {
            return Ok(());
        }
        *self.decorating.borrow_mut() = on.clone();
        if let Some((cell, words)) = on {
            let said = format!("{words}, {}", bearing(r#where.cell, cell));
            (self.log)(&format!("field cursor: {said}"));
            self.speech.say(&said, Category::Menu);
        }
        Ok(())
    }

    /// Say where the player is when the place changes, not the zone: a town and every
    /// building in it share a place, so walking out of a house arrives nowhere, as the game has
    /// it (spike 87). The name comes from RAM's index and the ROM's words, ahead of AreaNav's.
    fn arrive(&self, zone: i64) -> Result<()> {
        let Some(text) = &self.text else {
            return Ok(());
        };
        let index = self.field().place_index(zone)?;
        let Some(index) = index else {
            return Ok(());
        };
        if Some(index) == self.place.get() {
            return Ok(());
        }
        self.place.set(Some(index));
        if let Some(name) = text.place(index) {
            self.screen_announcer.borrow_mut().place(&name);
        }
        Ok(())
    }

    /// The layout handles of the message windows read, where they lead out of the heap.
    ///
    /// Only pools out of the heap are read this way. The heap's layouts wait for the sweep, which
    /// finds them with the objects that own them: a list's layout taken in before its list has
    /// its labels read as screen text (`Layouts.revive`).
    ///
    /// Python's is a generator, so each handle is read as `watch_pools` asks for it, between its
    /// own reads. A read that fails ends the iteration and is left in `failed` for the caller to
    /// raise, where the generator raised through `watch_pools`.
    fn linear_handles<'a>(
        &'a self,
        failed: &'a RefCell<Option<Error>>,
    ) -> impl Iterator<Item = i64> + 'a {
        let mut windows = self.dialogue.borrow().windows().into_iter();
        std::iter::from_fn(move || {
            loop {
                let window = windows.next()?;
                let entry = self
                    .memory
                    .read(window + msgwin::LAYOUT_HANDLE, 4)
                    .and_then(|read| u32(&read, 0));
                match entry {
                    Err(error) => {
                        *failed.borrow_mut() = Some(error);
                        return None;
                    }
                    Ok(entry) if within(LINEAR_SPAN, entry) => return Some(entry),
                    Ok(_) => {}
                }
            }
        })
    }

    /// Find the layout files of every layout tracked since the last look, in its own archive.
    ///
    /// Read outside any phase, since each archive is read once.
    fn read_archives(&self) -> Result<()> {
        for (work, root) in self.layouts.works() {
            if self.archived.borrow().contains(&(work, root)) {
                continue;
            }
            self.archived.borrow_mut().insert((work, root));
            let resource = u32(&self.memory.read(work + lyt::WORK_RESOURCE, 4)?, 0)?;
            if !in_a_heap(resource) {
                continue;
            }
            let archive = u32(&self.memory.read(resource + lyt::RESOURCE_ARCHIVE, 4)?, 0)?;
            if in_a_heap(archive) {
                self.files.add_archive(archive)?;
            }
        }
        Ok(())
    }
}

/// `Gen6Profile._soaring`, which the sky source is handed as a callable of its own.
fn soaring_of(apps: &RefCell<RunningApp>, known: &[Place]) -> Option<i64> {
    let apps = apps.borrow();
    let running = apps.running.as_deref() == Some(sky::APP) && !known.is_empty();
    if running { apps.process } else { None }
}

/// `speaker.name_for`, as the `Speaker` that `Announcements` asks who is showing a window.
fn name_for(speaker: &Rc<SpeakerReader>) -> Speaker {
    let speaker = speaker.clone();
    Rc::new(move |window| speaker.name_for(window))
}

/// What the game calls a character that RAM gives only a number: the rival, by the
/// sprite a work object was filled with (spike 89).
fn game_word(text: Option<&GameText>, character: i64) -> Option<String> {
    text.and_then(|text| text.character(character))
}

/// What the game calls a slot on a map: the building behind a door in from outdoors,
/// where the ROM's own tables pair the two (spike 86). Only a warp's slot is a door's: a sign
/// is numbered in a table of its own.
fn game_place(text: Option<&GameText>, zone: i64, slot: i64) -> Option<String> {
    let text = text?;
    if source_kind(slot) != WARP_SOURCE {
        return None;
    }
    text.door(zone, source_number(slot))
}

/// What the game calls the place an exit from `here` into `there` arrives in, or None when
/// it is the place the player is in already.
///
/// A place, not a zone, for the reason `arrive` gives: Littleroot and every house in it are
/// one place, so the door of a house leads nowhere new and says nothing more, while the gap
/// at the top of the town leads onto Route 101 and says so (the user, 2026-09-24). Both
/// indices are the zone table's in RAM and the words the ROM's, as for arriving.
///
/// The other zone of the player's own matrix is said all the same: Route 119 is two zones
/// the game calls by one name, and the edge between them was the only exit said bare, "edge"
/// (the user, 2026-10-03, spike 159).
fn leads_to_of(text: Option<Rc<GameText>>, field: Rc<RefCell<Rc<FieldReader>>>) -> LeadsTo {
    Rc::new(move |here, there| {
        let Some(text) = &text else {
            return Ok(None);
        };
        let field = field.borrow().clone();
        let (ours, theirs) = (field.place_index(here)?, field.place_index(there)?);
        let (Some(ours), Some(theirs)) = (ours, theirs) else {
            return Ok(None);
        };
        if ours == theirs && !field.one_matrix(here, there)? {
            return Ok(None);
        }
        Ok(text.place(theirs))
    })
}

/// The street of Lumiose's a warp between two of its zones comes out on, in the game's
/// own words (spike 107): its zones are all "Lumiose City", so the place says nothing.
fn street_of(text: Option<Rc<GameText>>) -> Street {
    Rc::new(move |zone, slot| {
        let Some(text) = &text else {
            return Ok(None);
        };
        if source_kind(slot) != WARP_SOURCE {
            return Ok(None);
        }
        Ok(text.street(zone, source_number(slot)))
    })
}

/// Is this in the heap or the linear heap, where a work's resource and archive have been?
fn in_a_heap(address: i64) -> bool {
    within(HEAP_SPAN, address) || within(LINEAR_SPAN, address)
}

impl Profile for Gen6Profile {
    fn track(&self, found: &Found) -> Result<()> {
        Gen6Profile::track(self, found)
    }

    fn poll_app(&self) -> Result<bool> {
        Gen6Profile::poll_app(self)
    }

    fn app_sweep_wanted(&self) -> bool {
        Gen6Profile::app_sweep_wanted(self)
    }

    fn poll_dialogue(&self) -> Result<bool> {
        Gen6Profile::poll_dialogue(self)
    }

    fn poll_menus(&self, discovery: &mut Discovery) -> Result<bool> {
        Gen6Profile::poll_menus(self, discovery)
    }

    fn poll_screen(&self) -> Result<()> {
        Gen6Profile::poll_screen(self)
    }

    fn poll_field(&self) -> Result<()> {
        Gen6Profile::poll_field(self)
    }

    fn poll_guidance(&self) -> Result<()> {
        Gen6Profile::poll_guidance(self)
    }

    fn poll_pointers(&self) -> Result<()> {
        Gen6Profile::poll_pointers(self)
    }

    fn poll_renames(&self) -> Result<()> {
        Gen6Profile::poll_renames(self)
    }

    fn press(&self, key: &str) -> Result<()> {
        Gen6Profile::press(self, key)
    }

    fn sweep_wanted(&self) -> Result<bool> {
        Gen6Profile::sweep_wanted(self)
    }
}
