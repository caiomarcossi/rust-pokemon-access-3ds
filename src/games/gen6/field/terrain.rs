//! The matrix the player is on, as the player's ROM has it: every chunk's cell table, floors and
//! ground heights by the file the field map lists for it, and what is read off them: where the
//! ground drops, which way a bumpy slope or a waterfall rises, the heights of the decks, and which
//! deep water the sea above joins (spikes 136, 156, 159, 160, 165, 170). Nothing here reads RAM:
//! the field reader says which files the matrix lists (`files`), and asks.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};

use crate::core::pathfind::{self, Cell};
use crate::games::common::field::chunk::{BLOCKED, CHUNK, Chunk, DIVE, Floor, KIND_SHIFT, SHORE};
use crate::games::gen6::field::mapblock;
use crate::py::{self, Error, Result};

/// What the list has where the matrix has no chunk: most of Route 109's 12 by 10 (spike 136).
pub const NO_CHUNK_FILE: i64 = 0xFFFF;

/// How many chunks flattened at a height are kept (`FieldReader._flattened`).
pub const FLATTENED: i64 = 64;

/// A chunk file's cell table out of the ROM, by its number (`archives.MapChunks`); None where
/// there is no ROM to read, and a chunk the game has not loaded is not known.
pub type FarCells = Rc<dyn Fn(i64) -> Result<Option<Vec<i64>>>>;

/// A chunk file's floors out of the ROM, by its number, where it has a deck over its ground
/// (`archives.MapChunks.floors`).
pub type FarFloors = Rc<dyn Fn(i64) -> Result<Vec<Floor>>>;

/// A chunk file's ground heights out of the ROM, by its number, which say where it drops
/// (`archives.MapChunks.heights`).
pub type FarHeights = Rc<dyn Fn(i64) -> Result<Option<Vec<Option<f64>>>>>;

/// A matrix's width in chunks and each chunk's file, out of the ROM, by its number
/// (`archives.MapChunks.matrix`): the sea's above the water, whose pockets say which deep water
/// joins which (`_surfaces`). None without a ROM.
pub type FarMatrices = Rc<dyn Fn(i64) -> Result<Option<(i64, Vec<i64>)>>>;

/// A cell's word on a matrix out of the ROM, None off it or where it has no chunk (`_sea_words`).
type Word = Rc<dyn Fn(Cell) -> Result<Option<i64>>>;

/// Every way `below` can be moved to land each of its cells on a cell of `deep`: each that lands
/// its first cell on one, given up at the first cell that lands elsewhere.
fn shifts(below: &IndexSet<Cell>, deep: &IndexSet<Cell>) -> Vec<Cell> {
    let Some(&first) = below.iter().min() else {
        return Vec::new();
    };
    deep.iter()
        .map(|&(x, z)| (x - first.0, z - first.1))
        .filter(|&(dx, dz)| below.iter().all(|&(x, z)| deep.contains(&(x + dx, z + dz))))
        .collect()
}

/// Every cell the player can reach from `start` above the water, surfing and on foot: open cells
/// side by side, and the shore, which Surf jumps.
fn sea(start: Cell, word: &Word) -> Result<HashSet<Cell>> {
    // ponytail: a ledge is a wall either way here; none is known to part the sea above the water a
    // player dives through.
    let mut seen: HashSet<Cell> = HashSet::from([start]);
    let mut todo = vec![start];
    while let Some(here) = todo.pop() {
        for near in beside(here) {
            let one = word(near)?;
            if let Some(one) = one
                && !seen.contains(&near)
                && (one & BLOCKED == 0 || one >> KIND_SHIFT == SHORE)
            {
                seen.insert(near);
                todo.push(near);
            }
        }
    }
    Ok(seen)
}

/// `statistics.median`: the middle of the sorted values, or the mean of the two middle ones.
fn median(values: &mut [f64]) -> f64 {
    // `sorted`: a height read from a freed mesh can be NaN, which only CPython's sort places
    // as Python does.
    py::sort_by_lt(values, |a, b| a < b);
    let (n, i) = (values.len(), values.len() / 2);
    if n % 2 == 1 {
        values[i]
    } else {
        (values[i - 1] + values[i]) / 2.0
    }
}

/// `pathfind.STEPS[way]`.
fn step_of(way: &str) -> Cell {
    pathfind::STEPS
        .iter()
        .find(|(known, _)| *known == way)
        .map(|&(_, step)| step)
        .expect("a way of the compass")
}

/// The four cells a step from this one.
fn beside(cell: Cell) -> [Cell; 4] {
    pathfind::STEPS.map(|(_, (dx, dz))| (cell.0 + dx, cell.1 + dz))
}

/// The chunk files the ways up a kind's cells were found for, and the ways (`rises`).
type Rises = (Rc<Vec<i64>>, IndexMap<Cell, &'static str>);

/// The ROM's chunks of the matrix the field map lists, read a file at a time and kept.
pub struct Terrain {
    cells: Option<FarCells>,
    floors: Option<FarFloors>,
    heights: Option<FarHeights>,
    matrices: Option<FarMatrices>,
    /// The matrix's width in chunks, and each chunk's file, row by row, as the field map lists
    /// them now: the field reader sets them as it reads the map.
    pub files: RefCell<(i64, Rc<Vec<i64>>)>,
    /// (column, row, file) -> the ROM's chunk there, once read.
    chunks: RefCell<HashMap<(i64, i64, i64), Option<Rc<Chunk>>>>,
    /// (id of a chunk with floors, the player's height rounded, or None for any floor) -> the
    /// chunk and the chunk flattened so: the ground is built again at every step guided.
    ///
    /// The id is the `Rc`'s address, which the chunk kept here holds fixed.
    flat: RefCell<HashMap<(usize, Option<i64>), (Rc<Chunk>, Rc<Chunk>)>>,
    /// (the chunk files, the height rounded) -> the drops found for them (`cliffs`).
    drops: RefCell<HashMap<(Vec<i64>, Option<i64>), IndexSet<(Cell, Cell)>>>,
    /// Kinds -> the chunk files the ways up their cells were found for, and the ways (`rises`).
    ///
    /// Python keys it by a `frozenset`; a `BTreeSet` is equal where the frozenset is.
    rises_of: RefCell<IndexMap<BTreeSet<i64>, Rises>>,
    /// The chunk files the floors' heights were read for, and the heights (`decks`).
    deck_heights: RefCell<(Option<Rc<Vec<i64>>>, Vec<i64>)>,
}

impl Terrain {
    /// Python takes the ROM's `MapChunks`, or None without a ROM, and keeps four of its methods;
    /// here they are handed over as they are kept.
    pub fn new(
        cells: Option<FarCells>,
        floors: Option<FarFloors>,
        heights: Option<FarHeights>,
        matrices: Option<FarMatrices>,
    ) -> Self {
        Terrain {
            cells,
            floors,
            heights,
            matrices,
            files: RefCell::new((0, Rc::new(Vec::new()))),
            chunks: RefCell::new(HashMap::new()),
            flat: RefCell::new(HashMap::new()),
            drops: RefCell::new(HashMap::new()),
            rises_of: RefCell::new(IndexMap::new()),
            deck_heights: RefCell::new((None, Vec::new())),
        }
    }

    /// Whether the ROM's chunks are read: without, a chunk the game has not loaded is not known.
    pub fn known(&self) -> bool {
        self.cells.is_some()
    }

    /// Each of `below`'s cells of deep water with every one the sea above joins it to: the sea
    /// is the matrix of one of `seas` laid over this one where every cell of deep water lands on
    /// deep water, which in ORAS it does exactly and in one place: under Route 126 the cells are
    /// the sea's less (80, 80), under Route 128 the same cells (spike 165). Where more than one
    /// fits, or none, nothing is joined: a way said wrong is worse than none.
    pub fn surfaces(
        &self,
        below: &IndexSet<Cell>,
        seas: &[i64],
    ) -> Result<IndexMap<Cell, Rc<HashSet<Cell>>>> {
        let mut found: Vec<(Word, Cell)> = Vec::new();
        for &number in seas {
            let matrix = match &self.matrices {
                None => None,
                Some(matrices) => matrices(number)?,
            };
            let Some(matrix) = matrix else {
                continue;
            };
            let (word, deep) = self.sea_words(matrix.0, &matrix.1)?;
            for shift in shifts(below, &deep) {
                found.push((word.clone(), shift));
            }
        }
        if found.len() != 1 || below.is_empty() {
            return Ok(IndexMap::new());
        }
        let (word, (dx, dz)) = &found[0];
        let mut out: IndexMap<Cell, Rc<HashSet<Cell>>> = IndexMap::new();
        for &cell in below {
            if out.contains_key(&cell) {
                continue;
            }
            let pocket = sea((cell.0 + dx, cell.1 + dz), word)?;
            let joined: Rc<HashSet<Cell>> = Rc::new(
                pocket
                    .iter()
                    .map(|&(x, z)| (x - dx, z - dz))
                    .filter(|here| below.contains(here))
                    .collect(),
            );
            for &here in joined.iter() {
                out.insert(here, joined.clone());
            }
        }
        Ok(out)
    }

    /// The chunk at (column, row) of the matrix as the ROM has it, by the file the field map
    /// lists there, or None off the matrix, where it lists none, or without a ROM.
    pub fn far(&self, column: i64, row: i64) -> Result<Option<Rc<Chunk>>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let Some(cells) = &self.cells else {
            return Ok(None);
        };
        if !(0 <= column && column < wide) || row < 0 {
            return Ok(None);
        }
        let at = row * wide + column;
        if at >= files.len() as i64 || files[at as usize] == NO_CHUNK_FILE {
            return Ok(None);
        }
        let file = files[at as usize];
        let key = (column, row, file);
        if !self.chunks.borrow().contains_key(&key) {
            let words = cells(file)?;
            let levels = match &self.floors {
                None => Vec::new(),
                Some(floors) => floors(file)?,
            };
            let chunk = words.map(|words| {
                Rc::new(Chunk {
                    floors: levels,
                    ..Chunk::new((column * CHUNK, row * CHUNK), words)
                })
            });
            self.chunks.borrow_mut().insert(key, chunk);
        }
        Ok(self.chunks.borrow()[&key].clone())
    }

    /// The chunk on the floor nearest `height` cell by cell, or on any floor open where
    /// `at_height` is False (`mapblock.flat`), made once a height.
    pub fn flattened(
        &self,
        chunk: &Rc<Chunk>,
        height: Option<f64>,
        at_height: bool,
    ) -> Result<Rc<Chunk>> {
        if chunk.floors.is_empty() || (at_height && height.is_none()) {
            return Ok(chunk.clone());
        }
        let rounded = match height {
            Some(height) if at_height => Some(py::round(height)?),
            _ => None,
        };
        let key = (Rc::as_ptr(chunk) as usize, rounded);
        if !self.flat.borrow().contains_key(&key) {
            // ponytail: emptied whole when full; a few chunks at a few heights is all a map holds.
            if self.flat.borrow().len() as i64 > FLATTENED {
                self.flat.borrow_mut().clear();
            }
            let flat = mapblock::flat(chunk, rounded.map(|height| height as f64));
            self.flat
                .borrow_mut()
                .insert(key, (chunk.clone(), Rc::new(flat)));
        }
        Ok(self.flat.borrow()[&key].1.clone())
    }

    /// Open cells side by side, each way, a drop apart (`mapblock.DROP`), but for the zone's
    /// `ladders`: Fortree City's platforms stand 108 over the ground beside them in one cell
    /// table, and P walked the player off one, "9 south" into nothing, where the game lets them
    /// down only by a ladder (spike 160). Read off the ROM's chunks of one floor across the whole
    /// matrix, a chunk with a deck on the floor each cell is walked on at `level`, the height the
    /// decks are walked at; none without the ROM.
    pub fn cliffs(
        &self,
        level: Option<f64>,
        ladders: &IndexMap<(Cell, Cell), (f64, f64)>,
    ) -> Result<IndexSet<(Cell, Cell)>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let key = (
            (*files).clone(),
            match level {
                None => None,
                Some(level) => Some(py::round(level)?),
            },
        );
        let held = self.drops.borrow().get(&key).cloned();
        let drops = match held {
            Some(drops) => drops,
            None => {
                // ponytail: emptied whole when full; a map is walked at a few heights.
                if self.drops.borrow().len() as i64 > FLATTENED {
                    self.drops.borrow_mut().clear();
                }
                let found = self.find_drops(wide, &files, level)?;
                self.drops.borrow_mut().insert(key, found.clone());
                found
            }
        };
        if drops.is_empty() {
            return Ok(drops);
        }
        Ok(drops
            .into_iter()
            .filter(|pair| !ladders.contains_key(pair))
            .collect())
    }

    /// Every drop on the matrix with its decks walked at `level` (`cliffs`).
    fn find_drops(
        &self,
        wide: i64,
        files: &[i64],
        level: Option<f64>,
    ) -> Result<IndexSet<(Cell, Cell)>> {
        if self.heights.is_none() || wide == 0 {
            return Ok(IndexSet::new());
        }
        let mut tables: IndexMap<(i64, i64), (Vec<i64>, Vec<Option<f64>>)> = IndexMap::new();
        for (at, &file) in files.iter().enumerate() {
            let at = at as i64;
            let Some(chunk) = self.far(py::modulo(at, wide), py::floordiv(at, wide))? else {
                continue;
            };
            if !chunk.floors.is_empty() {
                let flat = self.flattened(&chunk, level, true)?;
                let heights = if flat.heights.is_empty() {
                    chunk.floors[0].heights.clone()
                } else {
                    flat.heights.clone()
                };
                tables.insert(chunk.key(), (flat.words.clone(), heights));
            } else if let Some(heights) = self.heights.as_ref().map_or(Ok(None), |f| f(file))? {
                tables.insert(chunk.key(), (chunk.words.clone(), heights));
            }
        }

        let height = |cell: Cell| -> Result<Option<f64>> {
            let Some(table) =
                tables.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)))
            else {
                return Ok(None);
            };
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let word = *usize::try_from(i)
                .ok()
                .and_then(|i| table.0.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            if word & BLOCKED != 0 {
                return Ok(None);
            }
            Ok(*usize::try_from(i)
                .ok()
                .and_then(|i| table.1.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?)
        };

        let mut out: IndexSet<(Cell, Cell)> = IndexSet::new();
        for &(column, row) in tables.keys() {
            for z in row * CHUNK..(row + 1) * CHUNK {
                for x in column * CHUNK..(column + 1) * CHUNK {
                    let Some(here) = height((x, z))? else {
                        continue;
                    };
                    for near in [(x + 1, z), (x, z + 1)] {
                        let there = height(near)?;
                        if let Some(there) = there
                            && (here - there).abs() > mapblock::DROP
                        {
                            out.insert(((x, z), near));
                            out.insert((near, (x, z)));
                        }
                    }
                }
            }
        }
        Ok(out)
    }

    /// Each cell of `kinds` on the matrix with the way it goes up: between the two open cells on
    /// its opposite sides, towards the higher, as the ROM's heights have them. Every one of
    /// Jagged Pass's 24 bumpy slopes has one such pair, a cell's height apart, north for most and
    /// west for three (spike 159), and Ever Grande City's waterfall has the sea at 0 below and
    /// the water at 234 above (spike 170). None without the ROM's heights.
    pub fn rises(&self, kinds: &[i64]) -> Result<IndexMap<Cell, &'static str>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let key: BTreeSet<i64> = kinds.iter().copied().collect();
        if let Some(cached) = self.rises_of.borrow().get(&key)
            && cached.0 == files
        {
            return Ok(cached.1.clone());
        }
        let mut out: IndexMap<Cell, &'static str> = IndexMap::new();
        let mut tables: IndexMap<(i64, i64), (Vec<i64>, Vec<Option<f64>>)> = IndexMap::new();
        for (at, &file) in files.iter().enumerate() {
            let at = at as i64;
            let chunk = self.far(py::modulo(at, wide), py::floordiv(at, wide))?;
            let heights = match &self.heights {
                None => None,
                Some(heights) => heights(file)?,
            };
            if let (Some(chunk), Some(heights)) = (chunk, heights) {
                tables.insert(chunk.key(), (chunk.words.clone(), heights));
            }
        }

        let read = |cell: Cell| -> Result<Option<(i64, Option<f64>)>> {
            let table = tables.get(&(py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK)));
            let i = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
            let Some(table) = table else {
                return Ok(None);
            };
            let word = *usize::try_from(i)
                .ok()
                .and_then(|i| table.0.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            let height = *usize::try_from(i)
                .ok()
                .and_then(|i| table.1.get(i))
                .ok_or_else(|| Error::index("tuple index out of range"))?;
            Ok(Some((word, height)))
        };

        for (&(column, row), (words, _)) in &tables {
            for (i, &word) in words.iter().enumerate() {
                let i = i as i64;
                if !kinds.contains(&(word >> KIND_SHIFT)) {
                    continue;
                }
                let cell = (column * CHUNK + i % CHUNK, row * CHUNK + i / CHUNK);
                for (one, other) in [
                    (pathfind::NORTH, pathfind::SOUTH),
                    (pathfind::WEST, pathfind::EAST),
                ] {
                    let (first, second) = {
                        let first = step_of(one);
                        let first = read((cell.0 + first.0, cell.1 + first.1))?;
                        let second = step_of(other);
                        let second = read((cell.0 + second.0, cell.1 + second.1))?;
                        (first, second)
                    };
                    let (Some(first), Some(second)) = (first, second) else {
                        continue;
                    };
                    if (first.0 | second.0) & BLOCKED != 0 {
                        continue;
                    }
                    if let (Some(a), Some(b)) = (first.1, second.1)
                        && a != b
                    {
                        out.insert(cell, if a > b { one } else { other });
                    }
                }
            }
        }
        self.rises_of.borrow_mut().insert(key.clone(), (files, out));
        Ok(self.rises_of.borrow()[&key].1.clone())
    }

    /// Every floor's height on the matrix but the one at `level`, nearest it first: each floor of
    /// a chunk with a deck, at the middle of its open cells' heights: the `loaded` chunks', and
    /// with the ROM every chunk's of the matrix (spike 159).
    ///
    /// The heights are whole numbers, as `round` makes them, and so are the ground's `decks`.
    pub fn decks(
        &self,
        level: Option<f64>,
        loaded: &dyn Fn() -> Result<Vec<Rc<Chunk>>>,
    ) -> Result<Vec<f64>> {
        let Some(level) = level else {
            return Ok(Vec::new());
        };
        let mut others: Vec<i64> = self
            .floor_heights(loaded)?
            .into_iter()
            .filter(|&height| (height as f64 - level).abs() > mapblock::CLIMB)
            .collect();
        others.sort_by(|a, b| {
            let (a, b) = ((*a as f64 - level).abs(), (*b as f64 - level).abs());
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(others.into_iter().map(|height| height as f64).collect())
    }

    /// Whether a chunk of the matrix has a deck over its ground: the loaded chunks, and with
    /// the ROM every chunk of the matrix. Not whether a deck stands apart from the player:
    /// Mt. Chimney's ground and catwalk both sit at 108 over most of their cells, the ground
    /// falling to 4 only down the path by the crater, and a player on the summit was walked
    /// one floor a cell, an item ball on that path 104 below taken for one beside them
    /// (or_3d_pathfinding, 2026-10-24).
    pub fn stacked(&self, loaded: &dyn Fn() -> Result<Vec<Rc<Chunk>>>) -> Result<bool> {
        Ok(!self.floor_heights(loaded)?.is_empty())
    }

    /// Each floor's height of a chunk with a deck, at the middle of its open cells' heights,
    /// once a matrix (`decks`).
    fn floor_heights(&self, loaded: &dyn Fn() -> Result<Vec<Rc<Chunk>>>) -> Result<Vec<i64>> {
        let files = self.files.borrow().1.clone();
        if self.deck_heights.borrow().0.as_ref() != Some(&files) {
            let mut floors: Vec<Floor> = Vec::new();
            for chunk in loaded()? {
                floors.extend(chunk.floors.iter().cloned());
            }
            if let Some(far_floors) = &self.floors {
                for file in py::PySet::of(files.iter().copied())
                    .difference(&py::PySet::of([NO_CHUNK_FILE]))
                    .iter()
                {
                    floors.extend(far_floors(file)?);
                }
            }
            let mut heights: Vec<i64> = Vec::new();
            for floor in &floors {
                if floor.words.len() != floor.heights.len() {
                    return Err(Error::value(format!(
                        "zip() argument 2 is {} than argument 1",
                        if floor.heights.len() < floor.words.len() {
                            "shorter"
                        } else {
                            "longer"
                        }
                    )));
                }
                let mut open: Vec<f64> = Vec::new();
                for (&word, &height) in floor.words.iter().zip(&floor.heights) {
                    if let Some(height) = height
                        && word & BLOCKED == 0
                    {
                        open.push(height);
                    }
                }
                if !open.is_empty() {
                    heights.push(py::round(median(&mut open))?);
                }
            }
            heights.sort();
            heights.dedup();
            *self.deck_heights.borrow_mut() = (Some(files), heights);
        }
        Ok(self.deck_heights.borrow().1.clone())
    }

    /// The ROM's ground height at a cell of the current matrix, or None where the chunk is not
    /// listed or its heights are not read.
    pub fn height(&self, cell: Cell) -> Result<Option<f64>> {
        let (wide, files) = {
            let held = self.files.borrow();
            (held.0, held.1.clone())
        };
        let (column, row) = (py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK));
        let at = row * wide + column;
        let Some(heights) = &self.heights else {
            return Ok(None);
        };
        if !(0 <= column && column < wide) || row < 0 || at >= files.len() as i64 {
            return Ok(None);
        }
        let file = files[at as usize];
        if file == NO_CHUNK_FILE {
            return Ok(None);
        }
        let Some(heights) = heights(file)? else {
            return Ok(None);
        };
        let index = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
        heights
            .get(index as usize)
            .copied()
            .ok_or_else(|| Error::index("list index out of range"))
    }

    /// A cell's word on a matrix out of the ROM, None off it or where it has no chunk, and the
    /// matrix's deep water.
    fn sea_words(&self, wide: i64, files: &[i64]) -> Result<(Word, IndexSet<Cell>)> {
        let mut tables: Vec<Option<Vec<i64>>> = Vec::new();
        for &file in files {
            tables.push(match &self.cells {
                Some(cells) if file != NO_CHUNK_FILE => cells(file)?,
                _ => None,
            });
        }
        let mut deep: IndexSet<Cell> = IndexSet::new();
        for (at, table) in tables.iter().enumerate() {
            let at = at as i64;
            for (i, &one) in table.iter().flatten().enumerate() {
                let i = i as i64;
                if one >> KIND_SHIFT == DIVE {
                    deep.insert((
                        py::modulo(at, wide) * CHUNK + i % CHUNK,
                        py::floordiv(at, wide) * CHUNK + i / CHUNK,
                    ));
                }
            }
        }

        let word = move |cell: Cell| -> Result<Option<i64>> {
            let (column, row) = (py::floordiv(cell.0, CHUNK), py::floordiv(cell.1, CHUNK));
            let at = row * wide + column;
            let table = if 0 <= column && column < wide && 0 <= at && at < tables.len() as i64 {
                tables[at as usize].as_ref()
            } else {
                None
            };
            match table {
                None => Ok(None),
                Some(table) => {
                    let index = py::modulo(cell.1, CHUNK) * CHUNK + py::modulo(cell.0, CHUNK);
                    table
                        .get(index as usize)
                        .copied()
                        .map(Some)
                        .ok_or_else(|| Error::index("tuple index out of range"))
                }
            }
        };
        Ok((Rc::new(word), deep))
    }
}
