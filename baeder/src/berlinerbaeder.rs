use regex::Regex;
use chrono::DateTime;
use chrono::Local;

#[derive(Debug)]
struct DateTimeInterval {
    begin: chrono::DateTime<chrono::Local>,
    end: chrono::DateTime<chrono::Local>
}

#[derive(Debug)]
struct  Interval {
    begin: i64,
    end: i64
}

#[derive(Debug)]
struct OpeningTime {
    time: Interval,
    description: String
}

#[derive(Debug)]
pub struct Opening {
    caption: String,
    dates: DateTimeInterval,
    times: [Vec<OpeningTime>; 7]
}

#[derive(Debug)]
pub struct Pool {
    pub name: String,
    pub url: String
}

#[derive(Debug)]
pub struct BerlinerBaeder {
    pub pools: Vec<Pool>
}

impl BerlinerBaeder {
    // functions must be defined in impl block
    pub fn new() -> Self {
        let mut pools: Vec<Pool> = Vec::new();
    
        let mut next_url = "https://www.berlinerbaeder.de/baeder/".to_string(); 
        while !next_url.is_empty() {
            next_url = Self::scrape_pools_from_page(next_url, &mut pools);
        }
    
        pools.sort_by(|lhs, rhs| { lhs.name.cmp(&rhs.name) });

        BerlinerBaeder{pools}
    }

    fn scrape_pools_from_page(url: String, vec: &mut Vec<Pool>) -> String {
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
                    vec.push(Pool{name: name.trim().to_string(), url: url.trim().to_string()});
                }
            );

        match document.select(&page_selector).next() {
            None => "".to_string(),
            Some(e) => "https://www.berlinerbaeder.de".to_string() + e.value().attr("href").unwrap()
        }
    }
}

impl Pool {
    pub fn matches(&self, name: &str) -> bool {
        self.name.to_lowercase().contains(&name.to_lowercase())
    }

    pub fn scrape_times(&self) {
        let response = reqwest::blocking::get("https://www.berlinerbaeder.de".to_string() + &self.url)
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);

        // <table class="openingtime">
        let openingtime_selector = scraper::Selector::parse("table.openingtime").unwrap();

        // <caption><span class="d-none d-print-inline">Hallenbad</span> Öffnungszeiten 23.09.24 - 31.07.25
        // </caption>
        let caption_selector = scraper::Selector::parse("caption").unwrap();
        let regex_caption = Regex::new(r"([^\d]+)\s*(\d\d.\d\d.\d\d)[\n\s]+-[\n\s]+(\d\d.\d\d.\d\d)").unwrap();

        // <thead>
        //     <tr>
        //         <th scope="col" class="day">Tag</th>
        //         <th scope="col" class="times">Uhrzeit</th>
        //         <th scope="col" class="art">Art</th>
        //     </tr>
        // </thead>
        // <tbody>
        //     <tr class="even">
        //         <th rowspan="2" scope="rowgroup" class="day">
        //             Montag
        //         </th>
        //         <td title="öffentl. Schwimmen" class="time even time_0"><span class="mobileday">Montag</span>
        //             06:30  -  08:00 <span class="timelabel">Uhr</span></td>
        //         <td class="even time_0"><span class="mobileday"></span>öffentl. Schwimmen</td>
        //     </tr>        
        let tr_selector = scraper::Selector::parse("tbody > tr").unwrap();
        let regex_times = Regex::new(r"(\d\d:\d\d)[\n\s]+-[\n\s]+(\d\d:\d\d)").unwrap();

        let regex_open = Regex::new(r"öffentl").unwrap();

        document.select(&openingtime_selector).for_each(
            |table| {
                let concat_text = |element : scraper::ElementRef| {
                    let mut s = String::new();
                    for t in element.text() {
                        s += t.trim();
                        s += " ";
                    }
                    s
                };

                let caption = concat_text(table.select(&caption_selector).next().unwrap());
                let match_caption = regex_caption.captures(&caption).unwrap();
               
                println!("{}: {} - {}", 
                    match_caption.get(1).unwrap().as_str(), // Pool name
                    match_caption.get(2).unwrap().as_str(), // Starting date
                    match_caption.get(3).unwrap().as_str()  // End date
                );
                
                let mut weekday = String::new();
                table.select(&tr_selector).for_each(
                    |tr| {
                        let mut children = tr.children();
                        let mut next_tr_child_elem = || {
                            scraper::ElementRef::wrap(children.next().unwrap()).unwrap()
                        };

                        let mut cell  = next_tr_child_elem();
                        if "th" == cell.value().name() {
                            weekday = cell.inner_html().trim().to_string();
                            cell = next_tr_child_elem();
                        } 

                        let hours = cell.inner_html();
                        let reason = concat_text(next_tr_child_elem());
                        
                        if let Some(match_times) = regex_times.captures(&hours) {
                            if regex_open.is_match(&reason) {
                                println!("\t{} -- {}-{} -- {}", weekday, match_times.get(1).unwrap().as_str(), match_times.get(2).unwrap().as_str(), reason);
                            }
                        }
                    }
                );
                println!("");
            }
        );
    }
}