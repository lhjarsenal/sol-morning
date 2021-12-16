use crate::api;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::collections::HashMap;
use api::RawTokenAddr;
use bytemuck::__core::cmp::Ordering;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

#[derive(Debug, Serialize, Deserialize)]
pub struct OptResponse {
    pub code: u32,
    pub msg: String,
    pub data: Vec<OptRank>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenListResponse {
    pub total: u32,
    pub pagesize: u32,
    pub page: u32,
    pub data: Vec<RawTokenAddr>,
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct OptRank {
    pub amount_out: f64,
    pub quote_mint: String,
    pub base_mint: String,
    pub slippage: u32,
    pub opt: Vec<OptMarket>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct OptMarket {
    pub market: String,
    pub program_id: String,
    pub amount_out: f64,
    pub percentage: f32,
    pub routes: Vec<OptRoute>,
}

impl OptMarket {
    pub fn set_info(&mut self, market: String, program_id: String) {
        self.market = market;
        self.program_id = program_id;
    }

    pub fn get_amount(&self) -> f64 {
        self.amount_out
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct OptRoute {
    //pool_key或market_key
    pub route_key: String,

    pub source_amount: f64,
    pub source_name: String,
    pub source_mint: String,
    pub source_decimals: u8,

    pub destination_amount: f64,
    pub destination_name: String,
    pub destination_mint: String,
    pub destination_decimals: u8,

    pub source_value: u64,
    pub destination_value: u64,
    pub fee_factor: f64,

    pub data: HashMap<String, String>,

}

impl PartialOrd for OptMarket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Option::from(self.amount_out.total_cmp(&other.amount_out))
    }
}

impl OptRank {
    pub fn opt_best(&mut self) -> Result<Vec<Self>> {
        //按amount排序
        self.opt.sort_by(|a, b| b.partial_cmp(&a).unwrap());

        let mut opts = vec![];
        let mut manage_opt = HashMap::new();

        for opt in self.opt.iter() {
            let opt_clone = opt.clone();

            if !manage_opt.contains_key(&opt.market) {
                manage_opt.insert(&opt.market, 0);
                opts.push(opt_clone);
            }
        }

        //计算一个单独的
        let mut opt_res = vec![];
        if !opts.is_empty() {
            let one_step_best = self.cal_one_best_market_amount_out(opts[0].clone());
            opt_res.push(one_step_best);
        }


        if opts.is_empty() {
            Ok(vec![])
        } else if opts.len() == 1 {
            Ok(opt_res)
        } else {
            opt_res.push(OptRank {
                amount_out: opts[0].amount_out + opts[1].amount_out,
                quote_mint: self.quote_mint.to_string(),
                base_mint: self.base_mint.to_string(),
                slippage: self.slippage,
                opt: opts,
            });
            opt_res.sort_by(|a, b| b.partial_cmp(&a).unwrap());
            Ok(opt_res)
        }
    }

    fn cal_one_best_market_amount_out(&self, mut opt: OptMarket) -> OptRank {

        //手动100%
        opt.percentage = 1.0;

        let mut amount_in = opt.routes[0].source_amount * 2.0;

        let mut to_amount: f64 = 0.0;

        for mut step in &mut opt.routes {
            let basic: i128 = 10;
            let quote_pow = basic.pow(step.source_decimals as u32);
            let base_pow = basic.pow(step.destination_decimals as u32);
            let quote_amount = step.source_value;
            let base_amount = step.destination_value;
            if self.quote_mint.eq(&step.source_mint) || self.base_mint.eq(&step.destination_mint) {
                //pc to coin
                let from_amount = amount_in * (quote_pow as f64);
                let from_amount_with_fee = from_amount * step.fee_factor;
                let denominator = quote_amount as f64 + from_amount_with_fee;
                let amount_out = base_amount as f64 * from_amount_with_fee / denominator;
                let mut amount_out_format = Decimal::from_f64(amount_out / base_pow as f64).unwrap();
                // let mut amount_out_with_slippage = amount_out.div(Decimal::from(coin_base)).div(Decimal::from(1 + 5 / 100));
                amount_out_format.rescale(step.destination_decimals as u32);
                step.source_amount = amount_in.clone();
                step.destination_amount = amount_out_format.to_f64().unwrap();
                amount_in = amount_out_format.to_f64().unwrap();
                to_amount = amount_in.clone();
            } else {
                //coin to pc
                let from_amount = amount_in * (base_pow as f64);
                let from_amount_with_fee = from_amount * step.fee_factor;
                let denominator = base_amount as f64 + from_amount_with_fee;
                let amount_out = quote_amount as f64 * from_amount_with_fee / denominator;
                let mut amount_out_format = Decimal::from_f64(amount_out / quote_pow as f64).unwrap();
                // let mut amount_out_with_slippage = amount_out.div(Decimal::from(coin_base)).div(Decimal::from(1 + 5 / 100));
                amount_out_format.rescale(step.source_decimals as u32);
                step.source_amount = amount_in.clone();
                step.destination_amount = amount_out_format.to_f64().unwrap();
                amount_in = amount_out_format.to_f64().unwrap();
                to_amount = amount_in.clone();
            }
        };

        OptRank {
            amount_out: to_amount,
            quote_mint: self.quote_mint.to_string(),
            base_mint: self.base_mint.to_string(),
            slippage: self.slippage,
            opt: vec![opt],
        }
    }
}

impl PartialOrd for OptRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Option::from(self.amount_out.total_cmp(&other.amount_out))
    }
}






