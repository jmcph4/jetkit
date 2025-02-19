use std::{fs, path::PathBuf};

use clap::Parser;
use eyre::eyre;
use secp256k1::{rand, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use zeroize::Zeroize;

const ETHEREUM_ADDRESS_LEN: usize = 20;

/// Generates a SECP-256k1 keypair or displays the Ethereum address associated
/// with an existing one
#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
struct Opts {
    /// Do not write to standard output
    #[clap(long, short)]
    pub quiet: bool,
    /// Display the public key instead of the Ethereum address
    #[clap(long, short)]
    pub display_public_key: bool,
    /// Path to private key
    #[clap(long, short)]
    pub r#in: Option<PathBuf>,
    /// Path to save generated private key to
    #[arg(required_unless_present = "in")]
    pub out: Option<PathBuf>,
}

fn main() -> eyre::Result<()> {
    let opts = Opts::parse();

    let handle = Secp256k1::new();

    let (private_key, public_key) = if let Some(in_path) = opts.r#in {
        let mut private_key_bytes = fs::read(in_path)?;
        let private_key_slice: &[u8; 32] =
            match private_key_bytes.as_slice().try_into() {
                Ok(t) => t,
                Err(e) => {
                    return Err(eyre!(
                        "Incorrect data length for private key: {e:?}"
                    ))
                }
            };
        let private_key: SecretKey =
            match SecretKey::from_byte_array(private_key_slice) {
                Ok(t) => t,
                Err(e) => return Err(eyre!("Invalid private key: {e:?}")),
            };
        private_key_bytes.zeroize();
        (private_key, private_key.public_key(&handle))
    } else {
        handle.generate_keypair(&mut rand::thread_rng())
    };

    if let Some(out_path) = opts.out {
        fs::write(out_path, private_key.secret_bytes())?;
    }

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
