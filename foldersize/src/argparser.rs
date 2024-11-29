use std::env;

// Debug trait is used for automatic pretty-printing
// Struct is private in this file/module by default
#[derive(Debug)]
struct ArgOption {
    option: String,
    value: String
}

#[derive(Debug)]
pub struct ArgParser {
    // automatically private because ArgOption is private
    // private type is not leaked
    args: Vec<ArgOption>
}

impl ArgParser {
    // functions must be defined in impl block
    pub fn new() -> Self {
        let mut raw_args = env::args().skip(1).peekable(); // skip executable

        let mut args = Vec::<ArgOption>::new();
        while let Some(option) = raw_args.next() {
            if b'-'==option.as_bytes()[0] {
                args.push(ArgOption{
                    option: option.get(1..).unwrap().to_string(), 
                    value: "".to_string()
                });

                if let Some(next_option) = raw_args.peek() {
                    if b'-'!=next_option.as_bytes()[0] {
                        args.last_mut().unwrap().value = raw_args.next().unwrap();
                    }
                }
            } else {
                args.push(ArgOption{option: "".to_string(), value: option});
            }
        }

        ArgParser{args}
    }

    pub fn get_opt<T: std::str::FromStr>(&self, option: &str) -> Option<T> {
        self.args.iter().find(
            |argopt| argopt.option==option
        ).map(
            |argopt| argopt.value.parse::<T>().ok()
        ).flatten()
    }
}
