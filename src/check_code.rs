mod experience_code;
use std::env;

use experience_code::ExperienceCode;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let query = &args[1];
    let experience_code = ExperienceCode::from(query.to_owned());
    println!("{:#?}", experience_code.to_usize());
    Ok(())
}
