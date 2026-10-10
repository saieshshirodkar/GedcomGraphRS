#[derive(Debug, Clone, Default)]
pub(crate) struct RawNode {
    pub xref: String,
    pub tag: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Arena {
    pub nodes: Vec<RawNode>,
    pub kids: Vec<Vec<usize>>,
    pub roots: Vec<usize>,
    levels: Vec<i32>,
}

pub(crate) const VOID: &str = "VOID";

pub(crate) fn strip_ats(s: &str) -> &str {
    let b = s.as_bytes();
    let mut start = 0usize;
    let mut end = b.len();
    if end > 0 && b[0] == b'@' {
        start = 1;
    }
    if end > start && b[end - 1] == b'@' {
        end -= 1;
    }
    &s[start..end]
}

pub(crate) fn resolve_pointer(value: &str) -> Option<&str> {
    let id = strip_ats(value);
    if id == VOID || id.is_empty() {
        Option::None
    } else {
        Some(id)
    }
}

pub(crate) fn parse_line(line: &str) -> Option<(i32, String, String, String)> {
    let t = line.trim_end();
    if t.trim().is_empty() {
        return Option::None;
    }
    let mut parts = t.splitn(2, char::is_whitespace);
    let level_txt = parts.next().unwrap_or_default().trim();
    let rest = parts.next().unwrap_or_default().trim().to_string();
    let level: i32 = level_txt.parse().ok()?;
    if let Some(stripped) = rest.strip_prefix('@') {
        if let Some(end) = stripped.find('@') {
            let xref = rest[..end + 2].to_string();
            let tail = rest[end + 2..].trim().to_string();
            let mut tp = tail.splitn(2, char::is_whitespace);
            let tag = tp.next().unwrap_or_default().to_string();
            let value = tp.next().unwrap_or_default().to_string();
            return Some((level, xref, tag, value));
        }
        return Option::None;
    }
    let mut tp = rest.splitn(2, char::is_whitespace);
    let tag = tp.next().unwrap_or_default().to_string();
    let value = tp.next().unwrap_or_default().to_string();
    Some((level, String::new(), tag, value))
}

fn undouble_at(value: &str) -> &str {
    value.strip_prefix("@@").unwrap_or(value)
}

fn fold(arena: &mut Arena, level: i32, tag: &str, value: &str) -> bool {
    if tag != "CONT" && tag != "CONC" {
        return false;
    }
    let Some(prev) = arena.nodes.len().checked_sub(1) else {
        return false;
    };
    if !arena.kids[prev].is_empty() {
        return false;
    }
    let Some(prev_level) = arena.levels.get(prev) else {
        return false;
    };
    if *prev_level + 1 != level {
        return false;
    }
    let piece = undouble_at(value);
    let sep = if tag == "CONT" { "\n" } else { "" };
    let node = &mut arena.nodes[prev];
    node.value.push_str(sep);
    node.value.push_str(piece);
    true
}

pub(crate) fn build_arena(text: &str) -> Arena {
    let mut arena = Arena::default();
    arena.nodes.reserve(text.len() / 24);
    arena.kids.reserve(text.len() / 24);
    arena.levels.reserve(text.len() / 24);
    let mut stack: Vec<(i32, usize)> = Vec::new();
    for line in text.lines() {
        let line = line.strip_prefix('\u{FEFF}').unwrap_or(line);
        let Some((level, xref, tag, value)) = parse_line(line) else {
            continue;
        };
        if fold(&mut arena, level, &tag, &value) {
            continue;
        }
        let idx = arena.nodes.len();
        arena.nodes.push(RawNode { xref, tag, value });
        arena.kids.push(Vec::new());
        arena.levels.push(level);
        while let Some((lvl, _)) = stack.last() {
            if *lvl < level {
                break;
            }
            stack.pop();
        }
        if let Some((_, parent)) = stack.last().copied() {
            arena.kids[parent].push(idx);
        } else {
            arena.roots.push(idx);
        }
        stack.push((level, idx));
    }
    arena
}

pub(crate) fn child_date(arena: &Arena, node: usize) -> Option<String> {
    for c in &arena.kids[node] {
        let n = &arena.nodes[*c];
        if n.tag == "DATE" && !n.value.is_empty() {
            return Some(n.value.clone());
        }
    }
    Option::None
}

pub(crate) fn child_place(arena: &Arena, node: usize) -> Option<String> {
    for c in &arena.kids[node] {
        let n = &arena.nodes[*c];
        if n.tag == "PLAC" && !n.value.is_empty() {
            return Some(n.value.clone());
        }
    }
    Option::None
}

pub(crate) fn child_text(arena: &Arena, node: usize, tag: &str) -> Option<String> {
    for c in &arena.kids[node] {
        let n = &arena.nodes[*c];
        if n.tag == tag && !n.value.is_empty() {
            return Some(n.value.clone());
        }
    }
    Option::None
}
