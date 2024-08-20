
#[derive(Debug, PartialEq, Default)]
pub enum Timing {
    #[default]
    Vsync,
    Immediate,
    VsyncLimitFPS(f64),
    ImmediateLimitFPS(f64),
}
