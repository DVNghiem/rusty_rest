use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("direct redis error: {0}")]
    RedisError(#[from] RedisErr),
}

#[derive(Error, Debug)]
pub enum RedisErr {
    #[error("Redis key error")]
    RedisKeyErr,
    #[error("Redis error : {0}")]
    RedisErr(redis::RedisError),
}