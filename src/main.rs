use anyhow::Context;
use headless_chrome::protocol::page::PrintToPdfOptions;
use headless_chrome::Browser;
use indoc::indoc;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use url::Url;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone)]
struct PlaneType {
    name: String,
    highlight: bool,
}

fn main() -> anyhow::Result<()> {
    let file = File::open("gliderlist.csv")?;

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
            <title>DMSt Indexliste 2023</title>
            <link href="https://fonts.googleapis.com/css?family=Domine&display=swap" rel="stylesheet">
            <link href="https://fonts.googleapis.com/css?family=Roboto:300,400&display=swap" rel="stylesheet">
            <link href="styles.css" rel="stylesheet">
        </head>
        <body>
        <h1 class="header">
            <img src="logo.jpg" class="logo">
            DMSt Indexliste 2023
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

    let output_path = PathBuf::from("output");
    fs::create_dir_all(&output_path)?;
    let file_path = output_path.join("handicaps.html");
    let mut file = File::create(&file_path)?;
    file.write_all(output.as_bytes())?;

    let assets_path = PathBuf::from("assets");
    fs::copy(
        assets_path.join("normalize.css"),
        output_path.join("normalize.css"),
    )?;
    fs::copy(
        assets_path.join("styles.css"),
        output_path.join("styles.css"),
    )?;
    fs::copy(assets_path.join("logo.jpg"), output_path.join("logo.jpg"))?;

    let browser = Browser::default().unwrap();
    let tab = browser.wait_for_initial_tab().unwrap();

    let file_path = fs::canonicalize(file_path)?;
    let file_url = Url::from_file_path(&file_path).unwrap().to_string();
    tab.navigate_to(&file_url).unwrap();
    tab.wait_until_navigated().unwrap();

    let options = PrintToPdfOptions {
        landscape: None,
        display_header_footer: None,
        print_background: Some(true),
        scale: None,
        paper_width: None,
        paper_height: None,
        margin_top: None,
        margin_bottom: None,
        margin_left: None,
        margin_right: None,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(true),
    };
    let pdf_bytes = tab.print_to_pdf(Some(options)).unwrap();
    let pdf_path = output_path.join("handicaps.pdf");
    let mut pdf_file = File::create(&pdf_path)?;
    pdf_file.write_all(pdf_bytes.as_slice())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        super::main().unwrap();
    }
}
