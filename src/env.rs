use std::{cell::RefCell, fs, rc::Rc};

use rust_lisp::{
    default_env,
    interpreter::eval,
    model::{Env, RuntimeError, Symbol, Value},
    parser::parse,
};

use crate::value_to_string;

fn load(env: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let [Value::String(file)] = args.as_slice() else {
        return Err(RuntimeError {
            msg: "failed to get file path".to_string(),
        });
    };

    parse(&fs::read_to_string(file).map_err(|x| RuntimeError {
        msg: format!("failed to read file \"{file}\": {x}"),
    })?)
    .map(|x| {
        eval(
            env.clone(),
            &x.map_err(|x| RuntimeError {
                msg: format!("failed to parse file \"{file}\": {x}"),
            })?,
        )
    })
    .collect::<Result<Vec<_>, _>>()?;

    Ok(Value::NIL)
}

fn stringify(_: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError> {
    let [value] = args.as_slice() else {
        return Err(RuntimeError {
            msg: "failed to get value".to_string(),
        });
    };

    Ok(Value::String(value_to_string(value.clone())))
}

pub fn env() -> Env {
    let mut env = default_env();

    env.define(Symbol::from("load"), Value::NativeFunc(load));
    env.define(Symbol::from("stringify"), Value::NativeFunc(stringify));

    env
}
