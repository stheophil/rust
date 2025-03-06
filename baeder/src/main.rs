use std::env;
mod berlinerbaeder;
use crate::berlinerbaeder::BerlinerBaeder;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut baeder = BerlinerBaeder::new();
    
    println!("{} Baeder ausgelesen", baeder.pools.len());

    for p in &mut baeder.pools {
        if args.iter().any(|s| p.matches(s)) {

            println!("{} -> {}", p.name, p.url);
            p.scrape_times();
        }
    }
}