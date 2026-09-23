use crate::config::VERTICAL_SPACE;
use crate::core::node_base::NodeId;
use crate::engine::graph::Graph;

impl Graph {
    pub fn gedcom_person_id(&self, person: u32) -> String {
        self.gedcom
            .person(person)
            .map(|p| p.id.clone())
            .unwrap_or_default()
    }

    pub fn node_label(&self, nid: NodeId) -> String {
        self.anim.node_label(&self.gedcom, nid)
    }

    pub fn node_x(&self, nid: NodeId) -> f32 {
        self.anim.node_x(nid)
    }

    pub fn node_y(&self, nid: NodeId) -> f32 {
        self.anim.node_y(nid)
    }

    pub fn node_generation(&self, nid: NodeId) -> i32 {
        self.anim.node_generation(nid)
    }

    pub fn set_person_size(&mut self, pid: u32, w: f32, h: f32) {
        if let Some(p) = self.anim.persons.get_mut(pid as usize) {
            p.base.w = w;
            p.base.h = h;
        }
    }

    pub fn set_all_person_sizes(&mut self, w: f32, h: f32) {
        for p in &mut self.anim.persons {
            p.base.w = w;
            p.base.h = h;
        }
    }

    pub fn describe(&self) -> String {
        let mut out = String::new();
        for n in &self.anim.nodes {
            let generation = self.anim.node_generation(*n);
            out.push_str(&format!("{generation}:  | {} |\n", self.node_label(*n)));
        }
        out
    }

    pub fn line_count(&self) -> usize {
        self.anim.lines.len()
    }

    pub fn back_line_count(&self) -> usize {
        self.anim.back_lines.len()
    }

    pub fn duplicate_line_count(&self) -> usize {
        self.anim.duplicate_lines.len()
    }

    pub fn group_count(&self) -> usize {
        self.anim.groups.len()
    }

    pub fn fulcrum_persons(&self) -> Vec<u32> {
        self.anim
            .persons
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_fulcrum())
            .map(|(i, _)| i as u32)
            .collect()
    }

    pub fn person_gedcom_id(&self, pid: u32) -> String {
        self.anim
            .persons
            .get(pid as usize)
            .map(|p| self.gedcom_person_id(p.person))
            .unwrap_or_default()
    }

    pub fn vertical_space_default() -> f32 {
        VERTICAL_SPACE
    }
    pub fn get_width(&self) -> f32 {
        self.anim.width
    }

    pub fn get_height(&self) -> f32 {
        self.anim.height
    }

    pub fn need_max_bitmap_size(&self) -> bool {
        self.anim.max_bitmap_size == 0.0
    }

    pub fn set_max_bitmap_size(&mut self, size: f32) {
        self.anim.max_bitmap_size = size;
    }

    pub fn get_max_bitmap_size(&self) -> f32 {
        self.anim.max_bitmap_size
    }

    pub fn get_biggest_path_size(&self) -> f32 {
        self.anim.biggest_path_size
    }

    pub fn vertical_calc(&self) -> f32 {
        self.anim.spacing.vertical_calc
    }
}
