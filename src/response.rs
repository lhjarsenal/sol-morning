use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
use bytemuck::__core::ops::Mul;

#[derive(Debug, Serialize, Deserialize)]
pub struct OptResponse {
    pub code: u32,
    pub msg: String,
    pub data: Vec<OptRank>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptRank {
    pub amount_out: f32,
    pub quote_mint: String,
    pub base_mint: String,
    pub slippage: u32,
    pub opt: Vec<OptMarket>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub struct OptMarket {
    pub market: String,
    pub program_id: String,
    pub amount_out: f32,
    pub percentage: f32,
    pub routes: Vec<OptRoute>,
}

impl OptMarket {
    pub fn set_info(&mut self, market: String, program_id: String) {
        self.market = market;
        self.program_id = program_id;
    }

    pub fn get_amount(&self) -> f32 {
        self.amount_out
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
pub struct OptRoute {
    //pool_key或market_key
    pub route_key: String,

    pub source_amount: f32,
    pub source_name: String,
    pub source_mint: String,

    pub destination_amount: f32,
    pub destination_name: String,
    pub destination_mint: String,

}

impl OptRank {
    pub fn opt_best(&mut self) -> Result<Vec<Self>> {
        //按amount排序
        self.opt.sort_by(|a, b| b.amount_out.total_cmp(&a.amount_out));

        let mut opts = vec![];
        let mut manage_opt = HashMap::new();

        for opt in self.opt.iter() {

            let opt_clone = opt.clone();

            if !manage_opt.contains_key(&opt.market) {
                manage_opt.insert(&opt.market, 0);
                opts.push(opt_clone);
            }
        }

        if opts.is_empty() {
            Ok(vec![])
        } else if opts.len() == 1 {
            //todo 只有一个需单独计算
            let one_step_opt = OptRank {
                amount_out: opts[0].amount_out.mul(2.0),
                quote_mint: self.quote_mint.to_string(),
                base_mint: self.base_mint.to_string(),
                slippage: self.slippage,
                opt: opts,
            };
            Ok(vec![one_step_opt])
        } else {
            Ok(vec![OptRank {
                amount_out: opts[0].amount_out + opts[1].amount_out,
                quote_mint: self.quote_mint.to_string(),
                base_mint: self.base_mint.to_string(),
                slippage: self.slippage,
                opt: opts,
            }])
        }

    }
}




