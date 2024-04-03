use std::collections::{HashMap, VecDeque};
/*
A Deterministic Finite Automaton is a simple machine model that will recognize regular languages
A DFA consists of a 5-tuple (States, Alphabet, Initial, Transition, Accepting)
where   States is a set of states of the DFA
        Alphabet is a set of input symbols
        Initial is a state in States
        Transition is a function that takes a state and an input symbol and gives a new state
        Accepting is a set of states from States

You can give a DFA a string, and it will accept or reject it.
The set of all strings accepted by the DFA is its Language.
We can specify a language by making a DFA for it.

This struct implements a DFA by modeling it by the transition matrix associated to a transition graph.
This matrix has one row per state and one column per input symbol.
The (i,j) entry is a state x that the ith state transitions to
after the DFA reads the jth symbol.

When a DFA reads a string, it starts at the Initial state goes through the transition matrix
by reading each subsequent symbol in the string and the current state.
If the DFA ends in an accepting state after reading the whole string, we say that
the DFA accepts the given string. Otherwise, we reject the string.
 */
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

    /*
    This method is used to split the input string into tokens in an unamboguous way.
    Read the comment above the lexical_scan function for more information.
    */
    pub fn get_longest_accepted(&self, istream: &mut VecDeque<u8>) -> Vec<u8> {
        let mut state = 0;
        let mut last_accepting_index: Option<usize> = Option::None;

        for (index, i) in istream.iter().enumerate() {
            if !self.symbol_indices.contains_key(&(*i as char)) {
                // Stop reading once you find a symbol that isn't recognized by this DFA
                break
            }

            // transition to the next state
            state = self.transition[state][self.symbol_indices[&(*i as char)]];
            
            if self.accept[state] {
                last_accepting_index = Some(index);
            }
        }
        match last_accepting_index {
            None => vec![],
            Some(t) => {
                let mut out: Vec<u8> = vec![];
                for i in 0..=t {
                    out.push(istream[i])
                };
                //print!("\n");
                out
                
            }
        }
    }

}