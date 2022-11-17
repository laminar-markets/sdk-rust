use crate::types::order::Id;
use crate::types::{deserialize_from_str, u64_to_str};
use crate::{Side, TimeInForce};
use anyhow::Context;
use aptos_sdk::move_types::identifier::Identifier;
use aptos_sdk::move_types::language_storage::{StructTag, TypeTag};
use aptos_sdk::types::account_address::AccountAddress;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Formatter;
use std::str::FromStr;

pub(crate) trait EventStoreField<'a> {
    fn event_store_field() -> &'a str;
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct TypeInfo {
    pub account_address: AccountAddress,
    pub module_name: String,
    pub struct_name: String,
}

impl std::fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "{}::{}::{}",
            self.account_address.to_hex_literal(),
            &self.module_name,
            &self.struct_name
        );
        f.write_str(&s)
    }
}

impl FromStr for TypeInfo {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr, rest) = s.split_once("::").context("failed splitting typeinfo")?;
        let addr = AccountAddress::from_hex_literal(addr)?;
        let (module_name, struct_name) =
            rest.split_once("::").context("failed splitting typeinfo")?;
        Ok(Self {
            account_address: addr,
            module_name: module_name.to_string(),
            struct_name: struct_name.to_string(),
        })
    }
}

impl From<&Box<StructTag>> for TypeInfo {
    fn from(s: &Box<StructTag>) -> Self {
        Self {
            account_address: s.address,
            module_name: s.module.to_string(),
            struct_name: s.name.to_string(),
        }
    }
}

impl From<TypeInfo> for TypeTag {
    fn from(s: TypeInfo) -> Self {
        TypeTag::Struct(Box::new(StructTag {
            address: s.account_address,
            module: Identifier::from_str(&s.module_name).unwrap(),
            name: Identifier::from_str(&s.struct_name).unwrap(),
            type_params: vec![],
        }))
    }
}

impl<'de> Deserialize<'de> for TypeInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            AccountAddress,
            ModuleName,
            StructName,
        }

        struct TypeInfoVisitor;

        impl<'de> Visitor<'de> for TypeInfoVisitor {
            type Value = TypeInfo;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct TypeInfo")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut account_address = None;
                let mut module_name = None;
                let mut struct_name = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::AccountAddress => {
                            if account_address.is_some() {
                                return Err(Error::duplicate_field("account_address"));
                            }
                            let addr = map.next_value::<String>()?;
                            let addr = AccountAddress::from_hex_literal(&addr)
                                .map_err(|_| Error::custom("failed parsing address"))?;
                            account_address = Some(addr);
                        }
                        Field::ModuleName => {
                            if module_name.is_some() {
                                return Err(Error::duplicate_field("module_name"));
                            }
                            let hex_bytes = map.next_value::<String>()?;
                            let hex_bytes = hex_bytes.trim_start_matches("0x");
                            let bytes = hex::decode(hex_bytes)
                                .map_err(|_| Error::custom("failed parsing bytes"))?;
                            let name = String::from_utf8(bytes)
                                .map_err(|_| Error::custom("failed parsing string"))?;
                            module_name = Some(name);
                        }
                        Field::StructName => {
                            if struct_name.is_some() {
                                return Err(Error::duplicate_field("struct_name"));
                            }
                            let hex_bytes = map.next_value::<String>()?;
                            let hex_bytes = hex_bytes.trim_start_matches("0x");
                            let bytes = hex::decode(hex_bytes)
                                .map_err(|_| Error::custom("failed parsing bytes"))?;
                            let name = String::from_utf8(bytes)
                                .map_err(|_| Error::custom("failed parsing string"))?;
                            struct_name = Some(name);
                        }
                    }
                }

                let account_address =
                    account_address.ok_or_else(|| Error::missing_field("account_address"))?;
                let module_name = module_name.ok_or_else(|| Error::missing_field("module_name"))?;
                let struct_name = struct_name.ok_or_else(|| Error::missing_field("struct_name"))?;

                Ok(TypeInfo {
                    account_address,
                    module_name,
                    struct_name,
                })
            }
        }

        const FIELDS: &[&str] = &["account_address", "module_name", "struct_name"];
        deserializer.deserialize_struct("TypeInfo", FIELDS, TypeInfoVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateOrderBookEvent {
    pub book_id: Id,
    pub creator: AccountAddress,
    pub base: TypeInfo,
    pub quote: TypeInfo,
    pub price_decimals: u8,
    pub size_decimals: u8,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub min_size_amount: u64,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub time: u64,
}

impl<'a> EventStoreField<'a> for CreateOrderBookEvent {
    fn event_store_field() -> &'a str {
        "create_orderbook_events"
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaceOrderEvent {
    pub book_id: Id,
    pub order_id: Id,
    pub side: Side,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub price: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub size: u64,
    pub time_in_force: TimeInForce,
    pub post_only: bool,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub time: u64,
}

impl<'a> EventStoreField<'a> for PlaceOrderEvent {
    fn event_store_field() -> &'a str {
        "place_order_events"
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AmendOrderEvent {
    pub book_id: Id,
    pub order_id: Id,
    pub amend_id: Id,
    pub side: Side,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub price: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub size: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub time: u64,
}

impl<'a> EventStoreField<'a> for AmendOrderEvent {
    fn event_store_field() -> &'a str {
        "amend_order_events"
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CancelOrderEvent {
    pub book_id: Id,
    pub order_id: Id,
    pub cancel_id: Id,
    pub side: Side,
    // TODO change reason to enum
    pub reason: u8,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub time: u64,
}

impl<'a> EventStoreField<'a> for CancelOrderEvent {
    fn event_store_field() -> &'a str {
        "cancel_order_events"
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FillEvent {
    pub book_id: Id,
    pub order_id: Id,
    pub side: Side,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub price: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub fill_size: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub fee: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub fee_rate: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub time: u64,
    #[serde(
        deserialize_with = "deserialize_from_str",
        serialize_with = "u64_to_str"
    )]
    pub remaining_size: u64,
    pub is_maker: bool,
}

impl<'a> EventStoreField<'a> for FillEvent {
    fn event_store_field() -> &'a str {
        "fill_events"
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LaminarEvent {
    CreateOrderBook(CreateOrderBookEvent),
    PlaceOrder(PlaceOrderEvent),
    AmendOrder(AmendOrderEvent),
    CancelOrder(CancelOrderEvent),
    FillEvent(FillEvent),
}
