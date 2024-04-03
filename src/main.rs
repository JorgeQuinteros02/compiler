use std::{collections::VecDeque, env::args, fs::File, io::Read};

use compiler::lexical_scan;



fn main() {
    let args:Vec<String> = args().collect();

    let mut file = match File::open(args[1].clone()) {
        Ok(t) => t,
        Err(t) => panic!("{:?}", t)
    };

    let mut input = Vec::<u8>::new();
    let _ = file.read_to_end(&mut input);

    let input = VecDeque::from(input);

    println!("{:?}", lexical_scan(input));

}
