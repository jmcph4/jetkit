use clap::Parser;
use futures::{stream, StreamExt};
use reqwest::{Client, IntoUrl};
use url::Url;

const NUM_CONCURRENT_REQS: usize = 8;

#[inline]
fn request(private: bool, tx: &String) -> String {
    let method = if private {
        "eth_sendPrivateRawTransaction"
    } else {
        "eth_sendRawTransaction"
    };
    format!("{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"{}\",\"params\":[\"{}\"]}}", method, tx)
}

async fn send_tx<U>(url: U, tx: &String, private: bool) -> eyre::Result<String>
where
    U: IntoUrl,
{
    let resp = Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .body(request(private, tx))
        .send()
        .await?;
    Ok(resp.text().await?)
}

/// Gets transactions onto the Ethereum blockchain
#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
struct Opts {
    /// Hexadecimal string representing valid transaction RLP bytes
    pub tx: String,
    /// List of endpoints to submit the transaction to
    #[clap(long, short)]
    pub rpcs: Vec<Url>,
    /// Attempt to decode the RLP bytes into an EIP-2718 envelope and halt if this fails
    #[clap(long, short, action)]
    pub strict: bool,
    /// Do not print to standard output
    #[clap(long, short, action)]
    pub quiet: bool,
    /// Use `eth_sendPrivateRawTransaction`
    #[clap(long, short, action)]
    pub private: bool,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let opts = Opts::parse();

    if opts.strict {
        todo!()
    }

    let mut stream = stream::iter(opts.rpcs.iter().map(async |url| {
        (url, send_tx(url.clone(), &opts.tx, opts.private).await)
    }))
    .buffer_unordered(NUM_CONCURRENT_REQS);

    while let Some((url, res)) = stream.next().await {
        if !opts.quiet {
            println!("{} said: {}", url, res?);
        }
    }

    Ok(())
}
