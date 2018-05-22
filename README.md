| master | develop |
|:-:|:-:|
| [![Build Status (master)](https://travis-ci.org/LemonPancakes/rust-kb.svg?branch=master)](https://travis-ci.org/LemonPancakes/rust-kb) | [![Build Status (develop)](https://travis-ci.org/LemonPancakes/rust-kb.svg?branch=develop)](https://travis-ci.org/LemonPancakes/rust-kb) |

# Rust-KB

An implementation of knowledge base system library in Rust. We can initiate a knowledge base class from an input kb file, and then perform any queries/planning on the kb. Alternatively, users can instantiate a kb, then add rules and make queries in real time.

## Motivation

We wanted to make something that wasn’t done by many others yet. There doesn’t seem to be any work in progress for incorporating KB into Rust.

## Features

##### Must-Haves

* KB, Rule, Fact classes
* Add and Retract for rules and facts
* Query for KB
* Object hierarchy

##### Nice-to-Haves

* Efficiency for large KBs
* Concurrency management for real-time usage
* Follow the formal syntax of [PDDL](https://en.wikipedia.org/wiki/Planning_Domain_Definition_Language)

## Anticipated Difficulties

Efficiency (and maybe concurrency) will likely be the most challenging but most important element to be aware of. To adequately test our final KB, as well as testing in development, we will likely need to benchmark parts of our library. The library not only needs to be correct, it needs to be fast enough for others to actually want to use.

## Example Use-Cases

* Video games sometimes use knowledge bases to represent character knowledge or state. Various crates for game development focus on physics, game engines, or graphics, but none implement a KB.
* There are also common applications of KBs in artificial intelligence.

An example of a meld kb file:

```
(isa burger Food)
(isa John Person)

(action (eat ?person ?food) =>
  (type ?person Person)
  (type ?food Food))
```

We'd take in something of this form and enable queries on the kb:

```
(eat John ?x) -> ((?x . burger))
```

Which could return all possible bindings to ?x. The exact Rust representation remains to be figured out, but this is the general idea of what we want to accomplish.

## Team Members

* **Eric Hao** - [brotatotes](https://github.com/brotatotes)
* **Adam He** - [AdamHe17](https://github.com/AdamHe17)
* **Christopher Kober** - [ChristopherKober](https://github.com/ChristopherKober)
