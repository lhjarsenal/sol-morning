use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct OptResponse {
    pub amount_out: i32,
    pub quote_mint: String,
    pub base_mint: String,
    pub slippage: u32,
    pub opt: Vec<OptMarket>,
}

impl OptResponse {
    pub fn new() -> Self {
        OptResponse {
            amount_out: 0,
            quote_mint: "".to_string(),
            base_mint: "".to_string(),
            slippage: 0,
            opt: vec![OptMarket::new()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptMarket {
    pub market: String,
    pub program_id: String,
    pub percentage: i32,
    pub routes: Vec<OptRoute>,
}

impl OptMarket {
    fn new() -> Self {
        OptMarket {
            market: "".to_string(),
            program_id: "".to_string(),
            percentage: 0,
            routes: vec![OptRoute::new()],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptRoute {
    //pool_key或market_key
    pub route_key: String,

    pub source_amount: i32,
    pub source_name: String,
    pub source_mint: String,

    pub destination_amount: i32,
    pub destination_name: String,
    pub destination_mint: String,

}

impl OptRoute {
    fn new() -> Self {
        OptRoute {
            route_key: "".to_string(),
            source_amount: 0,
            source_name: "".to_string(),
            source_mint: "".to_string(),
            destination_amount: 0,
            destination_name: "".to_string(),
            destination_mint: "".to_string(),
        }
    }
}



