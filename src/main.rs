extern crate rust_kb;

use rust_kb::KnowledgeBase;
use std::io::{stdin, BufRead, BufReader};

const MENU: &str = "Select an option (1-4):\n 1. Assert Statement\n 2. Retract Statement\n 3. Ask Fact\n 4. Query Fact";

fn main() {
    let mut kb = KnowledgeBase::new();
    let mut selected_option: u32 = 0;

    let mut lines = BufReader::new(stdin()).lines();

    println!("{}", MENU);

    while let Some(Ok(line)) = lines.next() {
        if selected_option == 0 {
            selected_option = match line.as_ref() {
                "1" => 1,
                "2" => 2,
                "3" => 3,
                "4" => 4,
                _ => {
                    println!("Please select a valid option (1-4).\n");
                    0
                }
            };

            if selected_option != 0 {
                println!("Enter statement (rule or fact):");
            }
        } else {
            if let Ok(fact) = kb.create_fact(&("fact: ".to_string() + &line.clone() + ";")) {
                match selected_option {
                    1 => {
                        if let Ok(_) = kb.assert(fact) {
                            println!("Asserted Fact to KB\n");
                        }
                    },
                    2 => {
                        if let Ok(_) = kb.retract(fact) {
                            println!("Retracted Fact from KB\n");
                        } else {
                            println!("Retract failed, either because Fact does not exist or is supported by a Rule\n");
                        }
                    },
                    3 => {
                        if let Ok(res) = kb.ask(&fact) {
                            println!("{}\n", res);
                        }
                    },
                    4 => {
                        println!("{:?}\n", kb.query(&fact));
                    },
                    _ => {}
                }

            } else if let Ok(rule) = kb.create_rule(&("rule: ".to_string() + &line.clone() + ";")) {
                match selected_option {
                    1 => {
                        if let Ok(_) = kb.assert(rule) {
                            println!("Asserted Rule to KB\n");
                        }
                    },
                    2 => {
                        if let Ok(_) = kb.retract(rule) {
                            println!("Retracted Rule from KB\n");
                        } else {
                            println!("Retract failed, probably because Rule does not exist\n");
                        }
                    },
                    _ => println!("Cannot Ask or Query a Rule\n")
                }
            } else {
                println!("Failed to parse statement. Try again?\n");
            }


            selected_option = 0;
            println!("{}", MENU);
        }



    }
}
