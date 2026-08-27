use std::{cell::RefCell, io, rc::Rc};

use miette::{IntoDiagnostic, Report, WrapErr, miette};
use regex::{Captures, Regex};
use regex_try::RegexTry;
use rust_lisp::{default_env, interpreter::eval, parser::parse};

fn main() -> miette::Result<()> {
    let env = Rc::new(RefCell::new(default_env()));

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
