use crate::ast::UnaryOp;
pub use crate::ast::{BinOp, Expr};
use num_traits::identities::Zero;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Nil, _) => Some(std::cmp::Ordering::Equal),
            (_, Value::Nil) => Some(std::cmp::Ordering::Equal),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Array(a), Value::Array(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

type EnvRef = Rc<RefCell<Env>>;
#[derive(Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
    parent: Option<EnvRef>,
    modulus: Option<i64>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            vars: HashMap::new(),
            parent: None,
            modulus: None,
        }
    }

    pub fn child(parent: EnvRef) -> Self {
        Env {
            vars: HashMap::new(),
            parent: Some(parent.clone()),
            modulus: parent.borrow().modulus,
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
            .or_else(|| self.parent.as_ref()?.borrow().get(name))
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.vars.insert(name, val);
    }

    pub fn modify(&mut self, name: String, val: Value) -> EvalResult {
        if self.vars.contains_key(&name) {
            self.vars.insert(name, val.clone());
            Ok(val)
        } else {
            if let Some(p) = &self.parent {
                p.borrow_mut().modify(name, val)
            } else {
                Err(EvalError::UndefinedVariable(name))
            }
        }
    }

    pub fn set_modulus(&mut self, modulus: i64) {
        self.modulus = Some(modulus);
    }

    pub fn reset_modulus(&mut self) {
        self.modulus = None;
    }

    pub fn add(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(self.modded(a + b))),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),

            (Value::Array(mut a), Value::Array(mut b)) => {
                a.append(&mut b);
                Ok(Value::Array(a))
            }
            (Value::Array(mut a), b) => {
                a.push(b);
                Ok(Value::Array(a))
            }
            (b, Value::Array(mut a)) => {
                a.insert(0, b);
                Ok(Value::Array(a))
            }
            (_, _) => Err(EvalError::TypeError("unsupported types for `+`")),
        }
    }

    pub fn mul(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(self.modded(a * b))),
            (Value::String(a), Value::Number(b)) | (Value::Number(b), Value::String(a)) => {
                let mut res = String::new();
                for _ in 0..b {
                    res.push_str(a.as_str());
                }
                Ok(Value::String(res))
            }
            (Value::Array(a), Value::Number(b)) | (Value::Number(b), Value::Array(a)) => {
                let mut res = Vec::new();
                for _ in 0..b {
                    res.extend(a.clone().into_iter())
                }
                Ok(Value::Array(res))
            }

            (_, _) => Err(EvalError::TypeError("unsupported types for `*`")),
        }
    }

    pub fn sub(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(self.modded(a - b))),

            (_, _) => Err(EvalError::TypeError("unsupported types for `-`")),
        }
    }

    pub fn rem(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => {
                Ok(Value::Number(self.modded(((a % b) + b) % b)))
            }

            (_, _) => Err(EvalError::TypeError("unsupported types for `%`")),
        }
    }

    pub fn xor(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(self.modded(a ^ b))),

            (_, _) => Err(EvalError::TypeError("unsupported types for `^`")),
        }
    }

    pub fn and(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(self.modded(a & b))),

            (_, _) => Err(EvalError::TypeError("unsupported types for `&`")),
        }
    }

    pub fn or(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(self.modded(a | b))),

            (_, _) => Err(EvalError::TypeError("unsupported types for `|`")),
        }
    }

    pub fn pow(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (lhs, rhs) {
            (Value::Number(mut lhs), Value::Number(mut rhs)) => {
                Ok(Value::Number(if let Some(modulus) = self.modulus {
                    let mut res = 1;

                    while rhs >= 1 {
                        if rhs % 2 == 1 {
                            res *= lhs
                        }
                        lhs *= lhs;
                        res %= modulus;
                        lhs %= modulus;
                        rhs /= 2;
                    }
                    res
                } else {
                    let mut res = 1;

                    while rhs >= 1 {
                        if rhs % 2 == 1 {
                            res *= lhs
                        }
                        lhs *= lhs;
                        rhs /= 2;
                    }
                    res
                }))
            }

            (_, _) => Err(EvalError::TypeError("unsupported types for `**`")),
        }
    }

    pub fn div(&self, lhs: Value, rhs: Value) -> EvalResult {
        match (&lhs, &rhs) {
            (&Value::Number(a), &Value::Number(b)) => {
                if let Some(modulus) = self.modulus {
                    let rhs = match self.pow(rhs, Value::Number(modulus - 2)) {
                        Ok(v) => v,
                        other => return other,
                    };

                    self.mul(rhs, lhs)
                } else {
                    Ok(Value::Number(a / b))
                }
            }

            (_, _) => Err(EvalError::TypeError("unsupported types for `/`")),
        }
    }

    pub fn modded(&self, num: i64) -> i64 {
        if let Some(modulus) = self.modulus {
            ((num % modulus) + modulus) % modulus
        } else {
            num
        }
    }

    pub fn neg(&self, num: i64) -> EvalResult {
        Ok(Value::Number(self.modded(-num)))
    }

    pub fn not(&self, num: i64) -> EvalResult {
        Ok(Value::Number((num == 0) as i64))
    }
}
impl std::fmt::Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env")
            .field("parent", &self.parent.as_ref().map(|_| "Env(...)"))
            .finish()
    }
}

#[derive(Debug, Clone, Error)]
pub enum EvalError {
    #[error("type error: {0}")]
    TypeError(&'static str),
    #[error("wrong number of args: {0}")]
    WrongNumberOfArgs(String),
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("division by zero")]
    DivisionByZero,
    #[error("continue outside of loop")]
    Continue,
    #[error("break outside of loop")]
    Break,
    #[error("return outside of function")]
    Return(Option<Value>),
}

#[derive(Debug, Clone, Default)]
pub enum Value {
    Number(i64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Lambda {
        args: Vec<String>,
        body: Box<Expr>,
        env: EnvRef,
    },
    Nil,
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    #[default]
    Uninit,
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Nil => write!(f, "nil"),
            Value::Lambda { .. } => write!(f, "<lambda>"),
            _ => write!(f, "<value>"),
        }
    }
}

type EvalResult = Result<Value, EvalError>;

pub fn eval(expr: &Expr, env: EnvRef) -> EvalResult {
    match expr {
        Expr::Number(n) => Ok(Value::Number(env.borrow().modded(n.clone()))),
        Expr::String(s) => Ok(Value::String(s.to_string())),
        Expr::Bool(b) => Ok(Value::Number(*b as i64)),
        Expr::Nil => Ok(Value::Nil),
        Expr::Array(exprs) => {
            let mut out = Vec::new();

            for expr in exprs {
                match eval(expr, env.clone()) {
                    Ok(v) => out.push(v),
                    other => return other,
                }
            }

            Ok(Value::Array(out))
        }
        Expr::If { cond, then, else_ } => eval_if(cond, then, else_, env),
        Expr::BinOp { op, left, right } => eval_bin_op(op, left, right, env),
        Expr::Assign { name, value } => eval_assign(name, value, env),
        Expr::Declare {name, value } => eval_declare(name, value, env),
        Expr::Ident(name) => env
            .borrow()
            .get(name)
            .ok_or(EvalError::UndefinedVariable(name.to_string())),
        Expr::UnaryOp { op, expr } => eval_unary_op(op, expr, env),

        Expr::Mod { modulus, body } => eval_mod(modulus, body, env),

        Expr::Range {
            start,
            end,
            inclusive,
        } => {
            let mut end = end.clone();
            if *inclusive {
                end = Box::new(Expr::BinOp {
                    op:BinOp::Add,
                    left: Box::new(Expr::Number(1)),
                    right: end.clone()
                });
            }
            eval_call(&vec![*start.clone(), *end], &Box::new(Expr::Ident("range".to_string())), env)
        }
        Expr::For {
            var,
            iter,
            body,
        } => eval_for(var, iter, body, env),
        Expr::Continue => Err(EvalError::Continue),
        Expr::Break => Err(EvalError::Break),
        Expr::Return(v) => {
            if let Some(v) = v {
                let v = eval(&**v, env);
                Err(EvalError::Return(Some(v?)))
            } else {
                Err(EvalError::Return(Some(Value::Nil)))
            }
        }
        Expr::Lambda { args, body } => Ok(Value::Lambda {
            args: args.clone(),
            body: body.clone(),
            env,
        }),
        Expr::Call { args, func } => eval_call(args, func, env),
        Expr::While { cond, body } => eval_while(cond, body, env),

        Expr::Block(exprs) => {
            let mut last = Value::Nil;
            for expr in exprs {
                last = eval(expr, env.clone())?;
            }
            Ok(last)
        }
        _ => unreachable!("unhandled expr type"),
    }
}

fn eval_mod(modulus: &Box<Expr>, body: &Box<Expr>, env: EnvRef) -> EvalResult {
    let modulus = eval(&**modulus, env.clone())?;

    let modulus = match modulus {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError("modulus has to be a number")),
    };
    env.borrow_mut().set_modulus(modulus);

    let res = eval(&**body, env.clone());
    env.borrow_mut().reset_modulus();
    res
}

fn eval_for(var: &str, iter: &Box<Expr>, body: &Box<Expr>, env: EnvRef) -> EvalResult {
    let iter = eval(&**iter, env.clone())?;

    match iter {
        Value::Lambda {
            args,
            body: iter_body,
            env: iter_env
        } if args.is_empty() => {
            loop {
                let new_env = Rc::new(RefCell::new(Env::child(iter_env.clone())));
                let val = eval(&iter_body, new_env)?;
                if val == Value::Nil {break}
                env.borrow_mut().set(var.to_string(), val);
                match eval(&**body, env.clone()) {
                    Err(EvalError::Break) => break,
                    Err(EvalError::Continue) => continue,
                    Ok(_) => (),
                    other => return other
                }
            }
            Ok(Value::Nil)
        }
        _ => Err(EvalError::TypeError("for only works on iterators")),
    }
}

fn eval_while(cond: &Box<Expr>, body: &Box<Expr>, env: EnvRef) -> EvalResult {
    let mut val = eval(&**cond, env.clone())?;

    while is_true(&val) {

        match eval(&**body, env.clone()) {
            Ok(v) => v,
            Err(EvalError::Continue) => continue,
            Err(EvalError::Break) => break,
            other => return other,
        };
        val = eval(&**cond, env.clone())?;
    }
    Ok(Value::Nil)
}


fn eval_call(args: &Vec<Expr>, func: &Box<Expr>, env: EnvRef) -> EvalResult {
    let func = match eval(&**func, env.clone()) {
        Ok(v) => match v {
            Value::Lambda { args, body, env } => (args, body, env),
            _ => return Err(EvalError::TypeError("can only call functions")),
        },
        other => return other,
    };
    let new_env = Rc::new(RefCell::new(Env::child(func.2.clone())));

    if func.0.len() != args.len() {
        return Err(EvalError::WrongNumberOfArgs(format!(
            "expected {} got {}",
            func.0.len(),
            args.len()
        )));
    }

    for (arg, name) in args.into_iter().zip(func.0) {
        let val = match eval(&*arg, env.clone()) {
            Ok(v) => v,
            other => return other,
        };
        new_env.borrow_mut().set(name, val);
    }

    let res = eval(&*func.1, new_env);

    match res {
        Ok(v) => Ok(v),
        Err(EvalError::Return(v)) => Ok(v.unwrap_or(Value::Nil)),
        o => o,
    }
}

fn eval_unary_op(op: &UnaryOp, expr: &Box<Expr>, env: EnvRef) -> EvalResult {
    let val = eval(&**expr, env.clone())?;
    let val = match val {
        Value::Number(n) => n,
        _ => return Err(EvalError::TypeError("unary op requires a number")),
    };
    match op {
        UnaryOp::Neg => env.borrow().neg(val),
        UnaryOp::Not => env.borrow().not(val),
    }
}

fn eval_declare(name: &str, value: &Box<Expr>, env: EnvRef) -> EvalResult {
    let value = eval(&**value, env.clone());
    match value {
        Ok(v) => {
            env.borrow_mut().set(name.to_string(), v.clone());
            Ok(v)
        }
        o => o,
    }
}

fn eval_assign(name: &str, value: &Box<Expr>, env: EnvRef) -> EvalResult {
    let value = eval(&**value, env.clone());
    match value {
        Ok(v) => {
            env.borrow_mut().modify(name.to_string(), v.clone())?;
            Ok(v)
        }
        o => o,
    }
}
fn is_true(val: &Value) -> bool {
    match val {
        Value::Nil => false,
        Value::Number(n) if n.is_zero() => false,
        Value::Array(v) if v.is_empty() => false,
        _ => true,
    }
}

fn eval_bin_op(op: &BinOp, left: &Box<Expr>, right: &Box<Expr>, env: EnvRef) -> EvalResult {
    let left = eval(&**left, env.clone())?;
    let right = eval(&**right, env.clone())?;
    let env = env.borrow();

    match op {
        BinOp::Add => env.add(left, right),
        BinOp::Sub => env.sub(left, right),
        BinOp::Mul => env.mul(left, right),
        BinOp::Div => env.div(left, right),
        BinOp::Rem => env.rem(left, right),
        BinOp::Xor => env.xor(left, right),
        BinOp::And => env.and(left, right),
        BinOp::Or => env.or(left, right),
        BinOp::Pow => env.pow(left, right),
        BinOp::Eq => Ok(Value::Number((left == right) as i64)),
        BinOp::Neq => Ok(Value::Number((left != right) as i64)),
        BinOp::Lt => Ok(Value::Number((left < right) as i64)),
        BinOp::Gt => Ok(Value::Number((left > right) as i64)),
        BinOp::Le => Ok(Value::Number((left <= right) as i64)),
        BinOp::Ge => Ok(Value::Number((left >= right) as i64)),
    }
}

fn eval_if(
    cond: &Box<Expr>,
    then: &Box<Expr>,
    else_: &Option<Box<Expr>>,
    env: EnvRef,
) -> EvalResult {
    let cond_val = eval(&**cond, env.clone())?;
    if is_true(&cond_val) {
        eval(&**then, env)
    } else {
        if let Some(else_) = else_ {
            eval(&**else_, env)
        } else {
            Ok(Value::Nil)
        }
    }
}
