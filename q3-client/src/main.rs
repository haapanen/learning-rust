use clap::Parser;
mod q3_client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 's', long)]
    host: String,
}

fn main() {
    let args = Args::parse();

    let client = q3_client::Q3Client::new(args.host);

    let status = client.get_status().unwrap();

    println!("{}", serde_json::to_string_pretty(&status).unwrap());
}
