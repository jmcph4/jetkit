use std::{fs, path::PathBuf};

use clap::Parser;
use secp256k1::{rand, Secp256k1};
use sha3::{Digest, Keccak256};

const ETHEREUM_ADDRESS_LEN: usize = 20;

#[derive(Clone, Debug, Parser)]
#[clap(author, version)]
struct Opts {
    #[clap(long, short)]
    pub quiet: bool,
    #[clap(long, short)]
    pub display_public_key: bool,
    pub out: PathBuf,
}

fn main() -> eyre::Result<()> {
    let opts = Opts::parse();

    let (private_key, public_key) =
        Secp256k1::new().generate_keypair(&mut rand::thread_rng());

    fs::write(opts.out, private_key.secret_bytes())?;

    if !opts.quiet {
        let public_key_bytes = public_key.serialize();

        if opts.display_public_key {
            println!("0x{}", hex::encode(public_key_bytes));
        } else {
            let mut hasher = Keccak256::new();
            hasher.update(public_key_bytes);
            let digest = hasher.finalize();
            let address_bytes = &digest[..ETHEREUM_ADDRESS_LEN];
            println!("0x{}", hex::encode(address_bytes));
        }
    }

    Ok(())
}
