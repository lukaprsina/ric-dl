use std::{error::Error, fs, path::PathBuf};

use headless_chrome::{Browser, LaunchOptions};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use url::Url;

fn main() -> Result<(), Box<dyn Error>> {
    let browser = Browser::new(LaunchOptions {
        // headless: false,
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;

    // let subject = String::from("racunalnistvo");
    let subject = inquire::Text::new("Katero maturo nalozim?")
        .with_default("racunalnistvo")
        .with_help_message("Napiši ime predmeta brez sumnikov")
        .prompt()?;
    let page_url = Url::parse(&format!(
        "https://www.ric.si/splosna-matura/predmeti/{}",
        subject
    ))?;

    let base_path = PathBuf::from(&format!("pole/{}", subject.to_lowercase()));
    fs::create_dir_all(base_path.clone()).unwrap();

    tab.navigate_to(&page_url.to_string())?;

    tab.wait_until_navigated()?;

    let list = tab.find_element("ul#exams-list")?;
    let exam_lists = list.find_elements("li.exam-item")?;

    exam_lists.into_par_iter().for_each(|exam_list| {
        let link_p = exam_list.find_element(".exam-text > p").unwrap();
        let links = link_p.find_elements("a.external").unwrap();

        // JESENSKI IZPITNI ROK 2012
        let name_elem = exam_list.find_element(".exam-name").unwrap();
        let name = name_elem.get_inner_text().unwrap();
        let name = name.rsplit_once(" ").unwrap();
        let year = name.1;
        let name = name.0;

        let exam_path = base_path.join(&format!("{year} {name}"));
        fs::create_dir_all(exam_path.clone()).unwrap();

        links.into_par_iter().for_each(|link| {
            let mut pdf_name = link.get_inner_text().unwrap();
            pdf_name += ".pdf";

            let attrs = link.get_attributes().unwrap().unwrap();
            let mut iter = attrs.into_iter();
            for attr in iter.by_ref() {
                if attr == "href" {
                    break;
                }
            }

            let url_str = iter.next().unwrap();
            let full_url = page_url.join(&url_str).unwrap();
            let mut pdf = reqwest::blocking::get(full_url).unwrap();
            let mut buf: Vec<u8> = vec![];
            pdf.copy_to(&mut buf).unwrap();
            fs::write(exam_path.join(pdf_name), buf).unwrap();
        });
    });

    println!("Koncano");
    Ok(())
}
