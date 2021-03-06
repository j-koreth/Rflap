#![feature(proc_macro_hygiene, decl_macro)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Error, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Result;

use multimap::MultiMap;

mod finite_automaton;
mod generate_tests;

fn api(input_automaton_json: finite_automaton::FiniteAutomatonJson) -> Result<String> {
    let (input_automaton, input_strings, hint) =
        finite_automaton::FiniteAutomaton::new_from_json(&input_automaton_json);

    let mut return_paths = Vec::new();

    for input_string in input_strings {
        return_paths.push(input_automaton.validate_string(input_string.to_owned()));
    }

    let callback = finite_automaton::FiniteAutomatonCallback {
        list_of_strings: return_paths.to_owned(),
        hint: hint.to_owned(),
    };

    let callback_string = serde_json::to_string(&callback)?;
    println!("{}", callback_string);
    Ok("".to_string())
}

fn tests(tests: generate_tests::TestsJson) -> Result<String> {
    let callback = generate_tests::generate_tests(tests);
    let t = serde_json::to_string(&callback)?;
    println!("{}", t);
    Ok("".to_string())
}

// return is a tuple of number of passed test cases vs total cases
fn grade(
    source: finite_automaton::FiniteAutomatonJson,
    target: finite_automaton::FiniteAutomatonJson,
    num_tests: u16,
) -> (
    u16,
    u16,
    std::vec::Vec<std::string::String>,
    std::vec::Vec<u8>,
    std::vec::Vec<std::string::String>,
    std::vec::Vec<u8>,
    bool,
) {
    // generate TestsJson array

    let mut alphabet = target.alphabet.to_owned();
    alphabet.remove(&'Ɛ');

    let test_strings_deterministic = generate_tests::generate_tests(generate_tests::TestsJson {
        alphabet: alphabet.to_owned(),
        size: (num_tests * 4) / 5, // how many strings for non deterministic
        length: 10,                // longest string
        random: false,
    })
    .return_vec;
    // then take out the first num_tests/2 elements from the vector

    let test_strings_nondeterministic = generate_tests::generate_tests(generate_tests::TestsJson {
        alphabet: alphabet.to_owned(),
        size: (num_tests) / 5, // how many strings for non deterministic
        length: 10,            // longest string
        random: true,
    })
    .return_vec;
    // run tests on source and target, check that the result is equal

    let source: finite_automaton::FiniteAutomaton =
        finite_automaton::FiniteAutomaton::new_from_json(&source).0;
    let target: finite_automaton::FiniteAutomaton =
        finite_automaton::FiniteAutomaton::new_from_json(&target).0;

    let mut deterministic_scores: Vec<u8> = Vec::new();
    let mut nondeterministic_scores: Vec<u8> = Vec::new();

    let mut passed: u16 = 0;

    for test in &test_strings_deterministic {
        let (_, accepted_source, _, _) = source.validate_string(test.to_string());
        let (_, accepted_target, _, _) = target.validate_string(test.to_string());

        if accepted_source == accepted_target {
            passed += 1;
        }

        deterministic_scores.push((accepted_source == accepted_target) as u8);
    }

    for test in &test_strings_nondeterministic {
        let (_, accepted_source, _, _) = source.validate_string(test.to_string());
        let (_, accepted_target, _, _) = target.validate_string(test.to_string());

        if accepted_source == accepted_target {
            passed += 1;
        }

        nondeterministic_scores.push((accepted_source == accepted_target) as u8);
    }

    (
        passed,
        num_tests,
        test_strings_deterministic,
        deterministic_scores,
        test_strings_nondeterministic,
        nondeterministic_scores,
        source.is_deterministic() || !target.is_deterministic(), // count as 5/15 for false
    )
}

#[derive(Serialize, Deserialize)]
struct Tests {
    score: f64,
    name: String,
    number: String,
    visibility: String,
}

fn main() -> io::Result<()> {
    use std::io::Read;

    let mut buffer = String::new();

    // student input
    io::stdin().read_to_string(&mut buffer)?;

    let args: Vec<String> = env::args().collect();

    if &args[1] == "automata" {
        api(serde_json::de::from_str::<finite_automaton::FiniteAutomatonJson>(&buffer).unwrap());
    } else if &args[1] == "tests" {
        tests(serde_json::de::from_str::<generate_tests::TestsJson>(&buffer).unwrap());
    } else if &args[1] == "grade" {
        // answer file will be passed in the second command line argument
        let buffer_answer = fs::read_to_string(&args[2])?;

        // for the actual grading, we should show like 20 shorter strings and hide 80,
        let public_tests = grade(
            serde_json::de::from_str::<finite_automaton::FiniteAutomatonJson>(&buffer).unwrap(),
            serde_json::de::from_str::<finite_automaton::FiniteAutomatonJson>(&buffer_answer)
                .unwrap(),
            10,
        );
        let hidden_tests = grade(
            serde_json::de::from_str::<finite_automaton::FiniteAutomatonJson>(&buffer).unwrap(),
            serde_json::de::from_str::<finite_automaton::FiniteAutomatonJson>(&buffer_answer)
                .unwrap(),
            90,
        );

        // then initialize a data structure which follows the output of results.json
        // the only members out of results.json which matter are score and tests
        // the only members of tests which we care about are
        let mut tests: Vec<Tests> = Vec::new();

        let problem_number: String = args[4].to_owned();

        let is_properly_deterministic = if public_tests.6 { 0.0 } else { -5.0 };

        tests.push(Tests {
            score: is_properly_deterministic,
            name: "determinism".to_string(),
            number: problem_number.to_owned(),
            visibility: "visible".to_string(),
        });

        for test in 0..public_tests.2.len() {
            tests.push(Tests {
                score: public_tests.3[test] as f64,
                name: public_tests.2[test].to_owned(),
                number: problem_number.to_owned(),
                visibility: "visible".to_string(),
            });
        }

        for test in 0..public_tests.4.len() {
            tests.push(Tests {
                score: public_tests.5[test] as f64,
                name: public_tests.4[test].to_owned(),
                number: problem_number.to_owned(),
                visibility: "visible".to_string(),
            });
        }

        for test in 0..hidden_tests.4.len() {
            tests.push(Tests {
                score: hidden_tests.5[test] as f64,
                name: hidden_tests.4[test].to_owned(),
                number: problem_number.to_owned(),
                visibility: "hidden".to_string(),
            });
        }

        for test in 0..hidden_tests.2.len() {
            tests.push(Tests {
                score: hidden_tests.3[test] as f64,
                name: hidden_tests.2[test].to_owned(),
                number: problem_number.to_owned(),
                visibility: "hidden".to_string(),
            });
        }

        let final_tests = serde_json::to_string(&tests)?;

        let mut output = File::create(&args[3])?;
        write!(output, "{}", final_tests)?;
    }

    Ok(())
}
