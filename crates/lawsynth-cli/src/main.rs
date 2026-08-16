fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    match lawsynth_cli::run(&arguments) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
