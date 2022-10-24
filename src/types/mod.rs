#[cfg(feature = "fuzzing")]
pub mod arbitrary;
pub mod events;
pub mod order;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::str::FromStr;

pub(crate) fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
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
