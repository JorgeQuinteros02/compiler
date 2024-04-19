use std::{clone, collections::VecDeque, env::args, fs::File, io::Read};

use compiler::{dfa::Dfa, lexical_scan, nfa::Nfa};



fn main() {
    
    let args:Vec<String> = args().collect();
    let args: Vec<&str> = args.iter().map(|x| x.as_str()).collect();

    match args[1] {
        "lex" => test_lexical_scanner(args[2]),
        "dfa" => test_dfa_from_regex(args[2], args[3]),
        "nfa" => test_nfa_union(),
        _ => println!("Incorrect argument. write 'lex <filename>' or 'dfa \"<regex>\" \"<alphabet>\"'")
    }
}

fn test_nfa_union() {
    let (regex1, alphabet1) = ("(a|b)*", "ab");
    let (regex2, alphabet2) = ("(cd)*", "cd");

    let nfa1 = Nfa::from_regex(regex1, alphabet1, 1);
    let nfa2 = Nfa::from_regex(regex2, alphabet2, 2);

    let nfa = Nfa::union(vec![&nfa1, &nfa2]);
    
    
    print!("{}\n\n", nfa1.to_string());
    print!("{}\n\n", nfa2.to_string());
    print!("{}\n\n", nfa.to_string());
    print!("{}\n\n",Dfa::from_nfa(&nfa1).to_string());
    print!("{}\n\n",Dfa::from_nfa(&nfa2).to_string());
    print!("{}\n\n",Dfa::from_nfa(&nfa).to_string());


}

fn test_lexical_scanner(arg:&str) {

    let mut file = match File::open(arg.clone()) {
        Ok(t) => t,
        Err(t) => panic!("{:?}", t)
    };

    let mut input = Vec::<u8>::new();
    let _ = file.read_to_end(&mut input);

    let input = VecDeque::from(input);

    println!("{:#?}", lexical_scan(input));
}

fn test_dfa_from_regex(arg1:&str, arg2:&str) {
    let regex = arg1;
    let alphabet = arg2;
    println!("Making NFA from regex {:#?} with alphabet {:#?}", regex, alphabet);
    let nfa = Nfa::from_regex(regex, alphabet, 2);
    println!("{}",nfa.to_string());
    println!("\n\nMaking DFA from NFA");
    
    let dfa = Dfa::from_nfa(&nfa);
    print!("{}",dfa.to_string());

    
    println!("\n\nEnter a word to run through the DFA or enter QUIT to exit");
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
