use crate::parser::Parser;

use std::{result::Result as StdResult, error::Error};

type Result = StdResult<(), Box<dyn Error>>;
/*
#[test]
fn test_parse_enum_simple() -> Result {
    let code = "
    enum Test {
        Value1,
        Value2,
        Value3
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}

#[test]
fn test_parse_enum_tuples() -> Result {
    let code = "
    enum Test {
        Value1(int),
        Value2(String),
        Value3(str, float, String)
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}

#[test]
fn test_parse_enum_conts() -> Result {
    let code = "
    enum Test {
        Value1 {
            balls: int
        },
        Value2 {
            name: String,
            age: int
        },
        Value3 {
            name: str,
            city: String,
            age: int
        }
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}
*/
#[test]
fn test_parse_enum_complex() -> Result {
    let code = "
    enum Test {
        Value1,
        Value2(int, String),
        Value3 {
            name: str,
            city: String,
            age: int
        }
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}
