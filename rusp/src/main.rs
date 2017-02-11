#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// #[allow(improper_ctypes)] // TODO: where to put this?
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::ffi::CString;

fn main() {
    unsafe {
        let number: *mut mpc_parser_t = mpc_new(b"number\0".as_ptr() as *const _);
        let operator: *mut mpc_parser_t = mpc_new(b"operator\0".as_ptr() as * const _);
        let expr: *mut mpc_parser_t = mpc_new(b"expr\0".as_ptr() as * const _);
        let lispy: *mut mpc_parser_t = mpc_new(b"lispy\0".as_ptr() as  *const _);

        let grammar_string = b"
              number   : /-?[1-9][0-9]*/ ;                   \
              operator : '+' | '-' | '*' | '/' | '%' ;            \
              expr     : <number> | '(' <operator> <expr>+ ')' ;  \
              lispy    : /^/ <operator> <expr>+ /$/ ;             \
            \0".as_ptr() as *const _;

        mpca_lang(0, grammar_string, number, operator, expr, lispy);

        let mut rl = Editor::<()>::new();
        loop {
            let readline = rl.readline("lispy >> ");
            match readline {
                Ok(line) => {
                    if line == "exit" || line == "quit" {
                        break;
                    }
                    rl.add_history_entry(&line);
                    // println!("{}", line);
                    let mut r: mpc_result_t = mpc_result_t { error: Default::default(), output:Default::default(), bindgen_union_field: 0u64};
                    let stdin_cstr = b"<stdin>\0".as_ptr() as *const _;
                    let input = CString::new(line).unwrap();
                    if (mpc_parse(stdin_cstr, input.as_ptr(), lispy, &mut r)) != 0 {
                        /* Success - print the AST */
                        mpc_ast_print(*r.output.as_ref() as *mut mpc_ast_t);
                        mpc_ast_delete(*r.output.as_ref() as *mut mpc_ast_t);
                    } else {
                        /* Not parsed. Print error */
                        mpc_err_print(*r.error.as_ref());
                        mpc_err_delete(*r.error.as_ref());
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
        mpc_cleanup(4, number, operator, expr, lispy);
    }
}

