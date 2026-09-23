use crate::config::UNION_DISTANCE;
use crate::core::node_base::NodeId;
use crate::engine::animator::Animator;

impl Animator {
    pub(crate) fn union_outdistance_ancestor_column(&mut self, uid: u32) {
        let ancestor = self.unions.get(uid as usize).and_then(|u| u.ancestor);
        let Some(a) = ancestor else { return };
        let Some(youth) = self.node_youth(a) else {
            return;
        };
        self.group_refresh_width(youth);
        let youth_cx = self.group_center_x(youth);
        let self_cx = self.union_center_x(uid);
        let youth_distance = youth_cx - self_cx;
        let mut shift = 0.0f32;
        let (prev, next) = self
            .unions
            .get(uid as usize)
            .map(|u| (u.prev, u.next))
            .unwrap_or((Option::None, Option::None));
        let desc0 = self
            .unions
            .get(uid as usize)
            .and_then(|u| u.descendants.first().copied());
        if let Some(p) = prev {
            let same = self
                .unions
                .get(p as usize)
                .and_then(|u| u.descendants.first().copied())
                == desc0;
            if same {
                let pw = self.union_refresh_width(p);
                let px = self.unions.get(p as usize).map(|u| u.x).unwrap_or(0.0);
                let ux = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
                shift = (px + pw + UNION_DISTANCE - ux).max(youth_distance);
            }
        }
        if let Some(n) = next {
            let same = self
                .unions
                .get(n as usize)
                .and_then(|u| u.descendants.first().copied())
                == desc0;
            if same {
                let uw = self.union_refresh_width(uid);
                let ux = self.unions.get(uid as usize).map(|u| u.x).unwrap_or(0.0);
                let nx = self.unions.get(n as usize).map(|u| u.x).unwrap_or(0.0);
                shift = (nx - ux - uw - UNION_DISTANCE).min(youth_distance);
            }
        }
        shift = self.find_ancestor_shift(uid, a, shift);
        if let Some(u) = self.unions.get_mut(uid as usize) {
            u.column_shift = shift;
        }
        if shift != 0.0 {
            self.union_move_ascending(uid, shift);
        }
    }

    fn find_ancestor_shift(&self, uid: u32, nid: NodeId, mut shift: f32) -> f32 {
        for o in self.family_or_single_origins(nid) {
            if let Some(ou) = self.node_union(o) {
                let (uprev, unext) = self
                    .unions
                    .get(ou as usize)
                    .map(|u| (u.prev, u.next))
                    .unwrap_or((Option::None, Option::None));
                if let Some(p) = uprev {
                    let pdesc = self
                        .unions
                        .get(p as usize)
                        .map(|u| u.descendants.clone())
                        .unwrap_or_default();
                    if !pdesc.contains(&uid) {
                        let pw = self.union_width_of(p);
                        let px = self.unions.get(p as usize).map(|u| u.x).unwrap_or(0.0);
                        let ox = self.unions.get(ou as usize).map(|u| u.x).unwrap_or(0.0);
                        let left = px + pw + UNION_DISTANCE - ox;
                        if left > shift {
                            shift = left;
                        }
                    }
                }
                if let Some(n) = unext {
                    let ndesc = self
                        .unions
                        .get(n as usize)
                        .map(|u| u.descendants.clone())
                        .unwrap_or_default();
                    if !ndesc.contains(&uid) {
                        let ow = self.union_width_of(ou);
                        let ox = self.unions.get(ou as usize).map(|u| u.x).unwrap_or(0.0);
                        let nx = self.unions.get(n as usize).map(|u| u.x).unwrap_or(0.0);
                        let right = nx - ox - ow - UNION_DISTANCE;
                        if right < shift {
                            shift = right;
                        }
                    }
                }
                shift = self.find_ancestor_shift(uid, o, shift);
            }
        }
        shift
    }

    fn union_width_of(&self, uid: u32) -> f32 {
        if let Some(u) = self.unions.get(uid as usize) {
            if u.list.len() == 1 {
                if let Some(n) = u.list.first() {
                    return self.node_width(*n);
                }
                return 0.0;
            }
            if let (Some(a), Some(b)) = (u.list.first(), u.list.last()) {
                return self.node_x(*b) + self.node_width(*b) - self.node_x(*a);
            }
        }
        0.0
    }
}
