mod intf;

mod cont;

use crate::{parser::Parser, codegen::decl};
use std::{result::Result as StdResult, error::Error};

type Result = StdResult<(), Box<dyn Error>>;

#[test]
fn test_parse_empty_fn() {
    let code = "
    fun main() {
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list_res = parser.parse();
    assert!(decl_list_res.is_ok());
}

#[test]
fn test_parse_simple_fn() {
    let code = "
    fun main() {
        var x: int = 4;
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list_res = parser.parse();
    assert!(decl_list_res.is_ok());
}

#[test]
fn test_parse_simple_expr() {
    let code = "
    fun main() {
        var x: int = (4 / 2) * 3;
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list_res = parser.parse();
    assert!(decl_list_res.is_ok());
}

#[test]
fn test_parse_simple_on() -> Result {
    let code = "
    fun main() {
        var x: int = 4;
        on x == 4 {
            x = x - 2;
        } else {
            x = x + 2;
        }
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}

#[test]
fn test_parse_complex_on() -> Result {
    let code = "
    fun main() {
        var x: int = 4;
        on x == 4 {
            x = x - 2;
        } else on x <= 2 {
            x = x + 2;
        } else {
            x = 7;
        }
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}
