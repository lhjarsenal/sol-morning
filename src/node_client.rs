use std::error::Error;

use anyhow::Result;
use solana_program::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_client::{client_error::Result as ClientResult, rpc_client::RpcClient};

pub struct NetworkOpts {
    url: String,
}

pub enum NetworkType {
    Mainnet,
    Devnet,
    Serum,
    Custom(NetworkOpts),
}

impl NetworkType {
    pub fn url(&self) -> &str {
        match self {
            NetworkType::Devnet => "https://api.devnet.solana.com",
            NetworkType::Mainnet => "https://api.mainnet-beta.solana.com",
            NetworkType::Serum => "https://solana-api.projectserum.com",
            NetworkType::Custom(nework_opts) => &nework_opts.url,
        }
    }
}

pub fn get_rpc_client(network: &NetworkType) -> ClientResult<RpcClient> {
    let client = RpcClient::new(network.url().to_string());

    let version = client.get_version()?;
    println!("RPC version: {:?}", version);
    Ok(client)
}

pub struct Client {
    rpc_client: RpcClient,
    payer: Keypair,
}

impl Client {
    pub fn new(network_type: NetworkType, payer: Keypair, _path: &String) -> Result<Self, Box::<dyn Error>> {
        let client = get_rpc_client(&network_type)?;
        Ok(Client { rpc_client: client, payer })
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.rpc_client
    }
    pub fn payer(&self) -> Pubkey {
        self.payer.pubkey()
    }
}

