#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use libc::{c_char, c_int, c_long, c_ulong, c_void};
use rustyline::{error::ReadlineError, Editor};
use std::{
    ffi::CString,
    mem::{size_of, MaybeUninit},
    ptr::null_mut,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
struct LispValue {
    tag: Tag,
    num: c_long,
    err: *mut c_char,
    sym: *mut c_char,
    count: c_int,
    cell: *mut *mut LispValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
enum Tag {
    Num,
    Err,
    Sym,
    Sexpr,
    Qexpr,
}

/* Construct a number */
unsafe fn lval_num(x: c_long) -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).tag = Tag::Num;
    (*v).num = x;
    v
}

/* Construct an error */
unsafe fn lval_err(m: *mut c_char) -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).tag = Tag::Err;
    (*v).err = malloc(strlen(m).wrapping_add(1)) as *mut c_char;
    strcpy((*v).err, m);
    v
}

/* Construct a new symbol */
unsafe fn lval_sym(s: *mut c_char) -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).tag = Tag::Sym;
    (*v).sym = malloc(strlen(s).wrapping_add(1)) as *mut c_char;
    strcpy((*v).sym, s);
    v
}

/* Construct new empty sexpr */
unsafe fn lval_sexpr() -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).tag = Tag::Sexpr;
    (*v).count = 0;
    (*v).cell = null_mut();
    v
}

/* Construct new empty qexpr */
unsafe fn lval_qexpr() -> *mut LispValue {
    let mut v: *mut LispValue = malloc(size_of::<LispValue>() as c_ulong) as *mut LispValue;
    (*v).tag = Tag::Qexpr;
    (*v).count = 0;
    (*v).cell = null_mut();
    v
}

/* Clean up a lispval */
unsafe fn lval_del(val: *mut LispValue) {
    match (*val).tag {
        Tag::Num => {}
        Tag::Err => free((*val).err as *mut c_void),
        Tag::Sym => free((*val).sym as *mut c_void),
        Tag::Sexpr | Tag::Qexpr => {
            /* If Qexpr or Sexpr then delete all elements inside */
            let mut i: c_int = 0;
            while i < (*val).count {
                lval_del(*(*val).cell.offset(i as isize));
                i += 1
            }
            /* Also free the memory allocated to contain the pointers */
            free((*val).cell as *mut c_void);
        }
    }
    /* Free the entire struct finally */
    free(val as *mut c_void);
}

unsafe fn lval_add(val: *mut LispValue, x: *mut LispValue) -> *mut LispValue {
    (*val).count += 1;
    (*val).cell = realloc(
        (*val).cell as *mut c_void,
        (size_of::<*mut LispValue>() as c_ulong).wrapping_mul((*val).count as c_ulong),
    ) as *mut *mut LispValue;
    let fresh = &mut (*(*val).cell.offset(((*val).count - 1) as isize));
    *fresh = x;
    val
}

unsafe fn lval_pop(mut v: *mut LispValue, i: c_int) -> *mut LispValue {
    let x: *mut LispValue = *(*v).cell.offset(i as isize);
    memmove(
        &mut *(*v).cell.offset(i as isize) as *mut *mut LispValue as *mut c_void,
        &mut *(*v).cell.offset((i + 1) as isize) as *mut *mut LispValue as *const c_void,
        (size_of::<*mut LispValue>() as c_ulong).wrapping_mul(((*v).count - i - 1) as c_ulong),
    );
    (*v).count -= 1;
    (*v).cell = realloc(
        (*v).cell as *mut c_void,
        (size_of::<*mut LispValue>() as c_ulong).wrapping_mul((*v).count as c_ulong),
    ) as *mut *mut LispValue;
    x
}

unsafe fn lval_take(v: *mut LispValue, i: c_int) -> *mut LispValue {
    let x: *mut LispValue = lval_pop(v, i);
    lval_del(v);
    x
}

/* Print a lispval */
unsafe fn lval_expr_print(v: *mut LispValue, open: c_char, close: c_char) {
    putchar(open as c_int);
    let mut i: c_int = 0;
    while i < (*v).count {
        lval_print(*(*v).cell.offset(i as isize));
        if i != (*v).count - 1 {
            putchar(' ' as i32);
        }
        i += 1
    }
    putchar(close as c_int);
}

unsafe fn lval_print(v: *mut LispValue) {
    match (*v).tag {
        Tag::Num => {
            printf(b"%li\0" as *const u8 as *const c_char, (*v).num);
        }
        Tag::Err => {
            printf(b"Error: %s\0" as *const u8 as *const c_char, (*v).err);
        }
        Tag::Sym => {
            printf(b"%s\0" as *const u8 as *const c_char, (*v).sym);
        }
        Tag::Sexpr => lval_expr_print(v, '(' as c_char, ')' as c_char),
        Tag::Qexpr => lval_expr_print(v, '{' as c_char, '}' as c_char),
    };
}

unsafe fn builtin_op(a: *mut LispValue, op: *mut c_char) -> *mut LispValue {
    let mut i: c_int = 0;
    while i < (*a).count {
        if (**(*a).cell.offset(i as isize)).tag != Tag::Num {
            lval_del(a);
            return lval_err(
                b"Cannot operate on non-number!\0" as *const u8 as *const c_char as *mut c_char,
            );
        }
        i += 1
    }

    /* Pop the first element */
    let mut x: *mut LispValue = lval_pop(a, 0);

    /* If no arguments and sub then perform unary negation */
    if strcmp(op, b"-\0" as *const u8 as *const c_char) == 0 && (*a).count == 0 {
        (*x).num = -(*x).num;
    }

    /* While there are still elements remaining */
    while (*a).count > 0 {
        let y: *mut LispValue = lval_pop(a, 0);
        if strcmp(op, b"+\0" as *const u8 as *const c_char) == 0 {
            (*x).num += (*y).num;
        }
        if strcmp(op, b"-\0" as *const u8 as *const c_char) == 0 {
            (*x).num -= (*y).num;
        }
        if strcmp(op, b"*\0" as *const u8 as *const c_char) == 0 {
            (*x).num *= (*y).num;
        }
        if strcmp(op, b"/\0" as *const u8 as *const c_char) == 0 {
            if (*y).num == 0 {
                lval_del(x);
                lval_del(y);
                x = lval_err(b"Division by zero.\0" as *const u8 as *const c_char as *mut c_char);
                break;
            } else {
                (*x).num /= (*y).num;
            }
        }
        lval_del(y);
    }
    lval_del(a);
    x
}

unsafe fn lval_println(v: *mut LispValue) {
    lval_print(v);
    putchar('\n' as i32);
}

unsafe fn builtin_list(mut a: *mut LispValue) -> *mut LispValue {
    (*a).tag = Tag::Qexpr;
    a
}

unsafe fn builtin_head(a: *mut LispValue) -> *mut LispValue {
    if (*a).count != 1 {
        lval_del(a);
        return lval_err(
            b"Function \'head\' passed too many arguments.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    if !((**(*a).cell.offset(0)).tag == Tag::Qexpr) {
        lval_del(a);
        return lval_err(
            b"Function \'head\' passed incorrect type.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    if (**(*a).cell.offset(0)).count == 0 {
        lval_del(a);
        return lval_err(
            b"Function \'head\' passed {}.\0" as *const u8 as *const c_char as *mut c_char,
        );
    }
    let v: *mut LispValue = lval_take(a, 0);
    while (*v).count > 1 {
        lval_del(lval_pop(v, 1));
    }
    v
}

unsafe fn builtin_tail(a: *mut LispValue) -> *mut LispValue {
    if (*a).count != 1 {
        lval_del(a);
        return lval_err(
            b"Function \'tail\' passed too many arguments.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    if (**(*a).cell.offset(0)).tag != Tag::Qexpr {
        lval_del(a);
        return lval_err(
            b"Function \'tail\' passed incorrect type.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    if (**(*a).cell.offset(0)).count == 0 {
        lval_del(a);
        return lval_err(
            b"Function \'tail\' passed {}.\0" as *const u8 as *const c_char as *mut c_char,
        );
    }
    let v: *mut LispValue = lval_take(a, 0);
    lval_del(lval_pop(v, 0));
    v
}

unsafe fn builtin_eval(a: *mut LispValue) -> *mut LispValue {
    if (*a).count != 1 {
        lval_del(a);
        return lval_err(
            b"Function \'eval\' passed too many arguments.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    if !((**(*a).cell.offset(0)).tag == Tag::Qexpr) {
        lval_del(a);
        return lval_err(
            b"Function \'eval\' passed incorrect type.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    let mut x: *mut LispValue = lval_take(a, 0);
    (*x).tag = Tag::Sexpr;
    lval_eval(x)
}

unsafe fn builtin_join(a: *mut LispValue) -> *mut LispValue {
    let mut i: c_int = 0;
    while i < (*a).count {
        if !((**(*a).cell.offset(i as isize)).tag == Tag::Qexpr) {
            lval_del(a);
            return lval_err(
                b"Function \'join\' passed incorrect type.\0" as *const u8 as *const c_char
                    as *mut c_char,
            );
        }
        i += 1
    }
    let mut x: *mut LispValue = lval_pop(a, 0);
    while (*a).count != 0 {
        x = lval_join(x, lval_pop(a, 0))
    }
    lval_del(a);
    x
}

unsafe fn lval_join(mut x: *mut LispValue, y: *mut LispValue) -> *mut LispValue {
    while (*y).count != 0 {
        x = lval_add(x, lval_pop(y, 0))
    }
    lval_del(y);
    x
}

unsafe fn builtin(a: *mut LispValue, func: *mut c_char) -> *mut LispValue {
    if strcmp(b"list\0" as *const u8 as *const c_char, func) == 0 {
        return builtin_list(a);
    }
    if strcmp(b"head\0" as *const u8 as *const c_char, func) == 0 {
        return builtin_head(a);
    }
    if strcmp(b"tail\0" as *const u8 as *const c_char, func) == 0 {
        return builtin_tail(a);
    }
    if strcmp(b"join\0" as *const u8 as *const c_char, func) == 0 {
        return builtin_join(a);
    }
    if strcmp(b"eval\0" as *const u8 as *const c_char, func) == 0 {
        return builtin_eval(a);
    }
    if !strstr(b"+-/*\0" as *const u8 as *const c_char, func).is_null() {
        return builtin_op(a, func);
    }
    lval_del(a);
    lval_err(b"Unknown Function!\0" as *const u8 as *const c_char as *mut c_char)
}

unsafe fn lval_eval_sexpr(v: *mut LispValue) -> *mut LispValue {
    let mut i: c_int = 0;
    while i < (*v).count {
        let fresh = &mut (*(*v).cell.offset(i as isize));
        *fresh = lval_eval(*(*v).cell.offset(i as isize));
        i += 1
    }
    let mut i_0: c_int = 0;
    while i_0 < (*v).count {
        if (**(*v).cell.offset(i_0 as isize)).tag == Tag::Err {
            return lval_take(v, i_0);
        }
        i_0 += 1
    }
    if (*v).count == 0 {
        return v;
    }
    if (*v).count == 1 {
        return lval_take(v, 0);
    }
    let f: *mut LispValue = lval_pop(v, 0);
    if (*f).tag != Tag::Sym {
        lval_del(f);
        lval_del(v);
        return lval_err(
            b"S-expression Does not start with symbol.\0" as *const u8 as *const c_char
                as *mut c_char,
        );
    }
    /* Call builtin with operator */
    let result: *mut LispValue = builtin(v, (*f).sym);
    lval_del(f);
    result
}

unsafe fn lval_eval(v: *mut LispValue) -> *mut LispValue {
    if (*v).tag == Tag::Sexpr {
        return lval_eval_sexpr(v);
    }
    v
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
    let qexpr = b"qexpr\0".as_ptr() as *const i8;
    let opening = b"(\0".as_ptr() as *const i8;
    let closing = b")\0".as_ptr() as *const i8;
    let opening_curly = b"{\0".as_ptr() as *const i8;
    let closing_curly = b"}\0".as_ptr() as *const i8;
    let regex = b"regex\0".as_ptr() as *const i8;

    let mut x = null_mut() as *mut LispValue;
    if strcmp((*t).tag, root) == 0 {
        x = lval_sexpr();
    }
    if !strstr((*t).tag, sexpr).is_null() {
        x = lval_sexpr();
    }
    if !strstr((*t).tag, qexpr).is_null() {
        x = lval_qexpr();
    }

    for i in 0..(*t).children_num {
        if strcmp((**(*t).children.offset(i as isize)).contents, opening) == 0 {
            continue;
        }
        if strcmp((**(*t).children.offset(i as isize)).contents, closing) == 0 {
            continue;
        }
        if strcmp((**(*t).children.offset(i as isize)).contents, opening_curly) == 0 {
            continue;
        }
        if strcmp((**(*t).children.offset(i as isize)).contents, closing_curly) == 0 {
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
        println!("Lispy version 0.0.5");
        println!("Press CTRL-C to exit");

        println!("Example expression: * 2 2 or * (+ 1 5) (* 1 3 7)");

        // Define parsers by name. b"...\0".as_ptr() as *const creates a C-String lookalike.
        let number: *mut mpc_parser_t = mpc_new(b"number\0".as_ptr() as *const _);
        let symbol: *mut mpc_parser_t = mpc_new(b"symbol\0".as_ptr() as *const _);
        let sexpr: *mut mpc_parser_t = mpc_new(b"sexpr\0".as_ptr() as *const _);
        let qexpr: *mut mpc_parser_t = mpc_new(b"qexpr\0".as_ptr() as *const _);
        let expr: *mut mpc_parser_t = mpc_new(b"expr\0".as_ptr() as *const _);
        let lispy: *mut mpc_parser_t = mpc_new(b"lispy\0".as_ptr() as *const _);

        // Define grammar
        let grammar_string = b"
              number : /-?[0-9]+/ ;                                           \
              symbol : \"list\" | \"head\" | \"tail\" | \"join\" | \"eval\" | \
                        '+' | '-' | '*' | '/' | '%' ;                         \
              sexpr  : '(' <expr>* ')' ;                                      \
              qexpr  : '{' <expr>* '}' ;                                      \
              expr   : <number> | <symbol> | <sexpr> | <qexpr> ;              \
              lispy  : /^/ <expr>* /$/ ;                                      \
            \0"
        .as_ptr() as *const _;

        // Generate lispy language
        mpca_lang(
            MPCA_LANG_DEFAULT as i32,
            grammar_string,
            number,
            symbol,
            sexpr,
            qexpr,
            expr,
            lispy,
        );

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
        mpc_cleanup(6, number, symbol, sexpr, qexpr, expr, lispy);
    }
}
