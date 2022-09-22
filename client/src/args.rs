use clap::Parser;

/// Simple client that sends messages to the server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Message to send to the server
    #[clap(short, long, value_parser)]
    pub message: String,

    /// Number of times to send the memssage
    #[clap(short, long, value_parser, default_value_t = 1)]
    pub repeat: u64,

    /// Time in ms to wait between messages
    #[clap(short, long, value_parser, default_value_t = 0)]
    pub delay: u64,

    /// Wait for response before sending next message
    #[clap(short, long, value_parser, default_value_t = false)]
    pub wait: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
