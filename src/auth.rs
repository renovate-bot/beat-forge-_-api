use std::ops::{DerefMut, Deref};

use actix_web_lab::sse::Data;
use chrono::{DateTime, Utc};
use juniper::{GraphQLValue, GraphQLEnum, GraphQLType, ScalarValue, DefaultScalarValue, DynGraphQLValue};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{Database, KEY, Key, users::User};

bitflags::bitflags! {
    pub struct Permission: i32 {
        // self permissions
        const VIEW_SELF = 1 << 0;
        const EDIT_SELF = 1 << 1;
        // mod permissions
        const CREATE_MOD = 1 << 3;
        const EDIT_MOD = 1 << 4;
        const APPROVE_MOD = 1 << 6;
        // admin permissions
        const EDIT_OTHER_USERS = 1 << 7;
        const EDIT_OTHER_MODS = 1 << 8;
        const VIEW_OTHER = 1 << 9;
    }
}

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

    pub fn decode(dec: String, key: Key) -> Option<Self> {
        let token = match jsonwebtoken::decode::<JWTAuth>(
                    &dec,
                    &jsonwebtoken::DecodingKey::from_secret(&key.0),
                    &jsonwebtoken::Validation::default(),
                ) {
            Err(_) => { return None; },
            Ok(t) => Some(t),
        };

        Some(token.unwrap().claims)
    }

    pub fn encode(&self, key: Key) -> String {
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &self,
            &jsonwebtoken::EncodingKey::from_secret(&key.0),
        )
        .unwrap();

        token
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Authorization {
    Session(String),
    ApiKey(Uuid),
    None
}

impl Authorization {
    pub fn parse(s: Option<String>) -> Self {
        match s {
            Some(s) => {
                match Uuid::parse_str(&s) {
                    Ok(uuid) => Self::ApiKey(uuid),
                    Err(_) => Self::Session(s)
                }
            },
            None => Self::None
        }
    }

    pub async fn get_user(&self, db: &Database) -> Option<entity::users::Model> {
        match self {
            Self::Session(s) => {
                let auth = JWTAuth::decode(s.to_string(), *KEY.clone());
                match auth {
                    Some(auth) => {
                        let user = entity::users::Entity::find_by_id(auth.user.id)
                            .one(&db.pool)
                            .await
                            .unwrap()
                            .unwrap();

                        Some(user)
                    },
                    None => None
                }
            },
            Self::ApiKey(uuid) => {
                let user = entity::users::Entity::find()
                    .filter(entity::users::Column::ApiKey.eq(uuid.to_string()))
                    .one(&db.pool)
                    .await
                    .unwrap()
                    .unwrap();

                Some(user)
            },
            _ => None
        }
    }
}

pub async fn validate_permissions(user: &User, required: Permission) -> bool {
    let user_permissions = Permission::from_bits(user.permissions).unwrap();

    user_permissions.contains(required)
}