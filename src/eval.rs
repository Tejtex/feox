use std::ops::*;
use num_traits::identities::Zero;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ibig::IBig;
use num_traits::{One, Pow};
use crate::ast::{BinOp, Expr};

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(a + &b),
            (a, b) => panic!("cannot add {:?} and {:?}", a, b),
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (a, b) => panic!("cannot sub {:?} and {:?}", a, b),
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            (a, b) => panic!("cannot mul {:?} and {:?}", a, b),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            (a, b) => panic!("cannot div {:?} and {:?}", a, b),
        }
    }
}

impl Rem for Value {
    type Output = Value;
    fn rem(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
            (a, b) => panic!("cannot rem {:?} and {:?}", a, b),
        }
    }
}

impl BitAnd for Value {
    type Output = Value;
    fn bitand(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a & b),
            (a, b) => panic!("cannot & {:?} and {:?}", a, b),
        }
    }
}

impl BitOr for Value {
    type Output = Value;
    fn bitor(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a | b),
            (a, b) => panic!("cannot | {:?} and {:?}", a, b),
        }
    }
}

impl BitXor for Value {
    type Output = Value;
    fn bitxor(self, rhs: Value) -> Value {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a ^ b),
            (a, b) => panic!("cannot ^ {:?} and {:?}", a, b),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a),   Value::Bool(b))   => a == b,
            (Value::Nil,       Value::Nil)        => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

// impl Pow<Value> for Value {
//     type Output = Value;
//     fn pow(self, rhs: Value) -> Value {
//         match (self, rhs) {
//             (Value::Number(mut a), Value::Number(mut b)) => {
//                 let mut res = IBig::one();
//
//
//                 while b > IBig::zero() {
//                     if &b % 2 == 1 { res *= &a; }
//                     a = &a * &a;
//                     b /= 2;
//                 }
//                 Value::Number(res)
//             },
//             (a, b) => panic!("cannot pow {:?} and {:?}", a, b),
//         }
//     }
// }
type EnvRef = Rc<RefCell<Env>>;
#[derive(Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
    parent: Option<EnvRef>,
}

impl Env {
    pub fn new() -> Self {
        Env { vars: HashMap::new(), parent: None }
    }

    pub fn child(parent: EnvRef) -> Self {
        Env { vars: HashMap::new(), parent: Some(parent) }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
            .or_else(|| self.parent.as_ref()?.borrow().get(name))
    }

    pub fn set(&mut self, name: &str, val: Value) {
        self.vars.insert(name.to_string(), val);
    }
}
impl std::fmt::Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env")
            .field("vars", &self.vars.keys().collect::<Vec<_>>())
            .field("parent", &self.parent.as_ref().map(|_| "Env(...)"))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Lambda {
        args: Vec<String>,
        body: Box<Expr>,
        env: EnvRef,
    },
    Nil
}

#[derive(Debug)]
pub enum EvalResult {
    Return(Value),
    Continue,
    Break,
    Value(Value),
    Error(String)
}

pub fn eval(expr: &Expr, env: EnvRef) -> EvalResult {
    match expr {
        Expr::Number(n) => EvalResult::Value(Value::Number(n.clone())),
        Expr::String(s) => EvalResult::Value(Value::String(s.to_string())),
        Expr::Bool(b) => EvalResult::Value(Value::Bool(*b)),
        Expr::Nil => EvalResult::Value(Value::Nil),
        Expr::Array(exprs) => {
            let mut out = Vec::new();

            for expr in exprs {
                match eval(expr, env.clone()) {
                    EvalResult::Value(v) => out.push(v),
                    other => return other,
                }
            }

            EvalResult::Value(Value::Array(out))
        }
        Expr::If {cond, then, else_} => eval_if(cond, then, else_, env),
        Expr::BinOp {op, left, right} => eval_bin_op(op, left, right, env),
        Expr::Assign {name, value } => eval_assign(name, value, env),
        Expr::Ident(name) => EvalResult::Value(env.borrow().get(name).unwrap()),
        // Expr::UnaryOp {op, expr} => eval_unary_op(op, expr, env),
        Expr::Continue => EvalResult::Continue,
        Expr::Break => EvalResult::Break,
        Expr::Return(v) => {
            if let Some(v) = v {
                let v = eval(&**v, env);
                match v {
                    EvalResult::Value(v) => EvalResult::Return(v),
                    other => other
                }
            } else {
                EvalResult::Return(Value::Nil)
            }
        }
        Expr::Lambda {args, body} => EvalResult::Value(Value::Lambda {args: args.clone(), body: body.clone(), env}),
        Expr::Call {args, func} => eval_call(args, func, env),
        Expr::While {cond, body} => eval_while(cond, body, env),

        Expr::Block(exprs) => {
            let mut last = EvalResult::Value(Value::Nil);
            for expr in exprs {
                let res = eval(expr, env.clone());
                match res {
                    EvalResult::Value(val) => last = EvalResult::Value(val),
                    other => return other
                }
            }
            last
        }
        _ => panic!()


    }
}

fn eval_while(cond: &Box<Expr>, body: &Box<Expr>, env: EnvRef) -> EvalResult {
    let mut val = match eval(&**cond, env.clone()) {
        EvalResult::Value(v) => v,
        other => return other
    };

    while is_true(&val) {
        val = match eval(&**cond, env.clone()) {
            EvalResult::Value(v) => v,
            EvalResult::Continue => continue,
            EvalResult::Break => break,
            other => return other
        };

        match eval(&**body, env.clone()) {
            EvalResult::Value(v) => v,
            EvalResult::Continue => continue,
            EvalResult::Break => break,
            other => return other
        };
    }
    EvalResult::Value(Value::Nil)

}

fn eval_call(args: &Vec<Expr>, func: &Box<Expr>, env: EnvRef) -> EvalResult {
    let func = match eval(&**func, env.clone()) {
        EvalResult::Value(v) => match v {
            Value::Lambda { args, body, env } => (args, body, env),
            _ => panic!()
        },
        other => return other
    };
    let new_env = Rc::new(RefCell::new(Env::child(func.2.clone())));

    if func.0.len() != args.len() {
        panic!();
    }

    for (arg, name) in args.into_iter().zip(func.0) {
        let val = match eval(&*arg, env.clone()) {
            EvalResult::Value(v) => v,
            other => return other
        };
        new_env.borrow_mut().set(&name, val);
    }

    let res = eval(&*func.1, new_env);

    match res {
        EvalResult::Value(v) => EvalResult::Value(v),
        EvalResult::Return(v) => EvalResult::Value(v),
        _ => EvalResult::Value(Value::Nil)
    }
}

// fn eval_unary_op(op: &UnaryOp, expr: &Box<Expr>, env: EnvRef) -> EvalResult {
//     let val = eval(&**expr, env);
//     if let EvalResult::Value(val) = val {
//         match op {
//             UnaryOp::Neg => -val,
//             UnaryOp::Not => !val,
//         }
//     } else {
//         val
//     }
//
// }

fn eval_assign(name: &str, value: &Box<Expr>, env: EnvRef) -> EvalResult {
    let value = eval(&**value, env.clone());
    match value {
        EvalResult::Value(v) => {
            env.borrow_mut().set(name, v.clone());
            EvalResult::Value(v)
        }
        o => o
    }
}
fn is_true(val: &Value) -> bool {
    match val {
        Value::Bool(false) | Value::Nil => false,
        Value::Number(n) if n.is_zero() => false,
        Value::Array(v) if v.is_empty() => false,
        _ => true
    }
}

fn eval_bin_op(op: &BinOp, left: &Box<Expr>, right: &Box<Expr>, env: EnvRef) -> EvalResult {
    let left = eval(&**left, env.clone());
    if let EvalResult::Value(left) = left {
        let right = eval(&**right, env);
        if let EvalResult::Value(right) = right {
            EvalResult::Value(match op {
                BinOp::Add => left + right,
                BinOp::Sub => left - right,
                BinOp::Mul => left * right,
                BinOp::Div => left / right,
                BinOp::Rem => left % right,
                BinOp::Eq => Value::Bool(left == right),
                BinOp::Neq => Value::Bool(left != right),
                BinOp::Lt => Value::Bool(left < right),
                BinOp::Gt => Value::Bool(left > right),
                BinOp::Le => Value::Bool(left <= right),
                BinOp::Ge => Value::Bool(left >= right),
                BinOp::Xor => left ^ right,
                BinOp::And => left & right,
                BinOp::Or => left | right,
                // BinOp::Pow => left.pow(right)
                _ => panic!()
            })
            
        } else {
            right
        }
    } else {
        left
    }

}

fn eval_if(cond: &Box<Expr>, then: &Box<Expr>, else_: &Option<Box<Expr>>, env: EnvRef) -> EvalResult {
    let cond_val = eval(&**cond, env.clone());
    if let EvalResult::Value(cond_val) = cond_val {

        if is_true(&cond_val) {
            eval(&**then, env)
        } else {
            if let Some(else_) = else_ {
                eval(&**else_, env)
            } else {
                EvalResult::Value(Value::Nil)
            }
        }
    } else {
        cond_val
    }
}

