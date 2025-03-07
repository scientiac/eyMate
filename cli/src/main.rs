use anyhow::{Result, *};
use clap::{Command, arg, value_parser};
use figment::{
    Figment,
    providers::{Format, Toml},
};
use recognition::config::*;
use recognition::paths::*;

fn main() -> Result<()> {
    if whoami::username() != "root" {
        // return Err(anyhow!("You need to run eyMate with admin rights!"));
    }

    create_config_dir()?;
    create_data_dir()?;

    let config_file = get_config_file()?;

    let config: Config = Figment::new().merge(Toml::file(config_file)).extract()?;

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
            recognition::cmd_add(config, add_matches.get_one::<String>("USER").unwrap())
        }
        Some(("test", test_matches)) => {
            recognition::cmd_test(config, test_matches.get_one::<String>("USER").unwrap())
        }
        _ => unreachable!(),
    };

    if let Err(err) = err {
        println!("Command failed with:\n{}", err);
    }

    Ok(())
}
