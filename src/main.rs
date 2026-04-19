use feox::eval::{Env, Expr};
use feox::{eval, parser};
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::{env, fs};

fn run_line(line: &str, env: Rc<RefCell<Env>>) {
    if line.trim().is_empty() {
        return;
    }

    let ast = &match parser::parse(line) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let result = eval::eval(&eval::Expr::Block(ast.clone()), env);
    match result {
        Ok(ok) => println!("{}", ok),
        Err(err) => eprintln!("{}", err),
    }
}

fn run_file(path: &str, env: Rc<RefCell<Env>>) {
    let content = fs::read_to_string(path).expect("failed to read file");

    let ast = &match parser::parse(content.as_str()) {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let result = eval::eval(&eval::Expr::Block(ast.clone()), env);
    match result {
        Ok(ok) => println!("{}", ok),
        Err(err) => eprintln!("{}", err),
    }
}

fn read_input() -> String {
    let mut input = String::new();
    let mut depth = 0;

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        for c in line.chars() {
            match c {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => depth -= 1,
                _ => {}
            }
        }

        input.push_str(&line);

        if depth <= 0 { break; }
        print!("... ");
        std::io::stdout().flush().unwrap();
    }

    input
}

fn repl(env: &Rc<RefCell<Env>>) {
    println!("Feox REPL (type 'exit' to quit)");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let line = read_input();

        let line = line.trim();

        if line == "exit" {
            break;
        }

        run_line(line, env.clone());
    }
}

fn main() {
    let env = Rc::new(RefCell::new(Env::new()));
    const STDLIB: &str = r#"
        let range = |start, end| { let i = start; || { if i >= end { nil } else { i = i + 1; i - 1 } } };
        let map = |iter, f| || { let v = iter(); if v == nil { nil } else { f(v) } };
        let filter = |iter, f| || { let cur = nil; while ((cur = iter()) != nil){if f(cur) {return cur}} };
        let collect = |iter| { let arr = []; let v = nil; while (v = iter()) != nil { arr = arr + v }; arr };
        let sum = |iter| {let acc = 0; for el in iter { acc = acc + el; }; acc};
        let max = |iter| {let maxi = nil; for el in iter { if el >= maxi {maxi = el} }; maxi };
        let min = |iter| {let mini = nil; for el in iter { if el <= mini {mini = el} }; mini };
    "#;

    let ast = parser::parse(STDLIB).expect("stdlib should parse");
    eval::eval(&Expr::Block(ast), env.clone()).expect("stdlib should eval");



    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => repl(&env),

        2 => run_file(&args[1], env),

        _ => {
            eprintln!("Usage: feox [script.feox]");
        }
    }
}
