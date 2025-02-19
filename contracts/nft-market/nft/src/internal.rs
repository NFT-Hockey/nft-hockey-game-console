use crate::*;
use near_sdk::{log, CryptoHash, serde_json};
use std::mem::size_of;
use near_sdk::env::{attached_deposit};
use crate::extra::player_type::PlayerType;
use crate::pack::*;

pub(crate) fn royalty_to_payout(a: u32, b: Balance) -> U128 {
    U128(a as u128 * b / 10_000u128)
}

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn assert_one_yocto() {
    assert_eq!(
        attached_deposit(),
        1,
        "Requires attached deposit of exactly 1 yoctoNEAR",
    )
}

pub(crate) fn assert_at_least_one_yocto() {
    assert!(
        attached_deposit() >= 1,
        "Requires attached deposit of at least 1 yoctoNEAR",
    )
}

pub(crate) fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

// TODO: need a way for end users to determine how much an approval will cost.
pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    // The extra 4 bytes are coming from Borsh serialization to store the length of the string.
    account_id.len() as u64 + 4 + size_of::<u64>() as u64
}

pub(crate) fn refund_approved_account_ids_iter<'a, I>(
    account_id: AccountId,
    approved_account_ids: I,
) -> Promise
    where
        I: Iterator<Item = &'a AccountId>,
{
    let storage_released: u64 = approved_account_ids
        .map(bytes_for_approved_account_id)
        .sum();
    Promise::new(account_id).transfer(Balance::from(storage_released) * env::storage_byte_cost())
}

pub(crate) fn refund_approved_account_ids(
    account_id: AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
}

impl Contract {
    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Owner's method"
        );
    }

    pub(crate) fn internal_add_token_to_pack(
        &mut self,
        player_type: &PlayerType,
        rarity: &Rarity,
        token_id: &TokenId
    ) {
        match player_type {
            PlayerType::FieldPlayer => {
                let mut token_set = self.get_field_players_set(rarity);

                token_set.insert(token_id);
                self.field_players.insert(rarity, &token_set);
            }
            PlayerType::Goalie => {
                let mut token_set = self.get_goalies_set(rarity);

                token_set.insert(token_id);
                self.goalies.insert(rarity, &token_set);
            }
        };
    }

    pub(crate) fn internal_remove_token_from_pack(
        &mut self,
        player_type: &PlayerType,
        rarity: &Rarity,
        token_id: &TokenId
    ) {
        match player_type {
            PlayerType::FieldPlayer => {
                let mut token_set = self.get_field_players_set(rarity);

                token_set.remove(token_id);
                self.field_players.insert(rarity, &token_set);
            }
            PlayerType::Goalie => {
                let mut token_set = self.get_goalies_set(rarity);

                token_set.remove(token_id);
                self.goalies.insert(rarity, &token_set);
            }
        };
    }

    fn get_field_players_set(&self, rarity: &Rarity) -> UnorderedSet<TokenId> {
        let json_rarity: AccountId = serde_json::to_string(&rarity).expect("Cannot parse rarity");
        self.field_players.get(rarity).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::FieldPlayersInner {
                    field_player_hash: hash_account_id(&json_rarity.into()),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        })
    }

    fn get_goalies_set(&self, rarity: &Rarity) -> UnorderedSet<TokenId> {
        let json_rarity: AccountId = serde_json::to_string(&rarity).expect("Cannot parse rarity");
        self.goalies.get(rarity).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::GoaliesInner {
                    goalies_hash: hash_account_id(&json_rarity),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        })
    }

    pub(crate) fn internal_transfer_token_from_pack(
        &mut self,
        receiver_id: &AccountId,
        token_id: &TokenId,
        player_type: &PlayerType,
        rarity: &Rarity,
    ) -> TokenMetadata {
        let token = self.tokens_by_id.get(token_id).expect("No token");
        assert_ne!(
            &token.owner_id, receiver_id,
            "The token owner and the receiver should be different"
        );
        assert_eq!(&token.owner_id, &self.owner_id, "contract owner does not have a token");


        self.internal_remove_token_from_pack(player_type, rarity, token_id);
        self.internal_add_token_to_owner(receiver_id, token_id);

        let new_token = Token {
            owner_id: receiver_id.clone(),
            approved_account_ids: Default::default(),
            next_approval_id: token.next_approval_id,
            royalty: token.royalty.clone(),
            token_type: token.token_type.clone(),
        };
        self.tokens_by_id.insert(token_id, &new_token);

        match self.token_metadata_by_id.get(token_id) {
            Some(token_metadata) => token_metadata,
            _ => panic!("Token metadata not found")
        }
    }

    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self.tokens_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInner {
                    account_id_hash: hash_account_id(&account_id),
                }
                    .try_to_vec()
                    .unwrap(),
            )
        });

        tokens_set.insert(token_id);
        self.tokens_per_owner.insert(account_id, &tokens_set);
    }

    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self
            .tokens_per_owner
            .get(account_id)
            .expect("Token should be owned by the sender");
        tokens_set.remove(token_id);
        if tokens_set.is_empty() {
            self.tokens_per_owner.remove(account_id);
        } else {
            self.tokens_per_owner.insert(account_id, &tokens_set);
        }
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        let token = self.tokens_by_id.get(token_id).expect("No token");

        if sender_id != &token.owner_id {
            if !token.approved_account_ids.contains_key(sender_id) {
                env::panic(b"Unauthorized");
            }
            // If they included an enforce_approval_id, check the receiver approval id
            if let Some(enforced_approval_id) = approval_id {
                let actual_approval_id = token
                    .approved_account_ids
                    .get(sender_id)
                    .expect("Sender is not approved account");
                assert_eq!(
                    actual_approval_id, &enforced_approval_id,
                    "The actual approval_id {} is different from the given approval_id {}",
                    actual_approval_id, enforced_approval_id,
                );
            }
        }


        assert_ne!(
            &token.owner_id, receiver_id,
            "The token owner and the receiver should be different"
        );

        log!(
            "Transfer {} from @{} to @{}",
            token_id,
            &token.owner_id,
            receiver_id
        );

        self.internal_remove_token_from_owner(&token.owner_id, token_id);
        self.internal_add_token_to_owner(receiver_id, token_id);

        let new_token = Token {
            owner_id: receiver_id.clone(),
            approved_account_ids: Default::default(),
            next_approval_id: token.next_approval_id,
            royalty: token.royalty.clone(),
            token_type: token.token_type.clone(),
        };
        self.tokens_by_id.insert(token_id, &new_token);

        if let Some(memo) = memo {
            env::log(format!("Memo: {}", memo).as_bytes());
        }

        token
    }

    pub(crate) fn internal_get_pack_probabilities(&self) -> Vec<u8> {
        let deposit = attached_deposit();

        if deposit == BRILLIANT_PACK_COST {
            get_pack_probabilities(Pack::Brilliant)
        } else if deposit == PLATINUM_PACK_COST {
            get_pack_probabilities(Pack::Platinum)
        } else if deposit == GOLD_PACK_COST {
            get_pack_probabilities(Pack::Gold)
        } else if deposit == SILVER_PACK_COST {
            get_pack_probabilities(Pack::Silver)
        } else if deposit == BRONZE_PACK_COST {
            get_pack_probabilities(Pack::Bronze)
        } else {
            panic!("Wrong attached deposit")
        }
    }
}