//! What the port needs of Python's own behaviour, where Rust's differs and the difference would
//! change what pa3ds says or reads.
//!
//! The port is checked against the Python runtime line for line (`tools/tape`), so a percentage
//! rounded the other way at .5, a negative cell divided toward zero instead of down, or a float
//! logged as `1` instead of `1.0` is a regression. Every module uses these instead of the Rust
//! operator that looks the same: `docs/porting.md` lists which is which.

use std::fmt;
use std::panic::Location;
use std::rc::Rc;

/// `log: Callable[[str], None]`: where a reader writes what the log file shows.
pub type Log = Rc<dyn Fn(&str)>;

/// A Python exception class the port raises or catches by name.
///
/// pa3ds catches by class (`except (RomError, struct.error)`), so the port keeps the classes and
/// their bases: [`Error::is`] is `isinstance`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Exception,
    RuntimeError,
    ValueError,
    UnicodeDecodeError,
    LookupError,
    IndexError,
    KeyError,
    ArithmeticError,
    ZeroDivisionError,
    OverflowError,
    TypeError,
    /// `struct.error`: bytes too short for what was unpacked from them.
    StructError,
    OsError,
    TimeoutError,
    ConnectionResetError,
    /// `queue.Empty`.
    QueueEmpty,
    /// `tomllib.TOMLDecodeError`.
    TomlDecodeError,
    /// `pa3ds.core.rom.RomError`.
    RomError,
    /// `pa3ds.core.rpc.RpcError` and the classes under it.
    RpcError,
    RpcTimeoutError,
    RpcRefusedError,
    RpcLostError,
    /// `pa3ds.debug.gdb.GdbError` and `GdbTimeoutError`.
    GdbError,
    GdbTimeoutError,
}

impl Kind {
    /// The class this one derives from, as Python declares it.
    pub fn base(self) -> Option<Kind> {
        use Kind::*;
        match self {
            Exception => None,
            RuntimeError | ValueError | LookupError | ArithmeticError | TypeError | StructError
            | OsError | QueueEmpty | RomError => Some(Exception),
            ZeroDivisionError | OverflowError => Some(ArithmeticError),
            UnicodeDecodeError | TomlDecodeError => Some(ValueError),
            IndexError | KeyError => Some(LookupError),
            TimeoutError | ConnectionResetError => Some(OsError),
            RpcError | GdbError => Some(RuntimeError),
            RpcTimeoutError => Some(RpcError),
            RpcRefusedError | RpcLostError => Some(RpcTimeoutError),
            GdbTimeoutError => Some(GdbError),
        }
    }

    /// The class's name as a traceback gives it.
    pub fn name(self) -> &'static str {
        use Kind::*;
        match self {
            Exception => "Exception",
            RuntimeError => "RuntimeError",
            ValueError => "ValueError",
            UnicodeDecodeError => "UnicodeDecodeError",
            LookupError => "LookupError",
            IndexError => "IndexError",
            KeyError => "KeyError",
            ArithmeticError => "ArithmeticError",
            ZeroDivisionError => "ZeroDivisionError",
            OverflowError => "OverflowError",
            TypeError => "TypeError",
            StructError => "error",
            OsError => "OSError",
            TimeoutError => "TimeoutError",
            ConnectionResetError => "ConnectionResetError",
            QueueEmpty => "Empty",
            TomlDecodeError => "TOMLDecodeError",
            RomError => "RomError",
            RpcError => "RpcError",
            RpcTimeoutError => "RpcTimeoutError",
            RpcRefusedError => "RpcRefusedError",
            RpcLostError => "RpcLostError",
            GdbError => "GdbError",
            GdbTimeoutError => "GdbTimeoutError",
        }
    }
}

/// A raised exception: its class, `str(error)`, and the line that raised it, which the runtime
/// keys a fault by as Python keyed it by the traceback's last frame (`app.Faults`).
#[derive(Clone)]
pub struct Error {
    pub kind: Kind,
    pub message: String,
    pub location: &'static Location<'static>,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    #[track_caller]
    pub fn new(kind: Kind, message: impl Into<String>) -> Self {
        Error {
            kind,
            message: message.into(),
            location: Location::caller(),
        }
    }

    /// `isinstance(error, kind)`.
    pub fn is(&self, kind: Kind) -> bool {
        let mut at = Some(self.kind);
        while let Some(k) = at {
            if k == kind {
                return true;
            }
            at = k.base();
        }
        false
    }

    #[track_caller]
    pub fn value(message: impl Into<String>) -> Self {
        Error::new(Kind::ValueError, message)
    }

    #[track_caller]
    pub fn index(message: impl Into<String>) -> Self {
        Error::new(Kind::IndexError, message)
    }

    #[track_caller]
    pub fn key(message: impl Into<String>) -> Self {
        Error::new(Kind::KeyError, message)
    }

    #[track_caller]
    pub fn unpack(message: impl Into<String>) -> Self {
        Error::new(Kind::StructError, message)
    }

    /// The fault this is, as one line: what was raised and where (`app._fault`).
    pub fn fault(&self) -> String {
        let file = self
            .location
            .file()
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or("");
        format!("{} at {}:{}", self.kind.name(), file, self.location.line())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl fmt::Debug for Error {
    /// `repr(error)`: `RpcError('page 0x08000000 unread')`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.kind.name(), repr(&self.message))
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    #[track_caller]
    fn from(error: std::io::Error) -> Self {
        let kind = match error.kind() {
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => Kind::TimeoutError,
            std::io::ErrorKind::ConnectionReset => Kind::ConnectionResetError,
            _ => Kind::OsError,
        };
        Error::new(kind, error.to_string())
    }
}

/// `time`: the clock pa3ds reads, which a tape replaces (`tools/tape`).
///
/// Every `time.monotonic()` and `time.sleep()` in pa3ds is [`monotonic`] and [`sleep`] here, so a
/// replay can answer each from the tape in the order the Python runtime asked. The RPC client's
/// own timing (`perf_counter`) is below the tape and always real.
pub mod time {
    use std::cell::RefCell;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    /// Where the time comes from while a tape is replayed.
    pub trait Clock {
        fn monotonic(&mut self) -> f64;
        fn sleep(&mut self, seconds: f64);
    }

    thread_local! {
        static CLOCK: RefCell<Option<Box<dyn Clock>>> = const { RefCell::new(None) };
    }

    fn epoch() -> Instant {
        static EPOCH: OnceLock<Instant> = OnceLock::new();
        *EPOCH.get_or_init(Instant::now)
    }

    /// Answer this thread's [`monotonic`] and [`sleep`] from `clock`, or from the real clock
    /// again with None. Returns the clock it replaces.
    pub fn set_clock(clock: Option<Box<dyn Clock>>) -> Option<Box<dyn Clock>> {
        CLOCK.with(|cell| std::mem::replace(&mut *cell.borrow_mut(), clock))
    }

    /// `time.monotonic()`: seconds since some fixed moment.
    pub fn monotonic() -> f64 {
        let taken = CLOCK.with(|cell| cell.borrow_mut().as_mut().map(|clock| clock.monotonic()));
        taken.unwrap_or_else(|| epoch().elapsed().as_secs_f64())
    }

    /// `time.sleep(seconds)`.
    pub fn sleep(seconds: f64) {
        let replayed = CLOCK.with(|cell| match cell.borrow_mut().as_mut() {
            Some(clock) => {
                clock.sleep(seconds);
                true
            }
            None => false,
        });
        if !replayed && seconds > 0.0 {
            std::thread::sleep(Duration::from_secs_f64(seconds));
        }
    }

    /// `time.perf_counter()`: the real clock, never a tape's.
    pub fn perf_counter() -> f64 {
        epoch().elapsed().as_secs_f64()
    }
}

/// `a // b` for ints: rounded down, not toward zero.
pub fn floordiv(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

/// `a % b` for ints: the sign of `b`, not of `a`.
pub fn modulo(a: i64, b: i64) -> i64 {
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        r + b
    } else {
        r
    }
}

/// CPython's `divmod` for floats (`_float_div_mod` in `Objects/floatobject.c`), which `//` and `%`
/// are made of: the quotient is not `(a / b).floor()` (`1 // 0.1` is 9.0, not 10.0), and a zero
/// remainder takes the divisor's sign. `b` must not be 0, where Python raises.
fn float_div_mod(a: f64, b: f64) -> (f64, f64) {
    let mut modulo = a % b;
    let mut div = (a - modulo) / b;
    if modulo != 0.0 {
        if (b < 0.0) != (modulo < 0.0) {
            modulo += b;
            div -= 1.0;
        }
    } else {
        modulo = 0.0_f64.copysign(b);
    }
    let floordiv = if div != 0.0 {
        let floor = div.floor();
        if div - floor > 0.5 {
            floor + 1.0
        } else {
            floor
        }
    } else {
        0.0_f64.copysign(a / b)
    };
    (floordiv, modulo)
}

/// `a // b` for floats.
pub fn floordiv_f(a: f64, b: f64) -> f64 {
    float_div_mod(a, b).0
}

/// `a % b` for floats: the sign of `b`.
pub fn modulo_f(a: f64, b: f64) -> f64 {
    float_div_mod(a, b).1
}

/// A float made an int: Python refuses a NaN and an infinity where Rust would quietly make them
/// 0 and the largest `i64`, and a float read out of freed memory is either often enough.
#[track_caller]
fn whole(x: f64) -> Result<i64> {
    if x.is_nan() {
        return Err(Error::value("cannot convert float NaN to integer"));
    }
    if x.is_infinite() {
        return Err(Error::new(
            Kind::OverflowError,
            "cannot convert float infinity to integer",
        ));
    }
    Ok(x as i64)
}

/// `round(x)`: to the nearest int, halves to the even one.
#[track_caller]
pub fn round(x: f64) -> Result<i64> {
    whole(x.round_ties_even())
}

/// `int(x)` of a float: toward zero.
#[track_caller]
pub fn int(x: f64) -> Result<i64> {
    whole(x.trunc())
}

/// `math.floor(x)`.
#[track_caller]
pub fn floor(x: f64) -> Result<i64> {
    whole(x.floor())
}

/// `math.ceil(x)`.
#[track_caller]
pub fn ceil(x: f64) -> Result<i64> {
    whole(x.ceil())
}

/// `round(x, digits)`: rounded as the decimal it prints as, which is how Python rounds.
pub fn round_to(x: f64, digits: usize) -> f64 {
    format!("{x:.digits$}").parse().unwrap_or(x)
}

/// `str(x)` and `repr(x)` for a float: `1.0`, `0.1`, `1e-05`, `1e+16`.
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() {
            "-0.0".into()
        } else {
            "0.0".into()
        };
    }
    // The fewest digits that read back as `x`, each count rounded halves-to-even as Python's
    // repr rounds them. Rust's own shortest form breaks a tie between two 17-digit decimals the
    // other way, and a float widened from an f32 out of the game's memory often sits on one.
    let scientific = (1..=17)
        .map(|digits| format!("{:.*e}", digits - 1, x))
        .find(|text| text.parse::<f64>() == Ok(x))
        .unwrap_or_else(|| format!("{x:.16e}"));
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific notation");
    let exponent: i32 = exponent.parse().expect("an exponent");
    let negative = mantissa.starts_with('-');
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let mut out = String::from(if negative { "-" } else { "" });
    if (-4..16).contains(&exponent) {
        if exponent >= 0 {
            let whole = exponent as usize + 1;
            if digits.len() <= whole {
                out.push_str(digits);
                out.push_str(&"0".repeat(whole - digits.len()));
                out.push_str(".0");
            } else {
                out.push_str(&digits[..whole]);
                out.push('.');
                out.push_str(&digits[whole..]);
            }
        } else {
            out.push_str("0.");
            out.push_str(&"0".repeat((-exponent - 1) as usize));
            out.push_str(digits);
        }
    } else {
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        let sign = if exponent < 0 { '-' } else { '+' };
        out.push_str(&format!("e{sign}{:02}", exponent.abs()));
    }
    out
}

/// `str.isspace()` for one character: wider than [`char::is_whitespace`], which leaves out
/// U+001C to U+001F, the separators Python splits on.
pub fn is_space(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0b' | '\x0c' | '\r' | '\x1c'..='\x1f' | ' ' | '\u{85}'
        | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}'
        | '\u{202f}' | '\u{205f}' | '\u{3000}')
}

/// `text.split()`: the words between runs of whitespace.
pub fn split(text: &str) -> Vec<&str> {
    text.split(is_space)
        .filter(|word| !word.is_empty())
        .collect()
}

/// `text.strip()`.
pub fn strip(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// `text.lstrip()`.
pub fn lstrip(text: &str) -> &str {
    text.trim_start_matches(is_space)
}

/// `text.rstrip()`.
pub fn rstrip(text: &str) -> &str {
    text.trim_end_matches(is_space)
}

/// `str.isdigit()` for one character, as far as game text needs it: the Unicode decimal digits.
pub fn is_digit(c: char) -> bool {
    c.is_numeric()
        && matches!(
            unicode_general_category::get_general_category(c),
            unicode_general_category::GeneralCategory::DecimalNumber
        )
}

/// `str.isprintable()` for one character, which decides how `repr` writes it.
fn is_printable(c: char) -> bool {
    use unicode_general_category::GeneralCategory::*;
    if c == ' ' {
        return true;
    }
    !matches!(
        unicode_general_category::get_general_category(c),
        Control
            | Format
            | Surrogate
            | PrivateUse
            | Unassigned
            | LineSeparator
            | ParagraphSeparator
            | SpaceSeparator
    )
}

/// `repr(text)`: quoted the way Python quotes it, which pa3ds's log lines show (`{name!r}`).
pub fn repr(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(text.len() + 2);
    out.push(quote);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if is_printable(c) => out.push(c),
            c if (c as u32) < 0x100 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c if (c as u32) < 0x10000 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push_str(&format!("\\U{:08x}", c as u32)),
        }
    }
    out.push(quote);
    out
}

/// `data[start:end]`, clamped as Python clamps a slice instead of panicking. A negative index
/// counts from the end.
pub fn slice<T>(data: &[T], start: i64, end: i64) -> &[T] {
    let len = data.len() as i64;
    let at = |i: i64| -> usize { (if i < 0 { (len + i).max(0) } else { i.min(len) }) as usize };
    let (a, b) = (at(start), at(end));
    if a >= b { &data[0..0] } else { &data[a..b] }
}

/// `data[start:]`, clamped.
pub fn slice_from<T>(data: &[T], start: i64) -> &[T] {
    slice(data, start, data.len() as i64)
}

/// `bytes.decode("utf-16-le", "replace")`: a lone surrogate, or an odd byte at the end, is
/// U+FFFD.
pub fn decode_utf16le(data: &[u8]) -> String {
    let units: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_le_bytes(*p))
        .collect();
    let mut out = String::from_utf16_lossy(&units);
    if data.len() % 2 == 1 {
        out.push('\u{fffd}');
    }
    out
}

/// `bytes.decode("utf-16-le")`, strict: a UnicodeDecodeError where Python raises one.
#[track_caller]
pub fn decode_utf16le_strict(data: &[u8]) -> Result<String> {
    if data.len() % 2 == 1 {
        return Err(Error::new(Kind::UnicodeDecodeError, "truncated data"));
    }
    let units: Vec<u16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from_le_bytes(*p))
        .collect();
    String::from_utf16(&units)
        .map_err(|_| Error::new(Kind::UnicodeDecodeError, "illegal UTF-16 surrogate"))
}

/// `bytes.decode("ascii", "replace")`: every byte over 0x7f is U+FFFD.
pub fn decode_ascii(data: &[u8]) -> String {
    data.iter()
        .map(|&b| if b < 0x80 { b as char } else { '\u{fffd}' })
        .collect()
}

/// `bytes.decode("latin-1")`.
pub fn decode_latin1(data: &[u8]) -> String {
    data.iter().map(|&b| b as char).collect()
}

/// How CPython lays a `set` of ints out, for the order Python iterates one in (hash order, not
/// insertion order).
///
/// A set's order reaches what pa3ds reads and says wherever one is iterated: `layout.track` reads
/// each work in the order Python iterates `{*works, *known}`, and an insertion-ordered set read
/// them in another order. Only what pa3ds's sets need is here: adding and never removing, ints
/// below 2**61 - 1 hashing as themselves. Written against CPython's `Objects/setobject.c`
/// (`set_add_entry`, `set_table_resize`, `set_update_internal`), and checked against it in
/// `checks` below.
pub struct PySet {
    table: Vec<Option<i64>>,
    used: usize,
}

impl PySet {
    const LINEAR_PROBES: usize = 9;
    const PERTURB_SHIFT: u32 = 5;
    const MIN_SIZE: usize = 8;

    pub fn new() -> Self {
        PySet {
            table: vec![None; Self::MIN_SIZE],
            used: 0,
        }
    }

    /// `hash(key)`.
    fn hash(key: i64) -> i64 {
        const MODULUS: u64 = (1 << 61) - 1;
        let magnitude = (key.unsigned_abs() % MODULUS) as i64;
        let hash = if key < 0 { -magnitude } else { magnitude };
        if hash == -1 { -2 } else { hash }
    }

    /// The slot an insertion into a table of `mask + 1` slots finds free, by CPython's probing:
    /// ten slots from the hash's own, then a jump, and again.
    fn slot(table: &[Option<i64>], key: i64) -> Result<usize, usize> {
        let mask = table.len() - 1;
        let hash = Self::hash(key);
        let mut perturb = hash as u64;
        let mut i = (hash as u64 as usize) & mask;
        loop {
            let probes = if i + Self::LINEAR_PROBES <= mask {
                Self::LINEAR_PROBES
            } else {
                0
            };
            for at in i..=i + probes {
                match table[at] {
                    None => return Err(at),
                    Some(held) if held == key => return Ok(at),
                    Some(_) => {}
                }
            }
            perturb >>= Self::PERTURB_SHIFT;
            i = i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize)
                & mask;
        }
    }

    /// `set.add(key)`.
    pub fn add(&mut self, key: i64) {
        let Err(at) = Self::slot(&self.table, key) else {
            return;
        };
        self.table[at] = Some(key);
        self.used += 1;
        let mask = self.table.len() - 1;
        if self.used * 5 >= mask * 3 {
            self.resize(if self.used > 50000 {
                self.used * 2
            } else {
                self.used * 4
            });
        }
    }

    /// A table of the least power of two slots above `minused`, every key put in again in the
    /// order the old table holds them.
    fn resize(&mut self, minused: usize) {
        let mut size = Self::MIN_SIZE;
        while size <= minused {
            size <<= 1;
        }
        let old = std::mem::replace(&mut self.table, vec![None; size]);
        for key in old.into_iter().flatten() {
            if let Err(at) = Self::slot(&self.table, key) {
                self.table[at] = Some(key);
            }
        }
    }

    /// `set.update(some_dict)`, which makes room for every key before adding any.
    pub fn update_dict(&mut self, keys: &[i64]) {
        let mask = self.table.len() - 1;
        if (self.used + keys.len()) * 5 >= mask * 3 {
            self.resize((self.used + keys.len()) * 2);
        }
        for &key in keys {
            self.add(key);
        }
    }

    /// `set(iterable)` of an iterator, which adds one key at a time.
    pub fn of(keys: impl IntoIterator<Item = i64>) -> Self {
        let mut set = PySet::new();
        for key in keys {
            set.add(key);
        }
        set
    }

    /// `set(some_dict)`.
    pub fn of_dict(keys: &[i64]) -> Self {
        let mut set = PySet::new();
        set.update_dict(keys);
        set
    }

    /// The keys in the order Python iterates them.
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        self.table.iter().flatten().copied()
    }
}

/// `PySet` against orders CPython 3.14 gave, over works like the field's (0x104 apart) and the
/// screens' (0x10000 apart), which collide in a small table.
#[cfg(test)]
mod set_checks {
    use super::PySet;

    #[test]
    fn pyset_iterates_as_cpython_does() {
        let mut every = PySet::of([0x083002a4, 0x14B00000, 0x14B10000]);
        every.update_dict(&[0x14B00000, 0x14C00000, 0x14B20000]);
        assert_eq!(
            every.iter().collect::<Vec<_>>(),
            [0x14B00000, 0x14B10000, 0x14C00000, 0x14B20000, 0x083002a4]
        );
        let field: Vec<i64> = (0..12).map(|index| 0x08300000 + index * 0x104).collect();
        assert_eq!(
            PySet::of_dict(&field).iter().collect::<Vec<_>>(),
            [
                0x8300000, 0x8300820, 0x8300104, 0x8300924, 0x8300208, 0x8300a28, 0x830030c,
                0x8300b2c, 0x8300410, 0x8300514, 0x8300618, 0x830071c
            ]
        );
        assert_eq!(
            PySet::of([0x14B30600, 0x14B10600, 0x14B20600])
                .iter()
                .collect::<Vec<_>>(),
            [0x14B30600, 0x14B10600, 0x14B20600]
        );
        let mut small = PySet::of([5, 3, 1]);
        small.update_dict(&[9, 2, 8]);
        assert_eq!(small.iter().collect::<Vec<_>>(), [1, 2, 3, 5, 8, 9]);
    }
}

#[cfg(test)]
mod checks {
    use super::*;

    #[test]
    fn python_arithmetic() -> Result<()> {
        assert_eq!((floordiv(-7, 2), modulo(-7, 2)), (-4, 1));
        assert_eq!((floordiv(7, -2), modulo(7, -2)), (-4, -1));
        assert_eq!(
            (round(0.5)?, round(1.5)?, round(2.5)?, round(-0.5)?),
            (0, 2, 2, 0)
        );
        assert!(round(f64::NAN).unwrap_err().is(Kind::ValueError));
        assert!(int(f64::INFINITY).unwrap_err().is(Kind::OverflowError));
        assert_eq!((int(-2.7)?, floor(-2.5)?, ceil(-2.5)?), (-2, -3, -2));
        assert_eq!(round_to(2.675, 2), 2.67);
        assert_eq!(modulo_f(-1.0, 360.0), 359.0);
        assert_eq!(
            (floordiv_f(1.0, 0.1), modulo_f(1.0, 0.1)),
            (9.0, 0.09999999999999995)
        );
        assert!(modulo_f(-0.0, 360.0).is_sign_positive() && modulo_f(5.0, -5.0).is_sign_negative());
        assert_eq!((floordiv_f(-7.5, 2.0), modulo_f(-7.5, 2.0)), (-4.0, 0.5));
        Ok(())
    }

    #[test]
    fn python_text() {
        assert_eq!(float_repr(1.0), "1.0");
        assert_eq!(float_repr(0.1), "0.1");
        assert_eq!(float_repr(1e-5), "1e-05");
        assert_eq!(float_repr(1e16), "1e+16");
        assert_eq!(float_repr(-2.5), "-2.5");
        assert_eq!(
            float_repr(f64::from_bits(0x4062911c40000000)),
            "148.53469848632812"
        );
        assert_eq!(float_repr(1.5e16), "1.5e+16");
        assert_eq!(float_repr(-0.0), "-0.0");
        assert_eq!(float_repr(123.0), "123.0");
        assert_eq!(repr("it's"), "\"it's\"");
        assert_eq!(repr("a\nb\u{e08e}"), "'a\\nb\\ue08e'");
        assert_eq!(split(" a\u{a0}b\x1cc "), vec!["a", "b", "c"]);
        assert_eq!(slice(b"abc", 1, 10), b"bc");
        assert_eq!(slice(b"abc", -2, 3), b"bc");
        assert_eq!(
            decode_utf16le(&[0x41, 0, 0x00, 0xd8, 0x42]),
            "A\u{fffd}\u{fffd}"
        );
    }

    #[test]
    fn exception_classes() {
        let lost = Error::new(Kind::RpcLostError, "gone");
        assert!(lost.is(Kind::RpcTimeoutError) && lost.is(Kind::RpcError));
        assert!(lost.is(Kind::Exception) && !lost.is(Kind::OsError));
        assert!(lost.fault().starts_with("RpcLostError at py.rs:"));
        assert_eq!(format!("{lost:?}"), "RpcLostError('gone')");
    }
}
