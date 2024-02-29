use cosmwasm_schema::{ cw_serde, QueryResponses };
use cosmwasm_std::{ Addr, Uint128 };
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

use crate::state::{ UserInfo, FortuneBox };

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub native_token: String,
    pub founder_addr: Addr,
    pub dev_addr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateBox {
        box_id: String,
        price: Uint128,
        token_denom: String,
        token_decimals: u64,
        token_type: String,
        duration: u64,
    },
    ReceiveNft(Cw721ReceiveMsg),
    Receive(Cw20ReceiveMsg),
    AddTokenFactoryReward {
        box_id: String,
        rewards: Vec<TokenFactoryReward>,
    },
    OpenBox {
        box_id: String,
    },
    CancelBox {
        box_id: String,
    },
}

#[cw_serde]
pub enum NftReceiveMsg {
    AddNftReward {
        box_id: String,
        collection_addr: Addr,
        nft_id: String,
        odds: u64,
    },
}

#[cw_serde]
pub enum TokenReceiveMsg {
    CreateFortuneBoxWithCoin {
        box_id: String,
        creator: Addr,
        denom: String,
        amount: Uint128,
        odds: u64,
        price: Uint128,
        token_denom: String,
        token_decimals: u64,
        token_type: String,
        duration: u64,
    },
}

#[cw_serde]
pub struct TokenFactoryReward {
    pub id: u64,
    pub odds: u64,
    pub token_denom: String,
    pub token_decimals: u64,
    pub reward_type: RewardType,
    pub amount: Uint128,
    pub count: u64,
}

#[cw_serde]
pub struct RewardData {
    pub id: u64,
    pub reward_type: RewardType,
    pub collection_addr: Option<Addr>,
    pub nft_id: Option<String>,
    pub denom: Option<String>,
    pub amount: Option<Uint128>,
    pub decimals: Option<u64>,
    pub odds: u64,
    pub count: u64,
}

#[cw_serde]
pub enum RewardType {
    Cw20,
    Nft,
    TokenFactory,
}

#[cw_serde]
pub struct BoxesResponse {
    pub boxes: Vec<FortuneBox>,
}

#[cw_serde]
pub struct UsersInfoResponse {
    pub users: Vec<UserInfo>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Addr,
    pub token: String,
    pub injscribed_address: Addr,
    pub feature_fees: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)] GetConfig {},
    #[returns(BoxesResponse)] GetBoxes {},
    #[returns(UsersInfoResponse)] GetUsers {},
}
