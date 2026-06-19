use serde::{de, de::Visitor, Deserialize, Deserializer, Serializer};
use std::fmt;

pub fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn serialize_vec_u64_as_string<S>(values: &Vec<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(values.len()))?;
    for value in values {
        seq.serialize_element(&value.to_string())?;
    }
    seq.end()
}

pub fn deserialize_u64_from_string_or_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(U64StringOrNumberVisitor)
}

pub fn deserialize_vec_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(VecU64StringOrNumberVisitor)
}

pub fn serialize_option_vec_u64_as_string<S>(
    values: &Option<Vec<u64>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match values {
        Some(values) => serialize_vec_u64_as_string(values, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_option_vec_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<u64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Vec<U64StringOrNumber>> = Option::deserialize(deserializer)?;
    Ok(value.map(|items| items.into_iter().map(|item| item.0).collect()))
}

pub fn serialize_option_u64_as_string<S>(
    value: &Option<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&value.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_option_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<U64StringOrNumber>::deserialize(deserializer).map(|value| value.map(|value| value.0))
}

struct U64StringOrNumber(u64);

impl<'de> Deserialize<'de> for U64StringOrNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_u64_from_string_or_number(deserializer).map(Self)
    }
}

struct U64StringOrNumberVisitor;

impl<'de> Visitor<'de> for U64StringOrNumberVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u64 encoded as a JSON string or number")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u64::try_from(value).map_err(|_| E::custom("u64 value must not be negative"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .parse::<u64>()
            .map_err(|_| E::custom("u64 string must contain an unsigned integer"))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

struct VecU64StringOrNumberVisitor;

impl<'de> Visitor<'de> for VecU64StringOrNumberVisitor {
    type Value = Vec<u64>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a sequence of u64 values encoded as JSON strings or numbers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<U64StringOrNumber>()? {
            values.push(value.0);
        }
        Ok(values)
    }
}
