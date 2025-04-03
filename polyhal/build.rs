fn main() {
    // let ac = autocfg::new();
    // ac.set_no_std(true);
    // ac.
    println!(
        "cargo::rustc-env=HAL_ENV_ARCH={}",
        std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    );

    // set_var(
    //     "HAL_ENV_ARCH",
    //     std::env::var("CARGO_CFG_TARGET_ARCH").unwrap(),
    // );
}
