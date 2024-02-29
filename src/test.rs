#[cfg(test)]
mod test_module {
    use cosmwasm_std::Uint128;

    use crate::msg::{ TokenFactoryReward, RewardType };

    #[test]
    fn test() {
        let mut weighted_list = Vec::new();
        let mut rewards: Vec<_> = Vec::new();
        rewards.push(TokenFactoryReward {
            id: 1,
            token_denom: "inj".to_string(),
            amount: Uint128::from(10000000000000000u64),
            token_decimals: 18,
            odds: 10,
            count: 12,
            reward_type: RewardType::TokenFactory,
        });
        rewards.push(TokenFactoryReward {
            id: 2,
            token_denom: "inj".to_string(),
            amount: Uint128::from(1000000000000000000u64),
            token_decimals: 18,
            odds: 10,
            count: 3,
            reward_type: RewardType::TokenFactory,
        });

        for ticket_info in rewards.clone() {
            for _ in 0..ticket_info.odds {
                weighted_list.push(ticket_info.id.clone());
            }
        }

        let random_number = 1;
        Uint128::new(random_number as u128);

        let reward_id = weighted_list[random_number as usize].clone();
        //find the reward with id == winner_id
        let reward = rewards.iter().find(|x| x.id == reward_id);
        if reward.is_some() {
            let mut cnt = reward.unwrap().clone();
            cnt.count -= 1;
        }
        println!("{:}", reward.unwrap().count)
    }
}
