use regex::Regex;
use chrono::DateTime;
use chrono::Local;
use std::env;

struct DateTimeInterval {
    begin: chrono::DateTime<chrono::Local>,
    end: chrono::DateTime<chrono::Local>
}

struct Interval {
    begin: i64,
    end: i64
}

pub enum Open {
    fullyopen,
    limited
}

struct OpeningTime {
    time: Interval,
    open: Open
}

struct Opening {
    intvl: DateTimeInterval,
    times: [Vec<OpeningTime>; 7]
}

struct Pool {
    name: String,
    url: String,

    openings: Vec<Opening>
}

fn read_all(url: String, vec: &mut Vec<Pool>) -> String {
    let title_selector = scraper::Selector::parse(".bathlist_item_contents_text>h2>a").unwrap();
    let page_selector = scraper::Selector::parse(".pagination>li>a.next").unwrap();

    let response = reqwest::blocking::get(url)
            .unwrap()
            .text()
            .unwrap();

    let document = scraper::Html::parse_document(&response);

    document
        .select(&title_selector)
        .map(|x| (x.inner_html(), x.value().attr("href").unwrap()))
        .for_each(
            |(name, url)| {
                vec.push(Pool{name: name.trim().to_string(), url: url.trim().to_string(), openings: Vec::new()});
            }
        );

    match document.select(&page_selector).next() {
        None => "".to_string(),
        Some(e) => "https://www.berlinerbaeder.de".to_string() + e.value().attr("href").unwrap()
    }
}

fn main() {
    let mut vec: Vec<Pool> = Vec::new();
    let args: Vec<String> = env::args().collect();

    let mut next_url = "https://www.berlinerbaeder.de/baeder/".to_string(); 
    while !next_url.is_empty() {
        next_url = read_all(next_url, &mut vec);
    }

    vec.sort_by(|lhs, rhs| { lhs.name.cmp(&rhs.name) });
    println!("{} Baeder ausgelesen", vec.len());

    let openingtime_selector = scraper::Selector::parse("table.openingtime").unwrap();
    let caption_selector = scraper::Selector::parse("caption").unwrap();
    let tr_selector = scraper::Selector::parse("tbody > tr").unwrap();
    let regex = Regex::new(r"(\d\d:\d\d)[\n\s]+-[\n\s]+(\d\d:\d\d)").unwrap();

    for b in &vec {
        if args.iter().any(|s| b.name.to_lowercase().contains(&s.to_lowercase())) {

            println!("{} -> {}", b.name, b.url);
        
            let response = reqwest::blocking::get("https://www.berlinerbaeder.de".to_string() + &b.url)
                .unwrap()
                .text()
                .unwrap();

            let document = scraper::Html::parse_document(&response);

            document.select(&openingtime_selector).for_each(
                |e| {
                    let fn_get_text = |element : scraper::ElementRef| {
                        let mut s = String::new();
                        for t in element.text() {
                            s += t.trim();
                            s += " ";
                        }
                        s
                    };

                    let caption = fn_get_text(e.select(&caption_selector).next().unwrap());
                    println!("{}", caption);
                    
                    let mut weekday = String::new();
                    e.select(&tr_selector).for_each(
                        |tr| {
                            let mut children = tr.children();
                            let mut cell = children.next().unwrap();
                            
                            let first_cell = scraper::ElementRef::wrap(cell).unwrap();
                            if "th" == first_cell.value().name() {
                                weekday = first_cell.inner_html().trim().to_string();
                                cell = children.next().unwrap();
                            }

                            let hours = scraper::ElementRef::wrap(cell).unwrap().inner_html();
                            if !hours.contains("Geschlossen") {
                                let m = regex.captures(&hours).unwrap();

                                cell = children.next().unwrap();

                                let mut reason = String::new();
                                for text in scraper::ElementRef::wrap(cell).unwrap().text() {
                                    reason += text.trim();
                                }

                                println!("\t{} -- {}-{} -- {}", weekday, m.get(1).unwrap().as_str(), m.get(2).unwrap().as_str(), reason);
                            }
                        }
                    );
                    println!("");
                }
            );
        }
    }
}