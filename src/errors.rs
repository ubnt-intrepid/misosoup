#![allow(missing_docs)]

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {
        InvalidQuery {
            description("invalid query")
            display("invalid query")
        }
    }
}
