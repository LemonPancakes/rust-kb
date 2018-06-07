extern crate rust_kb;

use rust_kb::KnowledgeBase;
use std::env;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

const HELP: &str = "Options:\n Assert Statement 'assert: (isa this example)'\n Retract Statement 'retract: (isa this example)'\n Ask Fact 'ask: (isa this example)'\n Query Fact 'query: (isa this example)'\n Erase entire knowledge base 'new'\n Help 'h'\n Quit 'q'\n";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut kb = if args.len() == 2 {
        if let Ok(kb) = KnowledgeBase::from_file(&args[1]) {
            println!("Successfully parsed knowledge base from '{}'.", &args[1]);
            kb
        } else {
            println!(
                "Failed to parse knowledge base from '{}'. Quitting.",
                &args[1]
            );
            return;
        }
    } else {
        KnowledgeBase::new()
    };
    println!("KnowledgeBase Instantiated.\n");

    let mut lines = BufReader::new(stdin()).lines();

    help();
    prompt();

    while let Some(Ok(line)) = lines.next() {
        if let Some(index) = line.find(":") {
            let command = &line[..index];
            let statement = if line.len() > index + 2 {
                &line[(index + 2)..]
            } else {
                ""
            };
            let fact_attempt = "fact: ".to_string() + &statement + ";";
            let rule_attempt = "rule: ".to_string() + &statement + ";";

            match command.as_ref() {
                "assert" => {
                    if let Ok(fact) = kb.create_fact(&fact_attempt) {
                        if let Ok(_) = kb.assert(fact) {
                            println!("Asserted Fact '{}'.", &statement);
                        } else {
                            println!("Assert failed, probably because this fact has already been asserted.");
                        }
                    } else if let Ok(rule) = kb.create_rule(&rule_attempt) {
                        if let Ok(_) = kb.assert(rule) {
                            println!("Asserted Rule '{}'.", &statement);
                        } else {
                            println!("Assert failed, probably because this rule has already been asserted.");
                        }
                    } else {
                        println!("Failed to parse statement.");
                    }
                }
                "retract" => {
                    if let Ok(fact) = kb.create_fact(&fact_attempt) {
                        if let Ok(_) = kb.retract(fact) {
                            println!("Retracted Fact '{}'.", &statement);
                        } else {
                            println!("Retract failed, either because Fact does not exist or is supported by a Rule.");
                        }
                    } else if let Ok(rule) = kb.create_rule(&rule_attempt) {
                        if let Ok(_) = kb.retract(rule) {
                            println!("Retracted Rule '{}'.", &statement);
                        } else {
                            println!("Retract failed, probably because Rule does not exist.");
                        }
                    } else {
                        println!("Failed to parse statement.");
                    }
                }
                "ask" => {
                    if let Ok(fact) = kb.create_fact(&fact_attempt) {
                        if let Ok(res) = kb.ask(&fact) {
                            println!("{}", res.to_string().to_uppercase());
                        }
                    } else if let Ok(_) = kb.create_rule(&rule_attempt) {
                        println!("Ask can only accept a Fact.");
                    } else {
                        println!("Failed to parse statement.");
                    }
                }
                "query" => {
                    if let Ok(fact) = kb.create_fact(&fact_attempt) {
                        let query_result = kb.query(&fact);
                        if query_result.is_empty() {
                            println!("No results found.");
                        } else {
                            println!("Query results:");
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
                    } else if let Ok(_) = kb.create_rule(&rule_attempt) {
                        println!("Query can only accept a Fact.");
                    } else {
                        println!("Failed to parse statement.");
                    }
                }
                _ => println!("'{}' is an unrecognized command.", command),
            }
        } else if line == "new" {
            print!("Are you sure (y/n)? ");
            flush();
            if let Some(Ok(response)) = lines.next() {
                match response.as_ref() {
                    "y" | "Y" => {
                        kb = KnowledgeBase::new();
                        println!("Knowledge Base erased.");
                    }
                    _ => {
                        println!("Canceled.");
                    }
                }
            }
        } else if line == "h" {
            help();
        } else if line == "q" {
            return;
        }
        prompt();
    }
}

fn flush() {
    stdout().flush().unwrap();
}

fn prompt() {
    print!("kb> ");
    flush();
}

fn help() {
    println!("{}", HELP);
}
