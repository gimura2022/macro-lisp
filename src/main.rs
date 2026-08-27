use std::{cell::RefCell, fs, io, rc::Rc};

use miette::{IntoDiagnostic, Report, WrapErr, miette};
use regex::{Captures, Regex};
use regex_try::RegexTry;
use rust_lisp::{
    default_env,
    interpreter::eval,
    model::{RuntimeError, Symbol, Value},
    parser::parse,
};

fn main() -> miette::Result<()> {
    let env = Rc::new(RefCell::new(default_env()));

    env.borrow_mut().define(
        Symbol::from("load"),
        Value::NativeFunc(|env, args| {
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
        }),
    );

    print!(
        "{}",
        Regex::new(r"`([^`]*)`")
            .expect("failed to compile regex")
            .try_replace_all(
                &io::read_to_string(io::stdin())
                    .into_diagnostic()
                    .wrap_err("failed to read stdin")?,
                |caps: &Captures| {
                    Ok::<String, Report>(
                        parse(&caps[1])
                            .map(|x| {
                                eval(env.clone(), &x.map_err(|x| miette!(x.msg))?)
                                    .into_diagnostic()
                                    .wrap_err("failed to execute code")
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .last()
                            .wrap_err("failed to get expression from macro block")?
                            .to_string(),
                    )
                },
            )
            .wrap_err("failed to process macro code")?
    );

    Ok(())
}
