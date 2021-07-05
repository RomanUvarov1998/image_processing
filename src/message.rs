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
    SaveResults (SaveResults),
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
    ChainIsStarted { step_num: usize, process_until_end: bool },
    StepProgress { num: usize, step_percents: usize, total_percents: usize },
    Halted,
    CompletedStep { num: usize },
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

#[derive(Debug, Copy, Clone)]
pub enum SaveResults {
    Start,
    Completed { percents: usize, last_result_is_saved: bool },
    Error
}