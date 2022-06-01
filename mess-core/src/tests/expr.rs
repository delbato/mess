use crate::parser::Parser;

use std::{result::Result as StdResult, error::Error};

type Result = StdResult<(), Box<dyn Error>>;

#[test]
fn test_parse_fn_call() -> Result {
    let code = r#"
    fun main() {
        var x = call("four");
    }
    "#;

    let mut parser = Parser::new(code);
    let decl_list_res = parser.parse();
    assert!(decl_list_res.is_ok());
    Ok(())
}