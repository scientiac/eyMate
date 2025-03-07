#[macro_use]
extern crate pamsm;

use pamsm::{Pam, PamError, PamFlags, PamLibExt, PamServiceModule};
use recognition::cmd_auth;

struct PamFace;

impl PamServiceModule for PamFace {
    fn authenticate(pamh: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        let user = match pamh.get_user(None) {
            Ok(Some(u)) => u,
            Ok(None) => return PamError::USER_UNKNOWN,
            Err(e) => return e,
        };

        println!("User: {}", user.to_str().unwrap());

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
}

pam_module!(PamFace);
