use std::{cell::RefCell, io, rc::Rc};

use miette::{IntoDiagnostic, Report, WrapErr, miette};
use regex::{Captures, Regex};
use regex_try::RegexTry;
use rust_lisp::{interpreter::eval, model::Value, parser::parse};

use env::env;

mod env;

pub fn value_to_string(value: Value) -> String {
    match value {
        Value::String(x) => x,
        Value::List(x) => x
            .into_iter()
            .map(value_to_string)
            .collect::<Vec<_>>()
            .join("\n"),
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
