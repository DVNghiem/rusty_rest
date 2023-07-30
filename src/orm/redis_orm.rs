// use crate::errors::{Error, RedisErr::*};
// use redis::{aio, AsyncCommands};
// use serde_json::{self, Value};
// use tokio;

// trait RedisORMInterface {
//     fn get_key(&self, filter: Value) -> Result<String, Error>;
//     fn set_json(&mut self, data: &Value, ttl: u16) -> Result<(), Error>;
//     fn get_json(&mut self, filter: Value) -> Result<Value, Error>;
//     fn del_json(&mut self, filter: Value) -> Result<(), Error>;
// }
// pub struct RedisORM {
//     connection: aio::Connection,
// }

// impl RedisORMInterface for RedisORM {
//     fn get_key(&self, filter: Value) -> Result<String, Error> {
//         let json: serde_json::Value = serde_json::from_value(filter).unwrap();
//         let keys: String = match json.as_object() {
//             Some(map_data) => {
//                 let mut f_keys: Vec<&String> = map_data.keys().collect();
//                 f_keys.sort();
//                 let mut items: Vec<String> = Vec::new();
//                 for k in f_keys {
//                     let v: &Value = map_data.get(k).unwrap();
//                     items.push(format!("{}.{}", k, v))
//                 }
//                 items.join(":")
//             }
//             None => "".to_string(),
//         };
//         if keys.len() == 0 {
//             return Err(Error::RedisError(RedisKeyErr));
//         }
//         Ok(keys)
//     }

//     fn set_json(&mut self, data: &Value, ttl: u16) -> Result<(), Error> {
//         let key = self.get_key(data.clone()).unwrap();
//         tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(
//                 self.connection
//                     .set_ex(key, data.to_string().as_str(), ttl.into()),
//             )
//             .map_err(RedisErr)?;
//         Ok(())
//     }

//     fn get_json(&mut self, filter: Value) -> Result<Value, Error> {
//         let key = self.get_key(filter).unwrap();
//         let data: Result<String, crate::errors::RedisErr> = tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(self.connection.get(key))
//             .map_err(RedisErr);
//         let json_v = serde_json::from_str(data.unwrap().as_str());
//         Ok(json_v.unwrap())
//     }

//     fn del_json(&mut self, filter: Value) -> Result<(), Error> {
//         let key = self.get_key(filter).unwrap();
//         tokio::runtime::Runtime::new()
//             .unwrap()
//             .block_on(self.connection.del(key))
//             .map_err(RedisErr)?;
//         Ok(())
//     }
// }
