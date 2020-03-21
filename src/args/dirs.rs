use super::*;

pub fn dirs() -> ! {
    let config = get_config_path();
    println!("configuration directory is at..");
    println!("{}", config.display());
    println!();

    let data = get_data_path();
    println!("data directory is at..");
    println!("{}", data.display());
    println!();

    exit(0)
}
