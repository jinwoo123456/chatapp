//! `SeaORM` Entity for friends table

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Hash, Serialize, Deserialize)]
#[sea_orm(table_name = "friends")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,      // 내 user id
    pub friend_id: i32,    // 친구 user id
    pub friend_name: String,
    pub friend_avatar: String,
    pub friend_status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
