use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

#[derive(Debug, Clone, Default)]
pub struct Union {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub list: Vec<NodeId>,
    pub generation: i32,
    pub ancestor: Option<NodeId>,
    pub prev: Option<u32>,
    pub next: Option<u32>,
    pub descendants: Vec<u32>,
    pub youths: Vec<u32>,
    pub column_shift: f32,
}

impl Union {
    pub fn of(generation: i32) -> Union {
        Union {
            generation,
            ..Union::default()
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
    pub(crate) fn union_origins(&self, uid: u32) -> Vec<NodeId> {
        if let Some(u) = self.unions.get(uid as usize) {
            if let Some(a) = u.ancestor {
                return match a {
                    NodeId::Person(i) => self
                        .persons
                        .get(i as usize)
                        .and_then(|p| p.origin)
                        .map(|o| vec![o])
                        .unwrap_or_default(),
                    NodeId::Family(_) => self.family_origins(a),
                };
            }
        }
        Vec::new()
    }

    pub(crate) fn family_origins(&self, nid: NodeId) -> Vec<NodeId> {
        let mut out: Vec<NodeId> = Vec::new();
        if let NodeId::Family(i) = nid {
            if let Some(f) = self.families.get(i as usize) {
                for p in f.partners.clone() {
                    if let Some(q) = self.persons.get(p as usize) {
                        if let Some(o) = q.origin {
                            if !self.node_mini(o) && self.node_person_count(o) > 0 {
                                out.push(o);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    pub(crate) fn union_update_x(&mut self, uid: u32) {
        let fx = self
            .unions
            .get(uid as usize)
            .and_then(|u| u.list.first().copied())
            .map(|n| self.node_x(n))
            .unwrap_or(0.0);
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.x = fx;
        }
    }

    pub(crate) fn union_refresh_width(&mut self, uid: u32) -> f32 {
        let w = self
            .unions
            .get(uid as usize)
            .and_then(|u| u.list.last().copied().zip(u.list.first().copied()))
            .map(|(last, first)| {
                if self
                    .unions
                    .get(uid as usize)
                    .map(|u| u.list.len())
                    .unwrap_or(0)
                    == 1
                {
                    self.node_width(last)
                } else {
                    self.node_x(last) + self.node_width(last) - self.node_x(first)
                }
            })
            .unwrap_or(0.0);
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.w = w;
        }
        w
    }

    pub(crate) fn union_refresh_height(&self, uid: u32) -> f32 {
        let mut h = 0.0f32;
        if let Some(u) = self.unions.get(uid as usize) {
            for n in u.list.clone() {
                h = h.max(self.node_height(n));
            }
        }
        h
    }

    pub(crate) fn union_center_x(&self, uid: u32) -> f32 {
        let ux = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
        ux + self.union_center_rel_x(uid)
    }

    pub(crate) fn union_center_rel_x(&self, uid: u32) -> f32 {
        if let Some(u) = self.unions.get(uid as usize) {
            if let Some(a) = u.ancestor {
                return self.node_center_x(a) - u.x;
            }
        }
        0.0
    }

    pub(crate) fn union_set_x(&mut self, uid: u32, x: f32) {
        let gx = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
        let diff = x - gx;
        let members = self
            .unions
            .get(uid as usize)
            .map(|u| u.list.clone())
            .unwrap_or_default();
        for n in members {
            let nx = self.node_x(n);
            self.set_node_x(n, nx + diff);
        }
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.x = x;
        }
    }
}
