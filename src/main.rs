use std::fs;

use tree_sitter::{InputEdit, Language, Node, Parser, Point, TreeCursor};
use walkdir::WalkDir;

const AS_OPERATOR_ID: u16 = 234;

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let mut all_failures: Vec<Node> = Vec::new();
    for entry in WalkDir::new("./") {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                // error!("Error walking directory: {}\n{e}", path.to_str().unwrap());
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let source_code = fs::read(entry.path()).expect("Could not read path");
        let tree = parser.parse(source_code, None).unwrap();
        let cursor = tree.walk();
        let mut new_failures = traverse(cursor, |node| is_as(node));
        all_failures.append(&mut new_failures); // Merge into all_failures
    }
    println!("`as` count: {}", all_failures.len());

    for failure in all_failures {
        println!("{:#?}", failure);
    }
}

fn is_as(node: Node) -> Option<Node> {
    if node.grammar_id() == AS_OPERATOR_ID {
        return Some(node);
    }
    None
}

// Inspired by from: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<F>(mut cursor: TreeCursor, mut callback: F) -> Vec<Node>
where
    F: FnMut(Node) -> Option<Node>,
{
    let mut failures: Vec<Node> = Vec::new();
    loop {
        if let Some(failure) = callback(cursor.node()) {
            failures.push(failure);
        }

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        loop {
            if !cursor.goto_parent() {
                return failures;
            }

            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
