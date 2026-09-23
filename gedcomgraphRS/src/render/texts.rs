use crate::model::gedcom::GedcomData;

pub fn given_name(display: &str) -> String {
    let value = display.trim();
    let slash = value.find('/');
    let last = value.rfind('/');
    let given = match (slash, last) {
        (Some(0), Some(1)) if value.len() > 2 => &value[2..],
        (Some(0), Some(e)) if e > 1 => &value[1..e],
        (Some(s), _) if s > 0 => &value[..s],
        _ => value,
    };
    let t = given.trim().to_string();
    if t.is_empty() {
        "[No name]".to_string()
    } else {
        t
    }
}

pub fn name_lines(ged: &GedcomData, person: u32, duplicate: bool) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if let Some(p) = ged.person(person) {
        if let Some(n) = p.names.first() {
            lines.push(given_name(&n.display));
        } else {
            lines.push("[No name]".to_string());
        }
    } else {
        lines.push("[No name]".to_string());
    }
    if duplicate {
        if let Some(last) = lines.last_mut() {
            last.push_str(" (2)");
        } else {
            lines.push("(2)".to_string());
        }
    }
    if lines.is_empty() {
        lines.push("[No name]".to_string());
    }
    lines
}

pub fn titles_of(ged: &GedcomData, person: u32) -> String {
    if let Some(p) = ged.person(person) {
        let mut out = String::new();
        for f in &p.facts {
            if f.tag == "TITL" {
                if let Some(v) = &f.value {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(v);
                }
            }
        }
        return out;
    }
    String::new()
}

pub fn is_dead(ged: &GedcomData, person: u32) -> bool {
    if let Some(p) = ged.person(person) {
        for f in &p.facts {
            if f.tag == "DEAT" || f.tag == "BURI" {
                return true;
            }
        }
    }
    false
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

fn first_date(ged: &GedcomData, person: u32, tags: &[&str]) -> Option<String> {
    ged.person(person).and_then(|p| {
        p.facts
            .iter()
            .find(|f| tags.contains(&f.tag.as_str()) && f.date.is_some())
            .and_then(|f| f.date.clone())
    })
}

pub fn date_lines(ged: &GedcomData, person: u32) -> Vec<String> {
    let birth = first_date(ged, person, &["BIRT"]);
    let christening = if birth.is_none() {
        first_date(ged, person, &["CHR", "BAPM"])
    } else {
        Option::None
    };
    let death = first_date(ged, person, &["DEAT"]);
    let mut birth_line = String::new();
    if let Some(b) = birth {
        birth_line.push_str("\u{2605} ");
        birth_line.push_str(&b);
    } else if let Some(c) = christening {
        birth_line.push_str("\u{2248} ");
        birth_line.push_str(&c);
    }
    let mut death_line = String::new();
    if let Some(d) = death {
        death_line.push_str("\u{271B} ");
        death_line.push_str(&d);
    }
    let mut lines: Vec<String> = Vec::new();
    if !birth_line.is_empty() && !death_line.is_empty() {
        if utf16_len(&birth_line) > 7 || utf16_len(&death_line) > 7 {
            lines.push(birth_line);
            lines.push(death_line);
        } else {
            lines.push(birth_line + "  " + &death_line);
        }
    } else if !birth_line.is_empty() {
        lines.push(birth_line);
    } else if !death_line.is_empty() {
        lines.push(death_line);
    } else if let Some(p) = ged.person(person) {
        for f in &p.facts {
            if let Some(d) = &f.date {
                lines.push(d.clone());
                break;
            }
        }
    }
    lines
}
