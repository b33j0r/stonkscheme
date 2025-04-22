use crate::ast::Expr;
use indexmap::IndexMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Env {
    pub scope_stack: VecDeque<Arc<Mutex<Scope>>>,
}

/// Internal representation of numbers
enum Number {
    Zero,
    Unsigned(u64),
    Signed(i64),
    Float(f64),
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
                            "car" => {
                                if let Some(Expr::Combination(target, _)) = new_args.get(0) {
                                    Ok(*target.clone())
                                } else {
                                    panic!("car requires a list");
                                }
                            }
                            "cdr" => {
                                if let Some(Expr::Combination(_, args)) = new_args.get(0) {
                                    if args.len() > 1 {
                                        Ok(Expr::Combination(Box::new(args[0].clone()), args[1..].to_vec()))
                                    } else {
                                        panic!("cdr requires a list with at least two elements");
                                    }
                                } else {
                                    panic!("car requires a list");
                                }
                            }
                            "cons" => {
                                if let Some(Expr::Combination(target, args)) = new_args.get(0) {
                                    if args.len() > 0 {
                                        Ok(Expr::Combination(Box::new(*target.clone()), args.clone()))
                                    } else {
                                        panic!("cons requires a list");
                                    }
                                } else {
                                    panic!("cons requires a list");
                                }
                            }
                            "if" => {
                                if new_args.len() == 3 {
                                    let condition = self.eval(&new_args[0])?;
                                    if let Expr::Boolean(true) = condition {
                                        return Ok(new_args[1].clone());
                                    } else if let Expr::Boolean(false) = condition {
                                        return Ok(new_args[2].clone());
                                    } else {
                                        panic!("if requires a boolean condition");
                                    }
                                } else {
                                    panic!("if requires three arguments");
                                }
                            }
                            "+" => {
                                // Initialize accumulator as mutable
                                let mut number = Number::Zero;

                                for arg in new_args.iter() {
                                    // Match on both the current accumulator state and the argument type
                                    match (number, arg) {
                                        // Accumulator is Zero, initialize with the first number
                                        (Number::Zero, Expr::Integer(i)) => {
                                            number = Number::Signed(*i);
                                        }
                                        (Number::Zero, Expr::Float(f)) => {
                                            number = Number::Float(*f);
                                        }

                                        // Accumulator is Signed
                                        (Number::Signed(n), Expr::Integer(i)) => {
                                            // Add integer + integer
                                            // Consider using checked_add for overflow safety if needed
                                            number = Number::Signed(n + *i);
                                        }
                                        (Number::Signed(n), Expr::Float(f)) => {
                                            // Add integer + float -> promote to float
                                            number = Number::Float(n as f64 + *f);
                                        }

                                        // Accumulator is Float
                                        (Number::Float(n), Expr::Float(f)) => {
                                            // Add float + float
                                            number = Number::Float(n + *f);
                                        }
                                        (Number::Float(n), Expr::Integer(i)) => {
                                            // Add float + integer -> stays float
                                            number = Number::Float(n + *i as f64);
                                        }

                                        // Handle non-numeric arguments
                                        (_, other_expr) => {
                                            return Err(format!("Invalid argument for '+': expected Integer or Float, found {:?}", other_expr));
                                        }
                                    }
                                }

                                // Convert the final accumulator value back to an Expr
                                // This now becomes the return value for the '+' case
                                match number {
                                    // If no arguments were provided, or they summed to zero in their initial type
                                    Number::Zero => Ok(Expr::Integer(0)), // Default to integer 0 if no args
                                    Number::Signed(n) => Ok(Expr::Integer(n)),
                                    Number::Float(f) => Ok(Expr::Float(f)),
                                    // Assuming Number::Unsigned is not used in this logic based on Expr types
                                    Number::Unsigned(_) => unreachable!("Unsigned numbers not handled in addition logic"),
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
            Expr::Nil => Ok(expr.clone()),
            Expr::Comment(_) => Ok(expr.clone()),
            Expr::Boolean(_) => Ok(expr.clone()),
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