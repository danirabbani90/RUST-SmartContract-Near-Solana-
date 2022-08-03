use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn batch_mint(
        &mut self,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        num_to_mint: u64,                               // Num of Tokens to Mint
        amount: Option<U128>,                           // For NFT Price
        royalty: Option<HashMap<AccountId, u32>>,       // Royalties
        split_payment: Option<HashMap<AccountId, u32>>, // SplitPayments
    ) -> TokenSeriesJson {
        assert_at_least_one_yocto();
        assert!(num_to_mint > 0, "PLEASE ENTER NUM TO MINT  > 0 ");
        assert!(
            num_to_mint <= 125,
            "CANNOT MINT MORE THAN 125 TOKENS DUE TO GAS LIMITS"
        );

        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let mut total_perpetual = 0;
        let mut total_accounts = 0;
        let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
            for (k, v) in royalty.iter() {
                if !env::is_valid_account_id(k.as_bytes()) {
                    env::panic_str("ACCOUNT ID NOT VALID");
                };
                total_perpetual += *v;
                total_accounts += 1;
            }
            royalty
        } else {
            HashMap::new()
        };

        let split_res: HashMap<AccountId, u32> = if let Some(split_payment) = split_payment {
            for (k, v) in split_payment.iter() {
                if !env::is_valid_account_id(k.as_bytes()) {
                    env::panic_str("ACCOUNT ID NOT VALID");
                };
                total_perpetual += *v;
                total_accounts += 1;
            }
            split_payment
        } else {
            HashMap::new()
        };

        assert!(
            total_accounts <= 10,
            "LIMIT EXCEEDED ONLY 10 ACCOOUNTS ARE ALLOWED"
        );

        assert!(
            total_perpetual <= 5000,
            " LIMIT EXCEEDED ONLY ALLOWED  50% FOR ROYALTIES AND SPLIT PAYMENTS",
        );

        let price: Option<u128> = if amount.is_some() {
            Some(amount.unwrap().0)
        } else {
            None
        };

        //specify the token struct that contains the owner ID
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,

            // Price of NFT
            price,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty: royalty_res,

            splitpayments: split_res,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        let mut tokens: Vec<u64> = Vec::new();
        (0..num_to_mint).for_each(|i| {
            // let token_id = format!("{}" , self.tokens_by_id.len()+i) ;
            let token_id = self.tokens_minted + i;
            tokens.push(token_id);
            self.tokens_by_id.insert(&token_id, &token);

            //insert the token ID and metadata
            self.token_metadata_by_id.insert(&token_id, &metadata);

            //call the internal method for adding the token to the owner
            self.internal_add_token_to_owner(&token.owner_id, &token_id);

            // Construct the mint log as per the events standard.
            let nft_mint_log: EventLog = EventLog {
                // Standard name ("nep171").
                standard: NFT_STANDARD_NAME.to_string(),
                // Version of the standard ("nft-1.0.0").
                version: NFT_METADATA_SPEC.to_string(),
                // The data related with the event stored in a vector.
                event: EventLogVariant::NftMint(vec![NftMintLog {
                    // Owner of the token.
                    owner_id: token.owner_id.to_string(),
                    // Vector of token IDs that were minted.
                    token_ids: vec![token_id.to_string()],
                    // An optional memo to include.
                    memo: None,
                }]),
            };
            // Log the serialized json.
            env::log_str(&nft_mint_log.to_string());
        });

        self.tokens_minted += num_to_mint;

        let minted = self.tokens_minted - 1;

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);

        TokenSeriesJson {
            token_id: minted,
            metadata: metadata,
            owner_id: token.owner_id,
            token_ids: tokens,
        }
    }

    #[payable]
    pub fn unique_mint(
        &mut self,
        metadata: Vec<TokenMetadata>,
        receiver_id: AccountId,
        num_to_mint: u64,                               // Num of Tokens to Mint
        amount: Vec<Option<u128>>,                      // For NFT Price
        royalty: Option<HashMap<AccountId, u32>>,       // Royalties
        split_payment: Option<HashMap<AccountId, u32>>, // SplitPayments
    ) -> UniqueMintJson {
        assert_at_least_one_yocto();
        assert!(num_to_mint > 0, "PLEASE ENTER NUM TO MINT  > 0 ");
        assert!(
            num_to_mint <= 125,
            "CANNOT MINT MORE THAN 125 TOKENS DUE TO GAS LIMITS"
        );

        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let mut total_perpetual = 0;
        let mut total_accounts = 0;
        let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
            for (k, v) in royalty.iter() {
                if !env::is_valid_account_id(k.as_bytes()) {
                    env::panic_str("ACCOUNT ID NOT VALID");
                };
                total_perpetual += *v;
                total_accounts += 1;
            }
            royalty
        } else {
            HashMap::new()
        };

        let split_res: HashMap<AccountId, u32> = if let Some(split_payment) = split_payment {
            for (k, v) in split_payment.iter() {
                if !env::is_valid_account_id(k.as_bytes()) {
                    env::panic_str("ACCOUNT ID NOT VALID");
                };
                total_perpetual += *v;
                total_accounts += 1;
            }
            split_payment
        } else {
            HashMap::new()
        };

        assert!(
            total_accounts <= 10,
            "LIMIT EXCEEDED ONLY 10 ACCOOUNTS ARE ALLOWED"
        );

        assert!(
            total_perpetual <= 5000,
            " LIMIT EXCEEDED ONLY ALLOWED  50% FOR ROYALTIES AND SPLIT PAYMENTS",
        );

        //insert the token ID and token struct and make sure that the token doesn't exist
        let mut tokens: Vec<u64> = Vec::new();
        (0..num_to_mint).for_each(|i| {
            // let token_id = format!("{}" , self.tokens_by_id.len()+i) ;
            let token_id = self.tokens_minted + i;
            // let type_cast = i as usize;
            tokens.push(token_id);

            let type_cast = i as usize;

            let price: Option<u128> = if amount[type_cast].is_some() {
                Some(amount[type_cast].unwrap())
            } else {
                None
            };

            //specify the token struct that contains the owner ID
            let token = Token {
                //set the owner ID equal to the receiver ID passed into the function
                owner_id: receiver_id.clone(),
                //we set the approved account IDs to the default value (an empty map)
                approved_account_ids: Default::default(),
                //the next approval ID is set to 0
                next_approval_id: 0,

                // Price of NFT
                price,
                //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
                royalty: royalty_res.clone(),

                splitpayments: split_res.clone(),
            };

            self.tokens_by_id.insert(&token_id, &token);

            //insert the token ID and metadata
            self.token_metadata_by_id
                .insert(&token_id, &metadata[type_cast]);

            //call the internal method for adding the token to the owner
            self.internal_add_token_to_owner(&token.owner_id, &token_id);



            // Construct the mint log as per the events standard.
            let nft_mint_log: EventLog = EventLog {
                // Standard name ("nep171").
                standard: NFT_STANDARD_NAME.to_string(),
                // Version of the standard ("nft-1.0.0").
                version: NFT_METADATA_SPEC.to_string(),
                // The data related with the event stored in a vector.
                event: EventLogVariant::NftUniqueMint(vec![NftUniqueMintLog {
                    // Owner of the token.
                    owner_id: token.owner_id.to_string(),
                    // Vector of token IDs that were minted.
                    token_ids: vec![token_id.to_string()],
                    // An optional memo to include.
                    memo: None,
                }]),
            };
            // Log the serialized json.
            env::log_str(&nft_mint_log.to_string());
        });

        self.tokens_minted += num_to_mint;

        let minted = self.tokens_minted - 1;

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);

        UniqueMintJson {
            token_id: minted,
            token_ids: tokens,
            metadata,
        }
    }

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

    #[payable]
    pub fn update_price(
        &mut self,
        token_id: TokenId,
        amount: Option<U128>,
        art_id: String,
        updated_price: String,
    ) -> UpdatePriceJson {
        assert_one_yocto();
        let mut token_data = self.tokens_by_id.get(&token_id).expect("No Token Exists");
        assert_eq!(
            env::predecessor_account_id(),
            token_data.owner_id,
            "ONLY TOKEN OWNER CAN UPDATE PRICE",
        );
        let price: Option<u128> = Some(amount.unwrap().0);
        token_data.price = price;
        self.tokens_by_id.insert(&token_id, &token_data);
        UpdatePriceJson {
            token_id: token_id,
            updated_price,
            art_id,
        }
    }

    #[payable]
    pub fn set_transaction_fee(&mut self, next_fee: u16) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id.clone(),
            " UNAUTHORIZED - ONLY OWNER "
        );

        assert!(
            next_fee < 10_000,
            "YOU CANNOT SET TRANSACTION FEES TO 100% "
        );

        self.transaction_fee.current_fee = next_fee;
    }

    pub fn get_transaction_fee(&self) -> &TransactionFee {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id.clone(),
            "UNAUTHORIZED - ONLY OWNER"
        );
        &self.transaction_fee
    }

    pub fn calculate_current_transaction_fee(&mut self) -> u128 {
        self.transaction_fee.current_fee as u128
    }
}
