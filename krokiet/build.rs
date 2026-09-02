use std::env;

fn main() {
    if env::var("SLINT_STYLE").is_err() || env::var("SLINT_STYLE") == Ok(String::new()) {
        slint_build::compile_with_config("ui/main_window.slint", slint_build::CompilerConfiguration::new().with_style("cupertino-dark".into())).expect("Unable to compile slint file");
    } else {
        slint_build::compile("ui/main_window.slint").expect("Unable to compile slint file");
    }
}
