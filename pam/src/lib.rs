#[macro_use]
extern crate pamsm;

use pamsm::{Pam, PamError, PamFlags, PamLibExt, PamServiceModule};
use recognition::cmd_auth;

struct PamFace;

fn auth(pamh: Pam) -> PamError {
    let user = match pamh.get_user(None) {
        Ok(Some(u)) => u,
        Ok(None) => return PamError::USER_UNKNOWN,
        Err(e) => return e,
    };

    // println!("User: {}", user.to_str().unwrap());

    let res = cmd_auth(user.to_str().unwrap());

    if let Err(err) = res {
        println!("{}", err);
        return PamError::AUTH_ERR;
    } else if let Ok(worked) = res {
        if worked {
            return PamError::SUCCESS;
        }
    }

    PamError::AUTH_ERR
}

impl PamServiceModule for PamFace {
    fn open_session(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        auth(pamh)
    }

    fn authenticate(pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        auth(pamh)
    }

    fn close_session(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        PamError::IGNORE
    }

    fn setcred(_: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        PamError::IGNORE
    }

    fn acct_mgmt(_: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        PamError::IGNORE
    }

    fn chauthtok(_: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        PamError::IGNORE
    }
}

pam_module!(PamFace);
