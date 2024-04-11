use std::{clone, collections::VecDeque, env::args, fs::File, io::Read};

use compiler::{dfa::Dfa, lexical_scan, nfa::Nfa};



fn main() {
    test_lexical_scanner()
}

fn test_lexical_scanner() {
    let args:Vec<String> = args().collect();

    let mut file = match File::open(args[1].clone()) {
        Ok(t) => t,
        Err(t) => panic!("{:?}", t)
    };

    let mut input = Vec::<u8>::new();
    let _ = file.read_to_end(&mut input);

    let input = VecDeque::from(input);

    println!("{:#?}", lexical_scan(input));
}

fn test_dfa_from_regex() {
    let args: Vec<String> = args().collect();
    let regex = args[1].as_str();
    let alphabet = args[2].as_str();
    let nfa = Nfa::from_regex(regex, alphabet);
    let dfa = Dfa::from_nfa(&nfa);
    println!("Making DFA from regex {:#?} with alphabet {:#?}", regex, alphabet);

    
    println!("Enter a word to run through the DFA or enter QUIT to exit");
    loop {
        let mut word = String::new();
        let result = std::io::stdin().read_line(&mut word);
        if let Ok(_) = result {
            word.truncate(word.len() - 2);
            
            if word == "QUIT" {
                break;
            }
            println!("{}", dfa.accepts(word.clone()));
        } else {
            println!("Error");
        }
    }
}
