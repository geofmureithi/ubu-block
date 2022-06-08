use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "ubu-block")]
#[clap(bin_name = "ubu-block")]
enum UbuBlock {
    Pull(Pull),
    /// Run a query on the blockchain
    Query {
        #[clap(short)]
        q: String,
    },
    /// Add a new block
    Import {
        path: std::path::PathBuf,
    },
}

#[derive(clap::Args, Debug)]
#[clap(author, version, about, long_about = "Pull the blockchain from a url")]
struct Pull {
    #[clap(long)]
    manifest_path: Option<std::path::PathBuf>,
}

fn main() {
    let ubu = UbuBlock::parse();
    println!("{:?}", ubu);
}
