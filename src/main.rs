use std::io;
use std::process;

use clap::{Arg, ArgMatches, Command};
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::{parse_input, Preprocessor};

pub fn make_app() -> Command {
    Command::new("bib")
        .about(
            "A mdbook plugin (preprocessor) which loads/presents/allows citation of \
        bibliography entries in a .bib format",
        )
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    logging_initialization();
    let matches = make_app().get_matches();

    // Users will want to construct their own preprocessor here
    let preprocessor = mdbook_bib::Bibliography; // Explicit Bibliography processor ref in lib.rs

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("Errors: {e}");
        process::exit(1);
    }
}

fn logging_initialization() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_env_var("MDBOOK_LOG")
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env_lossy();
    let log_env = std::env::var("MDBOOK_LOG");
    // Silence some particularly noisy dependencies unless the user
    // specifically asks for them.
    let silence_unless_specified = |filter: tracing_subscriber::EnvFilter, target| {
        if !log_env
            .as_ref()
            .is_ok_and(|s| s.split(',').any(|directive| directive.starts_with(target)))
        {
            filter.add_directive(format!("{target}=warn").parse().unwrap())
        } else {
            filter
        }
    };
    let filter = silence_unless_specified(filter, "handlebars");
    let filter = silence_unless_specified(filter, "html5ever");

    // Don't show the target by default, since it generally isn't useful
    // unless you are overriding the level.
    let with_target = log_env.is_ok();

    tracing_subscriber::fmt()
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(with_target)
        .init();
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = parse_input(io::stdin())?;
    if ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook_preprocessor::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer).unwrap_or(false);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
