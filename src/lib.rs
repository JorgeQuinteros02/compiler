use std::{collections::{HashMap, VecDeque}, fmt::Error, rc::Rc};
use dfa::Dfa;
use itertools::Itertools;
use nfa::Nfa;



pub mod dfa;
pub mod nfa;


const DIGITS: &str = "0123456789";
const LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";

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
pub fn lexical_scan(mut istream: VecDeque<u8>) -> Result<HashMap<String, String>, Error> {
    let mut symbol_table = HashMap::<String,String>::new();

    // Set up DFA's and associate them with a class name
    let letter_regex: String = LETTERS.chars().collect_vec().into_iter().interleave(vec!['|';LETTERS.len() - 1]).collect::<String>();
    let digit_regex: String = DIGITS.chars().collect_vec().into_iter().interleave(vec!['|';DIGITS.len() - 1]).collect::<String>();

    let identifier_regex = ["(",letter_regex.as_str(),")(",letter_regex.as_str(),"|",digit_regex.as_str(),")*"].concat();
    let identifier_alphabet = [LETTERS, DIGITS].concat();


    let regexes = &[
        ("identifier", identifier_regex.as_str(), identifier_alphabet.as_str()),
        ("keyword", "(var|print|if)", "varpintf"),
        ("operator", "(\\+|-|/|\\*|=)", "+-/*="),
        ("integer", "(0|1|2|3|4|5|6|7|8|9)*", "0123456789"),
        ("ignored", "(;| |\t|\r|\n)", "; \t\r\n"),
    ];

    let machines: Vec<Nfa> = regexes.iter().enumerate().map(|(i, &(class,regex,alphabet))| {
        Nfa::from_regex(regex, alphabet, i+1)
    }).collect();

    let lexer_nfa = Nfa::union(machines.iter().collect_vec());
    let lexer = Dfa::from_nfa(&lexer_nfa);

    while !istream.is_empty() {
        // get the longest prefix that is accepted by any DFA
        let (name, mark) = lexer.get_longest_accepted(&mut istream);
        if name.is_empty() {break}
        let class = regexes[mark-1].0;
        // Skip ignored tokens
        if !(class.contains("ignored") || symbol_table.contains_key(&name)){
            symbol_table.insert(name, class.to_string());
        }
    }
    Result::Ok(symbol_table)
}