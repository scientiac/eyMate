use anyhow::Result;
use clap::{Command, arg, value_parser};
use figment::{
    Figment,
    providers::{Format, Toml},
};

mod detection;
mod config;


fn main() -> Result<()> {
    let config: config::Config = Figment::new().merge(Toml::file("eyMate.toml")).extract()?;

    println!("{:?}", config);

    let matches = Command::new(option_env!("CARGO_PKG_NAME").unwrap())
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap())
        .version(option_env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("add")
                .about("Add user to database.")
                .arg(arg!(<USER> "Affected user").value_parser(value_parser!(String)))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("test")
                .about("Test user login.")
                .arg(arg!(<USER> "Affected user").value_parser(value_parser!(String)))
                .arg_required_else_help(true),
        )
        .get_matches();

    let err = match matches.subcommand() {
        Some(("add", add_matches)) => {
            detection::cmd_add(config, add_matches.get_one::<String>("USER").unwrap())
        }
        Some(("test", test_matches)) => {
            detection::cmd_test(config, test_matches.get_one::<String>("USER").unwrap())
        }
        _ => unreachable!(),
    };

    if let Err(err) = err {
        println!("Command failed with:\n{}", err);
    }

    Ok(())
}
