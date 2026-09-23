use crate::config::{Branch, Spacing};
use crate::engine::animator::Animator;
use crate::layout::group::Group;
use crate::model::gedcom::GedcomData;

#[derive(Debug, Clone)]
pub struct Graph {
    pub gedcom: GedcomData,
    pub anim: Animator,
    pub fulcrum: Option<u32>,
    pub fulcrum_group: u32,
    pub which_family: usize,
    pub ancestor_generations: i32,
    pub great_uncles_generations: i32,
    pub with_spouses: bool,
    pub descendant_generations: i32,
    pub sibling_nephew_generations: i32,
    pub uncle_cousin_generations: i32,
    pub with_numbers: bool,
    pub with_duplicate_lines: bool,
    pub left_to_right: bool,
    pub max_above: i32,
    pub max_below: i32,
}

impl Default for Graph {
    fn default() -> Graph {
        Graph {
            gedcom: GedcomData::empty(),
            anim: Animator::fresh(),
            fulcrum: Option::None,
            fulcrum_group: 0,
            which_family: 0,
            ancestor_generations: 3,
            great_uncles_generations: 2,
            with_spouses: true,
            descendant_generations: 3,
            sibling_nephew_generations: 2,
            uncle_cousin_generations: 2,
            with_numbers: true,
            with_duplicate_lines: true,
            left_to_right: false,
            max_above: 0,
            max_below: 0,
        }
    }
}

impl Graph {
    pub fn create() -> Graph {
        Graph::default()
    }

    pub fn with_gedcom(gedcom: GedcomData) -> Graph {
        Graph {
            gedcom,
            ..Graph::default()
        }
    }

    pub fn set_gedcom(&mut self, gedcom: GedcomData) -> &mut Graph {
        self.gedcom = gedcom;
        self
    }

    pub fn show_family(&mut self, num: usize) -> &mut Graph {
        self.which_family = num;
        self
    }

    pub fn max_ancestors(&mut self, num: i32) -> &mut Graph {
        self.ancestor_generations = num;
        self
    }

    pub fn max_great_uncles(&mut self, num: i32) -> &mut Graph {
        self.great_uncles_generations = num;
        self
    }

    pub fn display_spouses(&mut self, display: bool) -> &mut Graph {
        self.with_spouses = display;
        self
    }

    pub fn max_descendants(&mut self, num: i32) -> &mut Graph {
        self.descendant_generations = num;
        self
    }

    pub fn max_siblings_nephews(&mut self, num: i32) -> &mut Graph {
        self.sibling_nephew_generations = num;
        self
    }

    pub fn max_uncles_cousins(&mut self, num: i32) -> &mut Graph {
        self.uncle_cousin_generations = num;
        self
    }

    pub fn display_numbers(&mut self, display: bool) -> &mut Graph {
        self.with_numbers = display;
        self.anim.spacing = Spacing::numbered(display);
        self
    }

    pub fn display_duplicate_lines(&mut self, display: bool) -> &mut Graph {
        self.with_duplicate_lines = display;
        self
    }

    pub fn set_layout_direction(&mut self, left_to_right: bool) -> &mut Graph {
        self.left_to_right = left_to_right;
        self.anim.left_to_right = left_to_right;
        self
    }

    pub fn init_nodes(&mut self) {
        let fg = self.fulcrum_group;
        let (above, below) = (self.max_above, self.max_below);
        let nums = self.with_numbers;
        self.anim.init_nodes(fg, above, below, nums);
    }

    pub fn place_nodes(&mut self) {
        self.anim.place_nodes();
    }

    pub fn create_group(
        &mut self,
        generation: i32,
        mini: bool,
        branch: Branch,
        before_fulcrum: bool,
    ) -> u32 {
        let group = Group::of(generation, mini, branch);
        let id = self.anim.alloc_group(group);
        if before_fulcrum {
            if let Some(pos) = self
                .anim
                .group_seq
                .iter()
                .position(|g| *g == self.fulcrum_group)
            {
                self.anim.group_seq.insert(pos, id);
            } else {
                self.anim.group_seq.push(id);
            }
        } else {
            self.anim.group_seq.push(id);
        }
        id
    }

    pub fn start_from(&mut self, fulcrum: u32) {
        self.walk(fulcrum);
    }
}
