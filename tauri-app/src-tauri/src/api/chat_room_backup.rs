use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    EntityTrait, FromQueryResult, ModelTrait, QueryFilter, QueryOrder, Statement,
};

use crate::entities::{
    chat::{Column as ChatCol, Entity as ChatEntity},
    room::{ActiveModel, Column, Entity as RoomEntity, Model},
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

#[derive(Debug, FromQueryResult)]
struct LastRead {
    last_read_id: Option<i32>,
}

pub async fn get_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<NewRoom>> {
    let mut condition = Condition::all();
    if let Some(id) = params.get("id") {
        if let Ok(id) = id.parse::<i32>() {
            condition = condition.add(Column::Id.eq(id));
        }
    }

    let rooms = RoomEntity::find().filter(condition).all(&conn).await.unwrap();
    let mut resp = Vec::new();
    for room in rooms {
        let participants: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        resp.push(NewRoom { id: Some(room.id), participants });
    }
    Json(resp)
}

pub async fn post_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let mut parts = room.participants.clone();
    parts.sort();
    parts.dedup();
    let participants = serde_json::to_string(&parts).unwrap();
    let room = ActiveModel { id: ActiveValue::not_set(), participants: ActiveValue::Set(participants) };
    Json(room.insert(&conn).await.unwrap())
}

pub async fn find_or_create_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let mut parts = room.participants.clone();
    parts.sort();
    parts.dedup();
    let key = serde_json::to_string(&parts).unwrap();
    if let Ok(Some(existing)) = RoomEntity::find().filter(Column::Participants.eq(key.clone())).one(&conn).await {
        return Json(existing);
    }
    let am = ActiveModel { id: ActiveValue::not_set(), participants: ActiveValue::Set(key) };
    Json(am.insert(&conn).await.unwrap())
}

pub async fn put_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let result = RoomEntity::find_by_id(room.id.unwrap()).one(&conn).await.unwrap().unwrap();
    let mut participants: Vec<String> = serde_json::from_str(&result.participants).unwrap_or_default();
    participants.extend(room.participants.clone());
    participants.sort();
    participants.dedup();
    let new_room = ActiveModel { id: ActiveValue::Set(result.id), participants: ActiveValue::Set(serde_json::to_string(&participants).unwrap()) };
    Json(new_room.update(&conn).await.unwrap())
}

pub async fn delete_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<&'static str> {
    let id = params.get("id").and_then(|v| v.parse::<i32>().ok()).unwrap();
    let chats = ChatEntity::find().filter(ChatCol::RoomId.eq(id)).all(&conn).await.unwrap();
    for chat in chats { chat.delete(&conn).await.unwrap(); }
    if let Some(room) = RoomEntity::find_by_id(id).one(&conn).await.unwrap() {
        room.delete(&conn).await.unwrap();
    }
    Json("Deleted")
}

pub async fn list_rooms_with_unread(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<RoomWithUnread>> {
    let Some(username) = params.get("username").cloned() else { return Json(Vec::new()) };
    let like_token = format!("\"{}\"", username);
    let rooms = RoomEntity::find().filter(Column::Participants.contains(&like_token)).all(&conn).await.unwrap_or_default();
    let mut out = Vec::new();
    for room in rooms {
        let parts: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        // last_read_id via raw SQL (no dedicated entity for room_read)
        let stmt = Statement::from_string(
            conn.get_database_backend(),
            format!(
                "SELECT last_read_id FROM room_read WHERE room_id = {} AND username = '{}'",
                room.id,
                username.replace("'", "''")
            ),
        );
        let last_read_row: Option<LastRead> = LastRead::find_by_statement(stmt).one(&conn).await.ok().flatten();
        let last_read_id = last_read_row.and_then(|r| r.last_read_id);
        // unread: messages by others after last_read_id
        let mut query = ChatEntity::find()
            .filter(ChatCol::RoomId.eq(room.id))
            .filter(ChatCol::Sender.ne(username.clone()));
        if let Some(lid) = last_read_id { query = query.filter(ChatCol::Id.gt(lid)); }
        let unread = query.count(&conn).await.unwrap_or(0) as i64;
        out.push(RoomWithUnread { id: room.id, participants: parts, unread_count: unread });
    }
    Json(out)
}

#[derive(Deserialize)]
pub struct ReadUpdate { pub room_id: i32, pub username: String, pub last_read_id: Option<i32> }

pub async fn mark_read(
    State(conn): State<DatabaseConnection>,
    Json(payload): Json<ReadUpdate>,
) -> Json<&'static str> {
    let mut last_id = payload.last_read_id;
    if last_id.is_none() {
        let last = ChatEntity::find().filter(ChatCol::RoomId.eq(payload.room_id)).order_by_desc(ChatCol::Id).one(&conn).await.ok().flatten();
        last_id = last.map(|m| m.id);
    }
    let escaped_user = payload.username.replace("'", "''");
    let value = match last_id { Some(v) => v.to_string(), None => "NULL".to_string() };
    let upsert_sql = format!(
        "INSERT INTO room_read (room_id, username, last_read_id, updated_at) VALUES ({}, '{}', {}, NOW()) \
        ON CONFLICT (room_id, username) DO UPDATE SET last_read_id = EXCLUDED.last_read_id, updated_at = NOW()",
        payload.room_id, escaped_user, value
    );
    let _ = conn.execute(Statement::from_string(conn.get_database_backend(), upsert_sql)).await;
    Json("OK")
}

pub async fn get_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<NewRoom>> {
    let mut condition = Condition::all();
    if let Some(id) = params.get("id") {
        condition = condition.add(Column::Id.eq(id.parse::<i32>().unwrap()));
    }
    let rooms = RoomEntity::find().filter(condition).all(&conn).await.unwrap();
    let mut resp = Vec::new();
    for room in rooms {
        let participants: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        resp.push(NewRoom { id: Some(room.id), participants });
    }
    Json(resp)
}

pub async fn post_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let mut parts = room.participants.clone();
    parts.sort();
    let participants = serde_json::to_string(&parts).unwrap();
    let room = ActiveModel { id: ActiveValue::not_set(), participants: ActiveValue::Set(participants) };
    Json(room.insert(&conn).await.unwrap())
}

pub async fn find_or_create_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let mut parts = room.participants.clone();
    parts.sort();
    let key = serde_json::to_string(&parts).unwrap();
    if let Ok(Some(existing)) = RoomEntity::find().filter(Column::Participants.eq(key.clone())).one(&conn).await {
        return Json(existing);
    }
    let am = ActiveModel { id: ActiveValue::not_set(), participants: ActiveValue::Set(key) };
    Json(am.insert(&conn).await.unwrap())
}

pub async fn put_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let result = RoomEntity::find_by_id(room.id.unwrap()).one(&conn).await.unwrap().unwrap();
    let mut participants: Vec<String> = serde_json::from_str(&result.participants).unwrap_or_default();
    if let Some(first) = room.participants.get(0) { participants.push(first.clone()); }
    let new_room = ActiveModel { id: ActiveValue::Set(result.id), participants: ActiveValue::Set(serde_json::to_string(&participants).unwrap()) };
    Json(new_room.update(&conn).await.unwrap())
}

pub async fn delete_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<&'static str> {
    let id = params.get("id").unwrap().parse::<i32>().unwrap();
    let chats = ChatEntity::find().filter(ChatCol::RoomId.eq(id)).all(&conn).await.unwrap();
    for chat in chats { chat.delete(&conn).await.unwrap(); }
    let room = RoomEntity::find_by_id(id).one(&conn).await.unwrap().unwrap();
    room.delete(&conn).await.unwrap();
    Json("Deleted")
}

pub async fn list_rooms_with_unread(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<RoomWithUnread>> {
    let Some(username) = params.get("username").cloned() else { return Json(Vec::new()) };
    let like_token = format!("\"{}\"", username);
    let rooms = RoomEntity::find().filter(Column::Participants.contains(&like_token)).all(&conn).await.unwrap_or_default();
    let mut out = Vec::new();
    for room in rooms {
        let parts: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        // last_read_id via raw SQL (no dedicated entity for room_read)
        let stmt = Statement::from_string(
            conn.get_database_backend(),
            format!(
                "SELECT last_read_id FROM room_read WHERE room_id = {} AND username = '{}'",
                room.id,
                username.replace("'", "''")
            ),
        );
        let last_read_row: Option<LastRead> = LastRead::find_by_statement(stmt).one(&conn).await.ok().flatten();
        let last_read_id = last_read_row.and_then(|r| r.last_read_id);
        let unread: i64 = if let Some(lid) = last_read_id {
            ChatEntity::find().filter(ChatCol::RoomId.eq(room.id)).filter(ChatCol::Id.gt(lid)).count(&conn).await.unwrap_or(0) as i64
        } else {
            ChatEntity::find().filter(ChatCol::RoomId.eq(room.id)).count(&conn).await.unwrap_or(0) as i64
        };
        out.push(RoomWithUnread { id: room.id, participants: parts, unread_count: unread });
    }
    Json(out)
}

#[derive(Deserialize)]
pub struct ReadUpdate { pub room_id: i32, pub username: String, pub last_read_id: Option<i32> }

pub async fn mark_read(
    State(conn): State<DatabaseConnection>,
    Json(payload): Json<ReadUpdate>,
) -> Json<&'static str> {
    let mut last_id = payload.last_read_id;
    if last_id.is_none() {
        let last = ChatEntity::find().filter(ChatCol::RoomId.eq(payload.room_id)).order_by_desc(ChatCol::Id).one(&conn).await.ok().flatten();
        last_id = last.map(|m| m.id);
    }
    let escaped_user = payload.username.replace("'", "''");
    let value = match last_id { Some(v) => v.to_string(), None => "NULL".to_string() };
    let upsert_sql = format!(
        "INSERT INTO room_read (room_id, username, last_read_id, updated_at) VALUES ({}, '{}', {}, NOW()) \
        ON CONFLICT (room_id, username) DO UPDATE SET last_read_id = EXCLUDED.last_read_id, updated_at = NOW()",
        payload.room_id, escaped_user, value
    );
    let _ = conn.execute(Statement::from_string(conn.get_database_backend(), upsert_sql)).await;
    Json("OK")
}
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    EntityTrait, FromQueryResult, ModelTrait, QueryFilter, QueryOrder, Statement,
};

use crate::entities::{
    chat::{self, Column as ChatCol, Entity as ChatEntity},
    room::{ActiveModel, Column, Entity as RoomEntity, Model},
};

pub async fn get_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<NewRoom>> {
    let mut condition = Condition::all();

    if let Some(id) = params.get("id") {
        condition = condition.add(Column::Id.eq(id.parse::<i32>().unwrap()));
    }

    let rooms = RoomEntity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap();

    let mut new_rooms: Vec<NewRoom> = Vec::new();

    for room in rooms {
        let participants: Vec<String> = serde_json::from_str(&room.participants).unwrap();

        new_rooms.push(NewRoom {
            id: Some(room.id),
            participants,
        });
    }

    Json(new_rooms)
}
use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    EntityTrait, FromQueryResult, ModelTrait, QueryFilter, Statement,
};

use crate::entities::{
    chat::{self, Column as ChatCol, Entity as ChatEntity},
    room::{ActiveModel, Column, Entity as RoomEntity, Model},
};

#[derive(Serialize, Deserialize)]
pub struct NewRoom {
    pub id: Option<i32>,
    pub participants: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RoomWithUnread {
    pub id: i32,
    pub participants: Vec<String>,
    pub unread_count: i64,
}

#[derive(Debug, FromQueryResult)]
struct LastRead { last_read_id: Option<i32> }

pub async fn get_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<NewRoom>> {
    let mut condition = Condition::all();

    if let Some(id) = params.get("id") {
        condition = condition.add(Column::Id.eq(id.parse::<i32>().unwrap()));
    }

    let rooms = RoomEntity::find()
        .filter(condition)
        .all(&conn)
        .await
        .unwrap();

    let mut new_rooms: Vec<NewRoom> = Vec::new();

    for room in rooms {
        let participants: Vec<String> = serde_json::from_str(&room.participants).unwrap();

        new_rooms.push(NewRoom {
            id: Some(room.id),
            participants,
        });
    }

    Json(new_rooms)
}

pub async fn post_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    // 참가자 정렬하여 동일한 조합은 동일한 문자열로 저장
    let mut parts = room.participants.clone();
    parts.sort();
    let participants = serde_json::to_string(&parts).unwrap();

    let room = ActiveModel {
        id: ActiveValue::not_set(),
        participants: ActiveValue::Set(participants),
    };

    Json(room.insert(&conn).await.unwrap())
}

// 참가자 목록으로 방을 찾거나 생성
pub async fn find_or_create_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let mut parts = room.participants.clone();
    parts.sort();
    let key = serde_json::to_string(&parts).unwrap();
    if let Ok(Some(existing)) = RoomEntity::find()
        .filter(Column::Participants.eq(key.clone()))
        .one(&conn)
        .await
    {
        return Json(existing);
    }
    let am = ActiveModel {
        id: ActiveValue::not_set(),
        participants: ActiveValue::Set(key),
    };
    Json(am.insert(&conn).await.unwrap())
}

pub async fn put_room(
    State(conn): State<DatabaseConnection>,
    Json(room): Json<NewRoom>,
) -> Json<Model> {
    let result = RoomEntity::find_by_id(room.id.unwrap())
        .one(&conn)
        .await
        .unwrap()
        .unwrap();

    let mut participants: Vec<String> = serde_json::from_str(&result.participants).unwrap();
    participants.push(room.participants[0].clone());

    let new_room = ActiveModel {
        id: ActiveValue::Set(result.id),
        participants: ActiveValue::Set(serde_json::to_string(&participants).unwrap()),
    };

    Json(new_room.update(&conn).await.unwrap())
}

pub async fn delete_room(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<&'static str> {
    let id = params.get("id").unwrap().parse::<i32>().unwrap();

    let chats = ChatEntity::find()
        .filter(ChatCol::RoomId.eq(id))
        .all(&conn)
        .await
        .unwrap();

    for chat in chats {
        chat.delete(&conn).await.unwrap();
    }

    let room = RoomEntity::find_by_id(id)
        .one(&conn)
        .await
        .unwrap()
        .unwrap();

    room.delete(&conn).await.unwrap();

    Json("Deleted")
}

// 리스트 API: 사용자별 방 목록과 미확인 메시지 수
pub async fn list_rooms_with_unread(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<RoomWithUnread>> {
    let Some(username) = params.get("username").cloned() else { return Json(Vec::new()) };
    // participants(JSON) 문자열에 "username" 포함된 방만 가져오기
    let like_token = format!("\"{}\"", username);
    let rooms = RoomEntity::find()
        .filter(Column::Participants.contains(&like_token))
        .all(&conn)
        .await
        .unwrap_or_default();

    let mut out = Vec::new();
    for room in rooms {
        let parts: Vec<String> = serde_json::from_str(&room.participants).unwrap_or_default();
        // last_read_id 조회 (raw SQL via SeaORM)
        let stmt = Statement::from_string(
            conn.get_database_backend(),
            format!(
                "SELECT last_read_id FROM room_read WHERE room_id = {} AND username = '{}'",
                room.id,
                username.replace("'", "''")
            ),
        );
        let last_read_row: Option<LastRead> = LastRead::find_by_statement(stmt)
            .one(&conn)
            .await
            .ok()
            .flatten();
        let last_read_id = last_read_row.and_then(|r| r.last_read_id);

        // unread 계산
        let unread: i64 = if let Some(lid) = last_read_id {
            ChatEntity::find()
                .filter(ChatCol::RoomId.eq(room.id))
                .filter(ChatCol::Id.gt(lid))
                .count(&conn)
                .await
                .unwrap_or(0) as i64
        } else {
            ChatEntity::find()
                .filter(ChatCol::RoomId.eq(room.id))
                .count(&conn)
                .await
                .unwrap_or(0) as i64
        };
        out.push(RoomWithUnread { id: room.id, participants: parts, unread_count: unread });
    }
    Json(out)
}

#[derive(Deserialize)]
pub struct ReadUpdate { pub room_id: i32, pub username: String, pub last_read_id: Option<i32> }

// 읽음 갱신 API: 방 마지막 읽은 메시지 id 업서트
pub async fn mark_read(
    State(conn): State<DatabaseConnection>,
    Json(payload): Json<ReadUpdate>,
) -> Json<&'static str> {
    // 마지막 메시지 id가 없으면 해당 방 최신 메시지 id로 대체
    let mut last_id = payload.last_read_id;
    if last_id.is_none() {
        let last = ChatEntity::find()
            .filter(ChatCol::RoomId.eq(payload.room_id))
            .order_by_desc(ChatCol::Id)
            .one(&conn)
            .await
            .ok()
            .flatten();
        last_id = last.map(|m| m.id);
    }
    // UPSERT (Postgres) via raw SQL
    let escaped_user = payload.username.replace("'", "''");
    let value = match last_id { Some(v) => v.to_string(), None => "NULL".to_string() };
    let upsert_sql = format!(
        "INSERT INTO room_read (room_id, username, last_read_id, updated_at) VALUES ({}, '{}', {}, NOW()) \
        ON CONFLICT (room_id, username) DO UPDATE SET last_read_id = EXCLUDED.last_read_id, updated_at = NOW()",
        payload.room_id, escaped_user, value
    );
    let _ = conn.execute(Statement::from_string(conn.get_database_backend(), upsert_sql)).await;
    Json("OK")
}
