#include <stdio.h>
#include <stdlib.h>

#include <editline/readline.h>
#include <histedit.h>

#include "mpc.h"

/* Create enum of error types */
typedef enum { LERR_DIV_ZERO, LERR_BAD_OP, LERR_BAD_NUM } error_type;

/* Create enum of lispval types */
typedef enum { LISPVAL_NUM, LISPVAL_ERR } expr_type;

/* lispval struct */
typedef struct {
    expr_type type;
    error_type err;
    long num;
} lisp_val;

/* lispval checked union */
typedef struct {
    expr_type type;
    union {
        error_type err;
        long num;
    } content;
} lispval;

/* Construct a number as expression result */
lispval lispval_num(long num) {
    lispval v;

    v.type = LISPVAL_NUM;
    v.content.num = num;
    return v;
}

/* Construct error as expression result */
lispval lispval_err(int error) {
    lispval v;

    v.type = LISPVAL_ERR;
    v.content.err = error;
    return v;
}

/* Print an lispval */
void lispval_print(lispval v) {
    switch (v.type) {
    case LISPVAL_NUM:
        printf("%li", v.content.num);
        break;
    case LISPVAL_ERR:
        switch (v.content.err) {
        case LERR_DIV_ZERO:
            printf("Error: Division by Zero!");
            break;
        case LERR_BAD_OP:
            printf("Error: Invalid operator!");
            break;
        case LERR_BAD_NUM:
            printf("Error: Invalid number!");
            break;
        }
    }
}

void lispval_println(lispval v) {
    lispval_print(v);
    putchar('\n');
}

lispval eval_op(lispval x, char *op, lispval y) {
    if (x.type == LISPVAL_ERR)
        return x;
    if (y.type == LISPVAL_ERR)
        return y;

    switch (op[0]) {
    case '+':
        return lispval_num(x.content.num + y.content.num);

    case '-':
        return lispval_num(x.content.num - y.content.num);

    case '*':
        return lispval_num(x.content.num * y.content.num);

    case '/':
        return y.content.num == 0 ? lispval_err(LERR_DIV_ZERO)
               : lispval_num(x.content.num / y.content.num);

    case '%':
        return lispval_num(x.content.num % y.content.num);
    }
    return lispval_err(LERR_BAD_OP);
}

lispval eval(mpc_ast_t *t) {
    /* Number: arrived at leaf */
    if (strstr(t->tag, "number")) {
        errno = 0;
        long x = strtol(t->contents, NULL, 10);
        return errno != ERANGE ? lispval_num(x) : lispval_err(LERR_BAD_NUM);
    }

    /* The operator always comes after '(' which is the 0'th child */
    char *op = t->children[1]->contents;

    /* We store the third child in 'x' */
    lispval x = eval(t->children[2]);

    /* Iterate over remaining children */
    int i = 3;
    while (strstr(t->children[i]->tag, "expr")) {
        x = eval_op(x, op, eval(t->children[i]));
        i++;
    }

    return x;
}

int main(int argc, char **argv) {
    // printf("4 * 2: %li\n", eval_op(4, "*", 2));

    /* Create some parsers */
    mpc_parser_t *Number = mpc_new("number");
    mpc_parser_t *Operator = mpc_new("operator");
    mpc_parser_t *Expr = mpc_new("expr");
    mpc_parser_t *Lispy = mpc_new("lispy");

    /* Define the parsers with the following language */
    mpca_lang(MPCA_LANG_DEFAULT, "                                               \
              number   : /(-?[0-9]+)/ ;                      \
              operator : '+' | '-' | '*' | '/' | '%' ;            \
              expr     : <number> | '(' <operator> <expr>+ ')' ;  \
              lispy    : /^/ <operator> <expr>+ /$/ ;             \
            ",
              Number, Operator, Expr, Lispy);

    /* Version and exit information */
    puts("Lispy version 0.0.0.0.4\n");
    puts("Press CTRL-C to exit\n");

    puts("Example expression: * 2 2 or * (+ 1 5) (* 1 3 7)");

    while (1) {
        char *input = readline("cispy >> ");

        add_history(input);

        /* Attempt to parse the input */
        mpc_result_t r;
        if (mpc_parse("<stdin>", input, Lispy, &r)) {
            /* Evaluate */
            lispval result = eval(r.output);
            lispval_println(result);
            /* Success - print the AST */
            mpc_ast_print(r.output);
            mpc_ast_delete(r.output);
        } else {
            /* Not parsed. Print error */
            mpc_err_print(r.error);
            mpc_err_delete(r.error);
        }

        /* readline does malloc */
        free(input);
    }

    /* Undefine and delete the parsers */
    mpc_cleanup(5, Number, Operator, Expr, Lispy);

    return 0;
}
