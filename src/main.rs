use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::{get, get_service, post},
    AddExtensionLayer, Json, Router, Server,
};
use bazaar::{AnyError, Symbol, apis::Session};
use chrono::{Date, DateTime, Utc, Duration};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::Serialize;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::try_join;
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting Muhtasib!");

    let pool = PgPoolOptions::new()
        .connect(&env::var("DATABASE_URL").unwrap())
        .await?;

    let app = Router::new()
        .route("/info", get(get_info))
        .route("/sessions", get(get_sessions))
        .route("/sessions/:id/info", get(get_session_info))
        .route("/sessions/:id/more", get(get_session_extended_info))
        .route("/sessions/:id/equity/:start", get(get_equity))
        .route("/sessions/:id/orders/:start", get(get_orders))
        /*
        .route(
            "/",
            get_service(ServeDir::new("./frontend/build")).handle_error(
                |error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                },
            ),
        )
        */
        .layer(AddExtensionLayer::new(pool))
        // TODO: Change CORS settings.
        .layer(CorsLayer::permissive());

    Server::bind(&"127.0.0.1:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct GetInfo {}

pub type GetSessionInfos = Vec<SessionInfo>;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    name: String,
    exchange: String,
    live_trading: bool,
    session_id: Uuid,
    create_time: DateTime<Utc>,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionExtendedInfo {
    info: SessionInfo,
    annual_rate_of_return: f64,
    operating_margin: f64,
    annual_turnover: f64,
    daily_rate_of_returns: Vec<f64>,
    avg_daily_rate_of_return: f64,
    stdev_daily_rate_of_return: f64,
}
/*
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    info: GetSessionInfo,
    equities: Equities,
    orders: Orders,

}
*/

pub type Equities = Vec<Equity>;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Equity {
    total: Decimal,
    time: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

pub type Orders = Vec<Order>;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    order_id: Uuid,
    market: String,
    side: Side,
    ordered_size: Decimal,
    ordered_price: Decimal,
    ordered_time: DateTime<Utc>,
    executed_size: Option<Decimal>,
    executed_price: Option<Decimal>,
    executed_time: Option<DateTime<Utc>>,
}

async fn get_info(Extension(pool): Extension<PgPool>) -> Json<Value> {
    /*
    sqlx::query("
            SELECT
        ")
        .bind(self.id().0)
        .bind(self.exit_time())
        .bind(self.exit_price())
        .execute(pool)
        .await?;
    */
    Json(serde_json::to_value(()).unwrap())
}

async fn get_sessions(Extension(pool): Extension<PgPool>) -> Json<Value> {
    let sessions: GetSessionInfos = sqlx::query_as(
        "
            SELECT session_id, name, exchange, live_trading, create_time
            FROM sessions
            ORDER BY create_time DESC
        ",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    Json(serde_json::to_value(sessions).unwrap())
}

async fn get_session_extended_info(Extension(pool): Extension<PgPool>, Path(id): Path<Uuid>) -> Json<Value> {

    let (info, equities, orders): (SessionInfo, Equities, Orders) = try_join!(
        sqlx::query_as("
                SELECT
                    session_id,
                    name,
                    exchange,
                    live_trading,
                    create_time
                FROM sessions
                WHERE session_id = $1
            ")
            .bind(id)
            .fetch_one(&pool),
        sqlx::query_as("
                SELECT
                    total,
                    time
                FROM equities
                WHERE session_id = $1
                ORDER BY time ASC
            ")
            .bind(id)
            .fetch_all(&pool),
        sqlx::query_as("
                SELECT
                    order_id,
                    market,
                    side,
                    ordered_size,
                    ordered_price,
                    ordered_time,
                    executed_size,
                    executed_price,
                    executed_time
                FROM orders
                WHERE session_id = $1
                ORDER BY ordered_time ASC
            ")
            .bind(id)
            .fetch_all(&pool),
    ).unwrap();

    let start = equities.first().unwrap();
    let end = equities.last().unwrap();

    let start_time = start.time;
    let end_time = end.time;
    let duration = end_time - start_time;
    let num_years = duration.num_days() as f64 / 365_f64;

    let start_equity = start.total.to_f64().unwrap();
    let end_equity = end.total.to_f64().unwrap();
    let returns = end_equity - start_equity;
    let relative_returns = returns / start_equity;

    let annual_rate_of_return = relative_returns / num_years;

    let turnover = orders
        .iter()
        .filter_map(|order| Some(order.executed_price? * order.executed_size?))
        .sum::<Decimal>()
        .to_f64().unwrap();
    let annual_turnover = turnover / num_years;

    let operating_margin = returns / turnover;

    let mut day_start_time = start_time;
    let mut day_start_equity = start_equity;
    let mut daily_rate_of_returns = Vec::new();
    for equity in &equities {
        if equity.time >= day_start_time + Duration::days(1) {
            let equity = equity.total.to_f64().unwrap();
            let daily_rate_of_return = (equity - day_start_equity) / day_start_equity;
            if daily_rate_of_return != 0.0 {
                daily_rate_of_returns.push(daily_rate_of_return);
            }
            day_start_time = day_start_time + Duration::days(1);
            day_start_equity = equity;
        }
    }

    let avg_daily_rate_of_return = daily_rate_of_returns.iter().sum::<f64>() / daily_rate_of_returns.len() as f64;
    let stdev_daily_rate_of_return = (daily_rate_of_returns.iter().map(|r| (r - avg_daily_rate_of_return).powi(2)).sum::<f64>() / daily_rate_of_returns.len() as f64).sqrt();

    Json(serde_json::to_value(SessionExtendedInfo {
        info,
        operating_margin,
        annual_turnover,
        annual_rate_of_return,
        daily_rate_of_returns,
        avg_daily_rate_of_return,
        stdev_daily_rate_of_return
    }).unwrap())
}

async fn get_session_info(Extension(pool): Extension<PgPool>, Path(id): Path<Uuid>) -> Json<Value> {
    let info: SessionInfo = sqlx::query_as(
        "
                SELECT
                    session_id,
                    name,
                    exchange,
                    live_trading,
                    create_time
                FROM sessions
                WHERE session_id = $1
            ",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();

    Json(serde_json::to_value(info).unwrap())
}

async fn get_equity(
    Extension(pool): Extension<PgPool>,
    Path((id, start)): Path<(Uuid, u32)>,
) -> Json<Value> {
    let equities: Equities = sqlx::query_as(
        "
                SELECT
                    total,
                    time
                FROM equities
                WHERE session_id = $1
                ORDER BY time ASC
                OFFSET $2
            ",
    )
    .bind(id)
    .bind(start)
    .fetch_all(&pool)
    .await
    .unwrap();

    Json(serde_json::to_value(equities).unwrap())
}

async fn get_orders(
    Extension(pool): Extension<PgPool>,
    Path((id, start)): Path<(Uuid, u32)>,
) -> Json<Value> {
    let orders: Orders = sqlx::query_as(
        "
                SELECT
                    order_id,
                    market,
                    side,
                    ordered_size,
                    ordered_price,
                    ordered_time,
                    executed_size,
                    executed_price,
                    executed_time
                FROM orders
                WHERE session_id = $1
                ORDER BY ordered_time ASC
                OFFSET $2
            ",
    )
    .bind(id)
    .bind(start)
    .fetch_all(&pool)
    .await
    .unwrap();

    Json(serde_json::to_value(orders).unwrap())
}
