use std::time::Duration;

use crate::{cli::HttpServerArgs, Error};

pub fn http_server(args: HttpServerArgs) -> Result<(), Error> {
    let HttpServerArgs { inactivity_timeout } = args;

    wotw_seedgen_http_server::start(inactivity_timeout.map(Duration::from))?;

    Ok(())
}
