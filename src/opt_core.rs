use crate::market;
use crate::response;
use crate::api;
use serde::{Serialize, Deserialize};
use market::market::MarketSwap;
use std::collections::HashMap;
use solana_sdk::account::Account;
use market::market::MarketType::*;
use response::{OptRoute, OptMarket};
use anyhow::Result;
use market::raydium::stats::AmmInfo;
use market::saber::state::SwapInfo;
use solana_program::account_info::AccountInfo;
use spl_token_swap::processor::Processor;
use rust_decimal::Decimal;
use api::TokenAddr;
use bytemuck::__core::ops::{Add, Mul, Div, Sub};
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use solana_program::pubkey::Pubkey;
use spl_token_swap::state::{SwapV1, SwapState};
use spl_token_swap::solana_program::program_pack::Pack;
use spl_token_swap::curve::stable::StableCurve;
use spl_token_swap::curve::calculator::{TradeDirection, CurveCalculator, SwapWithoutFeesResult};
use spl_token_swap::curve::fees::Fees;
use spl_token_swap::curve::base::{CurveType, SwapCurve};
use market::saber::curve::StableSwap;
use std::time::Instant;
use solana_program::clock::Clock;
use solana_program::sysvar::Sysvar;

#[derive(Debug, Serialize, Deserialize)]
pub struct OptInitData {
    pub amount_in: f64,
    pub tokens_adr: HashMap<String, TokenAddr>,
    pub account_map: HashMap<String, Account>,
    pub swaps: Vec<MarketSwap>,
    pub slippage: u32,
}

impl OptInitData {
    pub fn calculate(&self) -> Result<Vec<OptMarket>> {
        let mut res = vec![];

        for swap in self.swaps.iter() {
            let market_type = swap.market_type.clone();
            let swap_amount_in = self.amount_in.clone();
            match market_type {
                Raydium(x, y) => {
                    let mut market_swap = cal_raydium(swap_amount_in, swap,
                                                      &self.account_map, &self.tokens_adr, self.slippage).unwrap();
                    market_swap.set_info(x, y);
                    res.push(market_swap);
                }
                Orca(x, y) => {
                    let mut market_swap = cal_orca(swap_amount_in, swap,
                                                   &self.account_map, &self.tokens_adr, self.slippage).unwrap();
                    market_swap.set_info(x, y);
                    res.push(market_swap);
                }
                Saber(x, y) => {
                    let mut market_swap = cal_saber(swap_amount_in, swap,
                                                    &self.account_map, &self.tokens_adr, self.slippage).unwrap();
                    market_swap.set_info(x, y);
                    res.push(market_swap);
                }
                Swap(_x, _y) => {}
                Serum(_x, _y) => {}
            }
        }

        Ok(res)
    }
}

fn cal_raydium(amount_in: f64,
               swap: &MarketSwap,
               account_map: &HashMap<String, Account>,
               token_map: &HashMap<String, TokenAddr>,
               slippage: u32) -> Result<OptMarket> {
    let mut res = vec![];

    let mut amount_in = amount_in;

    let mut to_amount: f64 = 0.0;

    for step in swap.step.iter() {
        let pool_ac = account_map.get(&step.pool_key.to_string()).unwrap();
        let mut pool_clone = pool_ac.clone();
        let pool_ac_info = convert_to_info(&step.pool_key, &mut pool_clone);
        let pool_info = AmmInfo::load_amm_mut(&pool_ac_info, false).unwrap();

        let quote_ac = account_map.get(&step.quote_value_key.to_string()).unwrap();
        let mut quote_clone = quote_ac.clone();
        let quote_ac_info = convert_to_info(&step.quote_value_key, &mut quote_clone);
        let quote_info = Processor::unpack_token_account(&quote_ac_info, &quote_ac.owner).unwrap();

        let base_ac = account_map.get(&step.base_value_key.to_string()).unwrap();
        let mut base_clone = base_ac.clone();
        let base_ac_info = convert_to_info(&step.base_value_key, &mut base_clone);
        let base_info = Processor::unpack_token_account(&base_ac_info, &base_ac.owner).unwrap();

        let basic: i128 = 10;
        let quote_token = token_map.get(&step.quote_mint_key.to_string()).unwrap();
        let base_token = token_map.get(&step.base_mint_key.to_string()).unwrap();
        let quote_pow = basic.pow(quote_token.decimal as u32);
        let base_pow = basic.pow(base_token.decimal as u32);
        let quote_amount = Decimal::from(quote_info.amount);
        let base_amount = Decimal::from(base_info.amount);
        if step.is_quote_to_base {
            let from_amount = Decimal::from_f64(amount_in * (quote_pow as f64)).unwrap();
            let from_amount_with_fee = from_amount.mul(Decimal::from(pool_info.fees.swap_fee_denominator - pool_info.fees.swap_fee_numerator)).div(Decimal::from(pool_info.fees.swap_fee_denominator));
            let denominator = quote_amount.add(from_amount_with_fee);
            let amount_out = base_amount.mul(from_amount_with_fee).div(denominator);
            let mut amount_out_format = amount_out.div(Decimal::from(base_pow)).div(Decimal::from_f32(1 as f32 + slippage as f32 / 100 as f32).unwrap());
            // let mut amount_out_with_slippage = amount_out.div(Decimal::from(coin_base)).div(Decimal::from(1 + slippage / 100));
            amount_out_format.rescale(base_token.decimal as u32);

            res.push(OptRoute {
                route_key: step.pool_key.to_string(),
                source_amount: amount_in,
                source_name: quote_token.name.to_string(),
                source_mint: quote_token.mint.to_string(),
                source_decimals: quote_token.decimal,
                destination_amount: amount_out_format.to_f64().unwrap(),
                destination_name: base_token.name.to_string(),
                destination_mint: base_token.mint.to_string(),
                destination_decimals: base_token.decimal,
                source_value: quote_info.amount,
                destination_value: base_info.amount,
                fee_factor: ((pool_info.fees.trade_fee_denominator - pool_info.fees.trade_fee_numerator) as f64).div(pool_info.fees.trade_fee_denominator as f64),
                amp: None,
                data: step.data.clone(),
            });
            amount_in = amount_out_format.to_f64().unwrap();
            to_amount = amount_in.clone();
        } else {
            let from_amount = Decimal::from_f64(amount_in * (base_pow as f64)).unwrap();
            let from_amount_with_fee = from_amount.mul(Decimal::from(pool_info.fees.swap_fee_denominator - pool_info.fees.swap_fee_numerator)).div(Decimal::from(pool_info.fees.swap_fee_denominator));
            let denominator = base_amount.add(from_amount_with_fee);
            let amount_out = quote_amount.mul(from_amount_with_fee).div(denominator);
            let mut amount_out_format = amount_out.div(Decimal::from(quote_pow)).div(Decimal::from_f32(1 as f32 + slippage as f32 / 100 as f32).unwrap());
            // let mut amount_out_with_slippage = amount_out.div(Decimal::from(coin_base)).div(Decimal::from(1 + 5 / 100));
            amount_out_format.rescale(quote_token.decimal as u32);
            res.push(OptRoute {
                route_key: step.pool_key.to_string(),
                source_amount: amount_in,
                source_name: base_token.name.to_string(),
                source_mint: base_token.mint.to_string(),
                source_decimals: base_token.decimal,
                destination_amount: amount_out_format.to_f64().unwrap(),
                destination_name: quote_token.name.to_string(),
                destination_mint: quote_token.mint.to_string(),
                destination_decimals: quote_token.decimal,
                source_value: base_info.amount,
                destination_value: quote_info.amount,
                fee_factor: ((pool_info.fees.trade_fee_denominator - pool_info.fees.trade_fee_numerator) as f64).div(pool_info.fees.trade_fee_denominator as f64),
                amp: None,
                data: step.data.clone(),
            });
            amount_in = amount_out_format.to_f64().unwrap();
            to_amount = amount_in.clone();
        }
    }

    Ok(OptMarket {
        market: "".to_string(),
        program_id: "".to_string(),
        amount_out: to_amount,
        percentage: 0.5,
        routes: res,
    })
}

fn cal_orca(amount_in: f64,
            swap: &MarketSwap,
            account_map: &HashMap<String, Account>,
            token_map: &HashMap<String, TokenAddr>,
            slippage: u32) -> Result<OptMarket> {
    let mut res = vec![];

    let mut amount_in = amount_in;

    let mut to_amount: f64 = 0.0;

    for step in swap.step.iter() {
        let pool_ac = account_map.get(&step.pool_key.to_string()).unwrap();
        let pool_clone = pool_ac.clone();
        let pool_info = SwapV1::unpack_from_slice(&pool_clone.data).unwrap();

        let quote_ac = account_map.get(&step.quote_value_key.to_string()).unwrap();
        let mut quote_clone = quote_ac.clone();
        let quote_ac_info = convert_to_info(&step.quote_value_key, &mut quote_clone);
        let quote_info = Processor::unpack_token_account(&quote_ac_info, &quote_ac.owner).unwrap();

        let base_ac = account_map.get(&step.base_value_key.to_string()).unwrap();
        let mut base_clone = base_ac.clone();
        let base_ac_info = convert_to_info(&step.base_value_key, &mut base_clone);
        let base_info = Processor::unpack_token_account(&base_ac_info, &base_ac.owner).unwrap();

        let basic: i128 = 10;
        let quote_token = token_map.get(&step.quote_mint_key.to_string()).unwrap();
        let base_token = token_map.get(&step.base_mint_key.to_string()).unwrap();
        let quote_pow = basic.pow(quote_token.decimal as u32);
        let base_pow = basic.pow(base_token.decimal as u32);

        let fee_ratio = get_swap_fee_ratio(pool_info.fees.trade_fee_numerator,
                                           pool_info.fees.trade_fee_denominator,
                                           pool_info.fees.owner_trade_fee_numerator,
                                           pool_info.fees.owner_trade_fee_denominator);

        if step.is_quote_to_base {
            let from_amount = amount_in * (quote_pow as f64);

            let from_amount_with_fee = from_amount.sub(from_amount * (fee_ratio as f64));

            let mut amount_out;

            match step.amp {
                Some(amp) => {
                    let sc = StableCurve {
                        amp: 100
                    };
                    let sc_result = sc.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
                                                         quote_info.amount as u128,
                                                         base_info.amount as u128,
                                                         TradeDirection::AtoB).unwrap();
                    amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();
                }
                None => {
                    let sc = SwapCurve::default();
                    let sc_result = sc.calculator.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
                                                                    quote_info.amount as u128,
                                                                    base_info.amount as u128,
                                                                    TradeDirection::AtoB).unwrap();
                    amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();
                }
            };

            let mut amount_out_format = amount_out.div(Decimal::from(base_pow)).div(Decimal::from_f32(1 as f32 + slippage as f32 / 100 as f32).unwrap());
            amount_out_format.rescale(base_token.decimal as u32);
            res.push(OptRoute {
                route_key: step.pool_key.to_string(),
                source_amount: amount_in,
                source_name: quote_token.name.to_string(),
                source_mint: quote_token.mint.to_string(),
                source_decimals: quote_token.decimal,
                destination_amount: amount_out_format.to_f64().unwrap(),
                destination_name: base_token.name.to_string(),
                destination_mint: base_token.mint.to_string(),
                destination_decimals: base_token.decimal,
                source_value: quote_info.amount,
                destination_value: base_info.amount,
                fee_factor: fee_ratio as f64,
                amp: step.amp,
                data: step.data.clone(),
            });
            amount_in = amount_out_format.to_f64().unwrap();
            to_amount = amount_in.clone();
        } else {
            let from_amount = amount_in * (base_pow as f64);

            let from_amount_with_fee = from_amount - (from_amount * (fee_ratio as f64));

            let mut amount_out;

            match step.amp {
                Some(amp) => {
                    let sc = StableCurve {
                        amp: 100
                    };
                    let sc_result = sc.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
                                                         base_info.amount as u128,
                                                         quote_info.amount as u128,
                                                         TradeDirection::BtoA).unwrap();
                    amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();
                }
                None => {
                    let sc = SwapCurve::default();
                    let sc_result = sc.calculator.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
                                                                    base_info.amount as u128,
                                                                    quote_info.amount as u128,
                                                                    TradeDirection::BtoA).unwrap();
                    amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();
                }
            };

            let mut amount_out_format = amount_out.div(Decimal::from(quote_pow)).div(Decimal::from_f32(1 as f32 + slippage as f32 / 100 as f32).unwrap());
            amount_out_format.rescale(quote_token.decimal as u32);
            res.push(OptRoute {
                route_key: step.pool_key.to_string(),
                source_amount: amount_in,
                source_name: base_token.name.to_string(),
                source_mint: base_token.mint.to_string(),
                source_decimals: base_token.decimal,
                destination_amount: amount_out_format.to_f64().unwrap(),
                destination_name: quote_token.name.to_string(),
                destination_mint: quote_token.mint.to_string(),
                destination_decimals: quote_token.decimal,
                source_value: base_info.amount,
                destination_value: quote_info.amount,
                fee_factor: fee_ratio as f64,
                amp: step.amp,
                data: step.data.clone(),
            });
            amount_in = amount_out_format.to_f64().unwrap();
            to_amount = amount_in.clone();
        }
    }

    Ok(OptMarket {
        market: "".to_string(),
        program_id: "".to_string(),
        amount_out: to_amount,
        percentage: 0.5,
        routes: res,
    })
}

fn cal_saber(amount_in: f64,
             swap: &MarketSwap,
             account_map: &HashMap<String, Account>,
             token_map: &HashMap<String, TokenAddr>,
             slippage: u32) -> Result<OptMarket> {
    let mut res = vec![];

    let mut amount_in = amount_in;

    let mut to_amount: f64 = 0.0;

    for step in swap.step.iter() {
        let pool_ac = account_map.get(&step.pool_key.to_string()).unwrap();
        let pool_clone = pool_ac.clone();
        let pool_info = SwapInfo::unpack_from_slice(&pool_clone.data).unwrap();

        let quote_ac = account_map.get(&step.quote_value_key.to_string()).unwrap();
        let mut quote_clone = quote_ac.clone();
        let quote_ac_info = convert_to_info(&step.quote_value_key, &mut quote_clone);
        let quote_info = Processor::unpack_token_account(&quote_ac_info, &quote_ac.owner).unwrap();

        let base_ac = account_map.get(&step.base_value_key.to_string()).unwrap();
        let mut base_clone = base_ac.clone();
        let base_ac_info = convert_to_info(&step.base_value_key, &mut base_clone);
        let base_info = Processor::unpack_token_account(&base_ac_info, &base_ac.owner).unwrap();

        let basic: i128 = 10;
        let quote_token = token_map.get(&step.quote_mint_key.to_string()).unwrap();
        let base_token = token_map.get(&step.base_mint_key.to_string()).unwrap();
        let quote_pow = basic.pow(quote_token.decimal as u32);
        let base_pow = basic.pow(base_token.decimal as u32);
        let fee_ratio = get_swap_fee_ratio(pool_info.fees.trade_fee_numerator,
                                           pool_info.fees.trade_fee_denominator,
                                           pool_info.fees.admin_trade_fee_numerator,
                                           pool_info.fees.admin_withdraw_fee_denominator);

        let clock = Clock::default();

        let stable_swap = StableSwap::new(pool_info.initial_amp_factor, pool_info.target_amp_factor, clock.unix_timestamp,
                                          pool_info.start_ramp_ts, pool_info.stop_ramp_ts);

        if step.is_quote_to_base {
            let from_amount = amount_in * (quote_pow as f64);

            let from_amount_with_fee = from_amount.sub(from_amount * (fee_ratio as f64));

            // let sc = StableCurve {
            //     amp: pool_info.target_amp_factor,
            // };
            // let sc_result = sc.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
            //                                      quote_info.amount as u128,
            //                                      base_info.amount as u128,
            //                                      TradeDirection::AtoB).unwrap();
            // let amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();

            let sc_result = stable_swap.swap_to(from_amount as u64, quote_info.amount, base_info.amount, &pool_info.fees).unwrap();
            println!("saber_result={:?}", sc_result);
            println!("saber_amount_out={}", sc_result.amount_swapped);
            let amount_out = Decimal::from_u64(sc_result.amount_swapped).unwrap();

            let mut amount_out_format = amount_out.div(Decimal::from(base_pow)).div(Decimal::from_f32(1 as f32 + slippage as f32 / 100 as f32).unwrap());
            amount_out_format.rescale(base_token.decimal as u32);
            res.push(OptRoute {
                route_key: step.pool_key.to_string(),
                source_amount: amount_in,
                source_name: quote_token.name.to_string(),
                source_mint: quote_token.mint.to_string(),
                source_decimals: quote_token.decimal,
                destination_amount: amount_out_format.to_f64().unwrap(),
                destination_name: base_token.name.to_string(),
                destination_mint: base_token.mint.to_string(),
                destination_decimals: base_token.decimal,
                source_value: quote_info.amount,
                destination_value: base_info.amount,
                fee_factor: fee_ratio as f64,
                amp: Some(pool_info.target_amp_factor),
                data: step.data.clone(),
            });
            amount_in = amount_out_format.to_f64().unwrap();
            to_amount = amount_in.clone();
        } else {
            let from_amount = amount_in * (base_pow as f64);

            let from_amount_with_fee = from_amount - (from_amount * (fee_ratio as f64));

            // let sc = StableCurve {
            //     amp: pool_info.target_amp_factor,
            // };
            // let sc_result = sc.swap_without_fees(from_amount_with_fee.to_u128().unwrap(),
            //                                      quote_info.amount as u128,
            //                                      base_info.amount as u128,
            //                                      TradeDirection::AtoB).unwrap();
            // let amount_out = Decimal::from_u128(sc_result.destination_amount_swapped).unwrap();

            let sc_result = stable_swap.swap_to(from_amount as u64, base_info.amount, quote_info.amount, &pool_info.fees).unwrap();
            let amount_out = Decimal::from_u64(sc_result.amount_swapped).unwrap();
            println!("saber_result={:?}", sc_result);
            println!("saber_amount_out={}", sc_result.amount_swapped);
            let mut amount_out_format = amount_out.div(Decimal::from(quote_pow)).div(Decimal::from_f32(1 as f32 + slippage as f32 / 100 as f32).unwrap());
            amount_out_format.rescale(quote_token.decimal as u32);
            res.push(OptRoute {
                route_key: step.pool_key.to_string(),
                source_amount: amount_in,
                source_name: base_token.name.to_string(),
                source_mint: base_token.mint.to_string(),
                source_decimals: base_token.decimal,
                destination_amount: amount_out_format.to_f64().unwrap(),
                destination_name: quote_token.name.to_string(),
                destination_mint: quote_token.mint.to_string(),
                destination_decimals: quote_token.decimal,
                source_value: base_info.amount,
                destination_value: quote_info.amount,
                fee_factor: fee_ratio as f64,
                amp: Some(pool_info.initial_amp_factor),
                data: step.data.clone(),
            });
            amount_in = amount_out_format.to_f64().unwrap();
            to_amount = amount_in.clone();
        }
    }

    Ok(OptMarket {
        market: "".to_string(),
        program_id: "".to_string(),
        amount_out: to_amount,
        percentage: 0.5,
        routes: res,
    })
}

fn convert_to_info<'a>(key: &'a Pubkey, account: &'a mut Account) -> AccountInfo<'a> {
    AccountInfo::new(key,
                     false, false,
                     &mut account.lamports,
                     &mut account.data,
                     &account.owner, false,
                     *&account.rent_epoch)
}

pub fn get_swap_fee_ratio(trade_fee_numerator: u64,
                          trade_fee_denominator: u64,
                          owner_trade_fee_numerator: u64,
                          owner_trade_fee_denominator: u64) -> u64 {
    let trade_fee = if trade_fee_numerator == 0 {
        0
    } else {
        trade_fee_numerator / trade_fee_denominator
    };

    let owner_fee = if owner_trade_fee_numerator == 0 {
        0
    } else {
        owner_trade_fee_numerator / owner_trade_fee_denominator
    };

    trade_fee + owner_fee
}
