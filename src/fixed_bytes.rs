use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize, Serializer,
};

pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> Serialize for FixedBytes<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0)
    }
}

impl<U, const N: usize> AsRef<U> for FixedBytes<N>
where
    [u8; N]: AsRef<U>,
{
    fn as_ref(&self) -> &U {
        self.0.as_ref()
    }
}

impl<U, const N: usize> AsMut<U> for FixedBytes<N>
where
    [u8; N]: AsMut<U>,
{
    fn as_mut(&mut self) -> &mut U {
        self.0.as_mut()
    }
}

impl<const N: usize> Deref for FixedBytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for FixedBytes<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct FixedBytesVisitor<'de, const N: usize>(PhantomData<&'de [u8; N]>);

impl<'de, const N: usize> Visitor<'de> for FixedBytesVisitor<'de, N> {
    type Value = FixedBytes<N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a byte array of length {}", N)
    }

    fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
        self.visit_bytes(&v)
    }

    fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        self.visit_bytes(&v)
    }

    fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        if value.len() != N {
            return Err(E::custom(format!(
                "expected a byte array of length {}, but got {}",
                N,
                value.len()
            )));
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(value);
        Ok(FixedBytes(arr))
    }
}

impl<'de, const N: usize> Deserialize<'de> for FixedBytes<N> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_bytes(FixedBytesVisitor(PhantomData))
    }
}
