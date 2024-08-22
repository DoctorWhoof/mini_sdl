/// The timing strategy used on every frame advance.
///
#[derive(Debug, PartialEq, Default)]
pub enum Timing {
    #[default]
    /// The best, smoothest option! Frame timing will be perfectly in sync with the display. However,
    /// you will need to use delta-timing in your game logic to account for different display frequencies,
    /// considering that that both 60Hz and 120Hz displays are common.
    Vsync,
    /// Simply draw as fast as possible. Will max out CPU use, drain battery faster, etc.
    Immediate,
    /// Like Vsync, but prevents frame rate from going past a limit. Easier said than done, micro stutter CAN HAPPEN
    /// If the display frequency doesn't match the limit, and the results will also vary wildly per platform.
    VsyncLimitFPS(f64),
    /// Simply limits the frame rate without attempting Vsync. May yield better results on some platforms.
    ImmediateLimitFPS(f64),
}
