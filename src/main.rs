use crate::dependencies::Dependencies;

mod dependencies;
mod versions;

fn main() -> anyhow::Result<()> {
    let dependencies = Dependencies::get_all_dependencies()?;

    println!("{}", dependencies);

    Ok(())
}
