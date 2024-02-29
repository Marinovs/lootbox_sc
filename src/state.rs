use std::collections::HashMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ Addr, Uint128 };
use cw_storage_plus::{ Item, Map };

use crate::msg::RewardData;

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub creator: Addr,
    pub native_token: String,
    pub injscribed_address: Addr,
    pub feature_fees: Uint128,
    pub max_odds: u64,
    pub enabled: bool,
}

#[cw_serde]
pub struct FortuneBox {
    pub id: String,
    pub creator: Addr,
    pub rewards: Vec<RewardData>,
    pub max_odds: u64,
    pub price: Uint128,
    pub token_denom: String,
    pub token_decimals: u64,
    pub token_type: String,
    pub duration: u64,
    pub is_over: bool,
    pub is_featured: bool,
    pub winners: Option<Vec<WinnerStruct>>,
}

#[cw_serde]
pub struct WinnerStruct {
    pub address: Addr,
    pub rewards: Vec<RewardData>,
}

#[cw_serde]
pub struct UserInfo {
    pub address: Addr,
    pub box_created: u64,
    pub inj_spent: Uint128,
    pub tokens_spent: Uint128,
    pub box_opened: u64,
    pub rewards: HashMap<String, Vec<u64>>,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const ACCOUNT_MAP_PREFIX: &str = "account_map";
pub const ACCOUNT_MAP: Map<Addr, UserInfo> = Map::new(ACCOUNT_MAP_PREFIX);

pub const BOX_MAP_PREFIX: &str = "box_map";
pub const BOX_MAP: Map<String, FortuneBox> = Map::new(BOX_MAP_PREFIX);

pub const NOIS_PROXY: Item<Addr> = Item::new("nois_proxy");
