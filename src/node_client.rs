use std::{
    assert_eq,
    borrow::Cow,
    collections::HashMap,
    convert::identity,
    error::Error,
    mem::size_of,
    num::NonZeroU64,
    ops::DerefMut, sync::Arc};

use anyhow::{Result, format_err};
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{commitment_config::CommitmentConfig,
                 account::Account,
                 transaction::Transaction,
                 signature::Keypair,
                 signer::Signer,
};
use solana_client::{
    client_error::Result as ClientResult,
    rpc_client::RpcClient,
    rpc_config,
    rpc_filter,
};
use solana_client::rpc_response::RpcResult;
use solana_client::rpc_config::RpcAccountInfoConfig;


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
    pub fn new(network_type: NetworkType, payer: Keypair, path: &String) -> Result<Self, Box::<dyn Error>> {
        let client = get_rpc_client(&network_type)?;
        Ok(Client { rpc_client: client, payer})
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.rpc_client
    }
    pub fn payer(&self) -> Pubkey {
        self.payer.pubkey()
    }



}

