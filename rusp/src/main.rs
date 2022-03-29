#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// #[allow(improper_ctypes)] // TODO: where to put this?
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::ffi::CString;

fn eval_op(x: i64, op: &str, y: i64) -> i64 {
    match op {
        "+" => x + y,
        "-" => x - y,
        "*" => x * y,
        "/" => x / y,
        "%" => x % y,
        _ => {
            println!("Unknown operator: {}", op);
            0
        } // TODO fix C-style 0 return
    }
}

fn eval(t: &mpc_ast_t) -> i64 {
    let number = b"number\0".as_ptr() as *const _;
    let expr = b"expr\0".as_ptr() as *const _;

    if unsafe { strstr(t.tag, number) } != std::ptr::null_mut() {
        return unsafe { atoi(t.contents) } as i64;
    }

    let op = unsafe { &**t.children.offset(1) }.contents;
    let op = unsafe { std::ffi::CStr::from_ptr(op).to_str().unwrap() };

    let mut x = eval(unsafe { &**t.children.offset(2) });

    let mut i = 3;
    loop {
        let tag = unsafe { &**t.children.offset(i as isize) }.tag;
        if unsafe { strstr(tag, expr) } == std::ptr::null_mut() {
            break;
        }
        x = eval_op(x, op, eval(unsafe { &**t.children.offset(i as isize) }));
        i += 1;
    }
    x
}

fn number_of_nodes(t: &mpc_ast_t) -> usize {
    match t.children_num {
        0 => 1,
        n => {
            let mut total = 1;
            for i in 0..n {
                total += number_of_nodes(unsafe { &**t.children.offset(i as isize) });
            }
            total
        }
    }
}

fn main() {
    // // println!("12-13: {}", eval_op(12i64, "-", 13i64));
    // All functions from mpc are considered unsafe
    unsafe {
        /* Version and exit information */
        println!("Lispy version 0.0.0.0.1");
        println!("Press CTRL-C to exit");

        println!("Example expression: * 2 2 or * (+ 1 5) (* 1 3 7)");

        // Define parsers by name. b"...\0".as_ptr() as *const creates a C-String lookalike.
        let number: *mut mpc_parser_t = mpc_new(b"number\0".as_ptr() as *const _);
        let operator: *mut mpc_parser_t = mpc_new(b"operator\0".as_ptr() as *const _);
        let expr: *mut mpc_parser_t = mpc_new(b"expr\0".as_ptr() as *const _);
        let lispy: *mut mpc_parser_t = mpc_new(b"lispy\0".as_ptr() as *const _);

        // Define grammar
        let grammar_string = b"
              number   : /-?[1-9][0-9]*/ ;                        \
              operator : '+' | '-' | '*' | '/' | '%' ;            \
              expr     : <number> | '(' <operator> <expr>+ ')' ;  \
              lispy    : /^/ <operator> <expr>+ /$/ ;             \
            \0"
        .as_ptr() as *const _;

        // Generate lispy language
        mpca_lang(0, grammar_string, number, operator, expr, lispy);

        let mut prompt_editor = Editor::<()>::new();
        loop {
            let raw_input = prompt_editor.readline("lispy >> ");
            match raw_input {
                Ok(line) => {
                    if line == "exit" || line == "quit" {
                        break;
                    }
                    /* Add line to command-line history */
                    prompt_editor.add_history_entry(&line);

                    /* Initialize `result` with default members
                    The Default::default() method provides a useful default for a type */
                    let mut result = std::mem::MaybeUninit::zeroed().assume_init();
                    let stdin_cstr = b"<stdin>\0".as_ptr() as *const _;
                    let input = CString::new(line).unwrap();

                    /* Parse input, writing in the result */
                    if (mpc_parse(stdin_cstr, input.as_ptr(), lispy, &mut result)) != 0 {
                        /* Success - print the AST */
                        mpc_ast_print(result.output as *mut mpc_ast_t);
                        let reference = result.output as *const mpc_ast_t;
                        println!("{}", number_of_nodes(&*reference));
                        println!("{}", eval(&*reference));
                        mpc_ast_delete(result.output as *mut mpc_ast_t);
                    } else {
                        /* Not parsed. Print error */
                        mpc_err_print(result.error);
                        mpc_err_delete(result.error);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        /* Clean up the malloc'd ressources */
        mpc_cleanup(4, number, operator, expr, lispy);
    }
}
