use crate::types::deserialize_from_str;
use crate::types::events::FillEvent;
#[cfg(feature = "db")]
use anyhow::anyhow;
use aptos_api_types::{Address, U64};
use aptos_sdk::move_types::language_storage::TypeTag;
use aptos_sdk::types::account_address::AccountAddress;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Formatter;
use std::num::ParseIntError;
#[cfg(feature = "db")]
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Id {
    pub creation_num: U64,
    pub addr: Address,
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = format!(
            "{}:{}",
            self.addr.inner().to_hex_literal(),
            self.creation_num.0
        );
        f.write_str(&s)
    }
}

#[cfg(feature = "db")]
impl FromStr for Id {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (num, addr) = s
            .split_once(':')
            .ok_or_else(|| anyhow!("invalid ID string"))?;
        Ok(Self {
            creation_num: U64::from(num.parse::<u64>()?),
            addr: addr.parse()?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

#[cfg(feature = "db")]
impl TryFrom<i16> for Side {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bid),
            1 => Ok(Self::Ask),
            _ => Err(anyhow!("failed parsing side: {:?}", value)),
        }
    }
}

impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderSideVisitor;

        impl<'de> Visitor<'de> for OrderSideVisitor {
            type Value = Side;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("Bid=0 or Ask=1")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    0 => Ok(Side::Bid),
                    1 => Ok(Side::Ask),
                    _ => Err(E::custom("Bid=0 or Ask=1")),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let number = v
                    .parse::<u64>()
                    .map_err(|e| E::custom(format!("{:?} is an invalid OrderSide string", e)))?;
                self.visit_u64(number)
            }
        }

        deserializer.deserialize_any(OrderSideVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum TimeInForce {
    GoodTillCanceled = 0,
    ImmediateOrCancel = 1,
    FillOrKill = 2,
}

#[cfg(feature = "db")]
impl TryFrom<i16> for TimeInForce {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::GoodTillCanceled),
            1 => Ok(Self::ImmediateOrCancel),
            2 => Ok(Self::FillOrKill),
            _ => Err(anyhow!("failed parsing time_in_force: {:?}", value)),
        }
    }
}

impl<'de> Deserialize<'de> for TimeInForce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeInForceVisitor;

        impl<'de> Visitor<'de> for TimeInForceVisitor {
            type Value = TimeInForce;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("GTC=0 or IOC=1 or FOK=2")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    0 => Ok(TimeInForce::GoodTillCanceled),
                    1 => Ok(TimeInForce::ImmediateOrCancel),
                    2 => Ok(TimeInForce::FillOrKill),
                    _ => Err(E::custom("GTC=0 or IOC=1 or FOK=2")),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let number = v
                    .parse::<u64>()
                    .map_err(|e| E::custom(format!("{:?} is an invalid TimeInForce string", e)))?;
                self.visit_u64(number)
            }
        }

        deserializer.deserialize_any(TimeInForceVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Default)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum State {
    #[default]
    Open = 0,
    PartiallyFilled = 1,
    Closed = 2,
}

#[cfg(feature = "db")]
impl TryFrom<i16> for State {
    type Error = anyhow::Error;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::PartiallyFilled),
            2 => Ok(Self::Closed),
            _ => Err(anyhow!("failed parsing state: {:?}", value)),
        }
    }
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StateVisitor;

        impl<'de> Visitor<'de> for StateVisitor {
            type Value = State;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("Open=0, PartiallyFilled=1 or Closed=2")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    0 => Ok(State::Open),
                    1 => Ok(State::PartiallyFilled),
                    2 => Ok(State::Closed),
                    _ => Err(E::custom("GTC=0 or IOC=1 or FOK=2")),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let number = v
                    .parse::<u64>()
                    .map_err(|e| E::custom(format!("{:?} is an invalid TimeInForce string", e)))?;
                self.visit_u64(number)
            }
        }

        deserializer.deserialize_any(StateVisitor)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Instrument {
    pub owner: AccountAddress,
    pub price_decimals: u8,
    pub size_decimals: u8,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub min_size_amount: u64,
    pub base_decimals: u8,
    pub quote_decimals: u8,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
pub struct Order {
    pub id: Id,
    pub side: Side,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub price: u64,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub size: u64,
    pub post_only: bool,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub remaining_size: u64,
    #[serde(skip)]
    pub state: State,
    #[serde(skip)]
    pub fills: Vec<FillEvent>,
}

#[derive(Debug, Deserialize, Clone)]
struct GuardedIdx {
    #[serde(deserialize_with = "deserialize_from_str")]
    value: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct OrderOption {
    vec: Vec<Order>,
}

#[derive(Debug, Deserialize, Clone)]
struct OrderNode {
    next: GuardedIdx,
    value: OrderOption,
}

#[derive(Debug, Deserialize, Clone)]
struct OrderQueue {
    head: GuardedIdx,
    nodes: Vec<OrderNode>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
struct OrderPriceLevel {
    price: u64,
    orders: Vec<Order>,
}

impl<'de> Deserialize<'de> for OrderPriceLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Key,
            Left,
            Right,
            Value,
        }

        struct OrderPriceLevelVisitor;

        impl<'de> Visitor<'de> for OrderPriceLevelVisitor {
            type Value = OrderPriceLevel;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct OrderPriceLevel")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut price = None;
                let mut orders = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Key => {
                            if price.is_some() {
                                return Err(Error::duplicate_field("key"));
                            }
                            let price_str = map.next_value::<String>()?;
                            let price_num = price_str
                                .parse::<u64>()
                                .map_err(|_| Error::custom("failed parsing string as u64"))?;
                            price = Some(price_num)
                        }
                        Field::Value => {
                            if orders.is_some() {
                                return Err(Error::duplicate_field("value"));
                            }
                            let res = map.next_value::<OrderQueue>()?;
                            let mut order_queue = vec![];
                            let mut current = res.head;
                            while current.value != u64::MAX {
                                let o = res.nodes.get(current.value as usize).ok_or_else(|| {
                                    Error::custom("failed finding order in nodes")
                                })?;
                                current = o.next.clone();
                                let o = o.value.vec.get(0).ok_or_else(|| {
                                    Error::custom("failed fetching order out of option")
                                })?;
                                order_queue.push(o.clone());
                            }
                            orders = Some(order_queue);
                        }
                        Field::Left | Field::Right => {}
                    }
                }

                let price = price.ok_or_else(|| Error::missing_field("key"))?;
                let orders = orders.ok_or_else(|| Error::missing_field("value"))?;

                Ok(OrderPriceLevel { price, orders })
            }
        }

        const FIELDS: &[&str] = &["price", "orders"];
        deserializer.deserialize_struct("OrderPriceLevel", FIELDS, OrderPriceLevelVisitor)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
struct OrderBookSide {
    levels: Vec<OrderPriceLevel>,
}

impl<'de> Deserialize<'de> for OrderBookSide {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Max,
            Min,
            Nodes,
            RemovedNodes,
            Root,
            SingleSplay,
        }

        struct OrderBookSideVisitor;

        impl<'de> Visitor<'de> for OrderBookSideVisitor {
            type Value = OrderBookSide;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct OrderBookSide")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut nodes = None;
                let mut removed_nodes = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Nodes => {
                            if nodes.is_some() {
                                return Err(Error::duplicate_field("nodes"));
                            }
                            nodes = Some(map.next_value::<Vec<OrderPriceLevel>>()?);
                        }
                        Field::RemovedNodes => {
                            if removed_nodes.is_some() {
                                return Err(Error::duplicate_field("removed_nodes"));
                            }
                            let res = map
                                .next_value::<Vec<String>>()?
                                .into_iter()
                                .map(|s| s.parse::<usize>())
                                .collect::<Result<HashSet<usize>, ParseIntError>>()
                                .map_err(|_| Error::custom("failed parsing string as usize"))?;
                            removed_nodes = Some(res);
                        }
                        Field::Max | Field::Min | Field::Root | Field::SingleSplay => {}
                    }
                }

                let nodes = nodes.ok_or_else(|| Error::missing_field("nodes"))?;
                let removed_nodes =
                    removed_nodes.ok_or_else(|| Error::missing_field("removed_nodes"))?;
                let nodes = nodes
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !removed_nodes.contains(i))
                    .map(|(_, n)| n.clone())
                    .collect();
                Ok(OrderBookSide { levels: nodes })
            }
        }

        const FIELDS: &[&str] = &["levels"];
        deserializer.deserialize_struct("OrderBook", FIELDS, OrderBookSideVisitor)
    }
}

#[derive(Debug)]
pub struct OrderBook {
    pub id: Id,
    pub instrument: Instrument,
    pub bids: BTreeMap<u64, Vec<Order>>,
    pub asks: BTreeMap<u64, Vec<Order>>,
    pub type_tags: Vec<TypeTag>,
}

impl<'de> Deserialize<'de> for OrderBook {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Id,
            Instrument,
            Bids,
            Asks,
            Orders,
            SignerAddr,
        }

        struct OrderBookVisitor;

        impl<'de> Visitor<'de> for OrderBookVisitor {
            type Value = OrderBook;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct OrderBook")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id = None;
                let mut instrument = None;
                let mut bids = None;
                let mut asks = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value::<Id>()?);
                        }
                        Field::Instrument => {
                            if instrument.is_some() {
                                return Err(Error::duplicate_field("instrument"));
                            }
                            instrument = Some(map.next_value::<Instrument>()?);
                        }
                        Field::Bids => {
                            if bids.is_some() {
                                return Err(Error::duplicate_field("bids"));
                            }

                            let book_side = map.next_value::<OrderBookSide>()?;
                            let mut res = BTreeMap::<u64, Vec<Order>>::new();
                            for level in &book_side.levels {
                                res.insert(level.price, level.orders.clone());
                            }
                            bids = Some(res);
                        }
                        Field::Asks => {
                            if asks.is_some() {
                                return Err(Error::duplicate_field("asks"));
                            }

                            let book_side = map.next_value::<OrderBookSide>()?;
                            let mut res = BTreeMap::<u64, Vec<Order>>::new();
                            for level in &book_side.levels {
                                res.insert(level.price, level.orders.clone());
                            }
                            asks = Some(res);
                        }
                        Field::Orders | Field::SignerAddr => {}
                    }
                }

                let id = id.ok_or_else(|| Error::missing_field("id"))?;
                let instrument = instrument.ok_or_else(|| Error::missing_field("instrument"))?;
                if bids.is_none() && asks.is_none() {
                    return Err(Error::custom("either asks or bids needs to be provided"));
                }

                let bids = bids.unwrap_or_default();
                let asks = asks.unwrap_or_default();
                Ok(OrderBook {
                    id,
                    instrument,
                    bids,
                    asks,
                    type_tags: vec![],
                })
            }
        }

        const FIELDS: &[&str] = &["id", "instrument", "bids", "asks"];
        deserializer.deserialize_struct("OrderBook", FIELDS, OrderBookVisitor)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    //
    // #[test]
    // fn test_test_test() {
    //     let num = 42u64.to_le_bytes();
    //     let addr = AccountAddress::random();
    //     println!("ADDR IS {:?}", addr);
    //     let addr = addr.into_bytes();
    //     let mut bytes: Vec<u8> = vec![];
    //     bytes.extend_from_slice(&num);
    //     bytes.extend_from_slice(&addr);
    //     let mut u = Unstructured::new(&bytes);
    //     let y = TestId::arbitrary(&mut u).unwrap();
    //     println!("{:?}", y);
    // }
}
