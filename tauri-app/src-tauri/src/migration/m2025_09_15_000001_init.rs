// migration for users, friends, room, chat
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // users
        manager.create_table(
            Table::create()
                .table("users")
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Alias::new("username")).string().not_null().unique_key())
                .col(ColumnDef::new(Alias::new("password")).string().not_null())
                .to_owned()
        ).await?;

        // friends
        manager.create_table(
            Table::create()
                .table("friends")
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                .col(ColumnDef::new(Alias::new("friend_id")).integer().not_null())
                .col(ColumnDef::new(Alias::new("friend_name")).string().not_null())
                .col(ColumnDef::new(Alias::new("friend_avatar")).string().not_null())
                .col(ColumnDef::new(Alias::new("friend_status")).string().not_null())
                .to_owned()
        ).await?;

        // room
        manager.create_table(
            Table::create()
                .table("room")
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Alias::new("participants")).string().not_null())
                .to_owned()
        ).await?;

        // chat
        manager.create_table(
            Table::create()
                .table("chat")
                .if_not_exists()
                .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Alias::new("timestamp")).timestamp().not_null())
                .col(ColumnDef::new(Alias::new("sender")).string().not_null())
                .col(ColumnDef::new(Alias::new("message")).string().not_null())
                .col(ColumnDef::new(Alias::new("room_id")).integer().not_null())
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("chat").to_owned()).await?;
        manager.drop_table(Table::drop().table("room").to_owned()).await?;
        manager.drop_table(Table::drop().table("friends").to_owned()).await?;
        manager.drop_table(Table::drop().table("users").to_owned()).await?;
        Ok(())
    }
}
