use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Error};

use tree_sitter::{Node, Parser, Point, TreeCursor};
use walkdir::{DirEntry, WalkDir};

const AS_OPERATOR_ID: u16 = 234;
const COMMENT_OPERATOR_ID: u16 = 410;

#[derive(Debug)]
struct FailureNode {
    grammar_name: String,
    grammar_id: u16,
    start_position: Point,
    end_position: Point,
}

impl TryInto<FailureNode> for (&'_ [u8], Node<'_>) {
    type Error = Error;

    fn try_into(self) -> Result<FailureNode, Self::Error> {
        let (source, node) = self;
        if node.grammar_id() != COMMENT_OPERATOR_ID {
            return Ok(FailureNode {
                grammar_name: node.grammar_name().to_owned(),
                grammar_id: node.grammar_id(),
                start_position: node.start_position(),
                end_position: node.end_position(),
            });
        }

        let text = node.utf8_text(source)?.to_lowercase();

        if text.contains("todo") {
            return Ok(FailureNode {
                grammar_name: node.grammar_name().to_owned(),
                grammar_id: node.grammar_id(),
                start_position: node.start_position(),
                end_position: node.end_position(),
            });
        }
        Err(anyhow!("The comment does not contain a TODO"))
    }
}

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let failures: Vec<FailureNode> = WalkDir::new("test_files/comment")
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
        .map(|source_code| {
            (
                source_code.clone(),
                parser.parse(source_code, None).expect("Could not parse"),
            )
        })
        .flat_map(|(source_code, tree)| {
            let cursor = tree.walk();
            traverse(&source_code, cursor, |node| {
                is_as(node.clone()) || is_comment(node.clone())
            })
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

fn is_comment(node: Node) -> bool {
    node.grammar_id() == COMMENT_OPERATOR_ID
}

// Inspired by: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<F>(source: &[u8], mut cursor: TreeCursor, mut callback: F) -> Vec<FailureNode>
where
    F: FnMut(Node) -> bool,
{
    let mut failures: Vec<FailureNode> = Vec::new();
    loop {
        // println!("name: {}, id: {}", cursor.node().grammar_name(), cursor.node().grammar_id());
        let node = cursor.node();
        if callback(node) {
            match (source, node).try_into() {
                Ok(failure) => failures.push(failure),
                Err(_) => {
                    //TODO It's not really an error
                },
            }
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
