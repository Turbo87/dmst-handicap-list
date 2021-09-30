use indoc::indoc;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let file = File::open("gliderlist.csv")?;

    let mut handicaps: HashMap<String, HashMap<u8, Vec<String>>> = HashMap::new();

    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.records() {
        let record = result?;
        let name = record.get(2).unwrap().to_string();
        let handicap = record.get(16).unwrap().parse::<u8>()?;
        let class = record.get(4).unwrap().to_string();

        let class_handicaps = handicaps.entry(class).or_insert_with(|| HashMap::new());
        let glider_list = class_handicaps.entry(handicap).or_insert_with(|| vec![]);
        glider_list.push(name);
    }

    let mut output = String::new();
    output += indoc! {r#"
        <!DOCTYPE html>
        <html lang="de">
        <head>
          <meta charset="UTF-8">
          <title>DMSt-Wettbewerbsordnung</title>
          <link href="https://fonts.googleapis.com/css?family=Domine&display=swap" rel="stylesheet">
          <link href="https://fonts.googleapis.com/css?family=Roboto:300,400&display=swap" rel="stylesheet">
          <link href="styles.css" rel="stylesheet">
        </head>
        <body>
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
        output += &format!("<h1>{}</h1>\n", label);
        output += "<table>\n";

        let handicaps = handicaps.get(key).unwrap();

        let mut keys: Vec<_> = handicaps.keys().collect();
        keys.sort_by(|a, b| b.cmp(a));

        for key in keys {
            let mut glider_list = handicaps.get(key).unwrap().clone();
            glider_list.sort();
            let glider_list = glider_list.join(r#" <span class="sep">|</span> "#);

            output += &format!(
                "  <tr>\n    <td>{}</td>\n    <td>{}</td>\n  </tr>\n",
                glider_list, key,
            );
        }

        output += "</table>\n";
    }

    output += indoc! {r#"
        </body>
        </html>
    "#};

    let output_path = PathBuf::from("output");
    fs::create_dir_all(&output_path)?;
    let file_path = output_path.join("handicaps.html");
    let mut file = File::create(file_path)?;
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

    Ok(())
}
