/// The scaling strategy. A "VariableAspectRatio" option may be provided in the future.

#[derive(Debug, PartialEq, Default)]
pub enum Scaling {
    #[default]
    /// Fits the render target to the window while preserving the aspect ratio with black bars.
    /// Curenly does not work well with vertical aspect ratios, this will be fixed.
    PreserveAspect,

    /// Same as PreserveAspect, but performs integer scaling for best results with pixel art.
    /// Usually increases the amount of black bars around the render target.
    Integer,

    /// Stretches the render target to the window, completely disregarding the aspect ratio. The
    /// Picture Gods will smite you if you release anything using this.
    StretchToWindow,

    /// Does not stretch, changes the canvas dimensions instead causing graphics to stay at same size,
    /// but with more or less space around them.
    Fill
}
