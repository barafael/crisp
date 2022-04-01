#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// #[allow(improper_ctypes)] // TODO: where to put this?
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use libc::{c_char, c_ulong, c_void};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::{
    ffi::CString,
    mem::{size_of, MaybeUninit},
    ptr::null_mut,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
struct LVal {
    ty: LValTag,
    num: i64,
    err: *mut c_char,
    sym: *mut c_char,
    count: usize,
    cell: *mut *mut LVal,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
enum LValTag {
    Num,
    Err,
    Sym,
    Sexpr,
}

unsafe fn lval_num(num: i64) -> *mut LVal {
    let v = malloc(size_of::<LVal>() as u64);
    let v = v as *mut LVal;
    (*v).ty = LValTag::Num;
    (*v).num = num;
    v
}

unsafe fn lval_err(err: *mut c_char) -> *mut LVal {
    let v = malloc(size_of::<LVal>() as u64) as *mut LVal;
    (*v).ty = LValTag::Err;
    (*v).err = malloc(strlen(err) + 1) as *mut c_char;
    strcpy((*v).err, err);
    v
}

unsafe fn lval_sym(sym: *mut c_char) -> *mut LVal {
    let v = malloc(size_of::<LVal>() as u64) as *mut LVal;
    (*v).ty = LValTag::Sym;
    (*v).sym = malloc(strlen(sym) + 1) as *mut c_char;
    strcpy((*v).sym, sym);
    v
}

unsafe fn lval_sexpr() -> *mut LVal {
    let v = malloc(size_of::<LVal>() as u64) as *mut LVal;
    (*v).ty = LValTag::Sexpr;
    (*v).count = 0;
    (*v).cell = null_mut();
    v
}

unsafe fn lval_del(v: *mut LVal) {
    match (*v).ty {
        LValTag::Num => {}
        LValTag::Err => free((*v).err as *mut c_void),
        LValTag::Sym => free((*v).sym as *mut c_void),
        LValTag::Sexpr => {
            for i in 0..(*v).count {
                lval_del((*(*v).cell).offset(i as isize));
            }
            free(*(*v).cell as *mut c_void);
        }
    }
    free(v as *mut c_void)
}

unsafe fn lval_add(v: *mut LVal, x: *mut LVal) -> *mut LVal {
    (*v).count += 1;
    (*v).cell = realloc(
        (*v).cell as *mut c_void,
        size_of::<*mut LVal>() as u64 * (*v).count as u64,
    ) as *mut *mut LVal;
    *(*v).cell.offset((*v).count as isize - 1) = x;
    v
}

unsafe fn lval_pop(v: *mut LVal, i: isize) -> *mut LVal {
    let x = *(*v).cell.offset(i);
    memmove(
        *(*v).cell.offset(i) as *mut c_void,
        *(*v).cell.offset(i + 1) as *mut c_void,
        (size_of::<*mut LVal>() * ((*v).count - i as usize - 1)) as u64,
    );

    (*v).count -= 1;

    (*v).cell = realloc(
        *(*v).cell as *mut c_void,
        (size_of::<*mut LVal>() * (*v).count) as c_ulong,
    ) as *mut *mut LVal;
    x
}

unsafe fn lval_take(v: *mut LVal, i: isize) -> *mut LVal {
    let x = lval_pop(v, i);
    lval_del(v);
    x
}

unsafe fn lval_expr_print(v: *mut LVal, open: c_char, close: c_char) {
    putchar(open as i32);
    for i in 0..(*v).count {
        lval_print(*(*v).cell.offset(i as isize));

        if i != (*v).count - 1 {
            putchar(' ' as i32);
        }
    }
    putchar(close as i32);
}

unsafe fn lval_print(v: *mut LVal) {
    match (*v).ty {
        LValTag::Num => {
            printf(b"%li".as_ptr() as *const _, (*v).num);
        }
        LValTag::Err => {
            printf(b"Error: %s".as_ptr() as *const _, (*v).err);
        }
        LValTag::Sym => {
            printf(b"%s".as_ptr() as *const _, (*v).sym);
        }
        LValTag::Sexpr => lval_expr_print(v, '(' as c_char, ')' as c_char),
    }
}

unsafe fn builtin_op(a: *mut LVal, op: *mut c_char) -> *mut LVal {
    /* Ensure all arguments are numbers */
    for i in 0..(*a).count {
        let n = *a;
        let cell = n.cell;
        let first = *cell;
        let offset = first.offset(i as isize);
        if (*offset).ty != LValTag::Num {
            lval_del(a);
            return lval_err(b"Cannot operate on non-number!".as_ptr() as *mut _);
        }
    }

    /* Pop the first element */
    let mut x: *mut LVal = lval_pop(a, 0);

    /* If no arguments and sub then perform unary negation */
    if (strcmp(op, b"-".as_ptr() as *const i8) == 0) && (*a).count == 0 {
        (*x).num = -(*x).num;
    }

    /* While there are still elements remaining */
    while (*a).count > 0 {
        /* Pop the next element */
        let y = lval_pop(a, 0);

        /* Perform operation */
        if strcmp(op, b"+".as_ptr() as *const _) == 0 {
            (*x).num += (*y).num;
        }
        if strcmp(op, b"-".as_ptr() as *const _) == 0 {
            (*x).num -= (*y).num;
        }
        if strcmp(op, b"*".as_ptr() as *const _) == 0 {
            (*x).num *= (*y).num;
        }
        if strcmp(op, b"/".as_ptr() as *const _) == 0 {
            if (*y).num == 0 {
                lval_del(x);
                lval_del(y);
                x = lval_err(b"Division By Zero.".as_ptr() as *mut _);
                break;
            }
            (*x).num /= (*y).num;
        }

        /* Delete element now finished with */
        lval_del(y);
    }

    /* Delete input expression and return result */
    lval_del(a);
    return x;
}

unsafe fn lval_eval_sexpr(v: *mut LVal) -> *mut LVal {
    /* Evaluate Children */
    for i in 0..(*v).count {
        *(*v).cell.offset(i as isize) = lval_eval(*(*v).cell.offset(i as isize));
    }

    /* Error Checking */
    for i in 0..(*v).count {
        if (**(*v).cell.offset(i as isize)).ty == LValTag::Err {
            return lval_take(v, i as isize);
        }
    }

    /* Empty Expression */
    if (*v).count == 0 {
        return v;
    }

    /* Single Expression */
    if (*v).count == 1 {
        return lval_take(v, 0);
    }

    /* Ensure First Element is Symbol */
    let f = lval_pop(v, 0);
    if (*f).ty != LValTag::Sym {
        lval_del(f);
        lval_del(v);
        return lval_err(b"S-expression does not start with symbol.".as_ptr() as *mut _);
    }

    /* Call builtin with operator */
    let result = builtin_op(v, (*f).sym);
    lval_del(f);
    return result;
}

unsafe fn lval_println(v: *mut LVal) {
    lval_print(v);
    putchar('\n' as i32);
}

unsafe fn lval_eval(v: *mut LVal) -> *mut LVal {
    if (*v).ty == LValTag::Sexpr {
        lval_eval_sexpr(v)
    } else {
        v
    }
}

unsafe fn lval_read_num(ast: *mut mpc_ast_t) -> *mut LVal {
    *__errno_location() = 0;
    let x = strtol((*ast).contents, null_mut(), 10);
    if *__errno_location() == ERANGE as i32 {
        lval_err(b"invalid number\0".as_ptr() as *mut _)
    } else {
        lval_num(x)
    }
}

unsafe fn lval_read(t: *mut mpc_ast_t) -> *mut LVal {
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

    let mut x = null_mut() as *mut LVal;
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
            //let raw_input = prompt_editor.readline("lispy >> ");
            let raw_input = Ok::<String, ReadlineError>("+ 2 2 (* 5 5)".into());
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
                        let evaluated = lval_eval(lval_read(reference));
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
