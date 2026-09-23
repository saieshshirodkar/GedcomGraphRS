use crate::core::node_base::NodeId;
use crate::engine::animator::{Animator, LineRowScratch};
use crate::lines::segments::LineSeg;

impl Animator {
    pub(crate) fn normalize_origin(&mut self) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for n in self.nodes.clone() {
            let (x, y, w, h) = (
                self.node_x(n),
                self.node_y(n),
                self.node_width(n),
                self.node_height(n),
            );
            if x < min_x {
                min_x = x;
            }
            if x + w > max_x {
                max_x = x + w;
            }
            if y < min_y {
                min_y = y;
            }
            if y + h > max_y {
                max_y = y + h;
            }
        }
        if min_x == f32::MAX {
            self.width = 0.0;
            self.height = 0.0;
            return;
        }
        self.width = max_x - min_x;
        self.height = max_y - min_y;
        let all = self.nodes.clone();
        for n in all {
            match n {
                NodeId::Person(i) => {
                    if let Some(p) = self.persons.get_mut(i as usize) {
                        p.base.x -= min_x;
                        p.base.y -= min_y;
                    }
                }
                NodeId::Family(i) => {
                    if let Some(f) = self.families.get_mut(i as usize) {
                        f.base.x -= min_x;
                        f.base.y -= min_y;
                        let partners = f.partners.clone();
                        for q in partners {
                            if let Some(p) = self.persons.get_mut(q as usize) {
                                p.base.x -= min_x;
                                p.base.y -= min_y;
                            }
                        }
                        if let Some(b) = f.bond.as_mut() {
                            b.x -= min_x;
                            b.y -= min_y;
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn mirror_horizontal(&mut self) {
        let w = self.width;
        for f in &mut self.families {
            f.base.x = w - f.base.x - f.base.w;
        }
        for p in &mut self.persons {
            p.base.x = w - p.base.x - p.base.w;
        }
        for f in &mut self.families {
            if let Some(b) = f.bond.as_mut() {
                b.x = w - b.x - b.w;
            }
        }
    }

    pub(crate) fn update_duplicate_lines(&mut self) {
        let vcalc = self.spacing.vertical_calc;
        let persons = self.persons.clone();
        for d in &mut self.duplicate_lines {
            d.update(&persons, vcalc);
        }
    }

    pub(crate) fn distribute_into(&mut self, back: bool) {
        if self.max_bitmap_size == 0.0 {
            return;
        }
        if back {
            let (lines, persons, families) = (&mut self.back_lines, &self.persons, &self.families);
            for l in lines.iter_mut() {
                l.update(persons, families);
            }
            lines.sort_by(LineSeg::compare);
        } else {
            let (lines, persons, families) = (&mut self.lines, &self.persons, &self.families);
            for l in lines.iter_mut() {
                l.update(persons, families);
            }
            lines.sort_by(LineSeg::compare);
        }
        let rows = if back {
            &mut self.back_rows
        } else {
            &mut self.line_rows
        };
        for r in rows.iter_mut() {
            r.restart_x = 0.0;
            for b in r.buckets.iter_mut() {
                b.clear();
            }
        }
        let maxb = self.max_bitmap_size;
        let mut biggest = self.biggest_path_size;
        let lines: &[LineSeg] = if back { &self.back_lines } else { &self.lines };
        let rows = if back {
            &mut self.back_rows
        } else {
            &mut self.line_rows
        };
        for l in lines {
            let mut row_num = (l.y2 / maxb).floor() as usize;
            if l.y2.is_sign_negative() {
                row_num = 0;
            }
            while row_num >= rows.len() {
                rows.push(LineRowScratch::default());
            }
            let row = &mut rows[row_num];
            if row.buckets.is_empty() || l.left() > row.restart_x + maxb {
                row.restart_x = l.left();
                row.buckets.push(Vec::new());
            }
            if let Some(last) = row.buckets.last_mut() {
                last.push(*l);
            }
            let path = l.right() - row.restart_x;
            if path > biggest {
                biggest = path;
            }
        }
        self.biggest_path_size = biggest;
        let groups_out = if back {
            &mut self.back_line_groups
        } else {
            &mut self.line_groups
        };
        groups_out.clear();
        let rows = if back {
            &self.back_rows
        } else {
            &self.line_rows
        };
        for r in rows {
            for b in &r.buckets {
                if !b.is_empty() {
                    groups_out.push(b.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_run_normalizes_to_zero() {
        let mut a = Animator::fresh();
        a.place_nodes();
        assert_eq!((a.width, a.height), (0.0, 0.0));
        assert_eq!(a.biggest_path_size, 0.0);
    }
}
