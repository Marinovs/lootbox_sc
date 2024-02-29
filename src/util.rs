use cosmwasm_std::{
    to_json_binary, Addr, BalanceResponse as NativeBalanceResponse, BankMsg, BankQuery, Coin,
    CosmosMsg, QuerierWrapper, QueryRequest, Response, StdResult, Storage, Uint128, WasmMsg,
    WasmQuery,
};
use cw20::{BalanceResponse as CW20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};

use crate::{state::CONFIG, ContractError};

pub fn check_owner(storage: &mut dyn Storage, address: Addr) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(storage)?;

    if address != cfg.owner && address != cfg.creator {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::new().add_attribute("action", "check_owner"))
}

pub fn execute_update_config(
    storage: &mut dyn Storage,
    address: Addr,
    native_token: String,
    injscribed_address: Addr,
    feature_fees: Uint128,
) -> Result<Response, ContractError> {
    check_owner(storage, address)?;

    CONFIG.update(storage, |mut exists| -> StdResult<_> {
        exists.native_token = native_token.clone();
        exists.injscribed_address = injscribed_address.clone();
        exists.feature_fees = feature_fees;
        Ok(exists)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("native_token", native_token.clone())
        .add_attribute("feature_fees", feature_fees.clone())
    )
}

pub fn transfer_token_message(
    denom: String,
    token_type: String,
    amount: Uint128,
    receiver: Addr,
) -> Result<CosmosMsg, ContractError> {
    if token_type == "native" {

        return Ok((BankMsg::Send {
            to_address: receiver.clone().into(),
            amount: vec![Coin {
                denom: denom.clone(),
                amount,
            }],
        })
        .into());
    } else {

        return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: denom.clone().into(),
            funds: vec![],
            msg: to_json_binary(
                &(Cw20ExecuteMsg::Transfer {
                    recipient: receiver.clone().into(),
                    amount,
                }),
            )?,
        }));
    }
}

pub fn get_token_amount(
    querier: QuerierWrapper,
    denom: String,
    contract_addr: Addr,
    token_type: String,
) -> Result<Uint128, ContractError> {
    if token_type == "native" {
        let native_response: NativeBalanceResponse =
            querier.query(&QueryRequest::Bank(BankQuery::Balance {
                address: contract_addr.clone().into(),
                denom: denom.clone(),
            }))?;
        return Ok(native_response.amount.amount);
    } else {
        let balance_response: CW20BalanceResponse =
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: denom.clone().into(),
                msg: to_json_binary(
                    &(Cw20QueryMsg::Balance {
                        address: contract_addr.clone().into(),
                    }),
                )?,
            }))?;
        return Ok(balance_response.balance);
    }
}
