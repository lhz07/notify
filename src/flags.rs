use crate::proto::Level;
use bitflags::bitflags;

impl From<Level> for LevelFlag {
    fn from(value: Level) -> Self {
        match value {
            Level::Info => LevelFlag::Info,
            Level::Notice => LevelFlag::Notice,
            Level::Warning => LevelFlag::Warning,
            Level::Critical => LevelFlag::Critical,
        }
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct LevelFlag: u8{
        const Info     = 0b0001;
        const Notice   = 0b0010;
        const Warning  = 0b0100;
        const Critical = 0b1000;
        const ALL      = 0b1111;
    }
}

impl Level {
    pub const LIST: [Level; 4] = [Level::Info, Level::Notice, Level::Warning, Level::Critical];
}

impl Default for LevelFlag {
    fn default() -> Self {
        Self::ALL
    }
}
