use gedcomgraph::render::font_draw::Fonts;
use gedcomgraph::render::measure::measure_all;
use gedcomgraph::render::svg::render_svg;
use gedcomgraph::render::theme::Scale;
use gedcomgraph::{GedcomData, Graph, parse_gedcom};
use std::fs::read_to_string;
use std::process::ExitCode;

#[derive(Debug)]
enum AppError {
    Read(String),
    Empty(String),
    Fonts,
    Write(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Read(path) => write!(f, "cannot read {path}"),
            AppError::Empty(path) => write!(f, "no people in {path}"),
            AppError::Fonts => write!(f, "no system fonts found"),
            AppError::Write(path) => write!(f, "cannot write {path}"),
        }
    }
}

impl std::error::Error for AppError {}

fn load(path: &str) -> Result<GedcomData, AppError> {
    match read_to_string(path) {
        Ok(text) => Ok(parse_gedcom(&text)),
        Err(_) => Err(AppError::Read(path.to_string())),
    }
}

fn pick_fulcrum(ged: &GedcomData, wanted: Option<&str>) -> u32 {
    if let Some(id) = wanted {
        if let Some(i) = ged.find_person(id) {
            return i.0;
        }
        eprintln!("person {id} not found, using default");
    }
    let mut fallback = Option::None;
    for (i, p) in ged.persons.iter().enumerate() {
        let idx = u32::try_from(i).unwrap_or(u32::MAX);
        if fallback.is_none() {
            fallback = Some(idx);
        }
        if !p.parent_fams.is_empty() && !p.spouse_fams.is_empty() {
            return idx;
        }
    }
    fallback.unwrap_or(0)
}

fn print_tree(graph: &Graph, ged: &GedcomData) {
    println!("width {:.1} height {:.1}", graph.width(), graph.height());
    println!(
        "nodes {} persons {} bonds {}",
        graph.anim.node_count(),
        graph.anim.person_count(),
        graph.anim.bond_views().len()
    );
    println!(
        "lines {} back {} duplicates {}",
        graph.line_count(),
        graph.back_line_count(),
        graph.duplicate_line_count()
    );
    let mut order: Vec<(i32, f32, String)> = Vec::new();
    for n in graph.anim.nodes.clone() {
        let generation = graph.node_generation(n);
        let x = graph.node_x(n);
        order.push((generation, x, graph.node_label(n)));
    }
    order.sort_by(|a, b| {
        debug_assert!(a.1.is_finite() && b.1.is_finite());
        a.0.cmp(&b.0)
            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    });
    for (generation, x, label) in order {
        println!("generation {generation:>3} x {x:>8.1}  {label}");
    }
    let _ = ged;
}

fn render_file(path: &str, out: &str, scale: f32, person: Option<&str>) -> Result<(), AppError> {
    let ged = load(path)?;
    if ged.person_count() == 0 {
        return Err(AppError::Empty(path.to_string()));
    }
    let fonts = Fonts::load();
    if fonts.regular.is_none() {
        return Err(AppError::Fonts);
    }
    let sc = Scale::of(scale);
    let fulcrum = pick_fulcrum(&ged, person);
    let mut graph = Graph::with_gedcom(ged.clone());
    graph
        .max_ancestors(10)
        .max_great_uncles(10)
        .max_descendants(10)
        .max_siblings_nephews(10)
        .max_uncles_cousins(10)
        .display_spouses(true)
        .display_numbers(true)
        .display_duplicate_lines(true)
        .set_layout_direction(true);
    graph.start_from(fulcrum);
    measure_all(&mut graph, &fonts, &sc);
    graph.init_nodes();
    if graph.need_max_bitmap_size() {
        graph.set_max_bitmap_size(1000.0);
    }
    graph.place_nodes();
    let img_w = (graph.width() * sc.s).round() as i32 + sc.pad_px() * 2;
    let img_h = (graph.height() * sc.s).round() as i32 + sc.pad_px() * 2;
    let svg = render_svg(
        &graph,
        &fonts,
        &sc,
        sc.pad_px() as f32,
        sc.pad_px() as f32,
        img_w,
        img_h,
    );
    if std::fs::write(out, svg.as_bytes()).is_err() {
        return Err(AppError::Write(out.to_string()));
    }
    println!("Saved: {out} ({img_w}x{img_h}) scale={scale}");
    Ok(())
}

fn run() -> Result<(), AppError> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("example.ged");
    if path.ends_with(".svg") || path.ends_with(".png") {
        eprintln!("input must be a .ged file");
        return Ok(());
    }
    if let Some(second) = args.get(2) {
        if second.ends_with(".svg") {
            let scale = args
                .get(3)
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(4.0);
            return render_file(path, second, scale, args.get(4).map(|s| s.as_str()));
        }
        if second.ends_with(".png") {
            eprintln!("png output removed, use .svg");
            return Ok(());
        }
    }
    let ged = load(path)?;
    if ged.person_count() == 0 {
        return Err(AppError::Empty(path.to_string()));
    }
    let fulcrum = pick_fulcrum(&ged, args.get(2).map(|s| s.as_str()));
    let mut graph = Graph::with_gedcom(ged.clone());
    graph.set_max_bitmap_size(2000.0);
    graph.start_from(fulcrum);
    graph.init_nodes();
    graph.place_nodes();
    println!("file {path}");
    println!(
        "fulcrum {}",
        ged.person(fulcrum).map(|p| p.id.as_str()).unwrap_or("?")
    );
    print_tree(&graph, &ged);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
