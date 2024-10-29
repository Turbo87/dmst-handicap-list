use anyhow::Context;
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::Browser;
use indoc::indoc;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
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

        let mut output = String::new();
        output += indoc! {r#"
            <!DOCTYPE html>
            <html lang="de">
            <head>
                <meta charset="UTF-8">
                <title>DAeC Indexliste: DMSt 2023</title>
                <link href="https://fonts.googleapis.com/css?family=Domine&display=swap" rel="stylesheet">
                <link href="https://fonts.googleapis.com/css?family=Roboto:300,400&display=swap" rel="stylesheet">
                <link href="styles.css" rel="stylesheet">
            </head>
            <body>
            <h1 class="header">
                <img src="logo.jpg" class="logo">
                DAeC Indexliste: DMSt 2023
            </h1>
            <table>
              <thead><tr><td>
                <div class="header-space">&nbsp;</div>
              </td></tr></thead>
              <tbody><tr><td>
                <div class="content">
        "#};

        let categories = vec![
            ("Open", "Offene Klasse"),
            ("18", "18m Klasse"),
            ("15", "15m Klasse"),
            ("Standard", "Standardklasse"),
            ("Club", "Clubklasse"),
            ("Double", "Doppelsitzer"),
        ];

        for (key, label) in categories {
            output += &format!("<h2>{}</h2>\n", label);
            output += "<table>\n";

            let handicaps = handicaps.get(key).unwrap();

            let mut keys: Vec<_> = handicaps.keys().collect();
            keys.sort_by(|a, b| b.cmp(a));

            for key in keys {
                let mut glider_list = handicaps.get(key).unwrap().clone();
                glider_list.sort();

                let glider_list = glider_list
                    .into_iter()
                    .map(|plane_type| {
                        let highlight = if plane_type.highlight {
                            " highlight"
                        } else {
                            ""
                        };

                        format!(
                            r#"<span class="glider-type{}">{}</span>"#,
                            highlight, plane_type.name
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(r#"<span class="sep">|</span></span> <span class="glider-type">"#);

                output += &format!(
                    "  <tr>\n    <td>{}</td>\n    <td>{}</td>\n  </tr>\n",
                    glider_list, key,
                );
            }

            output += "</table>\n";
        }

        output += indoc! {r#"
                </div>
              </td></tr></tbody>
              <tfoot><tr><td>
                <div class="footer-space">&nbsp;</div>
              </td></tr></tfoot>
            </table>
            </body>
            </html>
        "#};

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
        fs::copy(self.assets.join("logo.jpg"), self.output.join("logo.jpg"))?;

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
