#include <stdio.h>
#include <stdlib.h>

#include <histedit.h>
#include <readline/readline.h>

#include "mpc.h"

/* Create enum of lispval types */
typedef enum { LISPVAL_NUM, LISPVAL_ERR, LISPVAL_SYM, LISPVAL_SEXPR } expr_type;

/* lispval struct */
typedef struct lispval {
    expr_type type;
    long      num;
    /* String for error */
    char *err;
    char *symbol;
    /* Counter and pointer for list of "lval*" */
    int              count;
    struct lispval **cell;
} lispval;

/* lispval checked union */
typedef struct lispval_union {
    expr_type type;
    union {
        char *err;
        long  num;
        char *symbol;
        struct lisplist {
            int       count;
            lispval **cell;
        } lisplist;
    } content;
} lispval_union;

/* Construct a number */
lispval *lispval_num(long num) {
    lispval *v = malloc(sizeof(lispval));

    v->type = LISPVAL_NUM;
    v->num  = num;
    return v;
}

/* Construct an error */
lispval *lispval_err(char *error) {
    lispval *v = malloc(sizeof(lispval));

    v->type = LISPVAL_ERR;
    v->err  = malloc(strlen(error) + 1);
    strcpy(v->err, error);
    return v;
}

/* Construct a new symbol */
lispval *lispval_sym(char *sym) {
    lispval *v = malloc(sizeof(lispval));

    v->type   = LISPVAL_SYM;
    v->symbol = malloc(strlen(sym) + 1);
    strcpy(v->symbol, sym);
    return v;
}

/* Construct new empty sexpr */
lispval *lispval_sexpr(void) {
    lispval *v = malloc(sizeof(lispval));

    v->type  = LISPVAL_SEXPR;
    v->count = 0;
    v->cell  = NULL;
    return v;
}

/* Clean up a lispval */
void lispval_del(lispval *val) {
    switch (val->type) {
        case LISPVAL_NUM: break;

        case LISPVAL_ERR: free(val->err); break;
        case LISPVAL_SYM: free(val->symbol); break;

        /* If Sexpr then delete all elements inside */
        case LISPVAL_SEXPR:
            for (int i = 0; i < val->count; i++) { lispval_del(val->cell[i]); }
    }
    /* Free the entire struct finally */
    free(val);
}

lispval *lispval_read_num(mpc_ast_t *t) {
    errno  = 0;
    long x = strtol(t->contents, NULL, 10);
    return errno != ERANGE ? lispval_num(x) : lispval_err("Invalid number: Over- or underflow!");
}

lispval *lispval_add(lispval *val, lispval *x) {
    val->count++;
    val->cell                 = realloc(val->cell, sizeof(lispval *) * val->count);
    val->cell[val->count - 1] = x;
    return val;
}

lispval *lispval_read(mpc_ast_t *t) {
    if (strstr(t->tag, "number")) {
        return lispval_read_num(t);
    }
    if (strstr(t->tag, "symbol")) {
        return lispval_sym(t->contents);
    }

    lispval *x = NULL;

    /* If root (>) or sexpr then construct empty list */
    if (strcmp(t->tag, ">") == 0) {
        x = lispval_sexpr();
    }
    if (strstr(t->tag, "sexpr")) {
        x = lispval_sexpr();
    }

    for (int i = 0; i < t->children_num; i++) {
        if (strcmp(t->children[i]->contents, "(") == 0) {
            continue;
        }
        if (strcmp(t->children[i]->contents, ")") == 0) {
            continue;
        }
        if (strcmp(t->children[i]->tag, "regex") == 0) {
            continue;
        }
        x = lispval_add(x, lispval_read(t->children[i]));
    }
    return x;
}

void lispval_print(lispval *v);
void lispval_expr_print(lispval *v, char open, char close);

/* Print an lispval */
void lispval_expr_print(lispval *v, char open, char close) {
    putchar(open);
    for (int i = 0; i < v->count; i++) {
        lispval_print(v->cell[i]);
        if (i != (v->count - 1)) {
            putchar(' ');
        }
    }
    putchar(close);
}

void lispval_print(lispval *v) {
    switch (v->type) {
        case LISPVAL_NUM: printf("%li", v->num); break;
        case LISPVAL_ERR: printf("Error: %s", v->err); break;
        case LISPVAL_SYM: printf("%s", v->symbol); break;
        case LISPVAL_SEXPR: lispval_expr_print(v, '(', ')'); break;
    }
}

void lispval_println(lispval *v) {
    lispval_print(v);
    putchar('\n');
}

lispval *lispval_pop(lispval *v, int i) {
    /* Find item at i */
    lispval *x = v->cell[i];

    /* Shift memory after the item at i over the top */
    memmove(&v->cell[i], &v->cell[i + 1], sizeof(lispval *) * (v->count - i - 1));

    v->count--;

    v->cell = realloc(v->cell, sizeof(lispval *) * v->count);
    return x;
}

lispval *lispval_take(lispval *v, int i) {
    lispval *x = lispval_pop(v, i);
    lispval_del(v);
    return x;
}

lispval *lispval_eval(lispval *v);
lispval *builtin_op(lispval *a, char *op);

lispval *lispval_eval_sexpr(lispval *v) {
    /* Evaluate children */
    for (int i = 0; i < v->count; i++) { v->cell[i] = lispval_eval(v->cell[i]); }

    /* Error Checking */
    for (int i = 0; i < v->count; i++) {
        if (v->cell[i]->type == LISPVAL_ERR) {
            return lispval_take(v, i);
        }
    }

    /* Empty expression () */
    if (v->count == 0) {
        return v;
    }

    /* Single expression */
    if (v->count == 1) {
        return lispval_take(v, 0);
    }

    /* Remaining must be sexpr, first of it must be a symbol */
    lispval *f = lispval_pop(v, 0);
    if (f->type != LISPVAL_SYM) {
        lispval_del(f);
        lispval_del(v);
        return lispval_err("S-Expression does not start with symbol!");
    }

    /* Call builtin with operator */
    lispval *result = builtin_op(v, f->symbol);
    lispval_del(f);
    return result;
}

lispval *builtin_op(lispval *a, char *op) {
    for (int i = 0; i < a->count; i++) {
        if (a->cell[i]->type != LISPVAL_NUM) {
            lispval_del(a);
            return lispval_err("Cannot operate on non-number!");
        }
    }

    lispval *x = lispval_pop(a, 0);

    /* If no arguments and sub('-') then perform unary negation */
    if ((strcmp(op, "-") == 0) && a->count == 0) {
        x->num = -x->num;
    }

    while (a->count > 0) {
        lispval *y = lispval_pop(a, 0);

        if (strcmp(op, "+") == 0) {
            x->num += y->num;
        }
        if (strcmp(op, "-") == 0) {
            x->num -= y->num;
        }
        if (strcmp(op, "*") == 0) {
            x->num *= y->num;
        }
        if (strcmp(op, "/") == 0) {
            if (y->num == 0) {
                lispval_del(x);
                lispval_del(y);
                x = lispval_err("Division by zero!");
                break;
            }
            x->num /= y->num;
        }
    }
    lispval_del(a);
    return x;
}

lispval *lispval_eval(lispval *v) {
    /* Evaluate S-expressions */
    if (v->type == LISPVAL_SEXPR) {
        return lispval_eval_sexpr(v);
    }
    /* Treat all other types the same */
    return v;
}

int main(int argc, char **argv) {
    // printf("4 * 2: %li\n", eval_op(4, "*", 2));

    /* Create some parsers */
    mpc_parser_t *Number = mpc_new("number");
    mpc_parser_t *Symbol = mpc_new("symbol");
    mpc_parser_t *Expr   = mpc_new("expr");
    mpc_parser_t *Sexpr  = mpc_new("sexpr");
    mpc_parser_t *Lispy  = mpc_new("lispy");

    /* Define the parsers with the following language */
    mpca_lang(MPCA_LANG_DEFAULT, "                           \
              number   : /(-?[0-9]+)/ ;                      \
              symbol   : '+' | '-' | '*' | '/' | '%' ;       \
              sexpr    : '(' <expr>* ')' ;                   \
              expr     : <number> | <symbol> | <sexpr> ;     \
              lispy    : /^/ <expr>* /$/ ;                   \
            ",
              Number, Symbol, Sexpr, Expr, Lispy);

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
            lispval *result = lispval_eval(lispval_read(r.output));
            lispval_println(result);
            lispval_del(result);
            mpc_ast_delete(r.output);
        } else {
            /* Not parsed. Print error */
            mpc_err_print(r.error);
            mpc_err_delete(r.error);
        }
        /* readline does malloc */
        free(input);
        break;
    }
    /* Undefine and delete the parsers */
    mpc_cleanup(5, Number, Symbol, Sexpr, Expr, Lispy);

    return 0;
}
