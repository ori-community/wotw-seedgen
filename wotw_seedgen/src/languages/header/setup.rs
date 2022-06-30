use crate::util::UberIdentifier;

use super::CodeDisplay;

#[derive(Debug, Clone)]
/// Setup commands that the client executes after parsing the seed
pub enum Setup {
    Timer(SetupTimer),
}

impl Setup {
    pub fn code(&self) -> CodeDisplay<Setup> {
        CodeDisplay::new(self, |s, f| {
            write!(f, "setup ", )?;
            match s {
                Setup::Timer(timer) => write!(f, "{}", timer.code()),
            }
        })
    }
}

#[derive(Debug, Clone)]
/// Set up a toggleable timer uberState
pub struct SetupTimer {
    /// The value form this UberState will determine whether the timer is running
    /// 
    /// This should be a boolean
    pub switch: UberIdentifier,
    /// If the timer is running, this UberState will be counting the seconds passed
    /// 
    /// This should be a float
    pub counter: UberIdentifier,
}

impl SetupTimer {
    pub fn code(&self) -> CodeDisplay<SetupTimer> {
        CodeDisplay::new(self, |s, f| {
            write!(f, "timer|{}|{}", s.switch.code(), s.counter.code())
        })
    }
}
