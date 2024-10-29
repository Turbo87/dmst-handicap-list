use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::Browser;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use url::Url;

pub fn to_pdf(input: &Path, output: &Path) -> anyhow::Result<()> {
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
