mod reach_check;

use std::io::{self, BufRead, BufReader, StdinLock};

use reach_check::{new_world, reach_check, relevant_uber_states, GraphCache, ReachCheckMessage};
use serde::Deserialize;
use wotw_seedgen::{logic_language::ast, seed::SeedgenInfo, UberStates};

use crate::{cli::VerboseArgs, log_config::LogConfig, seed::LogicFiles, Error};

pub fn daemon(args: VerboseArgs) -> Result<(), Error> {
    LogConfig::from_args(args, "seedgen_daemon_log.txt").apply()?;

    let mut daemon = Daemon::new()?;
    let areas = ast::parse(&daemon.logic_files.areas_source.content).into_result()?;
    let uber_states = UberStates::new(&daemon.logic_files.uber_state_data);

    let mut graph_cache = GraphCache::new(&areas, &daemon.logic_files);
    let mut world = None;

    loop {
        let Some(message) = read_message(&mut daemon.stdin, &mut daemon.buf)? else {
            return Ok(());
        };

        match message {
            Message::RelevantUberStates => relevant_uber_states(&daemon.logic_files)?,
            Message::SetSeedgenInfo(info) => {
                graph_cache.set_settings(info.universe_settings)?;
                world = Some(new_world(
                    &info.spawn_identifier,
                    info.world_index,
                    &uber_states,
                    &graph_cache,
                )?);
            }
            Message::ReachCheck(message) => {
                reach_check(message, &daemon.logic_files.uber_state_data, &mut world)?;
            }
        }
    }
}

pub struct Daemon {
    stdin: BufReader<StdinLock<'static>>,
    buf: String,
    logic_files: LogicFiles,
}

impl Daemon {
    fn new() -> Result<Self, Error> {
        Ok(Self {
            stdin: BufReader::new(io::stdin().lock()),
            buf: String::new(),
            logic_files: LogicFiles::new()?,
        })
    }
}

fn read_message(
    stdin: &mut BufReader<StdinLock>,
    buf: &mut String,
) -> Result<Option<Message>, Error> {
    buf.clear();
    let bytes = stdin.read_line(buf)?;
    if bytes == 0 {
        return Ok(None);
    }

    let message =
        serde_json::from_str(buf).map_err(|err| format!("failed to parse message: {err}"))?;
    Ok(Some(message))
}

#[derive(Deserialize)]
enum Message {
    RelevantUberStates,
    SetSeedgenInfo(SeedgenInfo),
    ReachCheck(ReachCheckMessage),
}
