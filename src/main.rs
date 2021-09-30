use std::collections::HashMap;
use std::fs::File;

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

    println!("{:#?}", handicaps);

    Ok(())
}
