use feox::eval::{Env, Expr};
use feox::{eval, parser};
use rustyline::{Cmd, DefaultEditor, KeyEvent};
use rustyline::error::ReadlineError;
use std::cell::RefCell;
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
        Ok(_) => (),
        Err(err) => eprintln!("{}", err),
    }
}

fn read_input(rl: &mut DefaultEditor) -> Result<String, ReadlineError> {
    let mut input = String::new();
    let mut depth = 0;

    loop {
        let prompt = if depth > 0 { "... " } else { "> " };

        let line = rl.readline(prompt)?;

        for c in line.chars() {
            match c {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => depth -= 1,
                _ => {}
            }
        }
        
        input.push_str(&line);
        input.push('\n');

        if depth <= 0 {
            break;
        }
    }
    Ok(input)
}

fn repl(env: &Rc<RefCell<Env>>) {
    println!("Feox REPL (type 'exit' to quit)");
    let mut rl = DefaultEditor::new().unwrap();

    rl.bind_sequence(KeyEvent::ctrl('j'), Cmd::Insert(1, "\n".into()));
    rl.bind_sequence(KeyEvent::from('\t'), Cmd::Insert(1, "    ".into()));

    loop {
        let line = match read_input(&mut rl) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        let line = line.trim();
        rl.add_history_entry(line).unwrap();

        if line == "exit" {
            break;
        }

        run_line(line, env.clone());
    }
}

fn main() {
    let env = Rc::new(RefCell::new(Env::new()));
    const STDLIB: &str = include_str!("../stdlib.fe");

    let ast = match parser::parse(STDLIB){
        Ok(ast) => ast,
        Err(e) => panic!("{}", e)
    };
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
