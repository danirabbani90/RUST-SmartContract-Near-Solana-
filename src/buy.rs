use crate ::*;

#[near_bindgen]

impl Contract {


    // BUY NFT FUNCTION --- WITH ROYALTIES,SPLIT-PAYMENTS AND TRANSACTION FEE FOR BLOCKPERKS
    #[payable]
    pub fn nft_buy(
        &mut self,
        token_id: TokenId,
        memo: Option<String>,
        art_id: String,
    ) -> BuyNftjson {
        let initial_storage_usage = env::storage_usage();
        let receiver_id = env::predecessor_account_id();
        let previous_token = self
            .tokens_by_id
            .get(&token_id)
            .expect("TOKEN DOESNT EXIST");
        let mut token_data = self
            .tokens_by_id
            .get(&token_id)
            .expect("TOKEN DOESNT EXIST");
        assert_ne!(
            env::predecessor_account_id(),
            token_data.owner_id,
            "YOU CANNOT BUY YOUR OWN NFT"
        );
        let price: u128 = token_data.price.expect("NOT FOR SALE");
        let deposit = env::attached_deposit();
        let previous_owner_id = previous_token.owner_id.clone();
        let mut total_perpetual = 0;

        assert!(deposit >= price, "DEPOSIT IS LESS THAN PRICE :{}", price,);

        let mut payout_object = Payout {
            payout: HashMap::new(),
        };

        let royalty = previous_token.royalty;
        let splitpayments = previous_token.splitpayments;

        // assert!(royalty.len() as u32 <= max_len_payout.unwrap(), "Cannot payout to that many receivers");
        for (k, v) in royalty.iter() {
            let key = k.clone();
            if key != previous_owner_id {
                payout_object
                    .payout
                    .insert(key, royalty_to_payout(*v, price));
                total_perpetual += *v;
            }
        }

        for (k, v) in splitpayments.iter() {
            let key = k.clone();
            if key != previous_owner_id {
                payout_object
                    .payout
                    .insert(key, payment_splitter(*v, price));
                total_perpetual += *v;
            }
        }

        assert!(total_perpetual <= 10000, "PAYOUT OVERFLOW NOT ALLOWED");

        // Calculate the Commission for Blocperks Owner Account
        let for_treasury = price as u128 * self.calculate_current_transaction_fee() / 10_000u128;

        payout_object.payout.insert(
            token_data.owner_id,
            royalty_to_payout(10000 - total_perpetual, price),
        );

        Promise::new(self.owner_id.clone()).transfer(for_treasury);

        for (receiver, amount) in payout_object.payout {
            if receiver == previous_owner_id {
                Promise::new(receiver).transfer(amount.0 - for_treasury);
            } else {
                Promise::new(receiver).transfer(amount.0);
            }
        }
        token_data.splitpayments = HashMap::new();

        //create a new token struct
        let new_tokendata = Token {
            owner_id: receiver_id.clone(),
            //reset the approval account IDs
            approved_account_ids: Default::default(),
            next_approval_id: token_data.next_approval_id,
            price: token_data.price.clone(),
            //we copy over the royalties from the previous token
            royalty: token_data.royalty.clone(),

            splitpayments: token_data.splitpayments,
        };
        //insert that new token into the tokens_by_id, replacing the old entry
        self.tokens_by_id.insert(&token_id, &new_tokendata);

        //we then add the token to the receiver_id's set
        self.internal_add_token_to_owner(&new_tokendata.owner_id, &token_id);

        //if there was some memo attached, we log it.
        if let Some(memo) = memo.as_ref() {
            env::log_str(&format!("Memo: {}", memo).to_string());
        }

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: receiver_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        buy_refund_deposit(required_storage_in_bytes, price);

        BuyNftjson {
            token_id: token_id,
            owner_id: receiver_id.clone(),
            art_id,
        }
    }

}
