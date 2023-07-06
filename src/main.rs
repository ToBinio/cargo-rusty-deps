use crate::dependencies::Dependencies;
use clap::Parser;

mod dependencies;
mod versions;

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    RustyDeps(RustyDepsArgs),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct RustyDepsArgs {
    #[arg(short, long)]
    update: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = CargoCli::parse();
    let CargoCli::RustyDeps(cli) = cli;

    let mut dependencies = Dependencies::get_all_dependencies()?;

    if cli.update {
        dependencies.outdated();
        dependencies.update();
    }

    println!("{}", dependencies);

    Ok(())
}
