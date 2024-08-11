use std::fs;

use tree_sitter::{Node, Parser, Point, TreeCursor};
use walkdir::WalkDir;

const AS_OPERATOR_ID: u16 = 234;

#[derive(Debug)]
struct FailureNode {
    grammar_id: u16,
    grammar_name: String,
    start_position: Point,
    end_position: Point,
}

impl From<Node<'_>> for FailureNode {
    fn from(value: Node) -> Self {
        Self {
            grammar_id: value.grammar_id(),
            grammar_name: value.grammar_name().to_owned(),
            start_position: value.start_position(),
            end_position: value.end_position(),
        }
    }
}

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures: Vec<FailureNode> = WalkDir::new("test_files")
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .filter(|entry| !entry.file_type().is_dir())
        .filter_map(|entry| match fs::read(entry.path()) {
            Ok(source) => Some(source),
            Err(e) => {
                println!("{e}");
                None
            }
        })
        .map(|source_code| parser.parse(source_code, None).expect("Could not parse"))
        .flat_map(|tree| {
            let cursor = tree.walk();
            traverse(cursor, |node| is_as(node.clone()))
        })
        .collect();

    println!("Failures: {}", failures.len());
    for failure in failures {
        println!("{:#?}", failure)
    }
}

fn is_as(node: Node) -> bool {
    node.grammar_id() == AS_OPERATOR_ID
}

// Inspired by: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<F>(mut cursor: TreeCursor, mut callback: F) -> Vec<FailureNode>
where
    F: FnMut(Node) -> bool,
{
    let mut failures: Vec<FailureNode> = Vec::new();
    loop {
        if callback(cursor.node()) {
            failures.push(cursor.node().into());
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
