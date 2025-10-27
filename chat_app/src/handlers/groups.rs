use warp::Filter;
use sqlx::{SqlitePool, Row};
use serde::{Deserialize, Serialize};
use warp::Reply;
use std::convert::Infallible;

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub group_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JoinLeaveGroupRequest {
    pub group_id: i64,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<String>,
    pub is_member: bool,
}

#[derive(Debug, Serialize)]
pub struct GroupsListResponse {
    pub member_groups: Vec<GroupResponse>,
    pub available_groups: Vec<GroupResponse>,
}

// ---------------- Routes ----------------
pub fn extended_routes(pool: SqlitePool) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let pool4 = pool.clone();
    let pool5 = pool.clone();
    let pool6 = pool.clone();

    // Test route to check if routing works at all
    let test_join = warp::path!("groups" / "test-join")
        .and(warp::post())
        .map(|| {
            println!("TEST: Join route reached successfully!");
            warp::reply::json(&serde_json::json!({"status": "test_success"}))
        });

    // Simplified join route without JSON body parsing
    let join_group_simple = warp::path!("groups" / "join-simple")
        .and(warp::post())
        .map(|| {
            println!("SIMPLE: Join route reached without body parsing!");
            warp::reply::json(&serde_json::json!({"status": "simple_success"}))
        });

    // Join group route with proper error handling
    let join_group = warp::path!("groups" / "join")
        .and(warp::post())
        .and(warp::body::json::<JoinLeaveGroupRequest>())
        .and(warp::header::<String>("authorization"))
        .and(warp::any().map(move || pool4.clone()))
        .and_then(join_group_handler);

    let leave_group = warp::path!("groups" / "leave")
        .and(warp::post())
        .and(warp::body::json::<JoinLeaveGroupRequest>())
        .and(warp::header::<String>("authorization"))
        .and(warp::any().map(move || pool5.clone()))
        .and_then(leave_group_handler);

    let update_group = warp::path!("groups" / "update")
        .and(warp::put())
        .and(warp::body::json::<UpdateGroupRequest>())
        .and(warp::header::<String>("authorization"))
        .and(warp::any().map(move || pool2.clone()))
        .and_then(update_group_handler);

    let delete_group = warp::path!("groups" / "delete" / i64)
        .and(warp::delete())
        .and(warp::header::<String>("authorization"))
        .and(warp::any().map(move || pool3.clone()))
        .and_then(delete_group_handler);

    let create_group = warp::path("groups")
        .and(warp::post())
        .and(warp::body::json::<CreateGroupRequest>())
        .and(warp::header::<String>("authorization"))
        .and(warp::any().map(move || pool1.clone()))
        .and_then(create_group_handler);

    let list_groups = warp::path("groups")
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(warp::any().map(move || pool6.clone()))
        .and_then(list_groups_handler);

    // Order: most specific routes first
    test_join
        .or(join_group_simple)
        .or(join_group)
        .or(leave_group)
        .or(update_group)
        .or(delete_group)
        .or(create_group)
        .or(list_groups)
}

// Helper function to extract username from auth header
fn extract_username_from_auth_header(auth_header: String) -> Result<String, String> {
    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err("Invalid authorization header".to_string());
    };

    crate::verify_jwt(token).map_err(|_| "Invalid or expired token".to_string())
}

// ---------------- Handlers ----------------

async fn create_group_handler(
    req: CreateGroupRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, Infallible> {
    println!("Creating group: {:?}", req);
    
    let creator_username = match extract_username_from_auth_header(auth_header) {
        Ok(username) => username,
        Err(error) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": error})),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };
    
    let group_id = sqlx::query("INSERT INTO groups (name, description) VALUES (?, ?)")
        .bind(&req.name)
        .bind(&req.description)
        .execute(&pool)
        .await
        .expect("Failed to insert group")
        .last_insert_rowid();

    // Add creator as member
    let _ = sqlx::query("INSERT INTO group_members (group_id, username) VALUES (?, ?)")
        .bind(group_id)
        .bind(&creator_username)
        .execute(&pool)
        .await;

    // Add other members
    for member in &req.members {
        if member != &creator_username {
            let _ = sqlx::query("INSERT INTO group_members (group_id, username) VALUES (?, ?)")
                .bind(group_id)
                .bind(member)
                .execute(&pool)
                .await;
        }
    }

    let mut all_members = req.members.clone();
    if !all_members.contains(&creator_username) {
        all_members.push(creator_username);
    }

    let resp = GroupResponse {
        id: group_id,
        name: req.name,
        description: req.description,
        members: all_members,
        is_member: true,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&resp),
        warp::http::StatusCode::CREATED,
    ))
}

async fn join_group_handler(
    req: JoinLeaveGroupRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, Infallible> {
    println!("Join group request: {:?}", req);
    
    let auth_username = match extract_username_from_auth_header(auth_header) {
        Ok(username) => username,
        Err(error) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": error})),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    if auth_username != req.username {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Unauthorized to join for another user"})),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }
    
    let existing = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
        .bind(req.group_id)
        .bind(&req.username)
        .fetch_optional(&pool)
        .await;

    match existing {
        Ok(Some(_)) => {
            println!("User {} already in group {}", req.username, req.group_id);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"status": "already_member"})),
                warp::http::StatusCode::OK,
            ))
        }
        Ok(None) => {
            let result = sqlx::query("INSERT INTO group_members (group_id, username) VALUES (?, ?)")
                .bind(req.group_id)
                .bind(&req.username)
                .execute(&pool)
                .await;

            match result {
                Ok(_) => {
                    println!("Successfully added {} to group {}", req.username, req.group_id);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({"status": "joined"})),
                        warp::http::StatusCode::OK,
                    ))
                }
                Err(e) => {
                    println!("Failed to add member: {:?}", e);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({"status": "error", "message": format!("{:?}", e)})),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) => {
            println!("Database error: {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"status": "error", "message": "Database error"})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn update_group_handler(
    req: UpdateGroupRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, Infallible> {
    println!("Updating group: {:?}", req);
    
    let username = match extract_username_from_auth_header(auth_header) {
        Ok(username) => username,
        Err(error) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": error})),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
        .bind(req.group_id)
        .bind(&username)
        .fetch_optional(&pool)
        .await;

    if member_check.unwrap_or(None).is_none() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Not a member of this group"})),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }
    
    if let Some(name) = req.name {
        let _ = sqlx::query("UPDATE groups SET name = ? WHERE id = ?")
            .bind(name)
            .bind(req.group_id)
            .execute(&pool)
            .await;
    }
    if let Some(description) = req.description {
        let _ = sqlx::query("UPDATE groups SET description = ? WHERE id = ?")
            .bind(description)
            .bind(req.group_id)
            .execute(&pool)
            .await;
    }
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({"status": "updated"})),
        warp::http::StatusCode::OK,
    ))
}

async fn delete_group_handler(
    group_id: i64,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, Infallible> {
    println!("Deleting group: {}", group_id);
    
    let username = match extract_username_from_auth_header(auth_header) {
        Ok(username) => username,
        Err(error) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": error})),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
        .bind(group_id)
        .bind(&username)
        .fetch_optional(&pool)
        .await;

    if member_check.unwrap_or(None).is_none() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Not authorized to delete this group"})),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }
    
    let _ = sqlx::query("DELETE FROM group_members WHERE group_id = ?")
        .bind(group_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM groups WHERE id = ?")
        .bind(group_id)
        .execute(&pool)
        .await;
        
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({"status": "deleted"})),
        warp::http::StatusCode::OK,
    ))
}

async fn leave_group_handler(
    req: JoinLeaveGroupRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, Infallible> {
    println!("Leave group request: {:?}", req);
    
    let auth_username = match extract_username_from_auth_header(auth_header) {
        Ok(username) => username,
        Err(error) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": error})),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    if auth_username != req.username {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({"error": "Unauthorized to leave for another user"})),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }
    
    let result = sqlx::query("DELETE FROM group_members WHERE group_id = ? AND username = ?")
        .bind(req.group_id)
        .bind(&req.username)
        .execute(&pool)
        .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() > 0 {
                println!("Successfully removed {} from group {}", req.username, req.group_id);
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({"status": "left"})),
                    warp::http::StatusCode::OK,
                ))
            } else {
                println!("User {} was not in group {}", req.username, req.group_id);
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({"status": "not_member"})),
                    warp::http::StatusCode::OK,
                ))
            }
        }
        Err(e) => {
            println!("Failed to remove member: {:?}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"status": "error", "message": format!("{:?}", e)})),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn list_groups_handler(
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, Infallible> {
    println!("Listing groups");
    
    let username = match extract_username_from_auth_header(auth_header) {
        Ok(username) => username,
        Err(error) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({"error": error})),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };
    
    // Get groups where user is a member
    let member_groups_rows = sqlx::query(
        "SELECT g.id, g.name, g.description 
         FROM groups g 
         INNER JOIN group_members gm ON g.id = gm.group_id 
         WHERE gm.username = ?"
    )
    .bind(&username)
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch member groups");

    let mut member_groups = Vec::new();
    for row in member_groups_rows {
        let group_id: i64 = row.get("id");
        let members = get_group_members(&pool, group_id).await;
        member_groups.push(GroupResponse {
            id: group_id,
            name: row.get("name"),
            description: row.get("description"),
            members,
            is_member: true,
        });
    }

    // Get groups where user is NOT a member (available to join)
    let available_groups_rows = sqlx::query(
        "SELECT g.id, g.name, g.description 
         FROM groups g 
         WHERE g.id NOT IN (
             SELECT gm.group_id 
             FROM group_members gm 
             WHERE gm.username = ?
         )"
    )
    .bind(&username)
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch available groups");

    let mut available_groups = Vec::new();
    for row in available_groups_rows {
        let group_id: i64 = row.get("id");
        let members = get_group_members(&pool, group_id).await;
        available_groups.push(GroupResponse {
            id: group_id,
            name: row.get("name"),
            description: row.get("description"),
            members,
            is_member: false,
        });
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&GroupsListResponse {
            member_groups,
            available_groups,
        }),
        warp::http::StatusCode::OK,
    ))
}

// ---------------- Helper ----------------

pub async fn get_group_members(pool: &SqlitePool, group_id: i64) -> Vec<String> {
    let rows = sqlx::query("SELECT username FROM group_members WHERE group_id = ?")
        .bind(group_id)
        .fetch_all(pool)
        .await
        .expect("Failed to fetch group members");

    rows.into_iter()
        .map(|r| r.get("username"))
        .collect()
}