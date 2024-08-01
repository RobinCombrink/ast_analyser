use std::fs;

use tree_sitter::{InputEdit, Language, Node, Parser, Point, TreeCursor};

const AS_OPERATOR_ID: u16 = 234;

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let source_code = fs::read("test_files/test.dart").unwrap();

    let tree = parser.parse(source_code, None).unwrap();

    let mut failures: Vec<Node> = vec![];

    traverse(tree.walk(), |node| is_as(node), &mut failures);
    println!("`as` count: {}", failures.len());

    for failure in failures {
        println!("{:#?}", failure);
    }
}

fn is_as<'a>(node: Node<'a>) -> Option<Node<'a>> {
    if node.grammar_id() == AS_OPERATOR_ID {
        return Some(node);
    }
    None
}

// Taken from: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<'a, F>(mut cursor: TreeCursor<'a>, mut callback: F, failures: &mut Vec<Node<'a>>)
where
    F: FnMut(Node) -> Option<Node>,
{
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
                return;
            }

            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
