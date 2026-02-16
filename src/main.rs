fn main() {
    let cli = match watch::cli::Cli::parse_args() {
        Ok(cli) => cli,
        Err(err) => {
            err.exit();
        }
    };
    let config = match watch::Config::from_cli(cli) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("watch: {err}");
            std::process::exit(2);
        }
    };

    match watch::app::run(config) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("watch: {err}");
            std::process::exit(2);
        }
    }
}
