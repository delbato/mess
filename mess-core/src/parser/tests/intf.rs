use std::{result::Result as StdResult, error::Error};

use crate::parser::Parser;

type Result = StdResult<(), Box<dyn Error>>;

#[test]
fn test_parser_intf_simple() -> Result {
    let code = "
    intf ToString {
        fun to_string(&this) ~ String;
    }
    ";
    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}