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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PersonId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FamilyId(pub u32);

impl From<u32> for PersonId {
    fn from(v: u32) -> PersonId {
        PersonId(v)
    }
}

impl From<PersonId> for u32 {
    fn from(v: PersonId) -> u32 {
        v.0
    }
}

impl From<PersonId> for usize {
    fn from(v: PersonId) -> usize {
        v.0 as usize
    }
}

impl std::fmt::Display for PersonId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for FamilyId {
    fn from(v: u32) -> FamilyId {
        FamilyId(v)
    }
}

impl From<FamilyId> for u32 {
    fn from(v: FamilyId) -> u32 {
        v.0
    }
}

impl From<FamilyId> for usize {
    fn from(v: FamilyId) -> usize {
        v.0 as usize
    }
}

#[derive(Debug, Clone, Default)]
pub struct GPerson {
    pub id: String,
    pub names: Vec<Name>,
    pub facts: Vec<Fact>,
    pub parent_fams: Vec<FamilyId>,
    pub spouse_fams: Vec<FamilyId>,
}

#[derive(Debug, Clone, Default)]
pub struct GFamily {
    pub id: String,
    pub husbands: Vec<PersonId>,
    pub wives: Vec<PersonId>,
    pub children: Vec<PersonId>,
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
