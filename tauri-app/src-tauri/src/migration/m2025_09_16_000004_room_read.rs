use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("room_read"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("room_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("username")).string().not_null())
                    .col(ColumnDef::new(Alias::new("last_read_id")).integer().null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        // Unique index on (room_id, username)
        manager
            .create_index(
                Index::create()
                    .name("idx_room_read_room_user")
                    .table(Alias::new("room_read"))
                    .col(Alias::new("room_id"))
                    .col(Alias::new("username"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("room_read")).to_owned())
            .await
    }
}
