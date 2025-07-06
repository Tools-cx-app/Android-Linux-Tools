use clap::Parser;

use crate::utils::option_to_str;

#[derive(Parser)]
#[command(author,  about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Install the linux
    Install {
        /// Download the address of rootfs.
        #[arg(long, default_value = "")]
        mirror: Option<String>,
    },
}

pub fn run() {
    let args = Cli::parse();

    match args.command {
        Commands::Install { mirror } => {
            let url = {
                if mirror == None {
                    "https://images.linuxcontainers.org".to_string()
                } else if option_to_str(mirror.clone()).starts_with("https") {
                    option_to_str(mirror).to_string()
                } else {
                    String::from(format!("https://{}", option_to_str(mirror)))
                }
            };
            println!("{}", url);
        }
    }
}
