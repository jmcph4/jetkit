use std::{fs, path::PathBuf};

use clap::Parser;
use eyre::eyre;
use secp256k1::{rand, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use zeroize::Zeroize;

const ETHEREUM_ADDRESS_LEN: usize = 20;

/// The number of bytes in an uncompressed public key (**ignoring the
/// leading prefix indicating compression status**)
const PUBLIC_KEY_NUM_BYTES_UNCOMPRESSED: usize = 64;

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
    /// When reading or writing private keys, interpret them as 0x-prefixed
    /// UTF-8-encoded hexadecimal strings (unless `--no-hex-prefix` is
    /// specified)
    #[clap(long, short = 'a')]
    pub human_readable: bool,
    /// When reading or writing private keys in human-readable, hexadecimal
    /// form, do not use 0x-prefixes
    #[clap(long, short = 'b')]
    pub no_hex_prefix: bool,
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
        let mut private_key_bytes = if opts.human_readable {
            if opts.no_hex_prefix {
                hex::decode(fs::read(in_path)?)?
            } else {
                hex::decode(&fs::read(in_path)?[2..])?
            }
        } else {
            fs::read(in_path)?
        };
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
        if opts.human_readable {
            if opts.no_hex_prefix {
                fs::write(out_path, hex::encode(private_key.secret_bytes()))?;
            } else {
                fs::write(
                    out_path,
                    format!("0x{}", hex::encode(private_key.secret_bytes())),
                )?;
            }
        } else {
            fs::write(out_path, private_key.secret_bytes())?;
        }
    }

    if !opts.quiet {
        /* We must serialise the uncompressed public key, in accordance with
         * the Yellow Paper.
         * */
        let public_key_bytes = &public_key.serialize_uncompressed()[1..];
        assert!(public_key_bytes.len() == PUBLIC_KEY_NUM_BYTES_UNCOMPRESSED);

        if opts.display_public_key {
            println!("0x{}", hex::encode(public_key_bytes));
        } else {
            let mut hasher = Keccak256::new();
            hasher.update(public_key_bytes);
            let digest = hasher.finalize();
            let address_bytes = &digest[digest.len() - ETHEREUM_ADDRESS_LEN..];
            assert!(address_bytes.len() == ETHEREUM_ADDRESS_LEN);
            println!("0x{}", hex::encode(address_bytes));
        }
    }

    Ok(())
}
