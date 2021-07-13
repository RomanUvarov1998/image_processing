#[derive(RustEmbed)]
#[folder = "icons\\"]
pub struct Asset;

#[derive(Clone, Copy, Debug)]
pub enum AssetItem {
    AddStep,
    DeleteStep,
    EditStep,
    Export,
    Import,
    ReorderSteps,
    RunStepsChain,
    HaltProcessing,
    FitImage,
    CropImage,
}

impl AssetItem {
    pub fn to_path(&self) -> &'static str {
        match self {
            AssetItem::AddStep => "add step.png",
            AssetItem::DeleteStep => "delete step.png",
            AssetItem::EditStep => "edit step.png",
            AssetItem::Export => "export.png",
            AssetItem::Import => "import.png",
            AssetItem::ReorderSteps => "reorder steps.png",
            AssetItem::RunStepsChain => "run step.png",
            AssetItem::HaltProcessing => "stop processing.png",
            AssetItem::FitImage => "stretch.png",
            AssetItem::CropImage => "crop.png",
        }
    }
}