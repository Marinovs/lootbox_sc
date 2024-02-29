use std::collections::HashMap;

use cosmwasm_std::{
    entry_point,
    from_json,
    to_json_binary,
    CosmosMsg,
    DepsMut,
    Env,
    MessageInfo,
    WasmMsg,
    Response,
    Uint128,
    Deps,
    StdResult,
    Binary,
    Order,
};
use cw20::Cw20ReceiveMsg;
use cw721::{ Cw721ExecuteMsg, Cw721ReceiveMsg };
use cw_utils::must_pay;

use crate::{
    error::ContractError,
    msg::{
        ExecuteMsg,
        InstantiateMsg,
        NftReceiveMsg,
        RewardData,
        QueryMsg,
        BoxesResponse,
        ConfigResponse,
        UsersInfoResponse,
        TokenReceiveMsg,
        TokenFactoryReward,
        RewardType,
    },
    state::{ Config, CONFIG, BOX_MAP, FortuneBox, ACCOUNT_MAP, UserInfo },
    util,
};
use cw2::set_contract_version;

use sha2::{ Sha256, Digest };

const CONTRACT_NAME: &str = "A5TOUND FUNZONE";
const CONTRACT_VERSION: &str = "1.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        owner: msg.owner.clone(),
        creator: msg.owner.clone(),
        native_token: msg.native_token.clone(),
        injscribed_address: msg.dev_addr.clone(),
        feature_fees: Uint128::from(400000000000000000u128),
        max_odds: 1000,
        enabled: true,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBox {
            box_id,
            price,
            token_denom,
            token_decimals,
            token_type,
            duration,
        } =>
            execute_create_box(
                deps,
                info,
                box_id,
                price,
                token_denom,
                token_decimals,
                token_type,
                duration
            ),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, info, msg),
        ExecuteMsg::Receive(msg) => execute_receive_token(deps, info, msg),
        ExecuteMsg::AddTokenFactoryReward { box_id, rewards } =>
            execute_add_tokenfactory_rewards(deps, info, box_id, rewards),
        ExecuteMsg::CancelBox { box_id } => execute_cancel_box(deps, info, box_id),
        ExecuteMsg::OpenBox { box_id } => execute_open_box(deps, env, info, box_id),
    }
}

pub fn execute_create_box(
    deps: DepsMut,
    info: MessageInfo,
    box_id: String,
    price: Uint128,
    token_denom: String,
    token_decimals: u64,
    token_type: String,
    duration: u64
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let lootbox = BOX_MAP.load(deps.storage, box_id.clone());

    let is_featured =
        info.funds[0].amount == cfg.feature_fees && info.funds[0].denom == "inj".to_string();
    match lootbox {
        Ok(_) => {
            return Err(ContractError::ConflictID {});
        }
        Err(_) => {
            let fbox = FortuneBox {
                id: box_id.clone(),
                creator: info.sender.clone(),
                rewards: vec![],
                max_odds: cfg.max_odds.clone(),
                price,
                token_denom: token_denom.clone(),
                token_decimals,
                token_type,
                duration,
                is_over: false,
                is_featured,
                winners: None,
            };

            BOX_MAP.save(deps.storage, box_id.clone(), &fbox)?;
            Ok(
                Response::default()
                    .add_attribute("action", "create_box")
                    .add_attribute("id", box_id.clone())
                    .add_attribute("token_denom", token_denom.clone())
                    .add_attribute("price", price.clone())
                    .add_attribute("duration", duration.to_string().clone())
            )
        }
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg
) -> Result<Response, ContractError> {
    let msg: NftReceiveMsg = from_json(&wrapper.msg)?;

    match msg {
        NftReceiveMsg::AddNftReward { box_id, collection_addr, nft_id, odds } => {
            if info.sender.clone() != collection_addr.clone() {
                return Err(ContractError::InvalidCw721Token {});
            }

            let cfg = CONFIG.load(deps.storage)?;

            let fortune_box = BOX_MAP.load(deps.storage, box_id.clone());

            match fortune_box {
                Ok(mut fortune_box) => {
                    let mut last_reward_id = 0;
                    if fortune_box.rewards.len() > 0 {
                        last_reward_id = fortune_box.rewards.last().unwrap().id;
                    }

                    //sum all odds in fortune_box.rewards
                    let current_max_odds = fortune_box.rewards
                        .iter()
                        .fold(0, |acc, next| { acc + next.odds });

                    if current_max_odds + odds > cfg.max_odds {
                        return Err(ContractError::MaxOddsReached {
                            msg: current_max_odds.to_string() + "/" + &odds.to_string(),
                        });
                    }

                    let reward = RewardData {
                        id: last_reward_id + 1,
                        reward_type: crate::msg::RewardType::Nft,
                        collection_addr: Some(collection_addr.clone()),
                        nft_id: Some(nft_id.clone()),
                        decimals: None,
                        denom: None,
                        amount: None,
                        odds,
                        count: 1,
                    };

                    fortune_box.rewards.push(reward);
                    BOX_MAP.save(deps.storage, box_id.clone(), &fortune_box)?;
                    Ok(
                        Response::new()
                            .add_attribute("action", "execute_create_box")
                            .add_attribute("collection_addr", collection_addr.clone())
                            .add_attribute("nft_id", nft_id.clone())
                            .add_attribute("odds", odds.to_string().clone())
                    )
                }
                Err(_) => { Err(ContractError::BoxNotFound {}) }
            }
        }
    }
}

pub fn execute_receive_token(
    deps: DepsMut,
    _info: MessageInfo,
    wrapper: Cw20ReceiveMsg
) -> Result<Response, ContractError> {
    let msg: TokenReceiveMsg = from_json(&wrapper.msg)?;

    match msg {
        TokenReceiveMsg::CreateFortuneBoxWithCoin {
            box_id,
            creator,
            denom,
            amount,
            odds,
            price,
            token_denom,
            token_decimals,
            token_type,
            duration,
        } => {
            let cfg = CONFIG.load(deps.storage)?;

            let fortune_box = BOX_MAP.load(deps.storage, box_id.clone());

            match fortune_box {
                Ok(mut fortune_box) => {
                    let last_reward_id = fortune_box.rewards.last().unwrap().id;

                    //sum all odds in fortune_box.rewards
                    let current_max_odds = fortune_box.rewards
                        .iter()
                        .fold(0, |acc, next| { acc + next.odds });

                    if current_max_odds + odds > cfg.max_odds {
                        return Err(ContractError::MaxOddsReached { msg: "max_odds".to_string() });
                    }

                    let reward = RewardData {
                        id: last_reward_id + 1,
                        reward_type: crate::msg::RewardType::Cw20,
                        denom: Some(denom.clone()),
                        amount: Some(amount.clone()),
                        decimals: Some(token_decimals.clone()),
                        odds,
                        collection_addr: None,
                        nft_id: None,
                        count: 1,
                    };

                    fortune_box.rewards.push(reward);
                    BOX_MAP.save(deps.storage, box_id.clone(), &fortune_box)?;
                    return Ok(
                        Response::new()
                            .add_attribute("action", "execute_create_box")
                            .add_attribute("denom", denom.clone())
                            .add_attribute("amount", amount.clone())
                            .add_attribute("odds", odds.to_string().clone())
                    );
                }
                Err(_) => {
                    let reward = RewardData {
                        id: 1,
                        reward_type: crate::msg::RewardType::Nft,
                        collection_addr: None,
                        nft_id: None,
                        denom: Some(denom.clone()),
                        amount: Some(amount.clone()),
                        decimals: Some(token_decimals.clone()),
                        odds,
                        count: 1,
                    };

                    let fbox = FortuneBox {
                        id: box_id.clone(),
                        creator: creator.clone(),
                        rewards: vec![reward],
                        max_odds: cfg.max_odds.clone(),
                        price,
                        token_denom,
                        token_decimals,
                        token_type,
                        duration,
                        is_over: false,
                        is_featured: false,
                        winners: None,
                    };
                    BOX_MAP.save(deps.storage, box_id.clone(), &fbox)?;
                    return Ok(
                        Response::new()
                            .add_attribute("action", "execute_create_box")
                            .add_attribute("denom", denom.clone())
                            .add_attribute("amount", amount.clone())
                            .add_attribute("odds", odds.to_string().clone())
                    );
                }
            }
        }
    }
}

pub fn execute_add_tokenfactory_rewards(
    deps: DepsMut,
    info: MessageInfo,
    box_id: String,
    rewards: Vec<TokenFactoryReward>
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    let fortune_box = BOX_MAP.load(deps.storage, box_id.clone());

    match fortune_box {
        Ok(mut fortune_box) => {
            let mut last_reward_id = 0;
            if fortune_box.rewards.len() > 0 {
                last_reward_id = fortune_box.rewards.last().unwrap().id;
            }

            //sum all odds in fortune_box.rewards
            let current_max_odds = fortune_box.rewards
                .iter()
                .fold(0, |acc, next| { acc + next.odds });

            let rewards_max_odds = rewards.iter().fold(0, |acc, next| { acc + next.odds });

            let amount_sum = rewards
                .iter()
                .fold(Uint128::zero(), |acc, next| {
                    acc + Uint128::from(next.amount) * Uint128::from(next.count)
                });

            let is_correct_amount = match must_pay(&info, &cfg.native_token.clone()) {
                Ok(it) => it,
                Err(_err) => {
                    return Err(ContractError::PaymentFailed {});
                }
            };

            // if is_correct_amount != amount_sum {
            //     return Err(ContractError::AmountNotMatch {});
            // }

            if current_max_odds + rewards_max_odds > cfg.max_odds {
                return Err(ContractError::MaxOddsReached { msg: "max odds tf".to_string() });
            }

            for reward in rewards.clone() {
                let rwrd = RewardData {
                    id: last_reward_id + 1,
                    reward_type: crate::msg::RewardType::TokenFactory,
                    denom: Some(reward.token_denom.clone()),
                    amount: Some(reward.amount.clone()),
                    decimals: Some(reward.token_decimals.clone()),
                    odds: reward.odds.clone(),
                    collection_addr: None,
                    nft_id: None,
                    count: reward.count,
                };
                last_reward_id += 1;
                fortune_box.rewards.push(rwrd);
            }

            BOX_MAP.save(deps.storage, box_id.clone(), &fortune_box)?;
            return Ok(
                Response::new()
                    .add_attribute("action", "execute_add_rewards")
                    .add_attribute("totals", rewards.clone().iter().len().to_string())
                    .add_attribute("amount_send", is_correct_amount.to_string())
                    .add_attribute("total_amount", amount_sum.to_string())
                    .add_attribute("odds", rewards_max_odds.to_string().clone())
            );
        }
        Err(_) => {
            return Err(ContractError::BoxNotFound {});
        }
    }
}

pub fn execute_cancel_box(
    deps: DepsMut,
    info: MessageInfo,
    box_id: String
) -> Result<Response, ContractError> {
    let fortune_box = BOX_MAP.load(deps.storage, box_id.clone());
    match fortune_box {
        Ok(fortune_box) => {
            let mut msgs = Vec::new();
            for reward in fortune_box.rewards.iter() {
                if reward.reward_type == RewardType::Nft {
                    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                        contract_addr: reward.collection_addr.clone().unwrap().to_string().clone(),
                        msg: to_json_binary(
                            &(Cw721ExecuteMsg::TransferNft {
                                token_id: reward.nft_id.clone().unwrap(),
                                recipient: info.sender.to_string().clone(),
                            })
                        )?,
                        funds: vec![],
                    });

                    msgs.push(msg);
                } else if reward.reward_type == RewardType::TokenFactory {
                    let msg = util::transfer_token_message(
                        reward.denom.clone().unwrap(),
                        "native".to_string(),
                        reward.amount.unwrap() * Uint128::from(reward.count),
                        info.sender.clone()
                    )?;

                    msgs.push(msg);
                }
            }

            BOX_MAP.remove(deps.storage, fortune_box.id.clone());
            Ok(Response::new().add_messages(msgs).add_attribute("action", "cancel_fortune_box"))
        }
        Err(_) => { Err(ContractError::BoxNotFound {}) }
    }
}

pub fn execute_open_box(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    box_id: String
) -> Result<Response, ContractError> {
    let fortune_box = BOX_MAP.load(deps.storage, box_id.clone());
    let cfg = CONFIG.load(deps.storage)?;
    match fortune_box {
        Ok(mut fortune_box) => {
            if fortune_box.is_over {
                return Err(ContractError::BoxTerminated {});
            }
            let mut weighted_list = Vec::new();
            for ticket_info in &fortune_box.rewards {
                for _ in 0..ticket_info.odds {
                    weighted_list.push(ticket_info.id.clone());
                }
            }

            let mut hasher = Sha256::new();
            hasher.update(env.block.time.seconds().to_string());
            let result = hasher.finalize();

            // Manually convert the first 8 bytes of the hash into a u64
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&result[0..8]);
            let random_number = u64::from_be_bytes(bytes) % (weighted_list.len() as u64);
            Uint128::new(random_number as u128);

            let reward_id = weighted_list[random_number as usize].clone();
            //find the reward with id == winner_id

            let index = fortune_box.rewards
                .iter()
                .position(|x| x.id == reward_id)
                .unwrap();

            if let Some(reward) = fortune_box.rewards.get_mut(index) {
                let mut msgs = Vec::new();
                if reward.reward_type == RewardType::TokenFactory {
                    msgs.push(
                        util::transfer_token_message(
                            reward.denom.clone().unwrap(),
                            "native".to_string(),
                            reward.amount.unwrap(),
                            info.sender.clone()
                        )?
                    );
                } else if reward.reward_type == RewardType::Nft {
                    msgs.push(
                        CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: reward.collection_addr
                                .clone()
                                .unwrap()
                                .to_string()
                                .clone(),
                            msg: to_json_binary(
                                &(Cw721ExecuteMsg::TransferNft {
                                    token_id: reward.clone().nft_id.unwrap().clone(),
                                    recipient: info.sender.clone().into(),
                                })
                            )?,
                            funds: vec![],
                        })
                    );
                }

                let fees: Uint128 =
                    (info.funds[0].amount * Uint128::from(5u64)) / Uint128::from(100u64);
                msgs.push(
                    util::transfer_token_message(
                        info.funds[0].denom.clone(),
                        "native".to_string(),
                        info.funds[0].amount - fees,
                        fortune_box.creator.clone()
                    )?
                );

                msgs.push(
                    util::transfer_token_message(
                        info.funds[0].denom.clone(),
                        "native".to_string(),
                        fees,
                        cfg.injscribed_address.clone()
                    )?
                );

                // Directly modify the 'count' of the reward
                reward.count -= 1;
                let userinfo = ACCOUNT_MAP.load(deps.storage, info.sender.clone());
                match userinfo {
                    Ok(mut userinfo) => {
                        userinfo.box_opened += 1;
                        userinfo.rewards
                            .entry(box_id.clone())
                            .or_insert_with(|| vec![reward_id.clone()])
                            .push(reward_id.clone());
                        userinfo.inj_spent += fortune_box.price.clone();
                        ACCOUNT_MAP.save(deps.storage, info.sender.clone(), &userinfo)?;
                    }
                    Err(_) => {
                        let mut map: HashMap<String, Vec<u64>> = HashMap::new();
                        map.insert(box_id.clone(), vec![reward_id.clone()]);
                        let usr: UserInfo = UserInfo {
                            address: info.sender.clone(),
                            box_created: 0,
                            inj_spent: fortune_box.price.clone(),
                            tokens_spent: Uint128::from(0u128),
                            box_opened: 1,
                            rewards: map,
                        };

                        ACCOUNT_MAP.save(deps.storage, info.sender.clone(), &usr)?;
                    }
                }

                if reward.count == 0 {
                    fortune_box.is_over = true;
                    for reward in fortune_box.rewards.clone() {
                        if reward.count == 0 {
                            continue;
                        }
                        if reward.reward_type == RewardType::TokenFactory {
                            msgs.push(
                                util::transfer_token_message(
                                    reward.denom.clone().unwrap(),
                                    "native".to_string(),
                                    reward.amount.unwrap() * Uint128::from(reward.count),
                                    fortune_box.creator.clone()
                                )?
                            );
                        } else if reward.reward_type == RewardType::Nft {
                            msgs.push(
                                CosmosMsg::Wasm(WasmMsg::Execute {
                                    contract_addr: reward.collection_addr
                                        .unwrap()
                                        .to_string()
                                        .clone(),
                                    msg: to_json_binary(
                                        &(Cw721ExecuteMsg::TransferNft {
                                            token_id: reward.nft_id.unwrap().clone(),
                                            recipient: fortune_box.creator.clone().into(),
                                        })
                                    )?,
                                    funds: vec![],
                                })
                            );
                        }
                    }
                }

                BOX_MAP.save(deps.storage, box_id, &fortune_box)?;
                Ok(
                    Response::new()
                        .add_messages(msgs)
                        .add_attribute("action", "execute_open_box")
                        .add_attribute("seed", random_number.to_string().clone())
                        .add_attribute("reward_id", reward_id.to_string().clone())
                )
            } else {
                return Err(ContractError::RewardNotFound {});
            }
        }
        Err(_) => {
            return Err(ContractError::BoxNotFound {});
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GetBoxes {} => to_json_binary(&query_boxes(deps)?),
        QueryMsg::GetUsers {} => to_json_binary(&query_users(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config: Config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.clone(),
        token: config.native_token,
        feature_fees: config.feature_fees,
        injscribed_address: config.injscribed_address,
    })
}

pub fn query_boxes(deps: Deps) -> StdResult<BoxesResponse> {
    let boxes: StdResult<Vec<FortuneBox>> = BOX_MAP.range(
        deps.storage,
        None,
        None,
        Order::Ascending
    )
        .map(|item| item.map(|(_, v)| v))
        .collect();

    match boxes {
        Ok(boxes) => Ok(BoxesResponse { boxes }),
        Err(_) =>
            Ok(BoxesResponse {
                boxes: Vec::new(),
            }),
    }
}

pub fn query_users(deps: Deps) -> StdResult<UsersInfoResponse> {
    let users: StdResult<Vec<UserInfo>> = ACCOUNT_MAP.range(
        deps.storage,
        None,
        None,
        Order::Ascending
    )
        .map(|item| item.map(|(_, v)| v))
        .collect();

    match users {
        Ok(users) => Ok(UsersInfoResponse { users }),
        Err(_) => Ok(UsersInfoResponse { users: Vec::new() }),
    }
}
