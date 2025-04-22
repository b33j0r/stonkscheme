use crate::ast::Expr;
use indexmap::IndexMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Env {
    pub scope_stack: VecDeque<Arc<Mutex<Scope>>>,
}

impl Env {
    pub fn new() -> Self {
        let mut scope_stack = VecDeque::new();
        scope_stack.push_back(Arc::new(Mutex::new(Scope(IndexMap::new()))));
        Self { scope_stack }
    }


    pub fn clone_child(&self) -> Self {
        let mut new_env = self.clone();
        new_env.scope_stack.push_back(Arc::new(Mutex::new(Scope(IndexMap::new()))));
        new_env
    }

    pub fn get(&self, key: &str) -> Option<Expr> {
        for scope in self.scope_stack.iter().rev() {
            let scope = scope.lock().unwrap();
            if let Some(expr) = scope.0.get(key) {
                return Some(expr.clone());
            }
        }
        None
    }
    pub fn set(&mut self, key: String, value: Expr) {
        if let Some(scope) = self.scope_stack.back_mut() {
            let mut scope = scope.lock().unwrap();
            scope.0.insert(key, value);
        }
    }
}

pub struct Scope(IndexMap<String, Expr>);

pub struct Interpreter {
    pub env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Env::new(),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Expr, String> {
        match expr {
            Expr::Nil => Ok(expr.clone()),
            Expr::Comment(_) => Ok(expr.clone()),
            Expr::Combination(target, args) => {
                let target = self.eval(target)?;
                let mut new_args = Vec::new();
                for arg in args {
                    let arg = self.eval(arg)?;
                    new_args.push(arg);
                }
                match target.clone() {
                    Expr::Symbol(symbol) => {
                        match symbol.0.as_str() {
                            "set" => {
                                if let Some(Expr::Symbol(key)) = new_args.get(0) {
                                    if let Some(value) = new_args.get(1) {
                                        builtin_set(&mut self.env, (*key.clone()).to_string(), value.clone());
                                        return Ok(Expr::Nil);
                                    } else {
                                        panic!("set requires two arguments");
                                    }
                                } else {
                                    panic!("set requires a Symbol key");
                                }
                            }
                            "get" => {
                                if let Some(Expr::Symbol(key)) = new_args.get(0) {
                                    return Ok(builtin_get(&self.env, &*key));
                                } else {
                                    panic!("get requires a Symbol key");
                                }
                            }
                            _ => {
                                // Handle other operators
                                return Ok(Expr::Combination(Box::new(target), new_args));
                            }
                        }
                    }
                    _ => {
                        // Handle other combinations
                        return Ok(Expr::Combination(Box::new(target), new_args));
                    }
                }
            }
            Expr::Symbol(_) => Ok(expr.clone()),
            Expr::Float(_) => Ok(expr.clone()),
            Expr::String(_) => Ok(expr.clone()),
            Expr::Duration(_) => Ok(expr.clone()),
            Expr::Timestamp(_) => Ok(expr.clone()),
            Expr::Integer(_) => Ok(expr.clone()),
        }
    }
}

fn builtin_set(env: &mut Env, key: String, value: Expr) {
    env.set(key, value);
}

fn builtin_get(env: &Env, key: &str) -> Expr {
    env.get(key).unwrap_or(Expr::Nil)
}