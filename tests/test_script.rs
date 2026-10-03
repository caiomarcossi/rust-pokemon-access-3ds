//! A zone's script, read as far as what its signs show (spike 107).

use indexmap::IndexMap;

use crate::games::gen6::script;

/// A zone's script as the game compiles one: the entry point switches on a sign's number to
/// a case calling a function of the sign's own, which shows each of its messages, (line,
/// style), by pushing the style, the line and their size, and calling the helper.
fn sign_script(signs: &IndexMap<i64, Vec<(i64, i64)>>, compact: bool) -> Vec<u8> {
    fn push(value: i64) -> i64 {
        (value & 0xFFFF) << 16 | script::PUSH_CONSTANT
    }

    let count = signs.len() as i64;
    let cases = 5 + 2 * count;
    let functions = cases + 2 * count;
    let mut code = vec![script::SWITCH, 8, script::CASETBL, count, 0];
    let mut bodies: Vec<Vec<i64>> = Vec::new();
    let mut at = functions;
    let mut starts: Vec<i64> = Vec::new();
    for messages in signs.values() {
        starts.push(at);
        let mut body: Vec<i64> = Vec::new();
        for &(line, style) in messages {
            body.extend([push(style), push(line), push(8), script::CALL, 0]);
        }
        body.push(script::RETN);
        at += body.len() as i64;
        bodies.push(body);
    }
    let helper = at;
    for (j, &number) in signs.keys().enumerate() {
        let j = j as i64;
        code.extend([number, (cases + 2 * j - (5 + 2 * j)) * 4]);
    }
    for (j, &start) in starts.iter().enumerate() {
        code.extend([script::CALL, (start - (cases + 2 * j as i64)) * 4]);
    }
    for (&start, body) in starts.iter().zip(bodies.iter_mut()) {
        for i in (3..body.len()).step_by(5) {
            body[i + 1] = (helper - (start + i as i64)) * 4;
        }
        code.extend_from_slice(body);
    }
    code.push(script::RETN);
    let body_bytes: Vec<u8> = if compact {
        compact_cells(&code)
    } else {
        code.iter()
            .flat_map(|&cell| {
                i32::try_from(cell)
                    .expect("a cell fits struct's i")
                    .to_le_bytes()
            })
            .collect()
    };
    let size = script::HEADER.size() + body_bytes.len() as i64;
    let mut head = script::HEADER
        .pack([
            size,
            script::MAGIC,
            10,
            10,
            if compact { script::COMPACT } else { 0 },
            8,
            script::HEADER.size(),
            script::HEADER.size() + 4 * code.len() as i64,
            0,
            0,
            0,
        ])
        .unwrap();
    head.extend(body_bytes);
    head
}

/// Cells in Pawn's compact encoding: seven bits a byte, the most significant first.
fn compact_cells(cells: &[i64]) -> Vec<u8> {
    let mut out = Vec::new();
    for &cell in cells {
        let mut value = if cell & 0x8000_0000 != 0 {
            cell - (1 << 32)
        } else {
            cell
        };
        let mut groups = vec![value & 0x7F];
        value >>= 7;
        while !(value == 0 && groups[groups.len() - 1] & 0x40 == 0)
            && !(value == -1 && groups[groups.len() - 1] & 0x40 != 0)
        {
            groups.push(value & 0x7F);
            value >>= 7;
        }
        for (i, group) in groups.iter().enumerate().rev() {
            out.push((group | if i > 0 { 0x80 } else { 0 }) as u8);
        }
    }
    out
}

/// A furniture record's number is a case of the entry point, whose function shows the
/// sign's message: the line of the zone's text file, and the style, 6 a building's plate.
#[test]
fn test_a_sign_shows_the_line_and_style_its_case_calls_the_helper_with() {
    let blob = sign_script(
        &IndexMap::from([(7, vec![(12, script::PLATE)]), (9, vec![(3, 5), (4, 7)])]),
        false,
    );
    let code = script::read(&blob).unwrap();
    let code = code.expect("a script");
    assert_eq!(script::sign_lines(&code, 7).unwrap(), [(12, script::PLATE)]);
    assert_eq!(script::sign_lines(&code, 9).unwrap(), [(3, 5), (4, 7)]);
    assert_eq!(script::sign_lines(&code, 8).unwrap(), []);
}

/// The game stores a script's cells compact, seven bits a byte, and a data offset past the
/// stored bytes' end, since it counts the cells expanded.
#[test]
fn test_a_compact_script_reads_as_the_same_code() {
    let signs = IndexMap::from([(2001, vec![(40, script::PLATE)]), (5, vec![(0x1234, 9)])]);
    let code = script::read(&sign_script(&signs, true)).unwrap();
    let code = code.expect("a script");
    assert_eq!(
        script::sign_lines(&code, 2001).unwrap(),
        [(40, script::PLATE)]
    );
    assert_eq!(script::sign_lines(&code, 5).unwrap(), [(0x1234, 9)]);
    assert_eq!(script::read(b"not a script").unwrap(), None);
}
