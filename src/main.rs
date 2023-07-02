use crate::dependencies::get_all_dependencies;

mod dependencies;
mod versions;

fn main() -> anyhow::Result<()> {
    let dependencies = get_all_dependencies()?;

    println!("{}", dependencies);

    Ok(())
}
