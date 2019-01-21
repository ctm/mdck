use mdck::{Config, MdckError};

fn mdck() -> Result<(), MdckError> {
    let config = Config::new()?;

    mdck::ck_sources(&config)
}

fn main() {
    std::process::exit(match mdck() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    });
}
