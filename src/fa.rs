use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct Dfa {
    accept: Vec<bool>,
    transition: Vec<Vec<usize>>,
    symbol_indices: HashMap<char, usize>,
}

impl Dfa {
    pub fn new(states: Vec<(char, bool)>, alphabet: Vec<char>, transitions: Vec<Vec<(char, char)>>) -> Self {
        // Assume initial state is in index 0 and transition table is using states' order for rows
        let mut accept = vec![false; states.len()];
        let mut state_indices:HashMap<char, usize> = HashMap::new();
        for (i,(state, accepting)) in states.iter().enumerate() {
            accept[i] = *accepting;
            state_indices.insert(*state, i);
        }

        let mut symbol_indices: HashMap<char, usize> = HashMap::new();
        for (i, symbol) in alphabet.iter().enumerate() {
            symbol_indices.insert(*symbol, i);
        }

        let mut transition = vec![vec![0; alphabet.len()]; states.len()];
        for (state0_index, arrows) in transitions.iter().enumerate() {
            for  (symbol, state1) in arrows {
                let symbol_index = symbol_indices[symbol];
                let state1_index = state_indices[state1];
                transition[state0_index][symbol_index] = state1_index;
            }
        }       
        
        Dfa{
            accept, transition, symbol_indices
        }
    }
    
    pub fn accepts(&self, word: String) -> bool {
        let mut state = 0;

        for i in word.chars() {
            if !self.symbol_indices.contains_key(&i) {return false}
            state = self.transition[state][self.symbol_indices[&i]];
        }

        self.accept[state as usize]
    }

    pub fn get_longest_accepted(&self, istream: &mut VecDeque<u8>) -> Vec<u8> {
        let mut state = 0;
        let mut last_accepting_index: Option<usize> = Option::None;

        for (index, i) in istream.iter().enumerate() {
            if !self.symbol_indices.contains_key(&(*i as char)) {
                println!("unrecognized symbol '{}' at {}", *i as char, index);
                break
            }
            state = self.transition[state][self.symbol_indices[&(*i as char)]];
            if self.accept[state] {
                println!("recognized accepting symbol '{}' at {}", *i as char, index);
                last_accepting_index = Some(index);
            }
        }
        match last_accepting_index {
            None => vec![],
            Some(t) => {
                let mut out: Vec<u8> = vec![];
                for i in 0..=t {
                    print!("{}", istream[i] as char);
                    out.push(istream[i])
                };
                print!("\n");
                out
                
            }
        }
    }

}