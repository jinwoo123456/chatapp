use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::{Set, NotSet}, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection,
    EntityTrait, FromQueryResult, QueryFilter, QuerySelect, PaginatorTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::entities::{
    chat::{Column as ChatCol, Entity as ChatEntity},
    room::{ActiveModel, Entity as RoomEntity, Model},
    room_read,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewRoom {
    pub id: Option<i32>,
    pub participants: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomWithUnread {
    pub id: i32,
    pub participants: Vec<String>,
    pub unread_count: i64,
}

#[derive(Debug, FromQueryResult, Serialize)]
pub struct LastRead {
    pub last_read_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct ReadUpdate {
    pub username: String,
    pub last_read_id: Option<i32>,
}

pub async fn create_room(
    State(db): State<DatabaseConnection>,
    Json(new_room): Json<NewRoom>,
) -> Result<Json<Model>, StatusCode> {
    let mut parts = new_room.participants.clone();
    parts.sort();
    parts.dedup();
    
    // 1:1 채팅만 허용 (정확히 2명)
    if parts.len() != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let participants = serde_json::to_string(&parts).unwrap();
    
    let room = ActiveModel {
        participants: Set(participants),
        ..Default::default()
    };

    match room.insert(&db).await {
        Ok(model) => Ok(Json(model)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn post_room(
    State(db): State<DatabaseConnection>,
    Json(new_room): Json<NewRoom>,
) -> Result<Json<Model>, StatusCode> {
    create_room(State(db), Json(new_room)).await
}

pub async fn find_or_create_room(
    State(db): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Result<Json<Model>, StatusCode> {
    let mut parts = room.participants.clone();
    parts.sort();
    parts.dedup();
    
    // 1:1 채팅만 허용 (정확히 2명)
    if parts.len() != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let key = serde_json::to_string(&parts).unwrap();
    
    // Try to find existing room with same participants
    if let Ok(Some(existing)) = RoomEntity::find()
        .filter(crate::entities::room::Column::Participants.eq(key.clone()))
        .one(&db)
        .await 
    {
        return Ok(Json(existing));
    }
    
    // Create new room if not found
    let room = ActiveModel {
        participants: Set(key),
        ..Default::default()
    };
    
    match room.insert(&db).await {
        Ok(model) => Ok(Json(model)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn put_room(
    State(db): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Result<Json<Model>, StatusCode> {
    if let Some(id) = room.id {
        update_room(Path(id), State(db), Json(room)).await
    } else {
        create_room(State(db), Json(room)).await
    }
}

pub async fn get_rooms(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<Model>>, StatusCode> {
    match RoomEntity::find().all(&db).await {
        Ok(rooms) => Ok(Json(rooms)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_room(
    State(db): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<NewRoom>>, StatusCode> {
    let mut condition = sea_orm::Condition::all();
    if let Some(id) = params.get("id") {
        if let Ok(id) = id.parse::<i32>() {
            condition = condition.add(crate::entities::room::Column::Id.eq(id));
        }
    }

    let rooms = match RoomEntity::find().filter(condition).all(&db).await {
        Ok(rooms) => rooms,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    let mut resp = Vec::new();
    for room in rooms {
        let participants: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        resp.push(NewRoom { 
            id: Some(room.id), 
            participants 
        });
    }
    
    Ok(Json(resp))
}

pub async fn get_room_by_id(
    Path(id): Path<i32>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Model>, StatusCode> {
    match RoomEntity::find_by_id(id).one(&db).await {
        Ok(Some(room)) => Ok(Json(room)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_room(
    Path(id): Path<i32>,
    State(db): State<DatabaseConnection>,
    Json(room_data): Json<NewRoom>,
) -> Result<Json<Model>, StatusCode> {
    let room = match RoomEntity::find_by_id(id).one(&db).await {
        Ok(Some(room)) => room,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut parts = room_data.participants.clone();
    parts.sort();
    parts.dedup();
    let participants = serde_json::to_string(&parts).unwrap();

    let mut room: ActiveModel = room.into();
    room.participants = ActiveValue::Set(participants);

    match room.update(&db).await {
        Ok(model) => Ok(Json(model)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_room(
    State(db): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<&'static str>, StatusCode> {
    if let Some(id) = params.get("id") {
        if let Ok(id) = id.parse::<i32>() {
            match RoomEntity::delete_by_id(id).exec(&db).await {
                Ok(_) => Ok(Json("Room deleted successfully")),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn delete_room_by_id(
    Path(id): Path<i32>,
    State(db): State<DatabaseConnection>,
) -> Result<StatusCode, StatusCode> {
    match RoomEntity::delete_by_id(id).exec(&db).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_rooms_with_unread(
    Query(params): Query<HashMap<String, String>>,
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<RoomWithUnread>>, StatusCode> {
    let username = params.get("username").cloned().unwrap_or_default();

    // 사용자명이 없으면 방을 반환하지 않음(새 계정 초기 상태 보호)
    if username.trim().is_empty() {
        return Ok(Json(vec![]));
    }
    
    let rooms = match RoomEntity::find().all(&db).await {
        Ok(rooms) => rooms,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut rooms_with_unread = Vec::new();

    for room in rooms {
        // 참가자 목록 파싱 및 현재 사용자 포함 여부 필터링
        let participants: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        if !participants.iter().any(|p| p == &username) {
            continue;
        }
        // Get the last read message ID for this user in this room
        let last_read_result = room_read::Entity::find()
            .filter(room_read::Column::RoomId.eq(room.id))
            .filter(room_read::Column::Username.eq(&username))
            .one(&db)
            .await;

        let unread_count = match last_read_result {
            Ok(Some(last_read)) => {
                if let Some(lid) = last_read.last_read_id {
                    // Count messages after the last read ID
                    let query = ChatEntity::find()
                        .filter(ChatCol::RoomId.eq(room.id))
                        .filter(ChatCol::Id.gt(lid));
                    
                    query.count(&db).await.unwrap_or(0) as i64
                } else {
                    // No last read ID, count all messages
                    let query = ChatEntity::find().filter(ChatCol::RoomId.eq(room.id));
                    query.count(&db).await.unwrap_or(0) as i64
                }
            }
            Ok(None) => {
                // No record for this user in this room, count all messages
                let query = ChatEntity::find().filter(ChatCol::RoomId.eq(room.id));
                query.count(&db).await.unwrap_or(0) as i64
            }
            Err(_) => {
                // Error querying, assume 0 unread
                0
            }
        };

        rooms_with_unread.push(RoomWithUnread {
            id: room.id,
            participants,
            unread_count,
        });
    }

    Ok(Json(rooms_with_unread))
}

pub async fn mark_read(
    State(db): State<DatabaseConnection>,
    Path(room_id): Path<i32>,
    Json(read_data): Json<ReadUpdate>,
) -> Result<Json<LastRead>, (StatusCode, String)> {
    println!("Received mark_read request for room: {}, user: {}, last_read_id: {:?}", 
             room_id, read_data.username, read_data.last_read_id);

    // 먼저 room이 존재하는지 확인
    let room_exists = RoomEntity::find_by_id(room_id)
        .one(&db)
        .await
        .map_err(|e| {
            println!("Error checking room existence: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
        })?;

    if room_exists.is_none() {
        println!("Room {} not found", room_id);
        return Err((StatusCode::NOT_FOUND, "Room not found".to_string()));
    }

    // 기존 record가 있는지 확인
    match room_read::Entity::find()
        .filter(room_read::Column::RoomId.eq(room_id))
        .filter(room_read::Column::Username.eq(&read_data.username))
        .one(&db)
        .await
    {
        Ok(Some(existing)) => {
            println!("Updating existing room_read record with id: {}", existing.id);
            // 기존 record 업데이트
            let mut active_model: room_read::ActiveModel = existing.into();
            active_model.last_read_id = Set(read_data.last_read_id);
            active_model.updated_at = Set(chrono::Utc::now());
            
            active_model.update(&db).await
                .map_err(|e| {
                    println!("Error updating room_read record: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {}", e))
                })?;
        }
        Ok(None) => {
            println!("Creating new room_read record");
            // 새 record 생성
            let new_record = room_read::ActiveModel {
                id: NotSet,
                room_id: Set(room_id),
                username: Set(read_data.username.clone()),
                last_read_id: Set(read_data.last_read_id),
                updated_at: Set(chrono::Utc::now()),
            };
            
            new_record.insert(&db).await
                .map_err(|e| {
                    println!("Error inserting room_read record: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {}", e))
                })?;
        }
        Err(e) => {
            println!("Database query error: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)));
        }
    };

    println!("Successfully updated room_read for room: {}, user: {}", room_id, read_data.username);
    Ok(Json(LastRead {
        last_read_id: read_data.last_read_id,
    }))
}
