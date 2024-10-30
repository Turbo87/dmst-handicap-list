use crate::{pdf, Options};
use minijinja::{context, Environment, State};
use serde::Deserialize;
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct Record {
    model: String,
    handicap: f32,
    #[serde(rename(deserialize = "18"), deserialize_with = "deserialize_bool")]
    is_18m: bool,
    #[serde(rename(deserialize = "15"), deserialize_with = "deserialize_bool")]
    is_15m: bool,
    #[serde(rename(deserialize = "Std"), deserialize_with = "deserialize_bool")]
    is_standard: bool,
    #[serde(rename(deserialize = "Club"), deserialize_with = "deserialize_bool")]
    is_club: bool,
    #[serde(rename(deserialize = "Double"), deserialize_with = "deserialize_bool")]
    is_double_seater: bool,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let buf = <&str>::deserialize(deserializer)?;
    Ok(!buf.is_empty())
}

impl Record {
    fn with_handicap(&self, handicap: f32) -> Record {
        let mut r = self.clone();
        r.handicap = handicap;
        r
    }
}

#[derive(Debug, serde::Serialize)]
struct CompetitionClass {
    title: String,
    cutoff: f32,
    records: Vec<Record>,
}

impl CompetitionClass {
    fn from(
        title: impl Into<String>,
        reference: f32,
        cutoff: f32,
        all_records: &[Record],
        filter: impl Fn(&Record) -> bool,
    ) -> CompetitionClass {
        let title = title.into();
        let records = all_records
            .iter()
            .filter(|r| filter(*r) && r.handicap >= cutoff)
            .map(|r| r.with_handicap(r.handicap / reference))
            .collect::<Vec<_>>();

        CompetitionClass {
            title,
            cutoff,
            records,
        }
    }
}

pub fn generate_competition(opts: &Options) -> anyhow::Result<()> {
    let file = File::open("competition.csv")?;

    let handicaps = csv::Reader::from_reader(file)
        .deserialize()
        .collect::<Result<Vec<Record>, _>>()?;

    let m15 = CompetitionClass::from("15m Klasse", 114., 106., &handicaps, |r| r.is_15m);
    let standard =
        CompetitionClass::from("Standardklasse", 110., 102., &handicaps, |r| r.is_standard);

    let mut env = Environment::new();
    env.add_filter("format_handicap", format_handicap);

    let template = fs::read_to_string(opts.assets.join("competition.jinja"))?;
    let template = env.template_from_str(&template)?;
    let output = template.render(context! { handicaps, m15, standard })?;

    fs::create_dir_all(&opts.output)?;
    let file_path = opts.output.join("competition.html");
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
        opts.assets.join("daec-logo.svg"),
        opts.output.join("daec-logo.svg"),
    )?;

    let file_path = fs::canonicalize(file_path)?;
    let pdf_path = opts.output.join("competition.pdf");
    pdf::to_pdf(&file_path, &pdf_path)?;

    Ok(())
}

fn format_handicap(_state: &State, handicap: f32) -> String {
    format!("{handicap:.3}")
}
