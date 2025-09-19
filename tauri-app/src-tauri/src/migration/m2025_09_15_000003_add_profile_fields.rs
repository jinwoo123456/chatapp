use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add display_name, status, avatar columns to users if not exist
        // Using separate ALTERs for portability
        // display_name
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .add_column(
                        ColumnDef::new(Alias::new("display_name"))
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await;

        // status
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .add_column(ColumnDef::new(Alias::new("status")).string().null())
                    .to_owned(),
            )
            .await;

        // avatar
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .add_column(ColumnDef::new(Alias::new("avatar")).string().null())
                    .to_owned(),
            )
            .await;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the added columns (best-effort; some DBs support DROP COLUMN)
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("display_name"))
                    .to_owned(),
            )
            .await;
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("status"))
                    .to_owned(),
            )
            .await;
        let _ = manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("users"))
                    .drop_column(Alias::new("avatar"))
                    .to_owned(),
            )
            .await;
        Ok(())
    }
}
