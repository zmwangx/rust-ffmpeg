use std::env;

fn main() {
    for (name, _value) in env::vars() {
        if name.starts_with("DEP_FFMPEG_") {
            println!(
                r#"cargo:rustc-cfg=feature="{}""#,
                name["DEP_FFMPEG_".len()..name.len()].to_lowercase()
            );
        }
    }
}
