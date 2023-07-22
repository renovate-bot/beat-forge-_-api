use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTAuth {
    pub user: entity::users::Model,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub(crate) exp: DateTime<Utc>, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    #[serde(with = "chrono::serde::ts_seconds")]
    pub(crate) iat: DateTime<Utc>, // Optional. Issued at (as UTC timestamp)
}

impl JWTAuth {
    pub fn new(user: entity::users::Model) -> Self {
        let now = Utc::now();

        Self {
            user,
            exp: now + chrono::Duration::days(1),
            iat: now,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.exp > Utc::now()
    }

    pub fn decode(dec: String, key: &[u8]) -> Option<Self> {
        let token = match jsonwebtoken::decode::<JWTAuth>(
                    &dec,
                    &jsonwebtoken::DecodingKey::from_secret(key),
                    &jsonwebtoken::Validation::default(),
                ) {
            Err(_) => { return None; },
            Ok(t) => Some(t),
        };

        Some(token.unwrap().claims)
    }

    pub fn encode(&self, key: &[u8]) -> String {
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &self,
            &jsonwebtoken::EncodingKey::from_secret(key),
        )
        .unwrap();

        token
    }
}