use sea_orm::{entity::prelude::*, Condition, QueryFilter};
pub use super::_entities::articles::{ActiveModel, Column, Entity, Model};
pub type Articles = Entity;

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, insert: bool) -> std::result::Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if !insert && self.updated_at.is_unchanged() {
            let mut this = self;
            this.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().into());
            Ok(this)
        } else {
            Ok(self)
        }
    }
}

// implement your read-oriented logic here
impl Model {}

// implement your write-oriented logic here
impl ActiveModel {}

// implement your custom finders, selectors oriented logic here
impl Entity {
    pub async fn search(
        db: &DatabaseConnection,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Vec<Model>, DbErr> {
        Self::find()
            .filter(
                Condition::all()
                    .add_option(title.map(|t| Column::Title.contains(t)))
                    .add_option(content.map(|c| Column::Content.contains(c))),
            )
            .all(db)
            .await
    }
}
