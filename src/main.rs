use std::fs;

use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree, TreeCursor};
use walkdir::{DirEntry, WalkDir};

const AS_OPERATOR_ID: u16 = 234;

fn main() {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .expect("Could not load Dart grammar");

    let mut all_failures: Vec<Node> = Vec::new();
    // match find_failures_for_file(
    //     &mut parser,
    let entry = WalkDir::new("./test_files/test.dart")
        .into_iter()
        .next()
        .unwrap()
        .unwrap();
    let source_code = fs::read(entry.path()).expect("Could not read path");
    let tree = parser.parse(&source_code, None).expect("Could not parse");
    let cursor = tree.walk();
    all_failures.append(&mut traverse(cursor, |node| is_as(node)));
    // ) {
    //     Some(mut failures) => all_failures.append(&mut failures),
    //     None => todo!(),
    // }
    // for entry in WalkDir::new("./") {
    //     let entry = match entry {
    //         Ok(entry) => entry,
    //         Err(_) => {
    //             // error!("Error walking directory: {}\n{e}", path.to_str().unwrap());
    //             continue;
    //         }
    //     };
    // }
    println!("`as` count: {}", &all_failures.len());

    for failure in &all_failures {
        println!("{:#?}", failure);
    }
}

// fn find_failures_for_file<'a>(parser: &mut Parser, entry: DirEntry) -> Option<Vec<Node>> {
//     if !entry.file_type().is_file() {
//         return None;
//     }
//     let source_code = fs::read(entry.path()).expect("Could not read path");
//     let tree = parser.parse(&source_code, None);
//     let tree = match tree {
//         Some(tree) => tree,
//         None => {
//             println!("Could not parse source code at path: {:#?}", entry.path());
//             return None;
//         }
//     };
// let failures = traverse(cursor, |node| is_as(node));
// return Some(failures);
// }
//
fn is_as(node: Node) -> Option<Node> {
    if node.grammar_id() == AS_OPERATOR_ID {
        return Some(node);
    }
    None
}

// Inspired by: https://github.com/skmendez/tree-sitter-traversal/blob/main/src/lib.rs
fn traverse<'a, F>(mut cursor: TreeCursor<'a>, mut callback: F) -> Vec<Node<'a>>
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
