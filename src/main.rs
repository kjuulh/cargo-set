mod commands;

fn main() -> anyhow::Result<()> {
    let args = std::env::args();

    commands::cli_execute(args)
}
