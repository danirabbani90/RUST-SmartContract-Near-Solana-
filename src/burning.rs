use crate ::*;

#[near_bindgen]

impl Contract {
    // BURN NFT
    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId, art_id: String) -> BurnNFTJson {
        assert_one_yocto();

        let token_data = self
            .tokens_by_id
            .get(&token_id)
            .expect("TOKEN DOESNT EXIST");
        assert_eq!(
            token_data.owner_id,
            env::predecessor_account_id(),
            "ONLY TOKEN OWNER IS ALLOWED TO BURN"
        );

        self.internal_remove_token_from_owner(&token_data.owner_id, &token_id);

        // if let token_metadata_by_id = &mut self.token_metadata_by_id {
        //     token_metadata_by_id.remove(&token_id);
        // }
        self.token_metadata_by_id.remove(&token_id);

        self.tokens_by_id.remove(&token_id);

        // Construct the mint log as per the events standard.
        let nft_burn_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftBurn(vec![NftburnLog {
                owner_id: (token_data.owner_id).to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_burn_log.to_string());

        BurnNFTJson {
            deleted_token: token_id,
            art_id,
            status: "NFT-BURNED".to_string(),
        }
    }
}