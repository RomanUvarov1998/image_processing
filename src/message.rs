#[derive(Debug, Copy, Clone)]
pub enum Processing {
    StepsChainIsStarted { step_num: usize, do_until_end: bool },
    StepProgress { step_num: usize, cur_percents: usize },
    StepIsCompleted { step_num: usize },
}

#[derive(Debug, Copy, Clone)]
pub enum Project {
    LoadImage,
    SaveProject,
    LoadProject,
    SaveResults,
}

#[derive(Debug, Copy, Clone)]
pub enum MoveStep { Up, Down }

#[derive(Debug, Copy, Clone)]
pub enum AddStep {
    AddStepLinCustom, 
    AddStepLinMean, 
    AddStepLinGauss, 
    AddStepMed, 
    AddStepHistogramLocalContrast, 
    AddStepCutBrightness, 
    AddStepHistogramEqualizer, 
    AddStepRgb2Gray, 
    AddStepNeutralizeChannel, 
    AddStepExtractChannel, 
}

#[derive(Debug, Copy, Clone)]
pub enum StepOp {
    EditStep { step_num: usize }, 
    MoveStep { step_num: usize, direction: MoveStep }, 
    DeleteStep { step_num: usize },
}

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Project(Project),
    AddStep(AddStep),
    StepOp(StepOp),
    Processing(Processing),
}