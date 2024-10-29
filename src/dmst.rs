use crate::{pdf, Options};
use anyhow::Context;
use minijinja::{context, Environment};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, serde::Serialize)]
struct PlaneType {
    name: String,
    highlight: bool,
}

pub fn generate_dmst(opts: &Options) -> anyhow::Result<()> {
    let file = File::open(&opts.input)?;

    let mut handicaps: HashMap<String, HashMap<u8, Vec<PlaneType>>> = HashMap::new();

    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        let id = record
            .get(0)
            .unwrap()
            .parse::<u32>()
            .context("Failed to parse id")?;
        let name = record.get(2).unwrap().to_string();
        let old_handicap = record.get(16).unwrap().parse::<u8>()?;
        let handicap = record.get(17).unwrap().parse::<u8>()?;
        let class = record.get(4).unwrap().to_string();

        let highlight = id > 593 || handicap != old_handicap;

        let plane_type = PlaneType { name, highlight };

        let class_handicaps = handicaps.entry(class).or_default();
        let glider_list = class_handicaps.entry(handicap).or_default();
        glider_list.push(plane_type);
    }

    let categories = vec![
        ("Open", "Offene Klasse"),
        ("18", "18m Klasse"),
        ("15", "15m Klasse"),
        ("Standard", "Standardklasse"),
        ("Club", "Clubklasse"),
        ("Double", "Doppelsitzer"),
    ];

    let env = Environment::new();
    let template = fs::read_to_string(opts.assets.join("dmst.jinja"))?;
    let template = env.template_from_str(&template)?;
    let output = template.render(context! { categories, handicaps })?;

    fs::create_dir_all(&opts.output)?;
    let file_path = opts.output.join("dmst.html");
    let mut file = File::create(&file_path)?;
    file.write_all(output.as_bytes())?;

    fs::copy(
        opts.assets.join("normalize.css"),
        opts.output.join("normalize.css"),
    )?;
    fs::copy(
        opts.assets.join("styles.css"),
        opts.output.join("styles.css"),
    )?;
    fs::copy(
        opts.assets.join("dmst-logo.svg"),
        opts.output.join("dmst-logo.svg"),
    )?;

    let file_path = fs::canonicalize(file_path)?;
    let pdf_path = opts.output.join("dmst.pdf");
    pdf::to_pdf(&file_path, &pdf_path)?;

    Ok(())
}
