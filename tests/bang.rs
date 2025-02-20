#[cfg(test)]
mod tests {
    use ast_analyser::failure_finder::FailureFinder;

    #[test]
    fn analysing_file_with_no_failures_returns_no_failures() {
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
}
