use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use futures_util::stream::StreamExt;
use serde_json::json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::entities::{
    chat::{ActiveModel as ActiveChat, Column, Entity as ChatEntity, Model as Chat},
    room::{ActiveModel as ActiveRoom, Entity as RoomEntity},
};

use serde::Serialize;

pub async fn subscribe(
    State(queue): State<broadcast::Sender<Chat>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let room_filter = params.get("room_id").and_then(|v| v.parse::<i32>().ok());
    let stream = BroadcastStream::new(queue.subscribe()).filter_map(move |msg| {
        let room_filter = room_filter.clone();
        async move {
            match msg {
                Ok(chat) => {
                    if room_filter.map(|rid| chat.room_id == rid).unwrap_or(true) {
                        Some(Ok(Event::default()
                            .event("message")
                            .data(json!({
                                "id": chat.id,
                                "sender": chat.sender,
                                "message": chat.message,
                                "room_id": chat.room_id,
                                "timestamp": chat.timestamp
                            }).to_string())))
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(e)),
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(serde::Deserialize)]
pub struct NewMessage {
    pub sender: String,
    pub message: String,
    pub room_id: i32,
}

#[derive(Serialize)]
pub struct SendResponse {
    pub success: i32,
    pub error: Option<String>,
    pub chat: Option<Chat>,
}

pub async fn send(
    State(conn): State<DatabaseConnection>,
    State(queue): State<broadcast::Sender<Chat>>,
    Json(new_message): Json<NewMessage>,
) -> Json<SendResponse> {
    // 입력값 검증
    if new_message.sender.trim().is_empty() || new_message.message.trim().is_empty() {
        return Json(SendResponse { success: 0, error: Some("보내는 사람과 메시지를 모두 입력하세요.".to_string()), chat: None });
    }
    if new_message.message.len() > 500 {
        return Json(SendResponse { success: 0, error: Some("메시지는 500자 이내여야 합니다.".to_string()), chat: None });
    }
    // 방 존재 확인
    let room = match RoomEntity::find_by_id(new_message.room_id).one(&conn).await {
        Ok(Some(room)) => room,
        _ => return Json(SendResponse { success: 0, error: Some("존재하지 않는 방입니다.".to_string()), chat: None }),
    };
    // 참가자 목록 업데이트
    let mut participants: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
    if !participants.contains(&new_message.sender) {
        participants.push(new_message.sender.clone());
    }
    let participants = serde_json::to_string(&participants).unwrap();
    let room_update = ActiveRoom {
        id: ActiveValue::set(room.id),
        participants: ActiveValue::set(participants),
    };
    let _ = room_update.update(&conn).await;
    // 메시지 저장
    let chat_model = ActiveChat {
        id: ActiveValue::not_set(),
        sender: ActiveValue::set(new_message.sender.clone()),
        message: ActiveValue::set(new_message.message.clone()),
        room_id: ActiveValue::set(new_message.room_id),
        timestamp: ActiveValue::set(chrono::Utc::now().naive_utc()),
    };
    let chat = match chat_model.insert(&conn).await {
        Ok(chat) => chat,
        Err(_) => return Json(SendResponse { success: 0, error: Some("메시지 저장에 실패했습니다.".to_string()), chat: None }),
    };
    let _ = queue.send(chat.clone());
    Json(SendResponse { success: 1, error: None, chat: Some(chat) })
}

pub async fn get_chat(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<Chat>> {
    let room_id = params.get("room_id").unwrap();

    Json(
        ChatEntity::find()
            .filter(Column::RoomId.eq(room_id.parse::<i32>().unwrap()))
            .all(&conn)
            .await
            .unwrap(),
    )
}
