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
            .expect("can't compile regex")
            .try_replace_all(
                &io::read_to_string(io::stdin())
                    .into_diagnostic()
                    .wrap_err("can't read stdin")?,
                |caps: &Captures| {
                    Ok::<String, Report>(
                        eval(
                            env.clone(),
                            &parse(&caps[1])
                                .next()
                                .wrap_err("can't find expression inside macro block")?
                                .map_err(|x| miette!(x.msg))?,
                        )
                        .into_diagnostic()
                        .wrap_err("can't execute")?
                        .to_string(),
                    )
                },
            )
            .wrap_err("can't process macro code")?
    );

    Ok(())
}
