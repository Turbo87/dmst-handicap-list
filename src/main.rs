use anyhow::Context;
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::Browser;
use minijinja::{context, Environment};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, serde::Serialize)]
struct PlaneType {
    name: String,
    highlight: bool,
}

struct Generator {
    input: PathBuf,
    assets: PathBuf,
    output: PathBuf,
}

impl Generator {
    fn generate(&self) -> anyhow::Result<()> {
        let file = File::open(&self.input)?;

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
        let template = fs::read_to_string(self.assets.join("dmst.jinja"))?;
        let template = env.template_from_str(&template)?;
        let output = template.render(context! { categories, handicaps })?;

        fs::create_dir_all(&self.output)?;
        let file_path = self.output.join("handicaps.html");
        let mut file = File::create(&file_path)?;
        file.write_all(output.as_bytes())?;

        fs::copy(
            self.assets.join("normalize.css"),
            self.output.join("normalize.css"),
        )?;
        fs::copy(
            self.assets.join("styles.css"),
            self.output.join("styles.css"),
        )?;
        fs::copy(
            self.assets.join("dmst-logo.svg"),
            self.output.join("dmst-logo.svg"),
        )?;

        let file_path = fs::canonicalize(file_path)?;
        let pdf_path = self.output.join("handicaps.pdf");
        to_pdf(&file_path, &pdf_path)?;

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let generator = Generator {
        input: "gliderlist.csv".into(),
        assets: "assets".into(),
        output: "output".into(),
    };

    generator.generate()
}

fn to_pdf(input: &Path, output: &Path) -> anyhow::Result<()> {
    let file_url = Url::from_file_path(input)
        .map_err(|_| anyhow::anyhow!("Failed to convert file path to URL"))?
        .to_string();

    let browser = Browser::default()?;

    let tab = browser.new_tab()?;
    tab.navigate_to(&file_url)?;
    tab.wait_until_navigated()?;

    let options = PrintToPdfOptions {
        print_background: Some(true),
        prefer_css_page_size: Some(true),
        ..Default::default()
    };
    let pdf_bytes = tab.print_to_pdf(Some(options))?;
    let mut pdf_file = File::create(output)?;
    pdf_file.write_all(pdf_bytes.as_slice())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_main() {
        let tempdir = tempfile::tempdir().unwrap();

        let generator = Generator {
            input: "gliderlist.csv".into(),
            assets: "assets".into(),
            output: tempdir.path().into(),
        };

        generator.generate().unwrap();

        let html = fs::read_to_string(tempdir.path().join("handicaps.html")).unwrap();
        assert_snapshot!(html);
    }
}
