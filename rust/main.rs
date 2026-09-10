mod assets;
mod create;
mod install_check;
mod npm_packages;
mod path_utils;
mod process;
mod validate;

fn usage() -> String {
    format!(
        concat!(
            "Harness Alchemist {}\n\n",
            "Usage: harness-alchemist <command> [options]\n\n",
            "Commands:\n",
            "  create <directory>    Create a universal coding-agent plugin repository.\n",
            "  validate [directory] Validate a generated repository.\n",
            "  install-check        Install the plugin into local harness CLIs and verify\n",
            "                        discovery (claude, codex, agy, opencode, dsh).\n",
            "  templates            List bundled canonical templates.\n",
            "  version              Print the CLI version.\n",
            "  help                 Show this help.\n\n",
            "Aliases: init and new are aliases for create."
        ),
        env!("HARNESS_ALCHEMIST_VERSION")
    )
}

fn main() {
    let mut arguments = std::env::args().skip(1);
    let command = arguments.next();
    let args = arguments.collect::<Vec<_>>();
    let status = match command.as_deref() {
        None | Some("help" | "--help" | "-h") => {
            println!("{}", usage());
            0
        }
        Some("version" | "--version" | "-v") => {
            println!("{}", env!("HARNESS_ALCHEMIST_VERSION"));
            0
        }
        Some("templates") => {
            println!("{}", create::list_templates());
            0
        }
        Some("create" | "init" | "new") => create::run(&args),
        Some("validate") => validate::run(&args),
        Some("install-check") => install_check::run(&args),
        Some("__stage-native-packages") => npm_packages::run(&args),
        Some(command) => {
            eprintln!("Unknown command: {command}\n");
            eprintln!("{}", usage());
            2
        }
    };
    std::process::exit(status);
}
