use std::{collections::{HashMap, VecDeque}, hash::Hash, path::Display, thread::current};
use itertools::{enumerate, GroupingMapBy, Itertools};

use crate::nfa::Nfa;
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
#[derive(Default, Debug)]
pub struct Dfa {
    pub marks: Vec<usize>,
    pub transition: Vec<Vec<usize>>,
    symbol_indices: HashMap<char, usize>,
}

impl Dfa {
    pub fn new(states: Vec<(char, usize)>, alphabet: Vec<char>, transitions: Vec<Vec<(char, char)>>) -> Self {
        // Assume initial state is in index 0 and transition table is using states' order for rows
        let mut marks = vec![0; states.len()];
        let mut state_indices:HashMap<char, usize> = HashMap::new();
        for (i,(state, mark)) in states.iter().enumerate() {
            marks[i] = *mark;
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
            marks, transition, symbol_indices
        }
    }

    pub fn from_regex(regex:&str, alphabet:&str, mark_num:usize) -> Self {
        Self::from_nfa(&Nfa::from_regex(regex, alphabet, mark_num))
    }

    pub fn from_nfa(nfa: &Nfa) -> Self {
        
        let nfa_transitions:Vec<Vec<Option<Vec<usize>>>> = nfa.transition.clone();
        let alphabet_size = nfa.symbols_table.len();
        let alphabet:&Vec<usize> = &(0..alphabet_size).collect();

        let mut subset_table: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut transition: Vec<Vec<usize>> = vec![vec![0; alphabet_size]];

        
        // insert start state into DFA
        // This represents the epsilon-closure of 
        let start_state = nfa.empty_closure(vec![0]).unwrap_or(vec![]);
        subset_table.insert(start_state.clone(), 0);
        let mut marks= vec![start_state.clone().iter().map(|x| nfa.marks[*x]).max().unwrap_or_default()];

        let mut stack: Vec<Vec<usize>> = vec![start_state];

        let mut state_count = 0;
        while !stack.is_empty() {
            let dfa_state = stack.pop().unwrap();
            let dfa_index = subset_table[&dfa_state];
            for &symbol in alphabet{
                let mut candidate: Vec<usize> = vec![];
                for &nfa_state in &dfa_state {
                    if let Some(range_states) = &nfa_transitions[nfa_state][symbol] {
                        candidate = [candidate, range_states.clone()].concat()
                    }
                }
                candidate.sort();
                candidate.dedup();
                candidate = nfa.empty_closure(candidate).unwrap_or(vec![]);

                if  subset_table.contains_key(&candidate) {
                    let candidate_index = subset_table[&candidate];
                    transition[dfa_index][symbol] = candidate_index;
                } else {
                    state_count += 1;
                    transition.push(vec![0; alphabet_size]);
                    stack.push(candidate.clone());
                    marks.push(candidate.iter().map(|&x| nfa.marks[x]).max().unwrap_or_default());
                    subset_table.insert(candidate, state_count);
                    transition[dfa_index][symbol] = state_count;
                }
            }
        }




        let dfa = Dfa{transition, symbol_indices:nfa.symbols_table.clone(), marks};
        let minimized = dfa.minimized();
        minimized
    }



    pub fn minimized(&self) -> Self {
        let mut cloud = DfaCloud::from_dfa(self);
        cloud.divide();
        let (transition, marks) = cloud.into_dfa_graph();


        Self { marks, transition, symbol_indices: self.symbol_indices.clone()}
    }
        

    
    pub fn accepts(&self, word: String) -> bool {
        let mut state = 0;

        for i in word.chars() {
            if !self.symbol_indices.contains_key(&i) {return false}
            state = self.transition[state][self.symbol_indices[&i]];
        }

        self.marks[state] > 0
    }

    /*
    This method is used to split the input string into tokens in an unamboguous way.
    Read the comment above the lexical_scan function for more information.
    */
    pub fn get_longest_accepted(&self, istream: &mut VecDeque<u8>) -> (String, usize) {
        let mut current_state = 0;
        let mut last_mark = 0;
        let mut longest_accepted: Vec<u8> = vec![];
        let mut token_buffer: Vec<u8> = vec![];
        

        while !istream.is_empty() {
            let next_token = istream.pop_front().unwrap();
            current_state = self.transition[current_state][self.symbol_indices[&(next_token as char)]];
            token_buffer.push(next_token);
            if self.marks[current_state] > 0 {
                longest_accepted.extend(token_buffer.iter());
                last_mark = self.marks[current_state];
                token_buffer.clear();
            } else {
                token_buffer.into_iter().rev().for_each(|x| istream.push_front(x));
                break
            }
        }

        (longest_accepted.into_iter().map(|x| x as char).collect::<String>(), last_mark)

    }

}

struct DfaCloud {
    state_transition:Vec<Vec<usize>>,
    groups:Vec<Vec<usize>>,
    state_groups: Vec<usize>,
    marks: Vec<usize>,
}

impl DfaCloud {
    fn from_dfa(dfa:&Dfa) -> Self {
        let groups: Vec<Vec<usize>> = dfa.marks
                                                .iter()
                                                .enumerate()
                                                .map(|(i,mark)| (*mark, i))
                                                .into_group_map()
                                                .into_values()
                                                .sorted_by_key(|x| x[0])
                                                .collect_vec();

        let mut state_groups = vec![0; dfa.marks.len()];
        if groups.len() > 1 {
            for (group_index, group) in groups.iter().enumerate() {
                for &state_index in group {
                    state_groups[state_index] = group_index;
                }
            }
            
        }

        Self { state_transition: dfa.transition.clone(), groups, state_groups, marks:dfa.marks.clone()}
    }

    #[inline]
    fn get_group_transitions(&self, group_index: usize, translate:Option<&Vec<usize>>) -> Vec<usize> {

        self.state_transition[self.groups[group_index][0]].iter().map(|x| {
            if let Some(map) = translate {
                map[self.state_groups[*x]]
            } else {
                self.state_groups[*x]
            }
            
        }).collect_vec()
    }

    fn into_dfa_graph(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
        let group0_index = self.state_groups[0];
        let mut translated: Vec<usize> = (0..self.state_groups.len()).collect();
        translated.swap(0, group0_index);
        let state0_transition = self.get_group_transitions(group0_index, Some(&translated));
        let mut transition = vec![state0_transition];
        let mut marks = vec![0; self.groups.len()];


        for i in 0..self.groups.len() {
            marks[translated[i]] = self.groups[i].clone().into_iter().map(|x| self.marks[x]).max().unwrap_or_default();
            
            if i != group0_index {
                transition.push(self.get_group_transitions(i, Some(&translated)))
            }
        }

        (transition, marks)
    }

    fn divide(&mut self) {
        let mut current_group_index = 0;
        while current_group_index < self.groups.len() {
            let current_group = &self.groups[current_group_index];
            let subgroups: Vec<Vec<usize>> = current_group.into_iter()
                                                            .map(|&x|{(&self.state_transition[x], x)})
                                                            .into_group_map()
                                                            .into_values()
                                                            .sorted_by_key(|x| x[0])
                                                            .collect_vec();

            let subgroup_num = subgroups.len();

            if subgroup_num == 0 {
                panic!()
            } else if subgroup_num == 1 {
                current_group_index += 1;
                continue
            }

            self.groups.remove(current_group_index);

            for group in self.groups[current_group_index..].into_iter() {
                group.into_iter().for_each(|x| self.state_groups[*x] += subgroup_num-1);
            }

            for (i, subgroup) in subgroups.iter().enumerate() {
                let new_index = current_group_index + i;
                subgroup.iter().for_each(|&x| self.state_groups[x] = new_index);
            }
            let ( first_half,  second_half) = self.groups.split_at(current_group_index);

            let mut new_group:Vec<Vec<usize>> = vec![];
            first_half.iter().for_each(|x| {new_group.push(x.clone())});
            subgroups.iter().for_each(|x| {new_group.push(x.clone())});
            second_half.iter().for_each(|x| {new_group.push(x.clone())});
            
            self.groups = new_group;

            current_group_index = 0;
        }
    }
}

impl ToString for Dfa {
    fn to_string(&self) -> String {
        let mut string = String::new();
        let sorted_alphabet = self.symbol_indices.keys().sorted_by_key(|x| self.symbol_indices[x]).collect_vec();
        string.push_str(format!("{:?}\n",sorted_alphabet).as_str());
        for i in 0..self.marks.len() {
            string.push_str(format!("{i} {:?} {:?}\n", self.transition[i], self.marks[i]).as_str())
        }
        string
    }
}
