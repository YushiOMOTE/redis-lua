use redis::{RedisWrite, ToRedisArgs};
use rmp::encode;
use serde::{de, ser, Serialize};
use std::{
    fmt::{self, Display},
    io::{self, Write},
};

trait RedisArgWrite: RedisWrite {
    fn pack(&mut self);
}

#[doc(hidden)]
pub struct ScriptArg {
    buf: Vec<u8>,
    pack: bool,
}

impl ScriptArg {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(128),
            pack: false,
        }
    }

    pub fn pack(&self) -> bool {
        self.pack
    }
}

impl RedisWrite for ScriptArg {
    fn write_arg(&mut self, arg: &[u8]) {
        self.buf.extend(arg);
    }
}

impl RedisArgWrite for ScriptArg {
    fn pack(&mut self) {
        self.pack = true;
    }
}

pub fn script_arg<T: Serialize + ?Sized>(value: &T) -> ScriptArg {
    let mut arg = ScriptArg::new();
    let mut ser = Serializer::new(&mut arg);
    value.serialize(&mut ser).expect("Couldn't serialize");
    arg
}

impl ToRedisArgs for ScriptArg {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: RedisWrite,
    {
        self.buf.write_redis_args(out);
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Error(String);

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<rmp::encode::Error> for Error {
    fn from(e: rmp::encode::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<rmp::encode::ValueWriteError> for Error {
    fn from(e: rmp::encode::ValueWriteError) -> Self {
        Self(e.to_string())
    }
}

struct Serializer<'a, W: ?Sized>(&'a mut W);

impl<'a, W> Serializer<'a, W>
where
    W: RedisArgWrite + ?Sized,
{
    fn new(w: &'a mut W) -> Self {
        Self(w)
    }

    fn write_null(&mut self) {
        Vec::<u8>::new().write_redis_args(self.0);
    }
}

impl<'a, 'b, W> ser::Serializer for &'a mut Serializer<'b, W>
where
    W: RedisArgWrite + ?Sized,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<Arg<'a, W>, Seq>;
    type SerializeTuple = Compound<Arg<'a, W>, Seq>;
    type SerializeTupleStruct = Compound<Arg<'a, W>, Seq>;
    type SerializeTupleVariant = Compound<Arg<'a, W>, Seq>;
    type SerializeMap = Compound<Arg<'a, W>, Map>;
    type SerializeStruct = Compound<Arg<'a, W>, Map>;
    type SerializeStructVariant = Compound<Arg<'a, W>, Map>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0; 4];
        let len = v.encode_utf8(&mut buf).len();
        Ok((&buf[..len]).write_redis_args(self.0))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        Ok(v.write_redis_args(self.0))
    }

    fn serialize_none(self) -> Result<()> {
        Ok(Vec::<u8>::new().write_redis_args(self.0))
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(self.write_null())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(self.write_null())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Compound::new(Arg(self.0), Seq::new()))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Compound::new(Arg(self.0), Map::new()))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_map(Some(len))
    }
}

/// Complex serializer
struct ComplexSerializer<W>(W, Option<u8>);

impl<W> ComplexSerializer<W>
where
    W: io::Write,
{
    fn new(w: W) -> Self {
        Self(w, None)
    }

    /// Write a binary-safe string
    fn write_str(&mut self, s: &[u8]) -> Result<()> {
        encode::write_str_len(&mut self.0, s.len() as u32)?;
        Ok(self.0.write_all(s)?)
    }

    fn single_byte(&self) -> Option<u8> {
        self.1
    }
}

impl<'a, W> ser::Serializer for &'a mut ComplexSerializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<Buf<'a, W>, Seq>;
    type SerializeTuple = Compound<Buf<'a, W>, Seq>;
    type SerializeTupleStruct = Compound<Buf<'a, W>, Seq>;
    type SerializeTupleVariant = Compound<Buf<'a, W>, Seq>;
    type SerializeMap = Compound<Buf<'a, W>, Map>;
    type SerializeStruct = Compound<Buf<'a, W>, Map>;
    type SerializeStructVariant = Compound<Buf<'a, W>, Map>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        Ok(encode::write_bool(&mut self.0, v)?)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        encode::write_sint(&mut self.0, v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.1 = Some(v);
        encode::write_uint(&mut self.0, v as u64)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        encode::write_uint(&mut self.0, v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        encode::write_f32(&mut self.0, v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        encode::write_f64(&mut self.0, v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_str(v.as_bytes())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_str(v)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        encode::write_nil(&mut self.0)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        encode::write_array_len(&mut self.0, 0)?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        variant_index.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(Compound::new(Buf(&mut self.0), Seq::new()))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(Compound::new(Buf(&mut self.0), Map::new()))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_map(Some(len))
    }
}

trait CompoundWrite {
    fn write_raw(&mut self, buf: &[u8]) -> Result<()>;

    fn write_packed(&mut self, buf: &[u8]) -> Result<()>;

    fn is_arg(&self) -> bool;
}

struct Arg<'a, W: ?Sized>(&'a mut W);

struct Buf<'a, W>(&'a mut W);

impl<'a, W> CompoundWrite for Arg<'a, W>
where
    W: RedisArgWrite + ?Sized,
{
    fn write_raw(&mut self, buf: &[u8]) -> Result<()> {
        buf.write_redis_args(self.0);
        Ok(())
    }

    fn write_packed(&mut self, buf: &[u8]) -> Result<()> {
        self.0.pack();
        buf.write_redis_args(self.0);
        Ok(())
    }

    fn is_arg(&self) -> bool {
        true
    }
}

impl<'a, W> CompoundWrite for Buf<'a, W>
where
    W: io::Write,
{
    fn write_raw(&mut self, buf: &[u8]) -> Result<()> {
        self.0.write_all(buf)?;
        Ok(())
    }

    fn write_packed(&mut self, buf: &[u8]) -> Result<()> {
        self.write_raw(buf)
    }

    fn is_arg(&self) -> bool {
        false
    }
}

trait CompoundType {
    fn is_map() -> bool;

    fn add_byte(&mut self, _byte: Option<u8>) {}

    fn bytearray(&self) -> Option<&[u8]> {
        None
    }
}

struct Map;

impl Map {
    fn new() -> Self {
        Self
    }
}

struct Seq {
    bytearray: Vec<u8>,
    is_bytearray: bool,
}

impl Seq {
    fn new() -> Self {
        Self {
            bytearray: vec![],
            is_bytearray: true,
        }
    }
}

impl CompoundType for Map {
    fn is_map() -> bool {
        true
    }
}

impl CompoundType for Seq {
    fn is_map() -> bool {
        false
    }

    fn add_byte(&mut self, byte: Option<u8>) {
        self.is_bytearray &= byte.is_some();
        if let Some(byte) = byte {
            self.bytearray.push(byte);
        }
    }

    fn bytearray(&self) -> Option<&[u8]> {
        if self.is_bytearray {
            Some(&self.bytearray)
        } else {
            None
        }
    }
}

struct Compound<W, C> {
    buf: Vec<u8>,
    len: usize,
    wr: W,
    inner: C,
}

impl<W, C> Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    fn new(wr: W, inner: C) -> Self {
        Self {
            buf: vec![],
            len: 0,
            wr,
            inner,
        }
    }

    fn add<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.len += 1;

        self.inner.add_byte({
            let mut ser = ComplexSerializer::new(&mut self.buf);
            value.serialize(&mut ser)?;
            ser.single_byte()
        });

        Ok(())
    }

    fn end(mut self) -> Result<()> {
        let mut v = Vec::new();

        if C::is_map() {
            // Here divide the map length by 2
            // because `add` is called twice per a key/value pair
            encode::write_map_len(&mut v, self.len as u32 / 2)?;
            v.write_all(&self.buf)?;
            self.wr.write_packed(&v)?;
        } else {
            if let Some(bytearray) = self.inner.bytearray() {
                if self.len > 0 {
                    // Non-empty u8 sequence becomes a string
                    if self.wr.is_arg() {
                        self.wr.write_raw(bytearray)?;
                    } else {
                        encode::write_str_len(&mut v, self.len as u32)?;
                        v.write_all(bytearray)?;
                        self.wr.write_packed(&v)?;
                    }
                    return Ok(());
                }
            }
            encode::write_array_len(&mut v, self.len as u32)?;
            v.write_all(&self.buf)?;
            self.wr.write_packed(&v)?;
        }

        Ok(())
    }
}

impl<W, C> ser::SerializeSeq for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end()
    }
}

impl<W, C> ser::SerializeTuple for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<W, C> ser::SerializeTupleStruct for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<W, C> ser::SerializeTupleVariant for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<W, C> ser::SerializeMap for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<W, C> ser::SerializeStruct for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(key)?;
        self.add(value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}

impl<W, C> ser::SerializeStructVariant for Compound<W, C>
where
    W: CompoundWrite,
    C: CompoundType,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add(key)?;
        self.add(value)
    }

    fn end(self) -> Result<()> {
        self.end()
    }
}
