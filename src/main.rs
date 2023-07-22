extern crate log;

use std::env;
use std::io;
use std::process;

use chrono::Local;
use clap::{Arg, ArgMatches, Command};
use env_logger::Builder;
use log::LevelFilter;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};

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
    let preprocessor = mdbook_bib::Bibiography; // Explicit Bibliography processor ref in lib.rs

    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("Errors: {e}");
        process::exit(1);
    }
}

fn logging_initialization() {
    use std::io::Write;
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        builder.filter(None, LevelFilter::Info); // Default to Info
    }

    builder.init();
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        // We should probably use the `semver` crate to check compatibility
        // here...
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
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
    let supported = pre.supports_renderer(renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
