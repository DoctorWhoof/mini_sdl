#[derive(Debug, PartialEq, Default)]
pub enum Scaling {
    #[default]
    PreserveAspect,
    Integer,
    StretchToWindow,
}
