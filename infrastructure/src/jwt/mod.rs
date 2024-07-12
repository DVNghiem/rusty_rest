use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    payload: Value,
}

impl Claims {
    pub fn new() -> Self {
        Claims {
            exp: 0,
            iat: 0,
            payload: Value::Null,
        }
    }

    pub fn build(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }
}

pub struct JsonWebToken {
    public_key: String,
    private_key: String,
    algorithm: Algorithm,
}

impl JsonWebToken {
    pub fn new() -> Self {
        JsonWebToken {
            public_key: "".to_owned(),
            private_key: "".to_owned(),
            algorithm: Algorithm::HS256,
        }
    }

    pub fn build(mut self, public_key: &str, private_key: &str, algorithm: Algorithm) -> Self {
        self.public_key = public_key.to_owned();
        self.private_key = private_key.to_owned();
        self.algorithm = algorithm;
        self
    }

    pub fn get_encoding_key(self) -> EncodingKey {
        match self.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                EncodingKey::from_secret(self.private_key.as_ref())
            }
            _ => EncodingKey::from_rsa_der(self.private_key.as_ref()),
        }
    }

    pub fn get_decoding_key(&self) -> DecodingKey {
        match self.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                DecodingKey::from_secret(self.public_key.as_ref())
            }
            _ => DecodingKey::from_rsa_der(self.public_key.as_ref()),
        }
    }

    pub fn encode(self, payload: Value) -> String {
        let claims = Claims::new().build(payload);
        encode::<Claims>(&Header::default(), &claims, &self.get_encoding_key()).unwrap()
    }

    pub fn decode(&self, token: &str) -> Value {
        decode::<Value>(
            token,
            &self.get_decoding_key(),
            &Validation::new(self.algorithm),
        )
        .unwrap()
        .claims
    }
}
