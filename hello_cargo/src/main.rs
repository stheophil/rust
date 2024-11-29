// const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;

fn no_return_stmt(n: i32) -> i32 {
    n.wrapping_add(1) // last statement is return value
}

fn main() {
    {
        let constant : u8 =  255;
        let mut var = constant;
        var = var.wrapping_add(1);

        let c2 = no_return_stmt((2_i64.pow(31)-1).try_into().unwrap());
        println!("Hello, world {constant} {var} {c2}!");
    }

    // control flow
    { 
        // try_into needs lhs type specification so it knows what to cast to
        let o : Result<i32, _> = (2_i64.pow(31)-1).try_into();
        let reply = if o.is_ok() { "2^31 - 1 does fit into i32" } else { "2^31 - 1 does not fit into i32" };
        println!("{reply}");
    }

    {
        // loops can return values 
        let mut counter = 0;
        let result = loop {
            counter += 1;

            if counter == 10 {
                break counter * 2;
            }
        };
        println!("The result is {result}");
    }

    { // collatz conjecture
        let mut vec: Vec<i32> = Vec::new();
        let mut n = 27;
        loop {
            vec.push(n);
            if 1==n {
                break;
            } else if n % 2 == 0 {
                n = n/2;
            } else {
                n = 3*n + 1;
            }
        }
        println!("{:?}", vec);
    }

    { // loops with labels
        // let mut count = 0;
        // 'counting_up: loop {
        //     println!("count = {count}");
        //     let mut remaining = 10;

        //     loop {
        //         println!("remaining = {remaining}");
        //         if remaining == 9 {
        //             break;
        //         }
        //         if count == 2 {
        //             break 'counting_up;
        //         }
        //         remaining -= 1;
        //     }

        //     count += 1;
        // }
        // println!("End count = {count}");
    }
}

/* 
    value type is any type with the Copy Trait
    will be stack allocated and copyable
    must not have the drop trait at the same time, i.e., trivial destructor

    otherwise object is moved implicitly
    may add drop for destructor
    or clone for explicit copy

    function taking a reference arg:

    fn calculate_length(s: &String) -> usize {
        s.len()
    }

    otherwise argument would be moved in and out again

    called with  
        let len = calculate_length(&s1);

    mutable reference:

        fn change(some_string: &mut String) {
            some_string.push_str(", world");
        }

    can have only one at the same time.
*/
