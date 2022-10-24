pub mod types;

use crate::types::events::{
    AmendOrderEvent, CancelOrderEvent, CreateOrderBookEvent, EventStoreField, FillEvent,
    LaminarEvent, PlaceOrderEvent,
};
use crate::types::order::{Id, Order, OrderBook, Side, State, TimeInForce};
use anyhow::anyhow;
use aptos_api_types::{
    AptosErrorCode, MoveModuleId, MoveType, Transaction, TransactionInfo, UserTransactionRequest,
    U64,
};
use aptos_sdk::bcs;
use aptos_sdk::crypto::ed25519::Ed25519PrivateKey;
use aptos_sdk::crypto::ValidCryptoMaterialStringExt;
use aptos_sdk::move_types::ident_str;
use aptos_sdk::move_types::language_storage::{ModuleId, TypeTag};
use aptos_sdk::rest_client::aptos::Balance;
use aptos_sdk::rest_client::error::RestError;
use aptos_sdk::rest_client::Client;
use aptos_sdk::transaction_builder::TransactionFactory;
use aptos_sdk::types::account_address::AccountAddress;
use aptos_sdk::types::chain_id::ChainId;
use aptos_sdk::types::transaction::EntryFunction;
use aptos_sdk::types::{AccountKey, LocalAccount};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::str::FromStr;

pub const SUBMIT_ATTEMPTS: u8 = 10;

#[derive(Deserialize, Debug, Clone)]
struct AptosConfig {
    private_key: String,
    account: String,
}

type AptosConfigYaml = HashMap<String, HashMap<String, AptosConfig>>;

impl AptosConfig {
    pub fn from_config(path: &str, profile_name: &str) -> Self {
        let file = File::open(path).expect("invalid config path provided");
        let config =
            serde_yaml::from_reader::<File, AptosConfigYaml>(file).expect("config file is invalid");
        let profiles = config
            .get("profiles")
            .expect("profiles section missing in config file");
        profiles
            .get(profile_name)
            .expect("given profile name is missing in config file")
            .clone()
    }
}

pub struct LaminarTransaction {
    pub info: TransactionInfo,
    pub request: UserTransactionRequest,
    pub events: Vec<LaminarEvent>,
    pub timestamp: U64,
}

pub struct LaminarClient {
    laminar: AccountAddress,
    aptos_client: Client,
    chain_id: ChainId,
    account: LocalAccount,
}

impl LaminarClient {
    /// Connect to an Aptos node and initialize the Laminar Markets client.
    ///
    /// # Arguments:
    ///
    /// * `node_url` - Url of aptos node.
    /// * `laminar_address` - Aptos `AccountAddress`.
    /// * `account` - `LocalAccount` representing Aptos user account
    pub async fn connect(
        node_url: Url,
        laminar: AccountAddress,
        mut account: LocalAccount,
    ) -> Result<Self, anyhow::Error> {
        let aptos_client = Client::new(node_url);
        let index = aptos_client.get_index().await?;
        let index = index.inner();
        let chain_id = ChainId::new(index.chain_id);
        let account_info = aptos_client.get_account(account.address()).await?;
        let account_info = account_info.inner();
        let seq_num = account_info.sequence_number;
        let acc_seq_num = account.sequence_number_mut();
        *acc_seq_num = seq_num;

        Ok(Self {
            laminar,
            aptos_client,
            chain_id,
            account,
        })
    }

    /// Connect to an Aptos node and initialize the Laminar Markets client using
    /// url strings, account address string and private key string.
    ///
    /// # Arguments:
    ///
    /// * `node_url` - url string of aptos node to connect to.
    /// * `laminar_address` - hex encoded address string of account that holds the laminar modules.
    /// * `account_address` - hex encoded address string of user using this client.
    /// * `account_private_key` - hex encoded private key string of user using this client.
    ///
    /// # Panics:
    ///
    /// * If provided url is not valid.
    /// * If provided private key is invalid.
    pub async fn connect_with_strings(
        node_url: &str,
        laminar_address: &str,
        account_address: &str,
        account_private_key: &str,
    ) -> Result<Self, anyhow::Error> {
        let node_url = Url::parse(node_url).expect("node url is not valid");
        let laminar = AccountAddress::from_hex_literal(laminar_address)?;
        let account_address = AccountAddress::from_hex_literal(account_address)?;
        let private_key = Ed25519PrivateKey::from_encoded_string(account_private_key)
            .expect("private key provided is not valid");
        let account_key = AccountKey::from(private_key);
        let account = LocalAccount::new(account_address, account_key, 0);
        Self::connect(node_url, laminar, account).await
    }

    /// Connect to an Aptos node and initialize the Laminar Markets client using a config file.
    /// The config file format is the default format created by the aptos cli.
    ///
    /// # Arguments:
    ///
    /// * `node_url` - Url string of aptos node to connect to.
    /// * `laminar_address` - Hex encoded address string of account that holds the laminar modules.
    /// * `config_path` - Path to config file.
    /// * `config_profile_name` - Name of profile to use in the config file.
    pub async fn connect_with_config(
        node_url: &str,
        laminar_address: &str,
        config_path: &str,
        config_profile_name: &str,
    ) -> Result<Self, anyhow::Error> {
        let config = AptosConfig::from_config(config_path, config_profile_name);
        Self::connect_with_strings(
            node_url,
            laminar_address,
            &config.account,
            &config.private_key,
        )
        .await
    }

    pub fn laminar(&self) -> &AccountAddress {
        &self.laminar
    }

    pub fn aptos_client(&self) -> &Client {
        &self.aptos_client
    }

    pub fn account(&self) -> &LocalAccount {
        &self.account
    }

    /// Update the laminar clients aptos chain id.
    /// If the aptos team pushes out a new node deployment, the chain id may change.
    /// In case of a change the internal chain id needs to be updated
    pub async fn update_chain_id(&mut self) -> Result<(), anyhow::Error> {
        let index = self.aptos_client.get_index().await?;
        let index = index.inner();
        let chain_id = ChainId::new(index.chain_id);
        self.chain_id = chain_id;
        Ok(())
    }

    // TODO doc strings for these functions
    pub async fn get_sequence_number(&self) -> Result<u64, anyhow::Error> {
        let account = self
            .aptos_client
            .get_account(self.account.address())
            .await?
            .into_inner();
        Ok(account.sequence_number)
    }

    pub async fn does_coin_exist(&self, coin: &TypeTag) -> Result<bool, anyhow::Error> {
        let coin_info = format!("0x1::coin::CoinInfo<{}>", coin);
        let res = self
            .aptos_client
            .get_account_resource(self.account.address(), &coin_info)
            .await?
            .into_inner();
        Ok(res.is_some())
    }

    pub async fn is_registered_for_coin(&self, coin: &TypeTag) -> Result<bool, anyhow::Error> {
        let coin_store = format!("0x1::coin::CoinStore<{}>", coin);
        let res = self
            .aptos_client
            .get_account_resource(self.account.address(), &coin_store)
            .await?
            .into_inner();
        Ok(res.is_some())
    }

    pub fn register_for_coin(coin: &TypeTag) -> Result<EntryFunction, anyhow::Error> {
        let entry = EntryFunction::new(
            ModuleId::from(MoveModuleId::from_str("0x1::managed_coin")?),
            ident_str!("register").to_owned(),
            vec![coin.clone()],
            vec![],
        );

        Ok(entry)
    }

    pub async fn get_coin_balance(&self, coin: &TypeTag) -> Result<U64, anyhow::Error> {
        let coin_store = format!("0x1::coin::CoinStore<{}>", coin);
        let resp = self
            .aptos_client
            .get_account_resource(self.account.address(), &coin_store)
            .await?;
        let resp = resp
            .inner()
            .as_ref()
            .ok_or_else(|| anyhow!("could not find CoinStore"))?;
        let balance = serde_json::from_value::<Balance>(resp.data.clone())?;
        Ok(balance.coin.value)
    }

    /// Create payload for this client's account to be registered to trade on Laminar
    pub fn register_user_payload(&self) -> EntryFunction {
        EntryFunction::new(
            ModuleId::new(self.laminar, ident_str!("book").to_owned()),
            ident_str!("register_user").to_owned(),
            vec![],
            vec![],
        )
    }

    /// Create payload for creating an `OrderBook`.
    ///
    /// # Arguments:
    ///
    /// * `base` - Aptos `TypeTag` of the `OrderBook` base coin.
    /// * `quote` - Aptos `TypeTag` of the `OrderBook` quote coin.
    /// * `min_price_tick` - Minimum price difference between order prices.
    /// E.g. a min price size of 2 would mean that order prices can only be even numbers.
    /// * `min_size_tick` - Minimum size difference between order sizes.
    /// E.g. a min size tick of 2 would mean that order sizes can only be even numbers.
    /// * `min_size_amount` - Minimum order size for orders in the `OrderBook`.
    pub fn create_orderbook_payload(
        &self,
        base: &TypeTag,
        quote: &TypeTag,
        price_decimals: u8,
        size_decimals: u8,
        min_size_amount: u64,
    ) -> Result<EntryFunction, anyhow::Error> {
        let entry = EntryFunction::new(
            ModuleId::new(self.laminar, ident_str!("book").to_owned()),
            ident_str!("create_orderbook").to_owned(),
            vec![base.clone(), quote.clone()],
            vec![
                bcs::to_bytes(&price_decimals)?,
                bcs::to_bytes(&size_decimals)?,
                bcs::to_bytes(&min_size_amount)?,
            ],
        );

        Ok(entry)
    }

    fn get_book_bids_type(&self, base: &TypeTag, quote: &TypeTag) -> String {
        format!(
            "{}::book::OrderBookBids<{}, {}>",
            self.laminar.to_hex_literal(),
            base,
            quote
        )
    }

    fn get_book_asks_type(&self, base: &TypeTag, quote: &TypeTag) -> String {
        format!(
            "{}::book::OrderBookAsks<{}, {}>",
            self.laminar.to_hex_literal(),
            base,
            quote
        )
    }

    /// Fetch `OrderBook` information from Aptos node.
    ///
    /// # Arguments:
    ///
    /// * `base` - Aptos `TypeTag` of the orderbook base coin.
    /// * `quote` - Aptos `TypeTag` of the orderbook quote coin.
    /// * `book_owner` - Address of the account that owns the `OrderBook`.
    pub async fn fetch_orderbook(
        &self,
        base: &TypeTag,
        quote: &TypeTag,
        book_owner: &AccountAddress,
    ) -> Result<OrderBook, anyhow::Error> {
        let mut bids_book = self
            .fetch_orderbook_side(self.get_book_bids_type(base, quote), book_owner)
            .await?;
        let OrderBook { asks, .. } = self
            .fetch_orderbook_side(self.get_book_asks_type(base, quote), book_owner)
            .await?;

        bids_book.asks = asks;
        Ok(bids_book)
    }

    async fn fetch_orderbook_side(
        &self,
        book_type: String,
        book_owner: &AccountAddress,
    ) -> Result<OrderBook, anyhow::Error> {
        let res = self
            .aptos_client
            .get_account_resource(*book_owner, &book_type)
            .await?;
        if let Some(res) = res.inner() {
            let mut book = serde_json::from_value::<OrderBook>(res.data.clone())?;
            let types = res.resource_type.type_params.clone();
            book.type_tags.extend(types);
            Ok(book)
        } else {
            Err(anyhow!("book not found"))
        }
    }

    /// Checks if account using this client is eligible to trade on Laminar
    pub async fn is_user_registered(&self) -> Result<bool, anyhow::Error> {
        let event_store_type = format!("{}::book::OrderBookStore", self.laminar.to_hex_literal(),);

        let res = self
            .aptos_client
            .get_account_resource(self.account.address(), &event_store_type)
            .await?
            .into_inner();
        Ok(res.is_some())
    }

    /// Create payload for placing a limit order.
    ///
    /// # Arguments:
    ///
    /// * `base` - Aptos `TypeTag` of the orderbook base coin.
    /// * `quote` - Aptos `TypeTag` of the orderbook quote coin.
    /// * `book_owner` - Address of the account that owns the `OrderBook`.
    /// * `side` - `OrderSide`: Bid or Ask.
    /// * `price` - Price in `U64` of limit order.
    /// * `size` - `U64` size of limit order.
    /// * `time_in_force` - `TimeInForce` for limit order, can be GTC, IOC, or FOK.
    /// * `post_only` - Flag to specify whether or not the limit order is `post_only`.
    #[allow(clippy::too_many_arguments)]
    pub fn place_limit_order_payload(
        &self,
        base: &TypeTag,
        quote: &TypeTag,
        book_owner: &AccountAddress,
        side: Side,
        price: u64,
        size: u64,
        time_in_force: TimeInForce,
        post_only: bool,
    ) -> Result<EntryFunction, anyhow::Error> {
        let entry = EntryFunction::new(
            ModuleId::new(self.laminar, ident_str!("book").to_owned()),
            ident_str!("place_limit_order").to_owned(),
            vec![base.clone(), quote.clone()],
            vec![
                bcs::to_bytes(book_owner)?,
                bcs::to_bytes(&side)?,
                bcs::to_bytes(&price)?,
                bcs::to_bytes(&size)?,
                bcs::to_bytes(&time_in_force)?,
                bcs::to_bytes(&post_only)?,
            ],
        );

        Ok(entry)
    }

    /// Create payload for placing a market order.
    ///
    /// # Arguments:
    ///
    /// * `base` - Aptos `TypeTag` of the orderbook base coin.
    /// * `quote` - Aptos `TypeTag` of the orderbook quote coin.
    /// * `book_owner` - Address of the account that owns the `OrderBook`.
    /// * `side` - `Side`: Bid or Ask.
    /// * `size` - U64 size of market order.
    pub fn place_market_order_payload(
        &self,
        base: &TypeTag,
        quote: &TypeTag,
        book_owner: &AccountAddress,
        side: Side,
        size: u64,
    ) -> Result<EntryFunction, anyhow::Error> {
        let entry = EntryFunction::new(
            ModuleId::new(self.laminar, ident_str!("book").to_owned()),
            ident_str!("place_market_order").to_owned(),
            vec![base.clone(), quote.clone()],
            vec![
                bcs::to_bytes(book_owner)?,
                bcs::to_bytes(&side)?,
                bcs::to_bytes(&size)?,
            ],
        );

        Ok(entry)
    }

    /// Create payload for amending an order.
    ///
    /// # Arguments:
    ///
    /// * `base` - Aptos `TypeTag` of the orderbook base coin.
    /// * `quote` - Aptos `TypeTag` of the orderbook quote coin.
    /// * `book_owner` - Address of the account that owns the `OrderBook`.
    /// * `order_id` - ID of order to amend.
    /// * `side` - `OrderSide`: Bid or Ask.
    /// * `price` - Price to update, provide current price if no amendment needed.
    /// * `size` - Size to update, provide current size if no amendment needed.
    #[allow(clippy::too_many_arguments)]
    pub fn amend_order_payload(
        &self,
        base: &TypeTag,
        quote: &TypeTag,
        book_owner: &AccountAddress,
        order_id: &Id,
        side: Side,
        price: u64,
        size: u64,
    ) -> Result<EntryFunction, anyhow::Error> {
        let entry = EntryFunction::new(
            ModuleId::new(self.laminar, ident_str!("book").to_owned()),
            ident_str!("amend_order").to_owned(),
            vec![base.clone(), quote.clone()],
            vec![
                bcs::to_bytes(book_owner)?,
                bcs::to_bytes(&order_id.creation_num.0)?,
                bcs::to_bytes(&side)?,
                bcs::to_bytes(&price)?,
                bcs::to_bytes(&size)?,
            ],
        );

        Ok(entry)
    }

    /// Create payload for canceling an order.
    ///
    /// # Arguments:
    ///
    /// * `base` - Aptos `TypeTag` of the orderbook base coin.
    /// * `quote` - Aptos `TypeTag` of the orderbook quote coin.
    /// * `book_owner` - Address of the account that owns the `OrderBook`.
    /// * `order_id` - ID of order to cancel.
    /// * `side` - `OrderSide`: Bid or Ask.
    pub fn cancel_order_payload(
        &self,
        base: &TypeTag,
        quote: &TypeTag,
        book_owner: &AccountAddress,
        order_id: &Id,
        side: Side,
    ) -> Result<EntryFunction, anyhow::Error> {
        let entry = EntryFunction::new(
            ModuleId::new(self.laminar, ident_str!("book").to_owned()),
            ident_str!("cancel_order").to_owned(),
            vec![base.clone(), quote.clone()],
            vec![
                bcs::to_bytes(book_owner)?,
                bcs::to_bytes(&order_id.creation_num.0)?,
                bcs::to_bytes(&side)?,
            ],
        );

        Ok(entry)
    }

    async fn submit_tx(
        &mut self,
        payload: EntryFunction,
    ) -> Result<LaminarTransaction, anyhow::Error> {
        let addr = self.account.address();
        let tx = TransactionFactory::new(self.chain_id)
            .entry_function(payload)
            .sender(addr)
            .sequence_number(self.account.sequence_number())
            .max_gas_amount(1_000_000)
            .build();

        let signed_tx = self.account.sign_transaction(tx);
        let pending = match self.aptos_client.submit(&signed_tx).await {
            Ok(res) => res.into_inner(),
            Err(e) => {
                return if let RestError::Api(a) = e {
                    match a.error.error_code {
                        AptosErrorCode::InvalidTransactionUpdate
                        | AptosErrorCode::SequenceNumberTooOld
                        | AptosErrorCode::VmError => {
                            let seq_num = self.get_sequence_number().await?;
                            let acc_seq_num = self.account.sequence_number_mut();
                            *acc_seq_num = max(seq_num, *acc_seq_num + 1);
                            Err(anyhow!(a))
                        }
                        _ => Err(anyhow!(a)),
                    }
                } else {
                    Err(anyhow!(e))
                }
            }
        };

        let tx = self.aptos_client.wait_for_transaction(&pending).await?;
        if let Transaction::UserTransaction(ut) = tx.inner() {
            let events = ut
                .events
                .iter()
                .filter(|e| {
                    if let MoveType::Struct(s) = &e.typ {
                        s.address.inner() == self.laminar()
                    } else {
                        false
                    }
                })
                .map(|e| serde_json::from_value(e.data.clone()))
                .collect::<Result<Vec<LaminarEvent>, serde_json::Error>>()?;

            Ok(LaminarTransaction {
                info: ut.info.clone(),
                request: ut.request.clone(),
                events,
                timestamp: ut.timestamp,
            })
        } else {
            Err(anyhow!("not a user transaction"))
        }
    }

    /// Utility method for building and submitting a tx
    ///
    /// # Arguments:
    ///
    /// * `payload` - Entry function payload to be used in the tx.
    pub async fn build_and_submit_tx(
        &mut self,
        payload: EntryFunction,
    ) -> Result<LaminarTransaction, anyhow::Error> {
        for i in 0..SUBMIT_ATTEMPTS {
            match self.submit_tx(payload.clone()).await {
                Ok(lt) => return Ok(lt),
                Err(e) => {
                    if i == SUBMIT_ATTEMPTS - 1 {
                        return Err(e);
                    };
                }
            }
        }

        Err(anyhow!("failed submitting tx"))
    }

    async fn get_dex_events<'a, T>(&self) -> Result<Vec<T>, anyhow::Error>
    where
        T: EventStoreField<'a> + DeserializeOwned,
    {
        let event_store = format!("{}::book::OrderBookStore", self.laminar.to_hex_literal(),);
        let events = self
            .aptos_client
            .get_account_events(
                self.account.address(),
                &event_store,
                T::event_store_field(),
                None,
                None,
            )
            .await?;
        let result = events
            .inner()
            .iter()
            .map(|e| serde_json::from_value(e.clone().data))
            .collect::<Result<Vec<T>, serde_json::Error>>()?;

        Ok(result)
    }

    async fn get_filtered_dex_events<'a, E, P>(&self, predicate: P) -> Result<Vec<E>, anyhow::Error>
    where
        E: EventStoreField<'a> + DeserializeOwned + Clone + Send,
        P: Send + Fn(&E) -> bool,
    {
        let res = self.get_dex_events::<E>().await?;
        let mut result: Vec<E> = vec![];
        for e in &res {
            if predicate(e) {
                result.push(e.to_owned());
            }
        }

        Ok(res)
    }

    /// Fetch all order books.
    pub async fn fetch_order_books(&self) -> Result<Vec<CreateOrderBookEvent>, anyhow::Error> {
        let filter = |_e: &CreateOrderBookEvent| true;
        self.get_filtered_dex_events(filter).await
    }

    /// Fetch all place order events for this client's account for a given book.
    ///
    /// # Arguments:
    ///
    /// * `book_id` - `OrderBook` Id.
    pub async fn fetch_all_place_events(
        &self,
        book_id: &Id,
    ) -> Result<Vec<PlaceOrderEvent>, anyhow::Error> {
        let filter = |e: &PlaceOrderEvent| &e.book_id == book_id;
        self.get_filtered_dex_events(filter).await
    }

    /// Fetch place order event for a given order ID.
    ///
    /// # Arguments:
    ///
    /// * `order_id` - ID of order to fetch place event for.
    pub async fn get_place_event(&self, order_id: &Id) -> Result<PlaceOrderEvent, anyhow::Error> {
        self.get_dex_events::<PlaceOrderEvent>()
            .await?
            .iter()
            .find(|e| order_id == &e.order_id)
            .cloned()
            .ok_or_else(|| anyhow!("order not found"))
    }

    /// Fetch all amend order events for this client's account for a given book.
    ///
    /// # Arguments:
    ///
    /// * `book_id` - `OrderBook` Id.
    pub async fn fetch_all_amend_events(
        &self,
        book_id: &Id,
    ) -> Result<Vec<AmendOrderEvent>, anyhow::Error> {
        let filter = |e: &AmendOrderEvent| &e.book_id == book_id;
        self.get_filtered_dex_events(filter).await
    }

    async fn get_amends_internal(
        &self,
        order_id: &Id,
    ) -> Result<Vec<AmendOrderEvent>, anyhow::Error> {
        let filter = |e: &AmendOrderEvent| order_id == &e.order_id;
        self.get_filtered_dex_events(filter).await
    }

    /// Fetch amend order events for a given order ID.
    ///
    /// # Arguments:
    ///
    /// * `order_id` - ID of order to fetch amend events for.
    pub async fn get_amend_events(
        &self,
        order_id: &Id,
    ) -> Result<Vec<AmendOrderEvent>, anyhow::Error> {
        match self.get_place_event(order_id).await {
            Ok(_) => self.get_amends_internal(order_id).await,
            Err(e) => Err(e),
        }
    }

    /// Fetch all cancel order events for this client's account for a given book.
    ///
    /// # Arguments:
    ///
    /// * `book_id` - `OrderBook` Id.
    pub async fn fetch_all_cancel_events(
        &self,
        book_id: &Id,
    ) -> Result<Vec<CancelOrderEvent>, anyhow::Error> {
        let filter = |e: &CancelOrderEvent| &e.book_id == book_id;
        self.get_filtered_dex_events(filter).await
    }

    /// Fetch cancel order event for a given order ID.
    ///
    /// # Arguments:
    ///
    /// * `order_id` - ID of order to fetch cancel event for.
    pub async fn get_cancel_event(
        &self,
        order_id: &Id,
    ) -> Result<Option<CancelOrderEvent>, anyhow::Error> {
        let res = self
            .get_dex_events::<CancelOrderEvent>()
            .await?
            .iter()
            .find(|e| order_id == &e.order_id)
            .cloned();
        Ok(res)
    }

    /// Fetch all fill events for this client's account for all orders
    ///
    /// # Arguments:
    ///
    /// * `book_id` - `OrderBook` Id.
    pub async fn fetch_all_fill_events(
        &self,
        book_id: &Id,
    ) -> Result<Vec<FillEvent>, anyhow::Error> {
        let filter = |e: &FillEvent| &e.book_id == book_id;
        self.get_filtered_dex_events(filter).await
    }

    async fn get_fills_internal(&self, order_id: &Id) -> Result<Vec<FillEvent>, anyhow::Error> {
        let filter = |e: &FillEvent| order_id == &e.order_id;
        self.get_filtered_dex_events(filter).await
    }

    /// Fetch fill events for a given order ID.
    ///
    /// # Arguments:
    ///
    /// * `order_id` - ID of order to fetch fill events for.
    pub async fn get_fill_events(&self, order_id: &Id) -> Result<Vec<FillEvent>, anyhow::Error> {
        match self.get_place_event(order_id).await {
            Ok(_) => self.get_fills_internal(order_id).await,
            Err(e) => Err(e),
        }
    }

    /// Fetch order object given an order ID
    ///
    /// # Arguments:
    ///
    /// * `order_id` - ID of order to fetch fill events for.
    pub async fn get_order(&self, order_id: &Id) -> Result<Order, anyhow::Error> {
        let place_event = self.get_place_event(order_id).await?;
        let amend_events = self.get_amends_internal(order_id).await?;
        let cancel_event = self.get_cancel_event(order_id).await?;
        let fills = self.get_fills_internal(order_id).await?;

        let (price, size) = match amend_events.last() {
            Some(a) => (a.price, a.size),
            None => (place_event.price, place_event.size),
        };

        let remaining_size = fills.last().map_or(0, |f| f.remaining_size);

        let state = if remaining_size == 0 || cancel_event.is_some() {
            State::Closed
        } else if !fills.is_empty() {
            State::PartiallyFilled
        } else {
            State::Open
        };

        let o = Order {
            id: order_id.clone(),
            side: place_event.side,
            price,
            size,
            post_only: place_event.post_only,
            remaining_size,
            state,
            fills,
        };

        Ok(o)
    }
}

#[cfg(test)]
mod tests {}
