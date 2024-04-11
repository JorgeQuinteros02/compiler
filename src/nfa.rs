use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Nfa {
    pub accepting: Vec<bool>,
    pub transition: Vec<Vec<Option<Vec<usize>>>>, // 2d Array where elements are sets of states,
    pub symbols_table: HashMap<char, usize>,
}

impl Nfa {
    fn new(transition: Vec<Vec<Option<Vec<usize>>>>, accepting: Vec<bool>, alphabet: Vec<char>) -> Self {
        let symbols_table: HashMap<char, usize> = HashMap::from_iter(
            alphabet.iter().enumerate().map(|(i, &symbol)|{
                (symbol, i)
            })
        );


        Nfa { accepting, transition, symbols_table }
    }

    pub fn from_regex(regex: &str, alphabet: &str) -> Nfa {
        let alphabet: Vec<char> = alphabet.chars().collect();
        let fragment = NfaFragment::from_regex(regex, &alphabet);
        //println!("{regex}{fragment:#?}");

        let mut transition = fragment.transition;

        
        // Add new state 
        transition.push(vec![None; transition[0].len()]);


        // Add transition from out-state to new final state with out-state symbol
        let (out_state, out_symbol) = fragment.out;
        let num_states = transition.len();
        let out_symbol = out_symbol.unwrap_or(alphabet.len());

        let out_range = &mut transition[out_state][out_symbol];
        if let Some(states) = out_range {
            states.push(num_states - 1)
        } else {
            *out_range = Some(vec![num_states - 1])
        }

        // Make the new final state the only accepting state
        let mut accepting = vec![false; num_states];
        *accepting.last_mut().unwrap() = true;

        //println!("Made NFA");
        Self::new(transition, accepting, alphabet)
    }

    pub fn empty_closure(&self, states: Vec<usize>) -> Option<Vec<usize>> {
        let epsilon_index = self.transition[0].len() - 1;
        let mut target_stack = states;

        target_stack.sort();
        target_stack.dedup();

        let mut closure = vec![false; self.transition.len()];
        


        loop {
            let candidate = target_stack.pop()?;
            
            if !closure[candidate] {
                // add states in epsilon transition range of candidate state
                self.transition[candidate][epsilon_index].clone().unwrap_or(vec![]).iter().for_each(|&x |{
                    if !closure[x] {
                        target_stack.push(x)
                    }
                });
                closure[candidate] = true;
            }

            if target_stack.is_empty() {
                break
            }
        }

        let mut result = vec![];
        closure.iter().enumerate().for_each(|(state, &is_in)|{
            if is_in {
                result.push(state);
            }
        });

        Some(result)
    }
}

#[derive(Debug)]
struct NfaFragment {
    transition: Vec<Vec<Option<Vec<usize>>>>,
    out:(usize, Option<usize>) // (state, out_symbol)
}

impl NfaFragment {
const OPERATORS: &'static str = ")(|+*"; // in order of lower to higher precedence

    fn prec(operator:char) -> usize{
        NfaFragment::OPERATORS.find(operator).unwrap()
    }

    fn regex_to_postfix(regex: &str) -> Vec<char> {
        // Add concatenation + operators in between consecutive terms (tokens and parenthesized or starred expressions)
        let mut previous_was_term = false;
        let mut escape = false;
        let mut token_list: Vec<char> = Vec::new();
        let mut raw_stack: VecDeque<char> = regex.chars().collect();
        while !raw_stack.is_empty() {
            let token = raw_stack.pop_front().unwrap();
            let token_is_operator = Self::OPERATORS.contains(token); 

            if !escape {
                if token == '\\' {escape = true};
                if previous_was_term && (token == '(' || !token_is_operator) {
                    token_list.push('+');
                }
                if ")*".contains(token) || !token_is_operator {
                    previous_was_term = true
                } else {
                    previous_was_term = false
                }
            }
            
            token_list.push(token);
        }
        let mut postfix_list: Vec<char> = vec![];
        let mut op_stack: Vec<char> = vec![];
        let mut escape = false;

        for token in token_list {
            if escape {
                postfix_list.push(token);
                escape = false;
                continue
            } else if token == '\\' {
                escape = true;
            }
            if Self::OPERATORS.contains(token) {
                let precedence = Self::prec(token);
                match token {
                    '(' => op_stack.push(token),
                    ')' => {
                        if let Some( mut top_token) = op_stack.pop() {
                            while top_token != '(' {
                                postfix_list.push(top_token);
                                if let Some(op) = op_stack.pop() {
                                    top_token = op;
                                } else {
                                    panic!("Couldn't find matching parenthesis")
                                }
                            }
                        } else {
                            panic!("Couldn't find operation in the stack")
                        }
                    },
                    _ => {
                        while   !op_stack.is_empty() &&
                                Self::prec(*op_stack.last().unwrap()) >= precedence {
                                    postfix_list.push(op_stack.pop().unwrap())
                        }
                        op_stack.push(token)
                    }
                }
            } else {
                postfix_list.push(token)
            }
        }


        while !op_stack.is_empty() {
            postfix_list.push(op_stack.pop().unwrap())
        }

        postfix_list
    }

    fn from_regex(regex: &str, alphabet: &Vec<char>) -> Self {
        let postfix = Self::regex_to_postfix(regex);
        let mut eval_stack: Vec<Self> = vec![];
        let mut escape = false;

        for token in postfix {
            if escape {
                match token {
                    'e' => eval_stack.push(Self::epsilon(alphabet)),
                    _ => eval_stack.push(Self::symbol(token, alphabet))
                }
                escape = false;
                continue
            } else if token == '\\' {
                escape = true;
                continue
            }

            if !Self::OPERATORS.contains(token) {
                eval_stack.push(Self::symbol(token, &alphabet));
            } else {
                match token {
                    '*' => {
                        let fragment = eval_stack.pop().unwrap();
                        //println!("frag before star {fragment:#?}");
                        eval_stack.push(Self::star(fragment));
                        //println!("frag* {:#?}",eval_stack.last());
                    }, 
                    '+' => {
                        let frag2 = eval_stack.pop().unwrap(); 
                        let frag1 = eval_stack.pop().unwrap();
                        eval_stack.push(Self::concatenate(frag1, frag2));
                    },
                    '|' => {
                        let frag2 = eval_stack.pop().unwrap(); 
                        let frag1 = eval_stack.pop().unwrap();
                        eval_stack.push(Self::union(frag1, frag2))
                    
                    },
                    _ => panic!()
                }
            }
        }

        eval_stack.pop().unwrap()
    }

    fn symbol(symbol:char, alphabet: &Vec<char>) -> Self {
        let transition = vec![vec![None; alphabet.len() + 1]];
        if let Some(index) = alphabet.iter().position(|x| *x==symbol) {
            Self {transition, out:(0, Some(index))}
        } else {
            println!("calling symbol method");
            println!("symbol {symbol}");
            println!("alphabet{:?}", alphabet);
            panic!()
        }
    }

    fn epsilon(alphabet: &Vec<char>) -> Self {
        let transition = vec![vec![None; alphabet.len() + 1]];
        Self {transition, out:(0, None)}
    }

    fn shifted(self, shift:usize) -> Self {
        let transition = self.transition.iter().map(|state| {
            state.iter().map(|symbol| {
                match symbol {
                    None => None,
                    Some(states) => Some(
                        states.iter().map(|&x| {x + shift}).collect::<Vec<usize>>()
                    )
                }
            }).collect::<Vec<Option<Vec<usize>>>>()
        }).collect::<Vec<Vec<Option<Vec<usize>>>>>();

        let out = (self.out.0 + shift, self.out.1);

        Self{transition, out}
    }

    fn concatenate(a:Self, b:Self) -> Self {
        let a_num_states = a.transition.len();
        let extended_alphabet_size = a.transition[0].len();
        
        let b = b.shifted(a_num_states);

        let mut transition = [a.transition, b.transition].concat();
        
        // insert transition from out-state of a to in-state of b
        // NOTE: b in-state index has been shifted from 0 to a_len
        let out_state_index = a.out.0;
        // default position is the last column corresponding to epsilon-transitions
        let out_symbol_index = a.out.1.unwrap_or(extended_alphabet_size-1);
        let a_out = &mut transition[out_state_index][out_symbol_index];
        
        match a_out {
            Some(states) => states.push(a_num_states),
            None => *a_out = Some(vec![a_num_states])
        }

        Self {transition, out:b.out}
    }

    fn union(a: Self, b:Self) -> Self {
        let a_num_states = a.transition.len();
        let epsilon_index = a.transition[0].len() - 1;
        let alphabet_size = epsilon_index;
        

        // shift state indices in a and b
        // new lawout is 0, A, B, 1
        // where 0 is the new in-state
        //       A is the set of states of a
        //       B is the set of states of b
        //       1 is the new out-state
        let Self{transition:a_transition, out:a_out} = a.shifted(1);
        let Self{transition:b_transition, out:b_out} = b.shifted(1 + a_num_states);


        let mut transition = [
            vec![vec![None; alphabet_size + 1]],
            a_transition,
            b_transition,
            vec![vec![None; alphabet_size + 1]]
        ].concat();

        // set epsilon-transition from new state to start of a and b
        transition[0][epsilon_index] = Some(vec![1, 1 + a_num_states]);
        // set transitions from out-states of a and b to start of new out-state with respective out-symbols of a and b
        let new_out_state_index = transition.len()-1;
        for old_out in [a_out, b_out] {
            let old_out_range = &mut transition[old_out.0][old_out.1.unwrap_or(epsilon_index)];
            match old_out_range {
                Some(states) => states.push(new_out_state_index),
                None => *old_out_range = Some(vec![new_out_state_index])
            }
        }

        let out = (new_out_state_index, None);

        Self{transition, out}
    }

    fn star(old: Self) -> Self {
        let extended_alphabet_size = old.transition[0].len();
        // shift state indices to make space for new state
        let old = old.shifted(1);

        // add new state at the start
        let mut transition = [
            vec![vec![None; extended_alphabet_size]],
            old.transition
        ].concat();

        // set epsilon-transition from new start state to old start state
        let epsilon_index = extended_alphabet_size - 1;
        transition[0][epsilon_index] = Some(vec![1]);

        // set transition from old out-state to new out-state
        // use out symbol
        let out_symbol = if let Some(symbol) = old.out.1 {
            symbol
        } else {
            epsilon_index
        };
        
        if let Some(out_range) = &mut transition[old.out.0][out_symbol] {
            out_range.push(0)
        } else {
            transition[old.out.0][out_symbol] = Some(vec![0])
        }

        let out = (0, None);

        Self{transition, out}
    }
}