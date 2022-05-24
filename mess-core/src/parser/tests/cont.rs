use crate::parser::Parser;

use std::{result::Result as StdResult, error::Error};

type Result = StdResult<(), Box<dyn Error>>;

#[test]
fn test_parse_cont_simple() -> Result {
    let code = "
    cont Vector {
        pub x: float;
        pub y: float;
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}

#[test]
fn test_parse_cont_fns() -> Result {
    let code = "
    cont Vector {
        fun calculate(&this) ~ float {
            return 0;
        }

        pub fun length(&this) ~ float {
            return 2.2;
        }
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}

#[test]
fn test_parse_cont_complex() -> Result {
    let code = "
    cont Vector {
        x: float;
        y: float;

        fun calculate(&this) ~ float {
            return 0;
        }

        pub fun length(&this) ~ float {
            return (x + y) * 3;
        }
    }
    ";

    let mut parser = Parser::new(code);
    let decl_list = parser.parse()?;
    println!("{:#?}", decl_list);
    Ok(())
}