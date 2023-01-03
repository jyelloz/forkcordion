use clap::Parser;
use clio::Input;

use console::style;

use forkcordion::applesingle;

#[derive(Parser, Debug)]
#[clap(name = "applesingle-info", author, version, about)]
struct InfoCommand {
    #[clap(value_parser, default_value = "-")]
    input: Input,
}

fn main() {

    let cmd = InfoCommand::parse();

    eprintln!(
        "info on {:?}",
        style(&cmd.input).yellow(),
    );

    let archive = applesingle::parse(cmd.input)
        .expect("could not decode as applesingle");

    eprintln!("format={}", style(archive.format()).cyan());

}
