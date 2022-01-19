use std::collections::HashMap;
use std::fs;
use rocket_contrib::json::Json;
use crate::api;
use crate::response;
use api::RawTokenAddr;
use response::TokenListResponse;
use serde::{Serialize, Deserialize};

pub fn token_list(page: Option<u32>, pagesize: Option<u32>, search: Option<String>, address: Option<String>, symbol: Option<String>) -> Json<TokenListResponse> {
    let token_main_path = "./token_mint.json".to_string();
    let raw_info = fs::read_to_string(token_main_path).expect("Error read file");
    let mut vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();

    //查询固定某一个address
    match address {
        Some(a) => {
            for token in vec.iter() {
                if token.address.eq(&a) {
                    return Json(TokenListResponse {
                        total: 1,
                        pagesize: 1,
                        page: 1,
                        data: vec![token.clone()],
                    });
                }
            }
            return Json(TokenListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
        }
        None => {}
    }

    //模糊查询symbol
    match search {
        Some(search) => {
            let match_search = search.trim();
            vec = vec
                .into_iter()
                .filter(|x|
                    x.symbol.to_uppercase().trim().contains(match_search)
                ).collect();
        }
        None => {}
    }

    //精确查询symbol
    match symbol {
        Some(symbol) => {
            for token in vec.iter() {
                if token.symbol.eq(&symbol) {
                    return Json(TokenListResponse {
                        total: 1,
                        pagesize: 1,
                        page: 1,
                        data: vec![token.clone()],
                    });
                }
            }
            return Json(TokenListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
        }
        None => {}
    }
    let start_page;
    let mut size = 50;
    let start_index;
    let total = vec.len() as u32;

    if total == 0 {
        return Json(TokenListResponse {
            total,
            pagesize: total,
            page: 1,
            data: vec![],
        });
    }

    match page {
        Some(p) => {
            start_page = p - 1;
        }
        None => {
            return Json(TokenListResponse {
                total,
                pagesize: total,
                page: 1,
                data: vec,
            });
        }
    }

    match pagesize {
        Some(s) => {
            size = s;
        }
        None => {}
    }

    start_index = start_page * size;
    let mut end_index = start_index + size;

    if end_index >= total {
        end_index = total;
    }

    let res = vec[start_index as usize..end_index as usize].to_vec();
    Json(TokenListResponse {
        total,
        pagesize: size,
        page: start_page + 1,
        data: res,
    })
}

pub fn eth_tokens(page: Option<u32>, pagesize: Option<u32>, search: Option<String>, address: Option<String>, symbol: Option<String>) -> Json<TokenListResponse> {
    let token_main_path = "./resource/token/ethereum.json".to_string();
    let raw_info = fs::read_to_string(token_main_path).expect("Error read file");
    let mut vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();

    //查询固定某一个address
    match address {
        Some(a) => {
            for token in vec.iter() {
                if token.address.eq(&a) {
                    return Json(TokenListResponse {
                        total: 1,
                        pagesize: 1,
                        page: 1,
                        data: vec![token.clone()],
                    });
                }
            }
            return Json(TokenListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
        }
        None => {}
    }

    //模糊查询symbol
    match search {
        Some(search) => {
            let match_search = search.trim();
            vec = vec
                .into_iter()
                .filter(|x|
                    x.symbol.to_uppercase().trim().contains(match_search)
                ).collect();
        }
        None => {}
    }

    //精确查询symbol
    match symbol {
        Some(symbol) => {
            for token in vec.iter() {
                if token.symbol.eq(&symbol) {
                    return Json(TokenListResponse {
                        total: 1,
                        pagesize: 1,
                        page: 1,
                        data: vec![token.clone()],
                    });
                }
            }
            return Json(TokenListResponse {
                total: 0,
                pagesize: 1,
                page: 1,
                data: vec![],
            });
        }
        None => {}
    }

    let start_page;
    let mut size = 50;
    let start_index;
    let total = vec.len() as u32;

    if total == 0 {
        return Json(TokenListResponse {
            total,
            pagesize: total,
            page: 1,
            data: vec![],
        });
    }

    match page {
        Some(p) => {
            start_page = p - 1;
        }
        None => {
            return Json(TokenListResponse {
                total,
                pagesize: total,
                page: 1,
                data: vec,
            });
        }
    }

    match pagesize {
        Some(s) => {
            size = s;
        }
        None => {}
    }

    start_index = start_page * size;
    let mut end_index = start_index + size;

    if end_index >= total {
        end_index = total;
    }

    let res = vec[start_index as usize..end_index as usize].to_vec();
    Json(TokenListResponse {
        total,
        pagesize: size,
        page: start_page + 1,
        data: res,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawWorehole {
    #[serde(rename = "originAddress")]
    pub origin_address: String,
    #[serde(rename = "wrapAddress")]
    pub wrap_address: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawWrapAddress {
    pub ethereum: Option<String>,
    pub solana: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WoreholeAddress {
    pub origin_address: String,
    pub target_token: Option<RawTokenAddr>,
}

pub fn bridge_token_by_origin(source_chain: String, to_chain: String, origin_address: String) -> WoreholeAddress {
    //获取目标
    let wrap_token_path = format!("./resource/token/worehole_{}.json", source_chain);
    let raw_info = fs::read_to_string(wrap_token_path).expect("Error read file");
    let vec: Vec<RawWorehole> = serde_json::from_str(&raw_info).unwrap();

    let mut origin = origin_address.clone();
    if origin_address.eq("") && source_chain.eq("ethereum") {
        origin = String::from("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
    }

    let mut wrap_address = None;
    for worehole in vec.iter() {
        if worehole.origin_address.eq(&origin) {
            wrap_address = worehole.wrap_address.get(&to_chain);
            break;
        }
    }

    let mut worehole_address = WoreholeAddress {
        origin_address: origin,
        target_token: None,
    };

    if wrap_address.is_none() {
        return worehole_address;
    }

    let token_path;
    if to_chain.eq("solana") {
        token_path = "./token_mint.json".to_string();
    } else {
        token_path = format!("./resource/token/{}.json", to_chain);
    }
    let raw_info = fs::read_to_string(token_path).expect("Error read file");
    let vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();
    let tokens_adr: HashMap<String, RawTokenAddr> = vec
        .iter()
        .map(|x| {
            let key = x.address.clone();
            (key, x.clone())
        })
        .collect();

    let wrap_address = tokens_adr.get(wrap_address.unwrap());
    if wrap_address.is_some() {
        worehole_address.target_token = Some(wrap_address.unwrap().clone());
    }
    worehole_address
}

pub fn bridge_token_by_wrap(source_chain: String, to_chain: String, wrap_address: String) -> WoreholeAddress {
    //获取to_chain target token
    let wrap_token_path = format!("./resource/token/worehole_{}.json", to_chain);
    let raw_info = fs::read_to_string(wrap_token_path).expect("Error read file");
    let vec: Vec<RawWorehole> = serde_json::from_str(&raw_info).unwrap();

    let mut target_address = None;
    for worehole in vec.iter() {
        let wrap_opt = worehole.wrap_address.get(&source_chain);
        if wrap_opt.is_some() {
            if wrap_opt.unwrap().eq(&wrap_address) {
                target_address = Some(worehole.origin_address.clone());
                break;
            }
        }
    }

    let mut worehole_address = WoreholeAddress {
        origin_address: wrap_address,
        target_token: None,
    };

    if target_address.is_none() {
        return worehole_address;
    }

    let token_path;
    if to_chain.eq("solana") {
        token_path = "./token_mint.json".to_string();
    } else {
        token_path = format!("./resource/token/{}.json", to_chain);
    }
    let raw_info = fs::read_to_string(token_path).expect("Error read file");
    let vec: Vec<RawTokenAddr> = serde_json::from_str(&raw_info).unwrap();
    let tokens_adr: HashMap<String, RawTokenAddr> = vec
        .iter()
        .map(|x| {
            let key = x.address.clone();
            (key, x.clone())
        })
        .collect();

    let target_address = tokens_adr.get(&target_address.unwrap());
    if target_address.is_some() {
        worehole_address.target_token = Some(target_address.unwrap().clone());
    }
    worehole_address
}