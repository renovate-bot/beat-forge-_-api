use chrono::{DateTime, Utc};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::{Database, KEY, Key, users::User};

bitflags::bitflags! {
    pub struct Permission: i32 {
        // self permissions
        const VIEW_SELF = 1 << 0;
        const EDIT_SELF = 1 << 1;
        // mod permissions
        const CREATE_MOD = 1 << 2;
        const EDIT_MOD = 1 << 3;
        const APPROVE_MOD = 1 << 4;
        // admin permissions
        const EDIT_OTHER_USERS = 1 << 5;
        const EDIT_OTHER_MODS = 1 << 6;
        const VIEW_OTHER = 1 << 7;
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
            Ok(t) => if t.claims.is_valid() { Some(t) } else { None }
        };

        Some(token.unwrap().claims)
    }

    pub fn encode(&self, key: Key) -> String {
        

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &self,
            &jsonwebtoken::EncodingKey::from_secret(&key.0),
        )
        .unwrap()
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

    pub async fn get_user(&self, db: &DatabaseConnection) -> Option<entity::users::Model> {
        match self {
            Self::Session(s) => {
                let auth = JWTAuth::decode(s.to_string(), *KEY.clone());
                match auth {
                    Some(auth) => {
                        let user = entity::users::Entity::find_by_id(auth.user.id)
                            .one(db)
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
                    .filter(entity::users::Column::ApiKey.eq(sea_orm::prelude::Uuid::from_bytes(*uuid.as_bytes())))
                    .one(db)
                    .await
                    .unwrap()
                    .unwrap();

                Some(user)
            },
            _ => None
        }
    }
}

pub trait HasPermissions {
    fn permissions(&self) -> i32;
}

impl HasPermissions for &User {
    fn permissions(&self) -> i32 {
        self.permissions
    }
}

impl HasPermissions for &entity::users::Model {
    fn permissions(&self) -> i32 {
        self.permissions
    }
}

impl HasPermissions for &mut User {
    fn permissions(&self) -> i32 {
        self.permissions
    }
}

impl HasPermissions for &mut entity::users::Model {
    fn permissions(&self) -> i32 {
        self.permissions
    }
}

impl HasPermissions for User {
    fn permissions(&self) -> i32 {
        self.permissions
    }
}

impl HasPermissions for entity::users::Model {
    fn permissions(&self) -> i32 {
        self.permissions
    }
}

pub async fn validate_permissions<T: HasPermissions>(user: T, required: Permission) -> bool {
    required.bits() & user.permissions() != 0
}