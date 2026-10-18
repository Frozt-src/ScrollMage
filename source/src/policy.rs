pub fn intercept(
    delta: i32,
    injected: bool,
    modified: bool,
    eligible: bool,
    enabled: bool,
) -> bool {
    enabled && eligible && !injected && !modified && delta != 0 && delta % 120 == 0
}
