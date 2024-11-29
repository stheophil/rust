mod argparser;
use crate::argparser::ArgParser;

fn main() {
    // read command line argument depth
    // optional arg: directory, else cwd
    let argparser = ArgParser::new();
    println!("{:?}", argparser);

    println!("Option {:?}", argparser.get_opt::<i64>("d"));

    // create vec of folder(path, size)

    // depth-first traversal of directory
    // for each folder:
    //      if depth not reached
    //          push traversed folder
    //      else 
    //          accumulate size of files into 
    //          top of stack

    // vec has structure
    //  top/
    //  top/first/
    //  top/first/sub
    //  top/two/
    //  top/two/sub

    // go backwards over vec, accumulating size into n-ary vector
    //  accumulate third level until 2nd level reached, 
    //  clear third level, assign 2nd level, accumulate 1st level
    println!("Hello, world!");
}
