#[macro_use]
extern crate pamsm;

use lib::cmd_auth;
use pamsm::{Pam, PamError, PamFlags, PamLibExt, PamServiceModule};

mod config;
mod lib;
mod paths;

struct PamFace;

fn auth(pamh: Pam) -> PamError {
    let user = match pamh.get_user(None) {
        Ok(Some(u)) => u,
        Ok(None) => return PamError::USER_UNKNOWN,
        Err(e) => return e,
    };

    // println!("User: {}", user.to_str().unwrap());

    let res = cmd_auth(user.to_str().unwrap_or("_"));

    if let Err(err) = res {
        panic!("{:?}", err);

        // return PamError::AUTH_ERR;
    } else if let Ok(worked) = res {
        if worked {
            return PamError::SUCCESS;
        }
    }

    PamError::AUTH_ERR
}

impl PamServiceModule for PamFace {
    fn authenticate(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        auth(pamh)
    }
}

pam_module!(PamFace);
