#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::ffi::CString;
//use std::os::raw::c_char;
//use std::ptr;

fn main() {
    unsafe {
        let number_cstr = CString::new("number").unwrap();
        let number: *mut mpc_parser_t = mpc_new(number_cstr.as_ptr());

        let operator_cstr = CString::new("operator").unwrap();
        let operator: *mut mpc_parser_t = mpc_new(operator_cstr.as_ptr());

        let expr_cstr = CString::new("expr").unwrap();
        let expr: *mut mpc_parser_t = mpc_new(expr_cstr.as_ptr());

        let lispy_cstr = CString::new("lispy").unwrap();
        let lispy: *mut mpc_parser_t = mpc_new(lispy_cstr.as_ptr());

        let grammar_string = CString::new("
              number   : /-?[1-9][0-9]*/ ;                   \
              operator : '+' | '-' | '*' | '/' | '%' ;            \
              expr     : <number> | '(' <operator> <expr>+ ')' ;  \
              lispy    : /^/ <operator> <expr>+ /$/ ;             \
            ").unwrap();

        mpca_lang(0, grammar_string.as_ptr(), number, operator, expr, lispy);

        let mut rl = Editor::<()>::new();
        loop {
            let readline = rl.readline("lispy >> ");
            match readline {
                Ok(line) => {
                    if line == "exit" || line == "quit" {
                        break;
                    }
                    rl.add_history_entry(&line);
                    println!("{}", line);
                    let mut r: mpc_result_t = mpc_result_t { error: Default::default(), output:Default::default(), bindgen_union_field: 0u64};
                    let stdin_str = CString::new("<stdin>").unwrap();
                    let input = CString::new(line).unwrap();
                    if (mpc_parse(stdin_str.as_ptr(), input.as_ptr(), lispy, &mut r)) != 0 {
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

