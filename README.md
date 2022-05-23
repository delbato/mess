# mess - an embeddable scripting language for Rust

## What is this?

`mess` is a simple programming language intended to be embedded in other languages, primarly Rust and  
in the future, via the `mess-sys` crate, also C and other languages with a C interface.
This code is based on older projects of mine from around 2020.

## Why

Primarily to learn how parsers, lexers, compilers and bytecode interpreters work and play together.  
Also - why not?

## Language specifics

Nothing special really - syntax is similar to Rust and TypeScript.  
`mess` is statically typed and supports (non-checked) references, interfaces, structs (containers) etc.  

## Syntax sample

```
// Containers are like structs.
// They contain data and methods
cont Vector {
    // Member variable declaration
    pub x: float;
    pub y: float;

    // Static functions do not take a "this" parameter
    pub fun new(x: float, y: float) ~ Vector {
        return Vector {
            x: x,
            y: y
        };
    }

    // Member functions take a "this" parameter
    pub fun length(&this) ~ float {
        return float::sqrt((this.x * this.x) + (this.y * this.y));
    }
}

// Interfaces are... well, interfaces
intf Printable {
    fun print(&this);
}

// Syntax is inspired by rust
impl Printable for Vector {
    fun print(&this) {
        std::print(this.length());
    }
}

// The main entry point
fun main() {
    // Variable declarations support auto typing
    var vec = Vector::new(2.0, 3.0);
    // Or manually specified types
    var number: int = 4;
    // "on" statements are essentially "if"s
    on number > 4 {
        std::print("This is impossible!\n");
    } else {
        // Example of dynamic polymorphism
        var printable: &Printable = &vec;
        printable.print();
    }
}
```

## Project structure

* `mess` - Core crate that you will use when embedding in Rust
* `mess-api`- Contains structs, functions and macros for defining an API to use in scripts
* `mess-cli` - Simple CLI program for running script files
* `mess-core` - Internal crate containing common definitions
* `mess-jit` - AMD64 based JIT compiler
* `mess-vm` - Bytecode interpreter and accompanying compiler
* `mess-sys` - Crate for the (eventual) C bindings

## Further notes

This project is currently in a NON-WORKING state, as i am restructuring the entire codebase from my  
previous projects. Please be patient while i work this out.

## License

This project is licensed under the Apache v2 License.  
See `LICENSE.md` for details.
