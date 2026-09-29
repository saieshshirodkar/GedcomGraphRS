use gedcomgraph::model::{Fact, FamilyId, GFamily, GPerson, Name, PersonId};
use gedcomgraph::{GedcomData, Graph};

pub struct Builder {
    ged: GedcomData,
}

impl Builder {
    pub fn create() -> Builder {
        Builder {
            ged: GedcomData::empty(),
        }
    }

    pub fn person(&mut self, id: &str, name: &str, sex: Option<&str>, dead: bool) -> PersonId {
        let mut facts: Vec<Fact> = Vec::new();
        if let Some(s) = sex {
            facts.push(Fact::of("SEX", Some(s), Option::None));
        }
        if dead {
            facts.push(Fact::dated("DEAT", Option::None));
        }
        self.ged.push_person(GPerson {
            id: id.to_string(),
            names: vec![Name::of(name)],
            facts,
            parent_fams: Vec::new(),
            spouse_fams: Vec::new(),
        })
    }

    pub fn family(
        &mut self,
        id: &str,
        husb: Option<PersonId>,
        wife: Option<PersonId>,
        kids: &[PersonId],
        marr: Option<&str>,
    ) -> FamilyId {
        let mut facts: Vec<Fact> = Vec::new();
        if let Some(d) = marr {
            facts.push(Fact::dated("MARR", Some(d)));
        }
        let f = self.ged.push_family(GFamily {
            id: id.to_string(),
            husbands: Vec::new(),
            wives: Vec::new(),
            children: Vec::new(),
            facts,
        });
        let mut hs: Vec<PersonId> = Vec::new();
        let mut ws: Vec<PersonId> = Vec::new();
        if let Some(h) = husb {
            hs.push(h);
        }
        if let Some(w) = wife {
            ws.push(w);
        }
        self.ged.link_spouses(f, &hs, &ws);
        self.ged.link_children(f, kids);
        f
    }

    pub fn finish(mut self) -> GedcomData {
        self.ged.reindex();
        self.ged
    }
}

pub fn run(graph: &mut Graph, fulcrum: PersonId) {
    graph.start_from(fulcrum.0);
    graph.init_nodes();
    graph.place_nodes();
}

pub fn run_sized(graph: &mut Graph, fulcrum: PersonId, w: f32, h: f32) {
    graph.start_from(fulcrum.0);
    graph.set_all_person_sizes(w, h);
    graph.init_nodes();
    graph.place_nodes();
}

pub fn finite(x: f32) -> bool {
    x.is_finite()
}

pub fn node_finite(graph: &Graph, n: gedcomgraph::core::NodeId) -> bool {
    finite(graph.node_x(n)) && finite(graph.node_y(n))
}

pub fn all_finite(graph: &Graph) -> bool {
    graph
        .anim
        .nodes
        .clone()
        .into_iter()
        .all(|n| node_finite(graph, n))
}

pub fn person_generation(graph: &Graph, pid: u32) -> i32 {
    graph
        .anim
        .persons
        .get(pid as usize)
        .map(|p| p.base.generation)
        .unwrap_or(999)
}

pub fn gedcom_of(graph: &Graph, pid: u32) -> PersonId {
    graph
        .anim
        .persons
        .get(pid as usize)
        .map(|p| PersonId(p.person))
        .unwrap_or(PersonId(u32::MAX))
}
