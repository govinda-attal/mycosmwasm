#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetPollResponse, InstantiateMsg, QueryMsg};
use crate::state::{Config, Poll, CONFIG, POLLS};

const CONTRACT_NAME: &str = "crates.io:mycosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let validated_admin_address = deps.api.addr_validate(&msg.admin_address)?;

    let config = Config {
        admin_address: validated_admin_address,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePoll { question } => execute_create_poll(deps, env, info, question),
        ExecuteMsg::Vote { question, choice } => execute_vote(deps, env, info, question, &choice),
    }
}

fn execute_create_poll(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    question: String,
) -> Result<Response, ContractError> {
    if POLLS.has(deps.storage, question.clone()) {
        return Err(ContractError::CustomError {
            val: "key already taken".to_string(),
        });
    }

    let poll = Poll {
        question: question.clone(),
        yes_votes: 0,
        no_votes: 0,
    };

    POLLS.save(deps.storage, question, &poll)?;

    Ok(Response::new().add_attribute("action", "create_poll"))
}

fn execute_vote(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    question: String,
    choice: &str,
) -> Result<Response, ContractError> {
    if !POLLS.has(deps.storage, question.clone()) {
        return Err(ContractError::CustomError {
            val: "poll doesn't exist!".to_string(),
        });
    }

    let mut poll = POLLS.load(deps.storage, question.clone())?;

    match choice {
        "yes" => poll.yes_votes += 1,
        "no" => poll.no_votes += 1,
        _ => {
            return Err(ContractError::CustomError {
                val: "invalid choice".to_string(),
            });
        }
    }

    POLLS.save(deps.storage, question, &poll)?;
    Ok(Response::new().add_attribute("action", "vote"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoll { question } => query_get_poll(deps, env, question),
        QueryMsg::GetConfig => to_binary(&CONFIG.load(deps.storage)?),
    }
}

fn query_get_poll(deps: Deps, _env: Env, question: String) -> StdResult<Binary> {
    let poll = POLLS.may_load(deps.storage, question)?;
    to_binary(&GetPollResponse { poll })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        attr, from_binary,
        testing::{mock_dependencies, mock_env, mock_info}, Addr,
    };

    use crate::msg::InstantiateMsg;

    use super::*;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr1".to_string(),
        };

        let result = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(result.attributes, vec![attr("action", "instantiate")])
    }

    #[test]
    fn test_create_poll() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr1".to_string(),
        };

        let _result = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            question: "Do you love spark IBC".to_string(),
        };

        let result = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(result.attributes, vec![attr("action", "create_poll")]);

        let msg = QueryMsg::GetConfig;

        let rs_binary = query(deps.as_ref(), env.clone(), msg).unwrap();

        let config: Config = from_binary(&rs_binary).unwrap();

        assert_eq!(config.admin_address, Addr::unchecked("addr1"));
    }

    #[test]
    fn test_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr1".to_string(),
        };

        let _result = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            question: "Do you love spark IBC".to_string(),
        };

        let result = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(result.attributes, vec![attr("action", "create_poll")]);

        let msg = ExecuteMsg::Vote {
            question: "Do you love spark IBC".to_string(),
            choice: "yes".to_string(),
        };

        let result = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(result.attributes, vec![attr("action", "vote")]);
    }

    #[test]
    fn get_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let msg = InstantiateMsg {
            admin_address: "addr1".to_string(),
        };

        let _result = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = QueryMsg::GetPoll {
            question: "Do you love spark IBC".to_string(),
        };

        let rs_binary = query(deps.as_ref(), env.clone(), msg).unwrap();

        let resp: GetPollResponse = from_binary(&rs_binary).unwrap();

        assert!(resp.poll.is_none());

        let msg = ExecuteMsg::CreatePoll {
            question: "Do you love spark IBC".to_string(),
        };

        let result = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        assert_eq!(result.attributes, vec![attr("action", "create_poll")]);

        let msg = ExecuteMsg::Vote {
            question: "Do you love spark IBC".to_string(),
            choice: "yes".to_string(),
        };

        let result = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        assert_eq!(result.attributes, vec![attr("action", "vote")]);

        let msg = QueryMsg::GetPoll {
            question: "Do you love spark IBC".to_string(),
        };

        let rs_binary = query(deps.as_ref(), env, msg).unwrap();

        let resp: GetPollResponse = from_binary(&rs_binary).unwrap();

        assert!(resp.poll.is_some());
    }
}
