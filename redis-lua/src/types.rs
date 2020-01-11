use redis::{RedisWrite, ToRedisArgs};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    hash::{BuildHasher, Hash},
};

pub trait Pack: ToRedisArgs + Serialize {
    fn pack(&self) -> bool {
        false
    }

    fn byte() -> bool {
        false
    }
}

pub struct Packer<T>(T);

impl<T: Pack> Packer<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: Pack> ToRedisArgs for Packer<T> {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        if self.0.pack() {
            let packed = rmp_serde::to_vec(&self.0).expect("Couldn't pack arguments");
            packed.write_redis_args(out);
        } else {
            self.0.write_redis_args(out);
        }
    }
}

macro_rules! impl_simple {
    ($($t:ty,)*) => {
        $(impl Pack for $t {})*
    };
}

macro_rules! impl_ref {
    ($($t:ty,)*) => {
        $(impl<'a> Pack for &'a $t {})*
    };
}

impl Pack for u8 {
    fn pack(&self) -> bool {
        false
    }

    fn byte() -> bool {
        // To specially handle byte arrays (e.g. `Vec<u8>`, `&[u8]`, `&[u8; N]`)
        true
    }
}

impl_simple! {
    i8, i16, u16, i32, u32, i64, u64, f32, f64, isize, usize, bool,
    String,
}

impl_ref! {
    String, str,
}

impl<T: ToRedisArgs + Serialize> Pack for Option<T> {
    fn pack(&self) -> bool {
        true
    }
}

impl<T: Pack> Pack for Vec<T> {
    fn pack(&self) -> bool {
        !T::byte()
    }
}

impl<'a, T: Pack> Pack for &'a [T] {
    fn pack(&self) -> bool {
        !T::byte()
    }
}

impl<T: ToRedisArgs + Hash + Eq + Serialize, S: BuildHasher + Default + Serialize> Pack
    for HashSet<T, S>
{
    fn pack(&self) -> bool {
        true
    }
}

impl<T: ToRedisArgs + Hash + Eq + Ord + Serialize> Pack for BTreeSet<T> {
    fn pack(&self) -> bool {
        true
    }
}

impl<T: ToRedisArgs + Hash + Eq + Ord + Serialize, V: ToRedisArgs + Serialize> Pack
    for BTreeMap<T, V>
{
    fn pack(&self) -> bool {
        true
    }
}

macro_rules! impl_tuple {
    () => ();
    ($($name:ident,)+) => (
        impl<$($name: ToRedisArgs + Serialize),*> Pack for ($($name,)*) {
            fn pack(&self) -> bool {
                true
            }
        }
        tuple_peel!($($name,)*);
    )
}

macro_rules! tuple_peel {
    ($name:ident, $($other:ident,)*) => (impl_tuple!($($other,)*);)
}

impl_tuple! { T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, }

macro_rules! impl_array {
    ($($N:expr)+) => {
        $(
            impl<'a, T: Pack> Pack for &'a [T; $N] {
                fn pack(&self) -> bool {
                    !T::byte()
                }
            }
        )+
    }
}

impl_array! {
    0  1  2  3  4  5  6  7  8  9
   10 11 12 13 14 15 16 17 18 19
   20 21 22 23 24 25 26 27 28 29
   30 31 32
}
