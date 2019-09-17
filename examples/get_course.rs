use std::error::Error;

use wut_jwc::Client;

const USER: &str = "";
const PASSWORD: &str = "";

fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    client.login(USER, PASSWORD)?;
    client.get_courses()?;
    Ok(())
}
