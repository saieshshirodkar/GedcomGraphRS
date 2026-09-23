#[derive(Debug, Clone, Default, PartialEq)]
pub struct Name {
    pub display: String,
    pub given: String,
    pub surname: String,
    pub nick: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Fact {
    pub tag: String,
    pub value: Option<String>,
    pub date: Option<String>,
    pub place: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GPerson {
    pub id: String,
    pub names: Vec<Name>,
    pub facts: Vec<Fact>,
    pub parent_fams: Vec<u32>,
    pub spouse_fams: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct GFamily {
    pub id: String,
    pub husbands: Vec<u32>,
    pub wives: Vec<u32>,
    pub children: Vec<u32>,
    pub facts: Vec<Fact>,
}

#[derive(Debug, Clone, Default)]
pub struct GedcomData {
    pub persons: Vec<GPerson>,
    pub families: Vec<GFamily>,
    pub(crate) person_order: Vec<u32>,
    pub(crate) family_order: Vec<u32>,
}

impl Name {
    pub fn of(display: &str) -> Name {
        Name {
            display: display.to_string(),
            given: String::new(),
            surname: String::new(),
            nick: String::new(),
        }
    }
}

impl Fact {
    pub fn of(tag: &str, value: Option<&str>, date: Option<&str>) -> Fact {
        Fact {
            tag: tag.to_string(),
            value: value.map(str::to_string),
            date: date.map(str::to_string),
            place: Option::None,
        }
    }

    pub fn dated(tag: &str, date: Option<&str>) -> Fact {
        Fact {
            tag: tag.to_string(),
            value: Option::None,
            date: date.map(str::to_string),
            place: Option::None,
        }
    }
}

pub fn essence(ged: &GedcomData, person: Option<u32>) -> String {
    if let Some(idx) = person {
        if let Some(p) = ged.person(idx) {
            if let Some(n) = p.names.first() {
                let s: String = n.display.chars().filter(|c| *c != '/').collect();
                let t = s.trim().to_string();
                if !t.is_empty() {
                    return t;
                }
            }
        }
    }
    "[No name]".to_string()
}
