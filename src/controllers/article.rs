#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sea_orm::{Order, QueryOrder};
// add crates to use
use axum::extract::Query;
use sea_orm::Condition;
use loco_rs::model::query::{PaginationQuery, PageResponse};
use chrono::NaiveDate;


use crate::models::_entities::articles::{ActiveModel, Entity, Model, Column};
use crate::models::articles::Articles;

#[derive(Debug, Deserialize, Serialize)]
pub struct ListResponse {
    pub id: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl From<Model> for ListResponse {
    fn from(article: Model) -> Self {
        Self {
            id: article.id,
            title: article.title.clone(),
            content: article.content.clone(),
            created_at: article.created_at.naive_local(),
            updated_at: article.updated_at.naive_local(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryParams {
    pub title: Option<String>,
    pub content: Option<String>,
    pub created_at_from: Option<String>,        // added here
    pub created_at_to: Option<String>,          // added here
    #[serde(flatten)]                                       // added
    pub pagination_query: PaginationQuery,                  // added
}
// pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
//     format::json(Entity::find().all(&ctx.db).await?)
// }
pub async fn list_inner(
    ctx: &AppContext,
    query_params: &QueryParams,
) -> Result<PageResponse<Model>> {
    let mut condition = Condition::all();
    if let Some(ref title) = query_params.title {
        if !title.is_empty() {
            condition = condition.add(Column::Title.contains(title));
        }
    }
    if let Some(ref content) = query_params.content {
        if !content.is_empty() {
            condition = condition.add(Column::Content.contains(content));
        }
    }
    let created_at_from_filter = query_params.created_at_from.as_ref().unwrap_or(&String::new()).clone();
    let created_at_to_filter = query_params.created_at_to.as_ref().unwrap_or(&String::new()).clone();
    if !created_at_from_filter.is_empty() {
        let parsed = chrono::NaiveDateTime::parse_from_str(&created_at_from_filter, "%Y-%m-%dT%H:%M");
        match parsed {
            Ok(dt) => {
                condition = condition.add(Column::CreatedAt.gte(dt));
            },
            Err(err) => {eprint!("{}", err)},
        }
    }

    if !created_at_to_filter.is_empty() {
        let parsed = chrono::NaiveDateTime::parse_from_str(&created_at_to_filter, "%Y-%m-%dT%H:%M");
        match parsed {
            Ok(dt) => {
                condition = condition.add(Column::CreatedAt.lte(dt));
            },
            Err(err) => {eprint!("{}", err)},
        }
    }
    model::query::paginate(
        &ctx.db, Entity::find(), Some(condition), &query_params.pagination_query,
    ).await
}
#[debug_handler]
pub async fn list2(
    Query(query_params): Query<QueryParams>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let response = list_inner(&ctx, &query_params).await?;
    let items: Vec<ListResponse> = response.page.into_iter().map(ListResponse::from).collect();
    format::json(data!({
        "results": items,
        "pagination": {
            "page": query_params.pagination_query.page,
            "page_size": query_params.pagination_query.page_size,
            "total_pages": response.total_pages,
        }
    }))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchParams {
    pub title: Option<String>,
    pub content: Option<String>,
    //pub order: Option<String>,  // "asc" 또는 "desc"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: Option<String>,
    pub content: Option<String>,
    }

impl Params {
    fn update(&self, item: &mut ActiveModel) {
      item.title = Set(self.title.clone());
      item.content = Set(self.content.clone());
      }
}

pub async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn search(
    State(ctx): State<AppContext>,
    Query(params): Query<SearchParams>,
) -> Result<Response> {
    let items = Articles::search(
        &ctx.db,
        params.title.as_deref(),
        params.content.as_deref(),
    )
    .await?;
    format::json(items)
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

// #[debug_handler]
// pub async fn add(State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
//     let mut item = ActiveModel {
//         ..Default::default()
//     };
//     params.update(&mut item);
//     let item = item.insert(&ctx.db).await?;
//     format::json(item)
// }

// #[debug_handler]
// pub async fn update(
//     Path(id): Path<i32>,
//     State(ctx): State<AppContext>,
//     Json(params): Json<Params>,
// ) -> Result<Response> {
//     let item = load_item(&ctx, id).await?;
//     let mut item = item.into_active_model();
//     params.update(&mut item);
//     let item = item.update(&ctx.db).await?;
//     format::json(item)
// }
pub async fn add_inner(ctx: &AppContext, params: Params) -> Result<Model> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await;
    match item {
        Ok(v) => Ok(v),
        Err(err) => core::result::Result::Err(loco_rs::Error::DB(err)),
    }
}

pub async fn add(State(ctx): State<AppContext>, Json(params): Json<Params>) -> Result<Response> {
    let item = add_inner(&ctx, params).await?;
    format::json(item)
}

pub async fn update_inner(id: i32, ctx: &AppContext, params: Params) -> Result<Model> {
    let item: Model = load_item(&ctx, id).await?;
    let mut item: ActiveModel = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await;
    match item {
        Ok(v) => Ok(v),
        Err(err) => core::result::Result::Err(loco_rs::Error::DB(err)),
    }
}

pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = update_inner(id, &ctx, params).await?;
    format::json(item)
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        // .prefix("api/articles/")
        // .add("/", get(list))
        // .add("/", post(add))
        // .add("search", get(search))
        // .add("{id}", get(get_one))
        // .add("{id}", delete(remove))
        // .add("{id}", put(update))
        // .add("{id}", patch(update))
        .prefix("api/articles")    // 'api/' added
        .add("/", get(list2))
        .add("/", post(add))
        .add("/{id}", get(get_one))
        .add("/{id}", delete(remove))
        .add("/{id}", post(update))
}
