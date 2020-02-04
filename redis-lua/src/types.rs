use redis::{RedisWrite, ToRedisArgs};
use serde::{ser, Serialize};
use std::io::{self, Write};

pub fn writer<S: Serialize + ?Sized>(arg: &S) -> Writer {
    arg.to_writer()
}

pub trait ToWriter {
    fn to_writer(&self) -> Writer;
}

impl<T> ToWriter for T
where
    T: Serialize,
{
    default fn to_writer(&self) -> Writer {
        Writer::packed(pack(self))
    }
}

impl<'a, T> ToWriter for &'a T
where
    T: Serialize,
{
    fn to_writer(&self) -> Writer {
        (**self).to_writer()
    }
}

macro_rules! impl_types {
    ($($name:ty),*) => {
        $(impl ToWriter for $name {
            fn to_writer(&self) -> Writer {
                let mut writer = Writer::new();
                self.write_redis_args(&mut writer);
                writer
            }
        })*
    };
}

impl_types! {
    i8, u8, i16, u16, i32, u32, i64, u64, f32, f64, isize, usize, bool,
    String, str, Vec<u8>, [u8],
    [u8; 0],  [u8; 1],  [u8; 2],  [u8; 3],  [u8; 4],  [u8; 5],  [u8; 6],  [u8; 7],  [u8; 8],  [u8; 9],
    [u8; 10], [u8; 11], [u8; 12], [u8; 13], [u8; 14], [u8; 15], [u8; 16], [u8; 17], [u8; 18], [u8; 19],
    [u8; 20], [u8; 21], [u8; 22], [u8; 23], [u8; 24], [u8; 25], [u8; 26], [u8; 27], [u8; 28], [u8; 29],
    [u8; 30], [u8; 31], [u8; 32]
}

fn pack<S: Serialize + ?Sized>(arg: &S) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);
    let mut ser = Serializer::new(&mut buf);
    arg.serialize(&mut ser)
        .expect("Couldn't serialize argument");
    buf
}

pub struct Writer {
    buf: Vec<u8>,
    pack: bool,
}

impl ToRedisArgs for Writer {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        self.buf.write_redis_args(out);
    }
}

impl Writer {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(128),
            pack: false,
        }
    }

    fn packed(buf: Vec<u8>) -> Self {
        Self { buf, pack: true }
    }

    pub fn pack(&self) -> bool {
        self.pack
    }
}

impl RedisWrite for Writer {
    fn write_arg(&mut self, arg: &[u8]) {
        self.buf.extend(arg);
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

type RmpSerializer<W> = rmp_serde::encode::Serializer<
    W,
    rmp_serde::config::StructMapConfig<rmp_serde::config::DefaultConfig>,
>;
type Result<T> = std::result::Result<T, rmp_serde::encode::Error>;

struct Serializer<W>(RmpSerializer<W>);

impl<W> Serializer<W>
where
    W: Write,
{
    fn new(w: W) -> Self {
        let ser = rmp_serde::encode::Serializer::new(w).with_struct_map();
        Self(ser)
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = rmp_serde::encode::Error;

    type SerializeSeq = <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeSeq;
    type SerializeTuple = <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeTuple;
    type SerializeTupleStruct = <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant =
        <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeTupleVariant;
    type SerializeMap = <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeMap;
    type SerializeStruct = <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeStruct;
    type SerializeStructVariant =
        <&'a mut RmpSerializer<W> as ser::Serializer>::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.0.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.0.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.0.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.0.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.0.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.0.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.0.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.0.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.0.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.0.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.0.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.0.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.0.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        rmp_serde::RawRef::from_utf8(v).serialize(self)
    }

    fn serialize_none(self) -> Result<()> {
        self.0.serialize_none()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_some(value)
    }

    fn serialize_unit(self) -> Result<()> {
        self.0.serialize_unit()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.0.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.0.serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.0
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.0.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.0.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.0.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.0
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.0.serialize_map(len)
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.0.serialize_struct(name, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.0
            .serialize_struct_variant(name, variant_index, variant, len)
    }
}
