use ast_analyser::failure_finder::FailureFinder;

#[test]
fn analysing_files_that_do_not_exist_return_err() {
    let failure_finder = FailureFinder::default();

    let failure = failure_finder.analyse_files(vec![
        "non_existent_file.dart".into(),
        "non_existent_file_2.dart".into(),
        "non_existent_file_3.dart".into(),
    ]);

    assert!(
        failure.is_err(),
        "File exists when when it shouldn't: {:#?}",
        failure
    );
}

#[test]
fn analysing_files_with_no_failures_returns_nones() {
    let failure_finder = FailureFinder::default();

    let failure = failure_finder.analyse_files(vec![
        "test_files/no_errors/clean.dart".into(),
        "test_files/no_errors/clean1.dart".into(),
    ]);

    assert!(
        failure.is_ok(),
        "Returned Err when not expected: {:#?}",
        failure
    );
    assert!(
        failure
            .as_ref()
            .unwrap()
            .iter()
            .all(|result| result.is_none()),
        "FailureFile had some errors when none were expected: {:#?}",
        failure
    );
}
#[test]
fn analysing_files_with_red_herrings_returns_nones() {
    let failure_finder = FailureFinder::default();

    let failure = failure_finder.analyse_files(vec![
        "test_files/no_errors/negation_operator.dart".into(),
        "test_files/no_errors/not_equals.dart".into(),
    ]);

    assert!(
        failure.is_ok(),
        "Returned Err when not expected: {:#?}",
        failure
    );
    assert!(
        failure
            .as_ref()
            .unwrap()
            .iter()
            .all(|result| result.is_none()),
        "FailureFile had some errors when none were expected: {:#?}",
        failure
    );
}

#[test]
fn analysing_files_with_failures_returns_failure_file() {
    let failure_finder = FailureFinder::default();

    let failure = failure_finder
        .analyse_files(vec![
            "test_files/bang/bang.dart".into(),
            "test_files/bang/bang_copy.dart".into(),
        ])
        .unwrap();

    assert!(
        failure.iter().all(|result| result.is_some()),
        "File found no errors in at least one file where there should be errors: {:#?}",
        failure
    );
    assert!(
        failure
            .iter()
            .filter_map(|result| result.clone())
            .all(|failure| failure.failure_nodes.len() > 0),
        "File found Some but had no failure nodes: {:#?}",
        failure
    );
}
