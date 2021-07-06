#[derive(Debug)]
pub enum BWError  {
	ThreadExchangeNotEmpty,
	TaskIsEmpty,
	TaskResultIsEmpty,
	NotFoundExpectedTask,
	NoInitialImage,
	NoPrevStepImg,
	NoStepImg,
	StepNumExceedsBounds,
	Custom { msg: String },
}