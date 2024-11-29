use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use core::ops::Add;
use core::ops::AddAssign;

use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
struct Point {
    x: i32,
    y: i32
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

fn main() -> std::io::Result<()>  {    
    let f = File::open("src/input.txt")?;

    let mut reader = BufReader::new(f);
    let mut buffer = String::new();

    let mut positions = HashSet::new();

    let mut pth = Point{x: 0, y: 0};
    let mut ptt = Point{x: 0, y: 0};

    positions.insert(ptt);

    while 0!=reader.read_line(&mut buffer)? {
        let m: u32 = buffer[2..].trim().parse().unwrap();
        let diff = match &buffer[0..1] {
            "L" => Point{x: -1, y: 0},
            "R" => Point{x: 1, y: 0},
            "U" => Point{x: 0, y: 1},
            "D" => Point{x: 0, y: -1},
            &_ => panic!()
        };
        println!("{buffer}");
        buffer.clear();

        for _i in 0..m {
            pth += diff;

            if 1<(pth.x-ptt.x).abs() || 1<(pth.y-ptt.y).abs() {
                if pth.x==ptt.x  {
                    ptt.y += diff.y;
                } else if pth.y==ptt.y {
                    ptt.x += diff.x;
                } else {
                    ptt.x += (pth.x - ptt.x).signum();
                    ptt.y += (pth.y - ptt.y).signum();
                }
                positions.insert(ptt);
            }
            println!("H = {:?} T = {:?}", pth, ptt);
        }
    }

    println!("Positions {}", positions.len());

    Ok(())
}
