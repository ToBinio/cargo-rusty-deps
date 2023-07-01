use crate::dependencies::{get_all_dependencies, print_dependencies};

mod dependencies;

fn main() -> anyhow::Result<()> {
    let dependencies = get_all_dependencies()?;

    print_dependencies(&dependencies);

    Ok(())
}
