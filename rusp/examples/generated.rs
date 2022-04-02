#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::{
    ffi::CString,
    mem::{size_of, MaybeUninit},
    ptr::null_mut,
};

use libc::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use rustyline::{error::ReadlineError, Editor};

use ExprType::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ExprType {
    LISPVAL_NUM = 0,
    LISPVAL_ERR = 1,
    LISPVAL_SYM = 2,
    LISPVAL_SEXPR = 3,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct LispValue {
    type_0: ExprType,
    num: c_long,
    err: *mut c_char,
    symbol: *mut c_char,
    count: c_int,
    cell: *mut *mut LispValue,
}

/* Construct a number */
unsafe fn lval_num(num: c_long) -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).type_0 = LISPVAL_NUM;
    (*v).num = num;
    v
}

/* Construct an error */
unsafe fn lval_err(error: *mut c_char) -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).type_0 = LISPVAL_ERR;
    (*v).err = malloc(strlen(error).wrapping_add(1 as c_int as c_ulong)) as *mut c_char;
    strcpy((*v).err, error);
    return v;
}

/* Construct a new symbol */
unsafe fn lval_sym(sym: *mut c_char) -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).type_0 = LISPVAL_SYM;
    (*v).symbol = malloc(strlen(sym).wrapping_add(1 as c_int as c_ulong)) as *mut c_char;
    strcpy((*v).symbol, sym);
    return v;
}

/* Construct new empty sexpr */
unsafe fn lval_sexpr() -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).type_0 = LISPVAL_SEXPR;
    (*v).count = 0 as c_int;
    (*v).cell = 0 as *mut *mut LispValue;
    return v;
}

/* Clean up a lispval */
unsafe fn lval_del(val: *mut LispValue) {
    match (*val).type_0 as c_uint {
        1 => {
            free((*val).err as *mut c_void);
        }
        2 => {
            free((*val).symbol as *mut c_void);
        }
        3 => {
            /* If Sexpr then delete all elements inside */
            let mut i: c_int = 0 as c_int;
            while i < (*val).count {
                lval_del(*(*val).cell.offset(i as isize));
                i += 1
            }
        }
        0 | _ => {}
    }
    /* Free the entire struct finally */
    free(val as *mut c_void);
}

unsafe fn lval_add(mut val: *mut LispValue, x: *mut LispValue) -> *mut LispValue {
    (*val).count += 1;
    (*val).cell = realloc(
        (*val).cell as *mut c_void,
        (size_of::<*mut LispValue>() as c_ulong).wrapping_mul((*val).count as c_ulong),
    ) as *mut *mut LispValue;
    let ref mut fresh0 = *(*val).cell.offset(((*val).count - 1 as c_int) as isize);
    *fresh0 = x;
    return val;
}

unsafe fn lval_read(t: *mut mpc_ast_t) -> *mut LispValue {
    let number = b"number\0".as_ptr() as *const i8;
    let symbol = b"symbol\0".as_ptr() as *const i8;

    if !strstr((*t).tag, number).is_null() {
        return lval_read_num(t);
    }
    if !strstr((*t).tag, symbol).is_null() {
        return lval_sym((*t).contents);
    }

    let root = b">\0".as_ptr() as *const i8;
    let sexpr = b"sexpr\0".as_ptr() as *const i8;
    let opening = b"(\0".as_ptr() as *const i8;
    let closing = b")\0".as_ptr() as *const i8;
    let regex = b"regex\0".as_ptr() as *const i8;

    let mut x = null_mut() as *mut LispValue;
    if strcmp((*t).tag, root) == 0 {
        x = lval_sexpr();
    }
    if !strstr((*t).tag, sexpr).is_null() {
        x = lval_sexpr();
    }

    for i in 0..(*t).children_num {
        if strcmp((**(*t).children.offset(i as isize)).contents, opening) == 0 {
            continue;
        }
        if strcmp((**(*t).children.offset(i as isize)).contents, closing) == 0 {
            continue;
        }
        if strcmp((**(*t).children.offset(i as isize)).tag, regex) == 0 {
            continue;
        }
        x = lval_add(x, lval_read(*(*t).children.offset(i as isize)));
    }
    x
}

unsafe fn lval_read_num(ast: *mut mpc_ast_t) -> *mut LispValue {
    *__errno_location() = 0;
    let x = strtol((*ast).contents, null_mut(), 10);
    if *__errno_location() == ERANGE as i32 {
        lval_err(b"invalid number\0".as_ptr() as *mut _)
    } else {
        lval_num(x)
    }
}

/* Print an lispval */
unsafe fn lval_expr_print(v: *mut LispValue, open: c_char, close: c_char) {
    putchar(open as c_int);
    let mut i: c_int = 0 as c_int;
    while i < (*v).count {
        lval_print(*(*v).cell.offset(i as isize));
        if i != (*v).count - 1 as c_int {
            putchar(' ' as i32);
        }
        i += 1
    }
    putchar(close as c_int);
}

unsafe fn lval_print(v: *mut LispValue) {
    match (*v).type_0 as c_uint {
        0 => {
            printf(b"%li\x00" as *const u8 as *const c_char, (*v).num);
        }
        1 => {
            printf(b"Error: %s\x00" as *const u8 as *const c_char, (*v).err);
        }
        2 => {
            printf(b"%s\x00" as *const u8 as *const c_char, (*v).symbol);
        }
        3 => {
            lval_expr_print(v, '(' as i32 as c_char, ')' as i32 as c_char);
        }
        _ => {}
    };
}

unsafe fn lval_println(v: *mut LispValue) {
    lval_print(v);
    putchar('\n' as i32);
}

unsafe fn lval_pop(mut v: *mut LispValue, i: c_int) -> *mut LispValue {
    /* Find item at i */
    let x: *mut LispValue = *(*v).cell.offset(i as isize);
    /* Shift memory after the item at i over the top */
    memmove(
        &mut *(*v).cell.offset(i as isize) as *mut *mut LispValue as *mut c_void,
        &mut *(*v).cell.offset((i + 1 as c_int) as isize) as *mut *mut LispValue as *const c_void,
        (size_of::<*mut LispValue>() as c_ulong)
            .wrapping_mul(((*v).count - i - 1 as c_int) as c_ulong),
    );
    (*v).count -= 1;
    (*v).cell = realloc(
        (*v).cell as *mut c_void,
        (size_of::<*mut LispValue>() as c_ulong).wrapping_mul((*v).count as c_ulong),
    ) as *mut *mut LispValue;
    return x;
}

unsafe fn lval_take(v: *mut LispValue, i: c_int) -> *mut LispValue {
    let x: *mut LispValue = lval_pop(v, i);
    lval_del(v);
    return x;
}

unsafe fn lval_eval_sexpr(v: *mut LispValue) -> *mut LispValue {
    /* Evaluate children */
    let mut i: c_int = 0 as c_int;
    while i < (*v).count {
        let ref mut fresh1 = *(*v).cell.offset(i as isize);
        *fresh1 = lval_eval(*(*v).cell.offset(i as isize));
        i += 1
    }
    /* Error Checking */
    let mut i_0: c_int = 0 as c_int;
    while i_0 < (*v).count {
        if (**(*v).cell.offset(i_0 as isize)).type_0 as c_uint == LISPVAL_ERR as c_int as c_uint {
            return lval_take(v, i_0);
        }
        i_0 += 1
    }
    /* Empty expression () */
    if (*v).count == 0 as c_int {
        return v;
    }
    /* Single expression */
    if (*v).count == 1 as c_int {
        return lval_take(v, 0 as c_int);
    }
    /* Remaining must be sexpr, first of it must be a symbol */
    let f: *mut LispValue = lval_pop(v, 0 as c_int);
    if (*f).type_0 as c_uint != LISPVAL_SYM as c_int as c_uint {
        lval_del(f);
        lval_del(v);
        return lval_err(
            b"S-Expression does not start with symbol!\x00" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    /* Call builtin with operator */
    let result: *mut LispValue = builtin_op(v, (*f).symbol);
    lval_del(f);
    return result;
}

unsafe fn builtin_op(a: *mut LispValue, op: *mut c_char) -> *mut LispValue {
    let mut i: c_int = 0 as c_int;
    while i < (*a).count {
        if (**(*a).cell.offset(i as isize)).type_0 as c_uint != LISPVAL_NUM as c_int as c_uint {
            lval_del(a);
            return lval_err(
                b"Cannot operate on non-number!\x00" as *const u8 as *const c_char as *mut c_char,
            );
        }
        i += 1
    }
    let mut x: *mut LispValue = lval_pop(a, 0 as c_int);
    /* If no arguments and sub('-') then perform unary negation */
    if strcmp(op, b"-\x00" as *const u8 as *const c_char) == 0 as c_int && (*a).count == 0 as c_int
    {
        (*x).num = -(*x).num
    }
    while (*a).count > 0 as c_int {
        let y: *mut LispValue = lval_pop(a, 0 as c_int);
        if strcmp(op, b"+\x00" as *const u8 as *const c_char) == 0 as c_int {
            (*x).num += (*y).num
        }
        if strcmp(op, b"-\x00" as *const u8 as *const c_char) == 0 as c_int {
            (*x).num -= (*y).num
        }
        if strcmp(op, b"*\x00" as *const u8 as *const c_char) == 0 as c_int {
            (*x).num *= (*y).num
        }
        if !(strcmp(op, b"/\x00" as *const u8 as *const c_char) == 0 as c_int) {
            continue;
        }
        if (*y).num == 0 as c_int as c_long {
            lval_del(x);
            lval_del(y);
            x = lval_err(b"Division by zero!\x00" as *const u8 as *const c_char as *mut c_char);
            break;
        } else {
            (*x).num /= (*y).num
        }
    }
    lval_del(a);
    return x;
}

unsafe fn lval_eval(v: *mut LispValue) -> *mut LispValue {
    /* Evaluate S-expressions */
    if (*v).type_0 as c_uint == LISPVAL_SEXPR as c_int as c_uint {
        return lval_eval_sexpr(v);
    }
    /* Treat all other types the same */
    return v;
}

fn main() {
    // // println!("12-13: {}", eval_op(12i64, "-", 13i64));
    // All functions from mpc are considered unsafe
    unsafe {
        /* Version and exit information */
        println!("Lispy version 0.0.4");
        println!("Press CTRL-C to exit");

        println!("Example expression: * 2 2 or * (+ 1 5) (* 1 3 7)");

        // Define parsers by name. b"...\0".as_ptr() as *const creates a C-String lookalike.
        let number: *mut mpc_parser_t = mpc_new(b"number\0".as_ptr() as *const _);
        let symbol: *mut mpc_parser_t = mpc_new(b"symbol\0".as_ptr() as *const _);
        let sexpr: *mut mpc_parser_t = mpc_new(b"sexpr\0".as_ptr() as *const _);
        let expr: *mut mpc_parser_t = mpc_new(b"expr\0".as_ptr() as *const _);
        let lispy: *mut mpc_parser_t = mpc_new(b"lispy\0".as_ptr() as *const _);

        // Define grammar
        let grammar_string = b"
              number : /-?[0-9]+/ ;                       \
              symbol : '+' | '-' | '*' | '/' | '%' ;      \
              sexpr  : '(' <expr>* ')' ;                  \
              expr   : <number> | <symbol> | <sexpr> ;    \
              lispy  : /^/ <expr>* /$/ ;                  \
            \0"
        .as_ptr() as *const _;

        // Generate lispy language
        mpca_lang(0, grammar_string, number, symbol, sexpr, expr, lispy);

        let mut prompt_editor = Editor::<()>::new();
        loop {
            let raw_input = prompt_editor.readline("lispy >> ");
            //let raw_input = Ok::<String, ReadlineError>("+ 2 2".into());
            match raw_input {
                Ok(line) => {
                    if line == "exit" || line == "quit" {
                        break;
                    }
                    /* Add line to command-line history */
                    prompt_editor.add_history_entry(&line);

                    /* Initialize `result` with default members
                    The Default::default() method provides a useful default for a type */
                    let mut result = MaybeUninit::zeroed().assume_init();
                    let stdin_cstr = b"<stdin>\0".as_ptr() as *const _;
                    let input = CString::new(line).unwrap();

                    /* Parse input, writing in the result */
                    if (mpc_parse(stdin_cstr, input.as_ptr(), lispy, &mut result)) != 0 {
                        /* Success - print the AST */
                        mpc_ast_print(result.output as *mut mpc_ast_t);

                        let reference = result.output as *mut mpc_ast_t;
                        let tree = lval_read(reference);

                        let evaluated = lval_eval(tree);

                        lval_println(evaluated);
                        lval_del(evaluated);
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
        mpc_cleanup(5, number, symbol, sexpr, expr, lispy);
    }
}
