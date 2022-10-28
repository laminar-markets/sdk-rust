#[cfg(feature = "fuzzing")]
pub mod arbitrary;
pub mod events;
pub mod order;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serializer};
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::str::FromStr;

pub fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr,
    D: Deserializer<'de>,
{
    struct StringVisitor<V> {
        phantom: PhantomData<V>,
    }

    impl<'de, V> Visitor<'de> for StringVisitor<V>
    where
        V: Deserialize<'de> + FromStr,
    {
        type Value = V;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a FromStr Deserializable type as a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            v.parse::<V>()
                .map_err(|_| Error::custom("string is not a valid"))
        }
    }

    deserializer.deserialize_any(StringVisitor::<T> {
        phantom: PhantomData::default(),
    })
}

pub fn u64_to_str<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn u64_option_to_str<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(v) => serializer.serialize_str(&v.to_string()),
    }
}
