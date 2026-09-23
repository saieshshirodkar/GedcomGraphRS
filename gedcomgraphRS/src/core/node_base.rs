use crate::config::Match;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
    Person(u32),
    Family(u32),
}

#[derive(Debug, Clone, Default)]
pub struct NodeBase {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub generation: i32,
    pub mini: bool,
    pub is_ancestor: bool,
    pub married_siblings: bool,
    pub union: Option<u32>,
    pub group: Option<u32>,
    pub youth: Option<u32>,
    pub prev: Option<NodeId>,
    pub next: Option<NodeId>,
    pub stamp: Match,
    pub origins: Option<Vec<NodeId>>,
    pub force: f32,
    pub column_shift: f32,
    pub spouse_family: Option<u32>,
}

impl NodeId {
    pub fn is_person(self) -> bool {
        matches!(self, NodeId::Person(_))
    }

    pub fn is_family(self) -> bool {
        matches!(self, NodeId::Family(_))
    }

    pub fn person_index(self) -> Option<u32> {
        match self {
            NodeId::Person(i) => Some(i),
            NodeId::Family(_) => Option::None,
        }
    }

    pub fn family_index(self) -> Option<u32> {
        match self {
            NodeId::Family(i) => Some(i),
            NodeId::Person(_) => Option::None,
        }
    }

    pub fn same_slot(self, other: NodeId) -> bool {
        self == other
    }
}

impl NodeBase {
    pub fn of_generation(generation: i32) -> NodeBase {
        NodeBase {
            generation,
            ..NodeBase::default()
        }
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.center_rel_simple()
    }

    pub fn center_rel_simple(&self) -> f32 {
        self.w / 2.0
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn is_multi_marriage(&self) -> bool {
        self.stamp.is_multi()
    }

    pub fn shift(&mut self, dx: f32) {
        self.x += dx;
    }

    pub fn grow(&mut self, dw: f32, dh: f32) {
        self.w += dw;
        self.h += dh;
    }

    pub fn has_finite_rect(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.w.is_finite() && self.h.is_finite()
    }

    pub fn link_next(&mut self, next: Option<NodeId>) {
        self.next = next;
    }

    pub fn link_prev(&mut self, prev: Option<NodeId>) {
        self.prev = prev;
    }

    pub fn attach_union(&mut self, union: Option<u32>) {
        self.union = union;
    }

    pub fn attach_group(&mut self, group: Option<u32>) {
        self.group = group;
    }

    pub fn attach_youth(&mut self, youth: Option<u32>) {
        self.youth = youth;
    }

    pub fn push_origin(&mut self, origin: NodeId) {
        let list = self.origins.get_or_insert_with(Vec::new);
        if !list.contains(&origin) {
            list.push(origin);
        }
    }

    pub fn clear_links(&mut self) {
        self.prev = Option::None;
        self.next = Option::None;
        self.union = Option::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_kinds() {
        let p = NodeId::Person(3);
        let f = NodeId::Family(5);
        assert!(p.is_person());
        assert!(!p.is_family());
        assert!(f.is_family());
        assert_eq!(p.person_index(), Some(3));
        assert_eq!(p.family_index(), Option::None);
        assert_eq!(f.family_index(), Some(5));
        assert!(p.same_slot(NodeId::Person(3)));
        assert!(!p.same_slot(f));
    }

    #[test]
    fn base_geometry() {
        let mut b = NodeBase::of_generation(-2);
        b.x = 10.0;
        b.w = 20.0;
        assert_eq!(b.center_x(), 20.0);
        assert_eq!(b.right(), 30.0);
        b.shift(5.0);
        assert_eq!(b.x, 15.0);
        b.grow(2.0, 4.0);
        assert_eq!((b.w, b.h), (22.0, 4.0));
        assert!(b.has_finite_rect());
    }

    #[test]
    fn links_and_unions() {
        let mut b = NodeBase::default();
        b.link_next(Some(NodeId::Person(1)));
        b.link_prev(Some(NodeId::Family(2)));
        b.attach_union(Some(7));
        b.attach_group(Some(8));
        b.attach_youth(Option::None);
        assert_eq!(b.next, Some(NodeId::Person(1)));
        b.push_origin(NodeId::Person(1));
        b.push_origin(NodeId::Person(1));
        assert_eq!(b.origins.as_ref().map(|v| v.len()), Some(1));
        b.clear_links();
        assert_eq!(b.next, Option::None);
        assert_eq!(b.union, Option::None);
    }

    #[test]
    fn multi_marriage_flag() {
        let mut b = NodeBase::default();
        assert!(!b.is_multi_marriage());
        b.stamp = Match::Near;
        assert!(b.is_multi_marriage());
        assert_eq!(b.bottom(), b.y + b.h);
    }
}
