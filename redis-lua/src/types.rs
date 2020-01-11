use redis::{RedisWrite, ToRedisArgs};
use serde::Serialize;

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
        let buf = rmp_serde::to_vec_named(self).expect("Coudln't serialize argument");
        Writer { buf, pack: true }
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

    pub fn pack(&self) -> bool {
        self.pack
    }
}

impl RedisWrite for Writer {
    fn write_arg(&mut self, arg: &[u8]) {
        self.buf.extend(arg);
    }
}
