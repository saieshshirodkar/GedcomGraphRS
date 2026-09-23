use crate::config::{Branch, HORIZONTAL_SPACE};
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

#[derive(Debug, Clone, Default)]
pub struct Group {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub list: Vec<NodeId>,
    pub origin: Option<NodeId>,
    pub first: Option<u32>,
    pub last: Option<u32>,
    pub generation: i32,
    pub mini: bool,
    pub branch: Branch,
}

impl Group {
    pub fn of(generation: i32, mini: bool, branch: Branch) -> Group {
        Group {
            generation,
            mini,
            branch,
            ..Group::default()
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

impl Animator {
    pub(crate) fn group_add_node(&mut self, gid: u32, nid: NodeId, index: Option<usize>) {
        if let Some(g) = self.groups.get_mut(gid as usize) {
            if let Some(at) = index {
                if at <= g.list.len() {
                    g.list.insert(at, nid);
                } else {
                    g.list.push(nid);
                }
            } else {
                g.list.push(nid);
            }
        }
        self.set_node_group(nid, Some(gid));
    }

    pub(crate) fn group_set_origin(&mut self, gid: u32) {
        let list = self
            .groups
            .get(gid as usize)
            .map(|g| g.list.clone())
            .unwrap_or_default();
        let branch = self
            .groups
            .get(gid as usize)
            .map(|g| g.branch)
            .unwrap_or(Branch::None);
        let mini = self
            .groups
            .get(gid as usize)
            .map(|g| g.mini)
            .unwrap_or(false);
        let mut found: Option<NodeId> = Option::None;
        for nid in &list {
            if self.node_is_ancestor(*nid) && self.node_person_count(*nid) > 1 {
                let slot = if branch == Branch::Mater { 1 } else { 0 };
                if let Some(o) = self.node_partner_origin(*nid, slot) {
                    found = Some(o);
                }
            }
        }
        if found.is_none() {
            for nid in &list {
                if let Some(o) = self.node_main_origin(*nid) {
                    found = Some(o);
                    break;
                }
            }
        }
        if let Some(g) = self.groups.get_mut(gid as usize) {
            g.origin = found;
        }
        if let Some(o) = found {
            self.set_node_youth(o, Some(gid));
        }
        if !mini {
            self.group_find_first(gid, &list, branch);
            self.group_find_last(gid, &list, branch);
        }
    }

    fn group_find_first(&mut self, gid: u32, list: &[NodeId], branch: Branch) {
        for nid in list {
            if self.node_is_ancestor(*nid)
                && branch == Branch::Mater
                && !self.node_married_siblings(*nid)
            {
                let w = self.node_wife(*nid);
                if let Some(g) = self.groups.get_mut(gid as usize) {
                    g.first = w;
                }
                return;
            }
            for p in self.node_persons(*nid) {
                if !self.person_acquired(p) {
                    if let Some(g) = self.groups.get_mut(gid as usize) {
                        g.first = Some(p);
                    }
                    return;
                }
            }
        }
    }

    fn group_find_last(&mut self, gid: u32, list: &[NodeId], branch: Branch) {
        for nid in list.iter().rev() {
            if self.node_is_ancestor(*nid)
                && branch == Branch::Pater
                && !self.node_married_siblings(*nid)
            {
                let hb = self.node_husband(*nid);
                if let Some(g) = self.groups.get_mut(gid as usize) {
                    g.last = hb;
                }
                return;
            }
            let persons = self.node_persons(*nid);
            for p in persons.iter().rev() {
                if !self.person_acquired(*p) {
                    if let Some(g) = self.groups.get_mut(gid as usize) {
                        g.last = Some(*p);
                    }
                    return;
                }
            }
        }
    }

    pub(crate) fn group_is_origin_mini_or_empty(&self, gid: u32) -> bool {
        if let Some(g) = self.groups.get(gid as usize) {
            if let Some(o) = g.origin {
                if self.node_is_multi(o) {
                    return false;
                }
                return self.node_mini(o) || self.node_person_count(o) == 0;
            }
        }
        false
    }
    pub(crate) fn group_basic_left_width(&self, gid: u32) -> f32 {
        let (list, first, branch) = self
            .groups
            .get(gid as usize)
            .map(|g| (g.list.clone(), g.first, g.branch))
            .unwrap_or_default();
        let mut w = 0.0f32;
        for nid in list {
            if let Some(f) = first {
                if self.node_persons(nid).contains(&f) {
                    w += self.node_left_width(nid, branch);
                    break;
                }
            }
            w += self.node_width(nid) + HORIZONTAL_SPACE;
        }
        w
    }

    pub(crate) fn group_basic_central_width(&self, gid: u32) -> f32 {
        let (list, first, last, branch) = self
            .groups
            .get(gid as usize)
            .map(|g| (g.list.clone(), g.first, g.last, g.branch))
            .unwrap_or_default();
        let mut w = 0.0f32;
        if let (Some(f), Some(l)) = (first, last) {
            if f != l {
                let start = self.person_family_node(f);
                let end = self.person_family_node(l);
                if let (Some(s), Some(e)) = (start, end) {
                    let a = list.iter().position(|n| *n == s).unwrap_or(0);
                    let b = list.iter().position(|n| *n == e).unwrap_or(a);
                    for n in &list[a..b] {
                        w += self.node_width(*n) + HORIZONTAL_SPACE;
                    }
                    w = w - self.node_left_width(s, branch) + self.node_left_width(e, branch);
                }
            }
        }
        w
    }
}
