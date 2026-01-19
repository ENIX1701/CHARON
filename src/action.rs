use crate::models::{Ghost, Task};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // system lifecycle
    Tick,
    Quit,
    Resize(u16, u16),

    // global navigation
    NextTab,
    PrevTab,
    ToggleHelp,

    // raw input
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Char(char),

    // dashboard
    OpenActionMenu,
    ConfirmKillGhost,

    // config
    SubmitGhostConfig,

    // builder
    ToggleBuilderSwitch,
    StartBuild,

    // results
    ReceiveGhosts(Result<Vec<Ghost>, String>),
    ReceiveTasks(Result<Vec<Task>, String>),
    ReceiveTaskSendResult(Result<String, String>),
    ReceiveConfigUpdateResult(Result<String, String>),
    ReceiveKillResult(Result<String, String>),
    ReceiveBuildResult(Result<String, String>),

    AutoRefresh
}
