use near_sdk::{near_bindgen, env, Gas, Promise, PromiseResult, AccountId, NearToken};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize}; // ✅ Import borsh properly
use near_sdk::serde::{Serialize, Deserialize}; // ✅ Use serde correctly
const GAS_FOR_POST_WITHDRAW: Gas = Gas::from_tgas(5); // ✅ Optimized to 5 TGas

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]  // ✅ Required for NEAR contract storage
pub struct Contract {
    campaigns: UnorderedMap<u64, Campaign>,
    next_campaign_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Campaign {
    campaign_id: u64,

    name: String, // The name of the campaign

    // The account of the project creator (owner of the campaign)
    owner: AccountId,

    // The campaign's funding goal in NEAR tokens
    funding_goal: NearToken,

    // Total funds collected so far
    total_funds: NearToken,

    // A mapping of contributors and their contributions
    contributions: UnorderedMap<AccountId, NearToken>,

    // Campaign deadline (in block height or timestamp)
    deadline: u64,

    // Flag to indicate if the campaign has been completed
    is_completed: bool,

    image_url: String, // URL of the image for the campaign
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CampaignDTO {
    campaign_id: u64,
    name: String,
    owner: AccountId,
    funding_goal: NearToken,
    total_funds: NearToken,
    contributions: Vec<(AccountId, NearToken)>,
    deadline: u64,
    is_completed: bool,
    image_url: String,
}

impl From<&Campaign> for CampaignDTO {
    fn from(campaign: &Campaign) -> Self {
        Self {
            campaign_id: campaign.campaign_id,
            name: campaign.name.clone(),
            owner: campaign.owner.clone(),
            funding_goal: campaign.funding_goal,
            total_funds: campaign.total_funds,
            contributions: campaign.contributions.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            deadline: campaign.deadline,
            is_completed: campaign.is_completed,
            image_url: campaign.image_url.clone(),
        }
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self 
        {
            campaigns: UnorderedMap::new(b"campaigns".to_vec()),       
            next_campaign_id: 0,
        }
    }
}


// Implement the contract structure
#[near_bindgen]
impl Contract {

    #[init]
    pub fn new() -> Self {
        Self {
            campaigns: UnorderedMap::new(b"campaigns".to_vec()),
            next_campaign_id: 0,
        }
    }

    
    pub fn create_campaign(&mut self, name: String, funding_goal: NearToken, duration_seconds: u64) -> u64 
    {
        let owner = env::predecessor_account_id();
        assert!(funding_goal > NearToken::from_yoctonear(0), "Funding goal must be greater than 0");
        assert!(duration_seconds > 0, "Duration must be greater than 0");

        let campaign_id = self.next_campaign_id;
        self.next_campaign_id += 1;

        let campaign = Campaign {
            campaign_id,
            name: name.clone(),
            owner: owner.clone(), 
            funding_goal, 
            total_funds: NearToken::from_yoctonear(0), 
            contributions: UnorderedMap::new(format!("contributions_{}", campaign_id).into_bytes()),
            deadline: env::block_timestamp() + duration_seconds * 1_000_000_000,
            is_completed: false,
            image_url: "".to_string(),
        };

        self.campaigns.insert(&campaign_id, &campaign);
        env::log_str(&format!("Campaign {} created by {}", campaign_id, owner));

        campaign_id
    }

    pub fn get_campaign(&self, campaign_id: u64) -> Option<CampaignDTO> {
        self.campaigns.get(&campaign_id).map(|c| CampaignDTO::from(&c))
    }

    pub fn get_campaign_details(&self, campaign_id: u64) -> (AccountId, String, NearToken, NearToken, u64, u64, bool, String) {
        let campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");

        let total_contributors = campaign.contributions.len() as u64; // ✅ Get number of contributors
        (
            campaign.owner.clone(),
            campaign.name.clone(),
            campaign.funding_goal,
            campaign.total_funds,
            campaign.deadline,
            total_contributors,
            campaign.is_completed,
            campaign.image_url.clone(),
        )
    }

    pub fn get_all_campaigns(&self) -> Vec<CampaignDTO> {
        self.campaigns.iter().map(|(_, campaign)| CampaignDTO::from(&campaign)).collect()
    }

    pub fn contribute(&mut self, campaign_id: u64) {
        let mut campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");

        assert!(!campaign.funding_goal.is_zero(), "Contract not initialized");
        assert!(!campaign.is_completed, "Campaign is completed");
        assert!(env::block_timestamp() <= campaign.deadline, "Campaign deadline has passed");
        assert!(!env::attached_deposit().is_zero(), "Attached deposit must be greater than 0");
        assert!(env::predecessor_account_id() != campaign.owner, "Owner cannot contribute to their own campaign");

        let account_id = env::predecessor_account_id();
        let amount = env::attached_deposit();

        campaign.total_funds = campaign.total_funds.saturating_add(amount);

        let previous_contribution = campaign.contributions.iter()
            .find(|(k, _)| k == &account_id)
            .map(|(_, v)| v.clone())
            .unwrap_or(NearToken::from_yoctonear(0));

        if let Some((_, v)) = campaign.contributions.iter_mut().find(|(k, _)| k == &account_id) {
            *v = v.saturating_add(amount); // ✅ Update existing contribution
        } else {
            campaign.contributions.push((account_id.clone(), amount)); // ✅ Add new contribution
        }

        let amount_near = amount.as_yoctonear() as f64 / 1_000_000_000_000_000_000.0;
        env::log_str(&format!("{} Contributed {} NEAR to the campaign", account_id, amount_near));

        self.campaigns.insert(&campaign_id, &campaign);

    }

    pub fn finalize_campaign(&mut self, campaign_id: u64) {
        let mut campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        assert!(env::predecessor_account_id() == campaign.owner, "Only the owner can finalize the campaign");
        assert!(!campaign.is_completed, "Campaign is already completed");
        campaign.is_completed = true;
        self.campaigns.insert(&campaign_id, &campaign);
        env::log_str(&format!("Campaign {} finalized by {}", campaign_id, campaign.owner));
    }
    
    fn check_campaign_completion(&mut self, campaign_id: u64) {
        let mut campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        assert!(!campaign.is_completed, "Campaign is already completed");
        if env::block_timestamp() >= campaign.deadline {
            campaign.is_completed = true;
            self.campaigns.insert(&campaign_id, &campaign);
            env::log_str(&format!("Campaign {} completed", campaign_id));
        }
        
    }

    pub fn withdraw_funds(&mut self, campaign_id: u64, amount: Option<NearToken>) {
        let campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        assert!(campaign.is_completed, "Campaign is not completed");
        assert!(env::predecessor_account_id() == campaign.owner, "Only the owner can withdraw funds");

        let withdraw_amount = amount.unwrap_or(campaign.total_funds);
        assert!(campaign.total_funds >= withdraw_amount, "Insufficient funds");

        // ✅ Transfer funds with a callback for safe state update
        Promise::new(campaign.owner.clone())
            .transfer(withdraw_amount)
            .then(Promise::new(env::current_account_id()).function_call(
                "post_withdraw".to_string(),
                near_sdk::serde_json::to_vec(&campaign_id).expect("Failed to serialize campaign"),
                NearToken::from_yoctonear(0), // Attach 0 NEAR for the callback
                GAS_FOR_POST_WITHDRAW, // ✅ Optimized gas for callback execution
            ));

        env::log_str(&format!(
            "{} initiated withdrawal of {} NEAR",
            campaign.owner,
            withdraw_amount.as_yoctonear()
        ));
    }

    #[private]
    pub fn post_withdraw(&mut self, campaign_id: u64, withdraw_amount: U128) {
        let mut campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        assert!(env::promise_results_count() == 1, "Expected one promise result.");
        
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                campaign.total_funds = campaign.total_funds.saturating_sub(NearToken::from_yoctonear(withdraw_amount.0));
                self.campaigns.insert(&campaign.campaign_id, &campaign);
                env::log_str(&format!(
                    "Withdrawal of {} NEAR successfully processed",
                    withdraw_amount.0
                ));
            }
            _ => {
                env::log_str("Withdrawal failed. Funds remain in contract.");
            }
        }
    }

    pub fn get_excess_funds(&self, campaign_id: u64) -> NearToken {
        let campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        if campaign.total_funds > campaign.funding_goal {
            campaign.total_funds.saturating_sub(campaign.funding_goal)
        } else {
            NearToken::from_yoctonear(0)
        }
    }

    pub fn refund(&mut self, campaign_id: u64) {
        let mut campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        assert!(campaign.is_completed, "Campaign is not completed");
        assert!(env::block_timestamp() >= campaign.deadline, "Campaign deadline has not passed");
        assert!(env::predecessor_account_id() != campaign.owner, "Owner cannot refund");
        assert!(campaign.total_funds < campaign.funding_goal, "Funding goal met, cannot refund");

        let account_id = env::predecessor_account_id();
        let amount = campaign.contributions.get(&account_id).unwrap_or(NearToken::from_yoctonear(0));
        assert!(!amount.is_zero(), "No funds to refund");

        campaign.contributions.remove(&account_id);

        Promise::new(account_id.clone()).transfer(amount);
        campaign.total_funds = campaign.total_funds.saturating_sub(amount);
        self.campaigns.insert(&campaign_id, &campaign);
        env::log_str(&format!("{} Refunded {} NEAR from the campaign", account_id, amount));
    }

    pub fn get_owner(&self, campaign_id: u64) -> AccountId {
        self.get_campaign_internal(campaign_id).map(|c| c.owner.clone()).expect("Campaign does not exist")
    }

    pub fn get_funding_goal(&self, campaign_id: u64) -> NearToken {
        self.get_campaign_internal(campaign_id).map(|c| c.funding_goal).expect("Campaign does not exist")
    }

    pub fn get_total_funds(&self, campaign_id: u64) -> NearToken {
        self.get_campaign_internal(campaign_id).map(|c| c.total_funds).expect("Campaign does not exist")
    }

    pub fn get_contributions(&self, campaign_id: u64) -> Vec<(AccountId, NearToken)> {
        self.get_campaign_internal(campaign_id).map(|c| c.contributions.iter().collect()).expect("Campaign does not exist")
    }

    pub fn get_total_contributors(&self, campaign_id: u64) -> u64 {
       self.get_campaign_internal(campaign_id).map(|c| c.contributions.len() as u64).expect("Campaign does not exist")
    }

    pub fn get_deadline(&self, campaign_id: u64) -> u64 {
        self.get_campaign_internal(campaign_id).map(|c| c.deadline).expect("Campaign does not exist")
    }

    pub fn is_completed(&self, campaign_id: u64) -> bool {
        self.get_campaign_internal(campaign_id).map(|c| c.is_completed).expect("Campaign does not exist")
    }

    pub fn get_image_url(&self, campaign_id: u64) -> String {
        self.get_campaign_internal(campaign_id).map(|c| c.image_url.clone()).expect("Campaign does not exist")
    }

    pub fn set_image_url(&mut self, campaign_id: u64, image_url: String) {
        let mut campaign = self.get_campaign_internal(campaign_id).expect("Campaign does not exist");
        assert!(env::predecessor_account_id() == campaign.owner, "Only the owner can set the image URL");
        campaign.image_url = image_url;
        self.campaigns.insert(&campaign_id, &campaign);
        env::log_str(&format!("Image URL set for campaign {}", campaign_id));
    }

   
}

impl Contract {
    fn get_campaign_internal(&self, campaign_id: u64) -> Option<CampaignDTO> {
        self.campaigns.get(&campaign_id).map(|c| c.into()) // ✅ Use CampaignDTO conversion
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, NearToken};

    fn get_context(predecessor: &str, attached_deposit: u128) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(AccountId::new_unchecked(predecessor.to_string()));
        builder.attached_deposit(attached_deposit);
        builder
    }

    #[test]
    fn test_contract_initialization() {
        let context = get_context("alice.testnet", 0).build();
        testing_env!(context);
        let contract = Contract::new(AccountId::new_unchecked("alice.testnet".to_string()), NearToken::from_yoctonear(10_000_000_000_000_000_000_000), 86400);
        
        assert_eq!(contract.get_owner(), "alice.testnet".parse().unwrap());
        assert_eq!(contract.get_funding_goal(), NearToken::from_yoctonear(10_000_000_000_000_000_000_000));
        assert_eq!(contract.get_total_funds(), NearToken::from_yoctonear(0));
        assert_eq!(contract.is_completed(), false);
    }

    #[test]
    fn test_contributions() {
        let mut context = get_context("bob.testnet", 5_000_000_000_000_000_000_000).build();
        testing_env!(context.clone());
        
        let mut contract = Contract::new(AccountId::new_unchecked("alice.testnet".to_string()), NearToken::from_yoctonear(10_000_000_000_000_000_000_000), 86400);
        
        contract.contribute();
        assert_eq!(contract.get_total_funds(), NearToken::from_yoctonear(5_000_000_000_000_000_000_000));
        assert_eq!(contract.get_total_contributors(), 1);
    }

    #[test]
    fn test_overfunding() {
        let mut context = get_context("charlie.testnet", 15_000_000_000_000_000_000_000).build();
        testing_env!(context.clone());
        
        let mut contract = Contract::new(AccountId::new_unchecked("alice.testnet".to_string()), NearToken::from_yoctonear(10_000_000_000_000_000_000_000), 86400);
        
        contract.contribute();
        assert_eq!(contract.get_total_funds(), NearToken::from_yoctonear(15_000_000_000_000_000_000_000));
        assert_eq!(contract.get_excess_funds(), NearToken::from_yoctonear(5_000_000_000_000_000_000_000));
    }

    #[test]
    fn test_withdrawal() {
        let mut context = get_context("alice.testnet", 0).build();
        testing_env!(context.clone());

        let mut contract = Contract::new(AccountId::new_unchecked("alice.testnet".to_string()), NearToken::from_yoctonear(10_000_000_000_000_000_000_000), 86400);

        // Simulate contributions
        let contributor_context = get_context("bob.testnet", 12_000_000_000_000_000_000_000).build();
        testing_env!(contributor_context);
        contract.contribute();

        // Simulate owner withdrawing funds
        testing_env!(context);
        contract.withdraw_funds(Some(NearToken::from_yoctonear(10_000_000_000_000_000_000_000)));
        assert_eq!(contract.get_total_funds(), NearToken::from_yoctonear(2_000_000_000_000_000_000_000));
    }

    #[test]
    fn test_refund() {
        let mut context = get_context("bob.testnet", 10_000_000_000_000_000_000_000).build();
        testing_env!(context.clone());

        let mut contract = Contract::new(AccountId::new_unchecked("alice.testnet".to_string()), NearToken::from_yoctonear(20_000_000_000_000_000_000_000), 86400);

        contract.contribute();

        // Simulate refund after deadline
        let mut owner_context = get_context("alice.testnet", 0).build();
        owner_context.block_timestamp(env::block_timestamp() + 2_000_000_000_000);
        testing_env!(owner_context);

        contract.finalize_campaign();
        assert!(contract.is_completed());

        let bob_context = get_context("bob.testnet", 0).build();
        testing_env!(bob_context);

        contract.refund();
        assert_eq!(contract.get_total_funds(), NearToken::from_yoctonear(0));
    }
}

*/