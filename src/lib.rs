use std::{cmp::max_by, collections::{HashMap, VecDeque}, fmt::Error, rc::Rc};

use fa::Dfa;

mod fa;

pub fn test_even() {
    let states = vec![('p', true), ('q', false)];
    let alphabet = vec!['0', '1'];
    let transitions = vec![
        vec![('0', 'q'),('1', 'p')],
        vec![('0', 'p'),('1', 'q')]
    ];

    let dfa = Dfa::new(states, alphabet, transitions);

    let test_cases = ["", "0", "1", "00", "01", "10", "11", "000", "001", "010", "011", "100", "101", "110", "111"];

    for (i, input) in test_cases.iter().enumerate() {
        println!("test {} : {} returns {}", i, input, dfa.accepts(input.to_string()))
    }
}

pub fn make_keyword_dfa() -> Dfa {
    let states = vec![('0', false), ('v', false), ('a', false), ('r', true),('E', false)];
    let alphabet: Vec<char> = "var".to_string().chars().collect();
    let transitions = states.iter().enumerate().map(|(i,(_, _))|{
        alphabet.iter().map(|&symbol|{
            if i <=2 && states[i+1].0 == symbol {
                (symbol, symbol)
            } else {
                (symbol, 'E')
            }
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

const DIGITS: &str = "0123456789";
pub fn make_integer_dfa() -> Dfa {
    let states = vec![('0', false), ('1', true)];
    let alphabet: Vec<char>= DIGITS.to_string().chars().collect();
    let transitions = states.iter().map(|(_state, _)|{
        alphabet.iter().map(|&symbol|{
            (symbol, '1')
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
pub fn make_identifier_dfa() -> Dfa {
    let states = vec![('0', false), ('L', true), ('D', false)];
    let mut alphabet = LETTERS.to_string();
    alphabet.push_str(DIGITS);
    let alphabet: Vec<char> = alphabet.chars().collect();

    let transitions = states.iter().map(|(state, _)| {
        alphabet.iter().map(|&symbol| {
            if *state == '0' {
                if LETTERS.contains(symbol) {
                    (symbol, 'L')
                } else {
                    (symbol, 'D')
                }
            } else {
                (symbol, *state)
            }
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

pub fn make_operator_dfa() -> Dfa {
    let states = vec![('0',false), ('1', true), ('E', false)];
    let alphabet: Vec<char> = "+-*/=".to_string().chars().collect();
    let transitions = states.iter().map(|(state, _)|{
        alphabet.iter().map(|&symbol|{
            if *state == '0' {
                (symbol, '1')
            } else {
                (symbol, 'E')
            }
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

pub fn make_ignored_dfa() -> Dfa {
    let states = vec![('0',false), ('1', true), ('E', false)];
    let alphabet: Vec<char> = " ;\n\r\t\0".to_string().chars().collect();
    let transitions = states.iter().map(|(state, _)|{
        alphabet.iter().map(|&symbol|{
            if *state == '0' {
                (symbol, '1')
            } else {
                (symbol, 'E')
            }
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

pub fn lexical_scan(mut istream: VecDeque<u8>) -> Result<HashMap<String, Rc<String>>, Error> {
    let mut symbol_table = HashMap::<Vec<u8>,Rc<String>>::new();

    let machines: [(Rc<String>, Dfa); 5] = [
        (Rc::new("ignored".to_string()), make_ignored_dfa()),
        (Rc::new("identifier".to_string()), make_identifier_dfa()),
        (Rc::new("keyword".to_string()), make_keyword_dfa()),
        (Rc::new("operator".to_string()), make_operator_dfa()),
        (Rc::new("integer".to_string()), make_integer_dfa()),
    ];

    while !istream.is_empty() {
        let longest_accepted = machines.iter().map(|(class, machine)| {
            println!("running {class} dfa ");
            (machine.get_longest_accepted(&mut istream), class)
        }).max_by_key(|(name, _class)| {
            name.len()
        });

        let Some((name, class)) = longest_accepted
        else {return Result::Err(Error)};

        let name_len = name.len();

        println!("{class}");
        if !(class.contains("ignored") || symbol_table.contains_key(&name)){
            symbol_table.insert(name.clone(), class.clone());
            println!("{:?}", symbol_table)
        }

        for _ in 0..name_len {
            istream.pop_front();
        }
    }

    let symbol_table = HashMap::<String, Rc<String>>::from_iter(symbol_table.iter().map(|(name_bytes,class)|{
        let mut name = String::new();
        name_bytes.iter().for_each(|&x| {name.push(x as char)});
        (name, class.clone())   
    }
));

    Result::Ok(symbol_table)
}