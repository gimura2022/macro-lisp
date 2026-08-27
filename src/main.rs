use std::{cell::RefCell, fs, io, rc::Rc};

use miette::{IntoDiagnostic, Report, WrapErr, miette};
use regex::{Captures, Regex};
use regex_try::RegexTry;
use rust_lisp::{
    default_env,
    interpreter::eval,
    model::{Env, RuntimeError, Symbol, Value},
    parser::parse,
};

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

fn env() -> Env {
    let mut env = default_env();

    env.define(Symbol::from("load"), Value::NativeFunc(load));

    env
}

fn value_to_string(value: Value) -> String {
    match value {
        Value::String(x) => x,
        x if x == Value::NIL => "".to_string(),
        _ => value.to_string(),
    }
}

fn main() -> miette::Result<()> {
    let env = Rc::new(RefCell::new(env()));

    print!(
        "{}",
        Regex::new(r"`([^`]*)`")
            .expect("failed to compile regex")
            .try_replace_all(
                &io::read_to_string(io::stdin())
                    .into_diagnostic()
                    .wrap_err("failed to read stdin")?,
                |caps: &Captures| {
                    Ok::<String, Report>(value_to_string(
                        parse(&caps[1])
                            .map(|x| {
                                eval(env.clone(), &x.map_err(|x| miette!(x.msg))?)
                                    .into_diagnostic()
                                    .wrap_err("failed to execute code")
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .last()
                            .wrap_err("failed to get expression from macro block")?
                            .clone(),
                    ))
                },
            )
            .wrap_err("failed to process macro code")?
    );

    Ok(())
}
