use crate::ctx::Ctx;
use crate::model::{ModelController, Ticket, TicketForCreate};
use crate::{Error, Result};
use axum::extract::{FromRef, Path};
use axum::routing::{delete, get, post};
use axum::Router;
use axum::{extract::State, Json};

#[derive(Clone, FromRef)]
struct AppState {
    mc: ModelController,
    tc: String,
}

pub fn routes(mc: ModelController) -> Router {
    let tc = "test".to_owned();
    let app_state = AppState { mc, tc };
    Router::new()
        .route("/tickets", post(create_ticket).get(list_tickets))
        .route("/tickets/:id", delete(delete_ticket))
        .with_state(app_state)
}

// region:  --- reset handlers
async fn create_ticket(
    State(mc): State<ModelController>, //直接解构
    State(tc): State<String>,
    ctx: Ctx, //Ctx Extracotr
    Json(ticket_fc): Json<TicketForCreate>,
) -> Result<Json<Ticket>> {
    println!("->> {:<12} - create_ticket", "HANDLER");
    let ticket = mc.create_ticket(ctx, ticket_fc).await?;
    Ok(Json(ticket))
}

async fn list_tickets(State(mc): State<ModelController>, ctx: Ctx) -> Result<Json<Vec<Ticket>>> {
    println!("->> {:<12} - list_tickets", "HANDLER");
    let tickets = mc.list_tickets(ctx).await?;
    Ok(Json(tickets))
}

async fn delete_ticket(
    State(mc): State<ModelController>,
    ctx: Ctx,
    Path(id): Path<u64>,
) -> Result<Json<Ticket>> {
    println!("->> {:<12} - delete_ticket", "HANDLER");
    let ticket = mc.delete_ticket(ctx, id).await?;
    Ok(Json(ticket))
}
// endregion
