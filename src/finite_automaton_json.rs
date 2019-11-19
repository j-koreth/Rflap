use std::collections::*;

pub struct FiniteAutomatonJSON {
	alphabet : HashSet<char>,
	start_state : String,
    states : HashMap<String, bool>,
	transition_function : Vec<(String, Option<char>, String)>,
	determinism : bool
}

impl FiniteAutomatonJSON {
	pub fn new(FiniteAutomaton) {

	}
}