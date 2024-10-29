use std::path::PathBuf;

mod competition;
mod dmst;
mod pdf;

#[derive(Debug, clap::Parser)]
struct Options {
    #[arg(long, default_value = "gliderlist.csv")]
    input: PathBuf,
    #[arg(long, default_value = "assets")]
    assets: PathBuf,
    #[arg(long, default_value = "output")]
    output: PathBuf,
    #[arg(long)]
    skip_competition: bool,
    #[arg(long)]
    skip_dmst: bool,
}

fn main() -> anyhow::Result<()> {
    let opts: Options = clap::Parser::parse();
    if !opts.skip_dmst {
        dmst::generate_dmst(&opts)?;
    }
    if !opts.skip_competition {
        competition::generate_competition(&opts)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::competition::generate_competition;
    use crate::dmst::generate_dmst;
    use insta::assert_snapshot;
    use std::fs::read_to_string;

    #[test]
    fn test_dmst() {
        let tempdir = tempfile::tempdir().unwrap();

        let opts = Options {
            input: "gliderlist.csv".into(),
            assets: "assets".into(),
            output: tempdir.path().into(),
            skip_competition: true,
            skip_dmst: false,
        };

        generate_dmst(&opts).unwrap();

        let html = read_to_string(tempdir.path().join("dmst.html")).unwrap();
        assert_snapshot!(html);
    }

    #[test]
    fn test_competition() {
        let tempdir = tempfile::tempdir().unwrap();

        let opts = Options {
            input: "gliderlist.csv".into(),
            assets: "assets".into(),
            output: tempdir.path().into(),
            skip_competition: false,
            skip_dmst: true,
        };

        generate_competition(&opts).unwrap();

        let html = read_to_string(tempdir.path().join("competition.html")).unwrap();
        assert_snapshot!(html);
    }
}
