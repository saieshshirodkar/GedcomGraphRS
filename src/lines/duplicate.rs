use crate::config::Gender;
use crate::model::gedcom::GedcomData;
use crate::nodes::person::PersonNodeData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DupLine {
    pub first: u32,
    pub second: u32,
    pub gender: Gender,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub x3: f32,
    pub y3: f32,
}

impl DupLine {
    pub fn of(ged: &GedcomData, first: u32, second: u32, persons: &[PersonNodeData]) -> DupLine {
        let fp = persons
            .get(first as usize)
            .map(|p| p.person)
            .unwrap_or(u32::MAX);
        DupLine {
            first,
            second,
            gender: Gender::of_person(ged, fp),
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            x3: 0.0,
            y3: 0.0,
        }
    }

    pub fn left(&self) -> f32 {
        self.x1.min(self.x2)
    }

    pub fn update(&mut self, persons: &[PersonNodeData], vertical_calc: f32) {
        let (a, b) = match (
            persons.get(self.first as usize),
            persons.get(self.second as usize),
        ) {
            (Some(x), Some(y)) => (x, y),
            _ => return,
        };
        let shift = 1.5f32;
        if a.base.generation == b.base.generation {
            if b.base.x > a.base.x {
                self.x1 = a.base.x + a.base.w - shift;
                self.x2 = b.base.x + shift;
            } else {
                self.x1 = a.base.x + shift;
                self.x2 = b.base.x + b.base.w - shift;
            }
            self.y1 = a.base.y + a.base.h - shift;
            self.y2 = b.base.y + b.base.h - shift;
            self.x3 = self.x1 + (self.x2 - self.x1) / 2.0;
            self.y3 = self.y1.max(self.y2) + vertical_calc.min((self.x2 - self.x1).abs());
        } else {
            if a.base.x < b.base.x {
                if b.base.x > a.base.x + a.base.w {
                    self.x1 = a.base.x + a.base.w - shift;
                    self.x2 = b.base.x + shift;
                } else {
                    self.x1 = a.base.x + (a.base.w / 4.0) * 3.0;
                    self.x2 = b.base.x + b.base.w / 4.0;
                }
            } else if a.base.x > b.base.x + b.base.w {
                self.x1 = a.base.x + shift;
                self.x2 = b.base.x + b.base.w - shift;
            } else {
                self.x1 = a.base.x + a.base.w / 4.0;
                self.x2 = b.base.x + (b.base.w / 4.0) * 3.0;
            }
            if a.base.y < b.base.y {
                self.y1 = a.base.y + a.base.h - shift;
                self.y2 = b.base.y + shift;
            } else {
                self.y1 = a.base.y + shift;
                self.y2 = b.base.y + b.base.h - shift;
            }
            self.x3 = self.x1 + (self.x2 - self.x1) / 2.0;
            self.y3 = self.y1 + (self.y2 - self.y1) / 2.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Card;
    use crate::model::gedcom::{Fact, GPerson, Name};

    fn persons_same_gen() -> Vec<PersonNodeData> {
        let mut a = PersonNodeData::single(0, Card::Regular, 0);
        a.base.x = 0.0;
        a.base.y = 0.0;
        a.base.w = 40.0;
        a.base.h = 30.0;
        let mut b = PersonNodeData::single(1, Card::Regular, 0);
        b.base.x = 100.0;
        b.base.y = 10.0;
        b.base.w = 40.0;
        b.base.h = 30.0;
        vec![a, b]
    }

    fn ged_two() -> GedcomData {
        let mut g = GedcomData::empty();
        g.push_person(GPerson {
            id: "I1".to_string(),
            names: vec![Name::of("A")],
            facts: vec![Fact::of("SEX", Some("M"), Option::None)],
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.push_person(GPerson {
            id: "I2".to_string(),
            names: vec![Name::of("B")],
            facts: Vec::new(),
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        });
        g.reindex();
        g
    }

    #[test]
    fn gender_from_first() {
        let g = ged_two();
        let p = persons_same_gen();
        let d = DupLine::of(&g, 0, 1, &p);
        assert_eq!(d.gender, Gender::Male);
        assert_eq!(d.left(), 0.0);
    }

    #[test]
    fn same_generation_layout() {
        let g = ged_two();
        let p = persons_same_gen();
        let mut d = DupLine::of(&g, 0, 1, &p);
        d.update(&p, 90.0);
        assert_eq!((d.x1, d.x2), (38.5, 101.5));
        assert_eq!((d.y1, d.y2), (28.5, 38.5));
        assert_eq!(d.x3, (38.5 + 101.5) / 2.0);
        assert_eq!(d.y3, 38.5 + 63.0f32.min(90.0));
    }

    #[test]
    fn cross_generation_layout() {
        let g = ged_two();
        let mut p = persons_same_gen();
        p[1].base.generation = 1;
        p[1].base.y = 200.0;
        let mut d = DupLine::of(&g, 0, 1, &p);
        d.update(&p, 90.0);
        assert_eq!((d.x1, d.x2), (38.5, 101.5));
        assert_eq!((d.y1, d.y2), (28.5, 201.5));
        assert_eq!(d.y3, (28.5 + 201.5) / 2.0);
    }

    #[test]
    fn missing_person_keeps_coords() {
        let g = ged_two();
        let p = persons_same_gen();
        let mut d = DupLine::of(&g, 0, 9, &p);
        d.update(&p, 90.0);
        assert_eq!((d.x1, d.y1, d.x2, d.y2), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn reversed_same_generation() {
        let g = ged_two();
        let mut p = persons_same_gen();
        p.swap(0, 1);
        let mut d = DupLine::of(&g, 0, 1, &p);
        d.update(&p, 90.0);
        assert_eq!((d.x1, d.x2), (101.5, 38.5));
        assert_eq!((d.y1, d.y2), (38.5, 28.5));
    }

    #[test]
    fn cross_generation_overlap() {
        let g = ged_two();
        let mut p = persons_same_gen();
        p[1].base.generation = 1;
        p[1].base.x = 10.0;
        p[1].base.y = 200.0;
        let mut d = DupLine::of(&g, 0, 1, &p);
        d.update(&p, 90.0);
        assert_eq!((d.x1, d.x2), (30.0, 20.0));
        assert_eq!((d.y1, d.y2), (28.5, 201.5));
    }

    #[test]
    fn cross_generation_gap_left() {
        let g = ged_two();
        let mut p = persons_same_gen();
        p[0].base.x = 100.0;
        p[0].base.y = 0.0;
        p[1].base.generation = 1;
        p[1].base.x = 0.0;
        p[1].base.y = 200.0;
        let mut d = DupLine::of(&g, 0, 1, &p);
        d.update(&p, 90.0);
        assert_eq!((d.x1, d.x2), (101.5, 38.5));
        assert_eq!((d.y1, d.y2), (28.5, 201.5));
    }
}
