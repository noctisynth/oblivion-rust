fn main() {
    #[cfg(all(feature = "safe", feature = "unsafe"))]
    compile_error!("Error: Only one of feature 'safe' and 'unsafe' can be enabled.");
}
