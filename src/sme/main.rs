#![feature(let_chains)]
use std::{
    fmt, fs,
    io::{self, stdin, BufRead, Read},
    path::PathBuf,
};

use clap::Parser;
use eyre::{eyre, ErrReport};
use serde::Deserialize;
use serde_with::{serde_as, skip_serializing_none};

/// Number of bytes that denote the length of the CBOR data
const CBOR_METADATA_LENGTH_LEN: usize = 2;

/// URL prefix to use for the construction of IPFS gateway URLs
///
/// It's somewhat unfortunate to have to kingmake a given IPFS gateway but the
/// feature (--gateway) is purely one of convenience and, from the user's
/// perspective, having to specify your own IPFS gateway defeats the point.
const IPFS_GATEWAY_URL_PREFIX: &str = "https://ipfs.io/ipfs";

/// Number of bytes that denote the version of the Solidity compiler
const SOLIDITY_VERSION_LEN: usize = 3;

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize)]
/// Represents smart contract metadata encoded by the Solidity reference
/// compiler
struct CanonicalMetadata<'a> {
    /* We must tolerate the explicit lifetime here due to `serde_cbor`'s idea of
     * which Rust types map to which CBOR types. Specifically, `serde_cbor`
     * considers a `Vec<u8>` to be a byte *array* not a *sequence*. We need to
     * deserialise the latter, hence the `&[u8]`.
     * */
    /// IPFS hash of the metadata file
    ipfs: Option<&'a [u8]>,
    /// Swarm (v0) hash of the metadata file
    bzzr0: Option<&'a [u8]>,
    /// Swarm (v1) hash of the metadata file
    bzzr1: Option<&'a [u8]>,
    /// Was this contract compiled with experimental compiler features enabled?
    experimental: Option<bool>,
    /// Version of the Solidity reference compiler used to compile this contract
    solc: Option<&'a [u8]>,
}

impl<'a> TryFrom<CanonicalMetadata<'a>> for Metadata {
    type Error = ErrReport;

    fn try_from(meta: CanonicalMetadata<'a>) -> Result<Self, Self::Error> {
        /* Always prefer the most recent Swarm version. In reality, there
         * should never be multiple fields specified but the entire metadata
         * concept is completely ad hoc anyway so our implementation had ought
         * to be as robust as is reasonably necessary.
         * */
        let swarm = match (meta.bzzr0, meta.bzzr1) {
            (Some(_), Some(b)) => Some(b.to_vec()),
            (Some(a), None) => Some(a.to_vec()),
            (None, Some(b)) => Some(b.to_vec()),
            (None, None) => None,
        };
        Ok(Self {
            digest: if let Some(ipfs) = meta.ipfs {
                /* Critically, the human-readable CID (i.e., that one might use
                 * to access via a gateway, for instance), is the
                 * Base58-encoded form of the IPFS digest byte sequence
                 * */
                Some(Digest::Ipfs(bs58::encode(&ipfs).into_string()))
            } else {
                swarm.map(|swarm| Digest::Swarm(hex::encode(swarm)))
            },
            experimental: meta.experimental.unwrap_or_default(),
            solidity_version: match meta.solc {
                Some(bytes) => Some(SolidityVersion::try_from(bytes)?),
                None => None,
            },
        })
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
/// Represents a version of the Solidity reference compiler
pub struct SolidityVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl TryFrom<&[u8]> for SolidityVersion {
    type Error = ErrReport;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() == SOLIDITY_VERSION_LEN {
            Ok(Self {
                major: bytes[0],
                minor: bytes[1],
                patch: bytes[2],
            })
        } else {
            Err(eyre!("Incorrect number of bytes for Solidity version"))
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Represents a digest within the contract metadata
///
/// The idea here is that we want *one* canonical digest despite the
/// possibility of there being multiple present in the metadata.
pub enum Digest {
    Ipfs(String),
    Swarm(String),
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Digest::Ipfs(ipfs) => write!(f, "ipfs://{ipfs}"),
            Digest::Swarm(swarm) => write!(f, "bzz://{swarm}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
/// Represents the high-level, "nice" view of contract metadata
pub struct Metadata {
    /// Digest that can be used to retrieve the standard metadata file
    /// from some form of distributed filesystem
    pub digest: Option<Digest>,
    /// Was this bytecode produced with experimental compiler features enabled?
    pub experimental: bool,
    /// Version of the Solidity reference compiler used to produce this contract
    pub solidity_version: Option<SolidityVersion>,
}

impl TryFrom<&[u8]> for Metadata {
    type Error = ErrReport;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() >= CBOR_METADATA_LENGTH_LEN {
            let cbor_len = usize::from_be_bytes([
                0,
                0,
                0,
                0,
                0,
                0,
                bytes[bytes.len() - CBOR_METADATA_LENGTH_LEN],
                bytes[bytes.len() - (CBOR_METADATA_LENGTH_LEN - 1)],
            ]);
            let cbor_bytes = &bytes[bytes.len()
                - (cbor_len + CBOR_METADATA_LENGTH_LEN)
                ..(bytes.len() - CBOR_METADATA_LENGTH_LEN)];
            assert!(
                cbor_bytes.len() == cbor_len,
                "INVARIANT VIOLATED: Decoding of CBOR bytes or the length of the CBOR bytes is incorrect."
            );
            let canonical_metadata: CanonicalMetadata =
                serde_cbor::from_slice(cbor_bytes)?;
            Ok(canonical_metadata.try_into()?)
        } else {
            Err(eyre!("Insufficient data"))
        }
    }
}

/// Extracts Solidity metadata from contract bytecode
#[derive(Clone, Debug, Parser)]
#[clap(version, about, author)]
struct Opts {
    /// Interpret input (from either standard input or a file via --bytecode) as literal bytes
    #[clap(short, long, action)]
    pub raw: bool,
    /// Print metadata representation to standard output
    #[clap(short, long, action)]
    pub metadata: bool,
    /// Display IPFS digests as URLs to the default IPFS web gateway
    #[clap(short, long, action)]
    pub gateway: bool,
    /// Provide file path to a file containing bytecode (interpretation depends on --raw)
    #[clap(short, long)]
    pub bytecode: Option<PathBuf>,
}

fn main() -> eyre::Result<()> {
    let opts = Opts::parse();
    let bytes = if let Some(path) = opts.bytecode {
        if opts.raw {
            fs::read(path)?
        } else {
            let s = fs::read_to_string(path)?;
            hex::decode(&s[2..])?
        }
    } else if opts.raw {
        let mut buf = Vec::new();
        io::stdin().lock().read_to_end(&mut buf)?;
        buf
    } else {
        let mut line = String::new();
        stdin().lock().read_line(&mut line)?;
        let line = line.trim_end();
        hex::decode(line[2..].trim_end())?
    };

    let metadata = Metadata::try_from(&bytes[..])?;

    if opts.metadata {
        println!("{:#?}", &metadata);
    }

    if let Some(digest) = metadata.digest {
        if let Digest::Ipfs(ref ipfs_digest) = digest
            && opts.gateway
        {
            println!("{IPFS_GATEWAY_URL_PREFIX}/{ipfs_digest}")
        } else {
            println!("{digest}");
        }
    }

    Ok(())
}
