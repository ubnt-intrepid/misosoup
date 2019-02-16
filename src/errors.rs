#![allow(missing_docs)]

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        InvalidQuery {
            description("invalid query")
            display("invalid query")
        }

        InvalidRecord {
            description("invalid record")
            display("invalid record")
        }

        FailedSpeculativeParse {
            description("failed to parse in speculative parsing mode")
            display("failed to parse in speculative parsing mode")
        }
    }
}
