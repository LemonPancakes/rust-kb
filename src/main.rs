extern crate rust_kb;

use rust_kb::KnowledgeBase;
use std::io::{stdin, BufRead, BufReader};

// TODO: use enum for options
const MENU: &str = "Select an option (1-5):\n 1. Assert Statement\n 2. Retract Statement\n 3. Ask Fact\n 4. Query Fact\n 5. Quit";

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
                "5" | "q" => return,
                _ => {
                    println!("Please select a valid option (1-5).\n");
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
                        let query_result = kb.query(&fact);
                        if query_result.is_empty() {
                            println!("No bindings found\n");
                        } else {
                            let mut crossbar = "-".to_string();
                            for _ in 0..query_result[0].len() {
                                crossbar.push_str("-----------");
                            }
                            println!("{}", crossbar);
                            for variable in &query_result[0] {
                                print!("|{:^10}", (*variable.0).to_string());
                            }
                            println!("|");
                            println!("{}", crossbar);

                            for binding in query_result {
                                for variable_tuple in binding {
                                    print!("|{:^10}", (*variable_tuple.1).to_string());
                                }
                                println!("|");
                            }

                            println!("{}", crossbar);
                        }
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
