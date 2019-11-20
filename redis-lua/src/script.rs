use futures::prelude::*;
use redis::{FromRedisValue, RedisError, RedisResult, ScriptInvocation};

pub struct Script<F> {
    script: redis::Script,
    call: F,
}

impl<F> Script<F>
where
    F: FnOnce(ScriptInvocation) -> ScriptInvocation,
{
    pub fn new(script: &str, call: F) -> Self {
        Self {
            script: redis::Script::new(script),
            call,
        }
    }

    pub fn invoke<T: FromRedisValue>(self, con: &mut dyn redis::ConnectionLike) -> RedisResult<T> {
        (self.call)(self.script.prepare_invoke()).invoke(con)
    }

    pub fn invoke_async<C, T>(self, con: C) -> impl Future<Item = (C, T), Error = RedisError>
    where
        C: redis::aio::ConnectionLike + Clone + Send + 'static,
        T: FromRedisValue + Send + 'static,
    {
        (self.call)(self.script.prepare_invoke()).invoke_async(con)
    }
}
