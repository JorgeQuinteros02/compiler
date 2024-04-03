use std::{collections::{HashMap, VecDeque}, fmt::Error, rc::Rc};

use fa::Dfa;

mod fa;

/* Generate a DFA that accepts keywords. Currently this includes only the keyword "var" */
pub fn make_keyword_dfa() -> Dfa {
    let states = vec![('0', false), ('v', false), ('a', false), ('r', true),('E', false)];
    let alphabet: Vec<char> = "var".to_string().chars().collect();
    
    // Each state will go to the next state if they read the symbol that represents that state . 0 -> v -> a -> r
    let transitions = states.iter().enumerate().map(|(i,(_, _))|{
        alphabet.iter().map(|&symbol|{
            if i <=2 && states[i+1].0 == symbol {
                (symbol, symbol)
            } else {
                // Any symbol other than the next symbol in the word automatically makes us reject the string.
                (symbol, 'E')
            }
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

const DIGITS: &str = "0123456789";
/* Generate a DFA that accepts integer constants. This will accept any string of digits (including any zeros on the left).*/
pub fn make_integer_dfa() -> Dfa {
    // State 1 represents having read a positive amount of integers and only having read integers
    let states = vec![('0', false), ('1', true)];
    
    // By our definition of DFA, something that isn't in the alphabet will be rejected.
    // This implies that all unicode scalars are implicitly in the actual alphabet of this DFA
    let alphabet: Vec<char>= DIGITS.to_string().chars().collect();
    let transitions = states.iter().map(|(_state, _)|{
        alphabet.iter().map(|&symbol|{
            // reading a digit will take to state 1
            // reading a non-digit will reject
            (symbol, '1')
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
/* Generate a DFA that accepts identifiers. These are given by the regex 
                L(L|D)*
where L is the set of all letters (and the underscore) and D is the set of all Digits.
This means that the string accepts any string that begins with a letter or underscore and is 
followed by any combination of letters, digits and underscores.
*/
pub fn make_identifier_dfa() -> Dfa {
    // State L represents that the first character was a letter or underscore
    // State D represents that the first character was a digit
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
                // We loop in states L or D as long as we have valid characters
                (symbol, *state)
            }
        }).collect()
    }).collect();

    Dfa::new(states, alphabet, transitions)
}

/* Generate DFA to recognize operators. This currently recognizes the arithmetic operators + - / *  
and the assignment operator =. The DFA recognizes a single character at a time */
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

/* Generate a DFA to recognize ignored tokens, like whitespace and semicolons. */
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

/* 
    Perform a lexical scan on given input file.
    The method simulates an overarching DFA by coordinating smaller DFA's
    in a way that produces the same logic.

    The input is a buffer for the bytes of a source file.
    The lexical scan splits the input string into tokens by the method of longest prefix,
    then each token is categorized into one of the following classes, defined by the following regular expressions
        keyword: var
        identifier: L{L|D}* where L is the set of letters and D is the set of digits
          operator: {'+'|'-'|'*'|'/'|'='}
           integer: DD* where D is the set of digits
           ignored: {'\n', ''}

    Note that there is ambiguity in how to split a string.
    For example, the string "55" could be split into 5 5, or it could be recognized as a single 55.
    Another example, "varx" could be recognized as (keyword var, identifier x) or (identifier varx).

    To solve this issue, we will use the longest prefix method.
    This method takes the longest string from the start accepted by any DFA as the correct next token.
    Then, it removes that token and starts where it ends.

*/
pub fn lexical_scan(mut istream: VecDeque<u8>) -> Result<HashMap<String, Rc<String>>, Error> {
    let mut symbol_table = HashMap::<Vec<u8>,Rc<String>>::new();

    // Set up DFA's and associate them with a class name
    let machines: [(Rc<String>, Dfa); 5] = [
        (Rc::new("ignored".to_string()), make_ignored_dfa()),
        (Rc::new("identifier".to_string()), make_identifier_dfa()),
        (Rc::new("keyword".to_string()), make_keyword_dfa()),
        (Rc::new("operator".to_string()), make_operator_dfa()),
        (Rc::new("integer".to_string()), make_integer_dfa()),
    ];

    while !istream.is_empty() {
        // get the longest prefix that is accepted by any DFA
        let longest_accepted = machines.iter().map(|(class, machine)| {
            //println!("running {class} dfa ");
            (machine.get_longest_accepted(&mut istream), class)
        }).max_by_key(|(name, _class)| {
            // sort each longest prefix by length
            name.len()
        });

        let Some((name, class)) = longest_accepted
        else {return Result::Err(Error)};

        let name_len = name.len();

        // Skip ignored tokens
        if !(class.contains("ignored") || symbol_table.contains_key(&name)){
            symbol_table.insert(name.clone(), class.clone());
            //println!("{:?}", symbol_table)
        }

        // Pop front of the string
        for _ in 0..name_len {
            istream.pop_front();
        }
    }

    // Convert symbol table keys from byte vectors to strings
    let symbol_table = HashMap::<String, Rc<String>>::from_iter(symbol_table.iter().map(|(name_bytes,class)|{
        let mut name = String::new();
        name_bytes.iter().for_each(|&x| {name.push(x as char)});
        (name, class.clone())   
    }
));

    Result::Ok(symbol_table)
}