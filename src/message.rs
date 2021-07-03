#[derive(Debug, Copy, Clone)]
pub enum Msg {
    Project(Project),
    StepOp(StepOp),
    Proc(Proc),
}

#[derive(Debug, Copy, Clone)]
pub enum Project {
    Import (ImportType),
    SaveProject,
    LoadProject,
    SaveResults,
}

#[derive(Debug, Copy, Clone)]
pub enum StepOp {
    AddStep(AddStep),
    Edit { step_num: usize }, 
    Move { step_num: usize, direction: MoveStep }, 
    Delete { step_num: usize },
}

#[derive(Debug, Copy, Clone)]
pub enum Proc {
    ChainIsStarted { step_num: usize, do_until_end: bool },
    Progress { step_num: usize, cur_percents: usize },
    Halt,
    Completed { step_num: usize },
}

#[derive(Debug, Copy, Clone)]
pub enum ImportType { File, SystemClipoard }

#[derive(Debug, Copy, Clone)]
pub enum AddStep {
    LinCustom, 
    LinMean, 
    LinGauss, 
    Median, 
    HistogramLocalContrast, 
    CutBrightness, 
    HistogramEqualizer, 
    Rgb2Gray, 
    NeutralizeChannel, 
    ExtractChannel, 
}

#[derive(Debug, Copy, Clone)]
pub enum MoveStep { Up, Down }