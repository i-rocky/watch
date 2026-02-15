fn main() {
    let cli = match watch::cli::Cli::parse_args() {
        Ok(cli) => cli,
        Err(err) => {
            err.exit();
        }
    };
    if let Err(err) = watch::Config::from_cli(cli) {
        eprintln!("watch: {err}");
        std::process::exit(2);
    }
}
