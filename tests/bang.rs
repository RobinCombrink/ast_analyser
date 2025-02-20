#[cfg(test)]
mod tests {
    use ast_analyser::failure_finder::FailureFinder;

    #[test]
    fn analysing_file_that_does_not_exist_returns_err() {
        let mut failure_finder = FailureFinder::default();

        let failure = failure_finder.analyse_file("non_existent_file.dart".into());

        assert!(
            failure.is_err(),
            "File exists when when it shouldn't: {:#?}",
            failure
        );
    }

    #[test]
    fn analysing_file_with_no_failures_returns_none() {
        let mut failure_finder = FailureFinder::default();

        let failure = failure_finder.analyse_file("test_files/clean.dart".into());

        assert!(
            failure.is_ok(),
            "Returned Err when not expected: {:#?}",
            failure
        );
        assert!(
            failure.as_ref().unwrap().is_none(),
            "FailureFile had some errors when none were expected: {:#?}",
            failure
        );
    }

    #[test]
    fn analysing_file_with_failures_returns_failure_file() {
        let mut failure_finder = FailureFinder::default();

        let failure = failure_finder
            .analyse_file("test_files/bang/bang.dart".into())
            .unwrap()
            .unwrap();

        assert_eq!(
            failure.failure_nodes.len(),
            2,
            "File exists when when it shouldn't: {:#?}",
            failure
        );
    }
}
