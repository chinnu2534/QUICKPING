use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use warp::ws::{Message, WebSocket};
use warp::{Filter, Reply};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration, Datelike};

use dotenv::dotenv;
use std::env;

mod handlers;
use handlers::groups;

use lazy_static::lazy_static;

lazy_static! {
    static ref TEMP_META: Arc<Mutex<HashMap<String, (String, String)>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    id: i64,
    group_id: Option<i64>,
    sender_username: String,
    receiver_username: String,
    message: String,
    timestamp: String,
    reactions: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
struct User {
    username: String,
    sender: broadcast::Sender<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    message: String,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct HistoryMessage {
    #[serde(rename = "type")]
    message_type: String,
    conversation_with: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserListResponse {
    users: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncomingMessage {
    #[serde(rename = "type")]
    message_type: String,
    #[serde(default)]
    receiver_username: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    message_id: Option<i64>,
    #[serde(default)]
    emoji: Option<String>,
    #[serde(default)]
    group_id: Option<i64>,
    // Poll-related fields
    #[serde(default)]
    poll_question: Option<String>,
    #[serde(default)]
    poll_options: Option<Vec<String>>,
    #[serde(default)]
    poll_allow_multiple: Option<bool>,
    #[serde(default)]
    poll_expires_at: Option<String>,
    #[serde(default)]
    poll_id: Option<i64>,
    #[serde(default)]
    poll_option_ids: Option<Vec<i64>>,
    // Game-related fields (NEW)
    #[serde(default)]
    game_type: Option<String>,
    #[serde(default)]
    game_id: Option<i64>,
    #[serde(default)]
    game_move: Option<String>,
    #[serde(default)]
    target_username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConversationHistoryResponse {
    #[serde(rename = "type")]
    message_type: String,
    conversation_with: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroupHistoryResponse {
    #[serde(rename = "type")]
    message_type: String,
    group_id: i64,
    messages: Vec<ChatMessage>,
}

// Poll-related structures
#[derive(Debug, Serialize, Deserialize)]
struct CreatePollRequest {
    group_id: i64,
    question: String,
    options: Vec<String>,
    allow_multiple_choices: Option<bool>,
    expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VotePollRequest {
    poll_id: i64,
    option_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PollOption {
    id: i64,
    option_text: String,
    vote_count: i64,
    voted_by_current_user: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Poll {
    id: i64,
    group_id: i64,
    creator_username: String,
    question: String,
    created_at: String,
    expires_at: Option<String>,
    is_active: bool,
    allow_multiple_choices: bool,
    options: Vec<PollOption>,
    total_votes: i64,
}

// Add these new structs after existing ones
//#[derive(Debug, Serialize, Deserialize)]
//struct HighlightRequest {
//    #[serde(rename = "type")]
//    highlight_type: String,
//    target_type: String,
//    target_id: Option<i64>,
//   date_range: Option<String>,
//}

#[derive(Debug, Serialize, Deserialize)]
struct Highlight {
    id: i64,
    user_username: String,
    target_type: String,
    target_id: Option<i64>,
    target_name: String,
    highlight_type: String,
    summary: String,
    key_topics: Vec<String>,
    message_count: i64,
    participant_count: i64,
    start_date: String,
    end_date: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HighlightResponse {
    highlights: Vec<Highlight>,
    period: String,
    total_messages: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct HighlightRequest {
    #[serde(rename = "type")]
    highlight_type: String,
    target_type: String,
    target_id: Option<i64>,
    date_range: Option<String>,
    #[serde(default)]
    specific_user: Option<String>, // Add this new field
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Serialize, Deserialize)]
struct AIAssistantRequest {
    query: String,
    context_type: Option<String>,
    target_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AIAssistantResponse {
    response: String,
    query_type: String,
    success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    id: i64,
    game_type: String, // "chess", "tictactoe", "trivia"
    player1_username: String,
    player2_username: Option<String>,
    game_state: String, // JSON serialized game state
    current_turn: String,
    status: String, // "waiting", "active", "finished"
    winner: Option<String>,
    created_at: String,
    conversation_type: String, // "private" or "group"
    conversation_id: Option<i64>, // group_id for groups, null for private
}

#[derive(Debug, Serialize, Deserialize)]
struct GameMove {
    game_id: i64,
    player_username: String,
    move_data: String, // JSON serialized move
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateGameRequest {
    game_type: String,
    target_username: Option<String>, // for private games
    group_id: Option<i64>, // for group games
}

#[derive(Debug, Serialize, Deserialize)]
struct GameMoveRequest {
    game_id: i64,
    move_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TriviaQuestion {
    id: i64,
    question: String,
    options: Vec<String>,
    correct_answer: i32,
    category: String,
}

type Users = Arc<Mutex<HashMap<String, User>>>;

const JWT_SECRET: &[u8] = b"your-secret-key-change-this-in-production";

async fn create_game_tables(pool: &SqlitePool) {
    // Games table
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_type TEXT NOT NULL,
            player1_username TEXT NOT NULL,
            player2_username TEXT,
            game_state TEXT NOT NULL,
            current_turn TEXT NOT NULL,
            status TEXT DEFAULT 'waiting',
            winner TEXT,
            created_at TEXT NOT NULL,
            conversation_type TEXT NOT NULL,
            conversation_id INTEGER
        )"
    ).execute(pool).await;

    // Game moves table
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS game_moves (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            player_username TEXT NOT NULL,
            move_data TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
        )"
    ).execute(pool).await;

    // Trivia questions table
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS trivia_questions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            question TEXT NOT NULL,
            options TEXT NOT NULL,
            correct_answer INTEGER NOT NULL,
            category TEXT NOT NULL
        )"
    ).execute(pool).await;

    // Insert some sample trivia questions
    let sample_questions = vec![
        ("What is the capital of France?", "[\"Paris\", \"London\", \"Berlin\", \"Madrid\"]", 0, "Geography"),
        ("What is 2 + 2?", "[\"3\", \"4\", \"5\", \"6\"]", 1, "Math"),
        ("Who painted the Mona Lisa?", "[\"Van Gogh\", \"Picasso\", \"Da Vinci\", \"Monet\"]", 2, "Art"),
        ("What year did World War II end?", "[\"1944\", \"1945\", \"1946\", \"1947\"]", 1, "History"),
        ("What is the largest planet?", "[\"Earth\", \"Mars\", \"Jupiter\", \"Saturn\"]", 2, "Science"),
    ];

    for (question, options, correct, category) in sample_questions {
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO trivia_questions (question, options, correct_answer, category) VALUES (?, ?, ?, ?)"
        )
        .bind(question)
        .bind(options)
        .bind(correct)
        .bind(category)
        .execute(pool)
        .await;
    }
}

async fn create_game(
    pool: &SqlitePool,
    game_type: &str,
    player1: &str,
    player2: Option<&str>,
    conversation_type: &str,
    conversation_id: Option<i64>,
) -> Result<Game, sqlx::Error> {
    let now = get_current_time();
    let initial_state = match game_type {
        "chess" => create_initial_chess_state(),
        "tictactoe" => create_initial_tictactoe_state(),
        "trivia" => create_initial_trivia_state(pool).await?,
        _ => "{}".to_string(),
    };

    let game_id = sqlx::query(
        "INSERT INTO games (game_type, player1_username, player2_username, game_state, current_turn, status, created_at, conversation_type, conversation_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(game_type)
    .bind(player1)
    .bind(player2)
    .bind(&initial_state)
    .bind(player1)
    .bind(if player2.is_some() { "active" } else { "waiting" })
    .bind(&now)
    .bind(conversation_type)
    .bind(conversation_id)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(Game {
        id: game_id,
        game_type: game_type.to_string(),
        player1_username: player1.to_string(),
        player2_username: player2.map(|s| s.to_string()),
        game_state: initial_state,
        current_turn: player1.to_string(),
        status: if player2.is_some() { "active".to_string() } else { "waiting".to_string() },
        winner: None,
        created_at: now,
        conversation_type: conversation_type.to_string(),
        conversation_id,
    })
}

fn create_initial_chess_state() -> String {
    serde_json::json!({
        "board": [
            ["r", "n", "b", "q", "k", "b", "n", "r"],
            ["p", "p", "p", "p", "p", "p", "p", "p"],
            [".", ".", ".", ".", ".", ".", ".", "."],
            [".", ".", ".", ".", ".", ".", ".", "."],
            [".", ".", ".", ".", ".", ".", ".", "."],
            [".", ".", ".", ".", ".", ".", ".", "."],
            ["P", "P", "P", "P", "P", "P", "P", "P"],
            ["R", "N", "B", "Q", "K", "B", "N", "R"]
        ],
        "turn": "white",
        "moves": []
    }).to_string()
}

fn create_initial_tictactoe_state() -> String {
    serde_json::json!({
        "board": [
            ["", "", ""],
            ["", "", ""],
            ["", "", ""]
        ],
        "turn": "X",
        "moves": []
    }).to_string()
}

async fn create_initial_trivia_state(pool: &SqlitePool) -> Result<String, sqlx::Error> {
    let question = sqlx::query_as::<_, (i64, String, String, i32, String)>(
        "SELECT id, question, options, correct_answer, category FROM trivia_questions ORDER BY RANDOM() LIMIT 1"
    )
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "current_question": {
            "id": question.0,
            "question": question.1,
            "options": serde_json::from_str::<Vec<String>>(&question.2).unwrap_or_default(),
            "category": question.4
        },
        "scores": {},
        "answered": []
    }).to_string())
}

async fn process_game_move(
    pool: &SqlitePool,
    game_id: i64,
    player: &str,
    move_data: &str,
) -> Result<Game, String> {
    // Get current game
    let game_row = sqlx::query(
        "SELECT id, game_type, player1_username, player2_username, game_state, current_turn, status, winner, created_at, conversation_type, conversation_id
         FROM games WHERE id = ?"
    )
    .bind(game_id)
    .fetch_one(pool)
    .await
    .map_err(|_| "Game not found")?;

    let mut game = Game {
        id: game_row.get("id"),
        game_type: game_row.get("game_type"),
        player1_username: game_row.get("player1_username"),
        player2_username: game_row.get("player2_username"),
        game_state: game_row.get("game_state"),
        current_turn: game_row.get("current_turn"),
        status: game_row.get("status"),
        winner: game_row.get("winner"),
        created_at: game_row.get("created_at"),
        conversation_type: game_row.get("conversation_type"),
        conversation_id: game_row.get("conversation_id"),
    };

    // Validate player
    if game.status != "active" {
        return Err("Game is not active".to_string());
    }

    if game.current_turn != player {
        return Err("Not your turn".to_string());
    }

    // Process move based on game type
    match game.game_type.as_str() {
        "chess" => process_chess_move(&mut game, player, move_data)?,
        "tictactoe" => process_tictactoe_move(&mut game, player, move_data)?,
        "trivia" => process_trivia_move(&mut game, player, move_data, pool).await?,
        _ => return Err("Unknown game type".to_string()),
    }

    // Save move
    let now = get_current_time();
    let _ = sqlx::query(
        "INSERT INTO game_moves (game_id, player_username, move_data, timestamp) VALUES (?, ?, ?, ?)"
    )
    .bind(game_id)
    .bind(player)
    .bind(move_data)
    .bind(&now)
    .execute(pool)
    .await;

    // Update game
    let _ = sqlx::query(
        "UPDATE games SET game_state = ?, current_turn = ?, status = ?, winner = ? WHERE id = ?"
    )
    .bind(&game.game_state)
    .bind(&game.current_turn)
    .bind(&game.status)
    .bind(&game.winner)
    .bind(game_id)
    .execute(pool)
    .await;

    Ok(game)
}

fn process_chess_move(game: &mut Game, _player: &str, move_data: &str) -> Result<(), String> {
    let move_json: serde_json::Value = serde_json::from_str(move_data)
        .map_err(|_| "Invalid move format")?;
    
    let mut state: serde_json::Value = serde_json::from_str(&game.game_state)
        .map_err(|_| "Invalid game state")?;
    
    // Basic chess move validation (simplified)
    let from = move_json["from"].as_array().ok_or("Invalid from position")?;
    let to = move_json["to"].as_array().ok_or("Invalid to position")?;
    
    if from.len() != 2 || to.len() != 2 {
        return Err("Invalid position format".to_string());
    }

    let from_row = from[0].as_u64().ok_or("Invalid row")? as usize;
    let from_col = from[1].as_u64().ok_or("Invalid col")? as usize;
    let to_row = to[0].as_u64().ok_or("Invalid row")? as usize;
    let to_col = to[1].as_u64().ok_or("Invalid col")? as usize;

    if from_row >= 8 || from_col >= 8 || to_row >= 8 || to_col >= 8 {
        return Err("Position out of bounds".to_string());
    }

    // Move the piece (simplified)
    let board = state["board"].as_array_mut().ok_or("Invalid board")?;
    let piece = board[from_row][from_col].clone();
    board[from_row][from_col] = serde_json::Value::String(".".to_string());
    board[to_row][to_col] = piece;

    // Switch turns
    let current_turn = state["turn"].as_str().unwrap_or("white");
    state["turn"] = serde_json::Value::String(
        if current_turn == "white" { "black" } else { "white" }.to_string()
    );

    game.current_turn = if game.current_turn == game.player1_username {
        game.player2_username.as_ref().unwrap_or(&game.player1_username).clone()
    } else {
        game.player1_username.clone()
    };

    game.game_state = state.to_string();
    Ok(())
}

fn process_tictactoe_move(game: &mut Game, player: &str, move_data: &str) -> Result<(), String> {
    let move_json: serde_json::Value = serde_json::from_str(move_data)
        .map_err(|_| "Invalid move format")?;
    
    let mut state: serde_json::Value = serde_json::from_str(&game.game_state)
        .map_err(|_| "Invalid game state")?;
    
    let row = move_json["row"].as_u64().ok_or("Invalid row")? as usize;
    let col = move_json["col"].as_u64().ok_or("Invalid col")? as usize;

    if row >= 3 || col >= 3 {
        return Err("Position out of bounds".to_string());
    }

    let board = state["board"].as_array_mut().ok_or("Invalid board")?;
    
    if !board[row][col].as_str().unwrap_or("X").is_empty() {
        return Err("Position already taken".to_string());
    }

    let symbol = if player == game.player1_username { "X" } else { "O" };
    board[row][col] = serde_json::Value::String(symbol.to_string());

    // Check for win
    if check_tictactoe_win(&board, symbol) {
        game.status = "finished".to_string();
        game.winner = Some(player.to_string());
    } else if check_tictactoe_draw(&board) {
        game.status = "finished".to_string();
        game.winner = Some("draw".to_string());
    } else {
        // Switch turns
        game.current_turn = if game.current_turn == game.player1_username {
            game.player2_username.as_ref().unwrap_or(&game.player1_username).clone()
        } else {
            game.player1_username.clone()
        };
    }

    game.game_state = state.to_string();
    Ok(())
}

async fn process_trivia_move(
    game: &mut Game,
    player: &str,
    move_data: &str,
    pool: &SqlitePool,
) -> Result<(), String> {
    let move_json: serde_json::Value =
        serde_json::from_str(move_data).map_err(|_| "Invalid move format")?;

    let mut state: serde_json::Value =
        serde_json::from_str(&game.game_state).map_err(|_| "Invalid game state")?;

    let answer = move_json["answer"]
        .as_u64()
        .ok_or("Invalid answer")? as i32;
    let question_id = state["current_question"]["id"]
        .as_i64()
        .ok_or("No current question")?;

    // Get correct answer
    let correct_answer = sqlx::query_scalar::<_, i32>(
        "SELECT correct_answer FROM trivia_questions WHERE id = ?",
    )
    .bind(question_id)
    .fetch_one(pool)
    .await
    .map_err(|_| "Question not found")?;

    // Update scores
    {
        let scores = state["scores"]
            .as_object_mut()
            .ok_or("Invalid scores")?;

        let current_score = scores
            .get(player)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if answer == correct_answer {
            scores.insert(
                player.to_string(),
                serde_json::Value::Number(serde_json::Number::from(current_score + 1)),
            );
        }
    }

    // Mark as answered
    {
        let answered = state["answered"]
            .as_array_mut()
            .ok_or("Invalid answered list")?;

        answered.push(serde_json::Value::String(player.to_string()));
    }

    // Check if both players answered
    let both_answered = {
        let answered = state["answered"]
            .as_array()
            .ok_or("Invalid answered list")?;

        game.player2_username.is_none()
            || answered.len() >= 2
            || (answered.len() == 1
                && game.player2_username.is_some()
                && answered.iter().any(|v| {
                    v.as_str() == game.player2_username.as_ref().map(|s| s.as_str())
                }))
    };

    if both_answered {
        // Get next question or end game
        let next_question = sqlx::query_as::<_, (i64, String, String, i32, String)>(
            "SELECT id, question, options, correct_answer, category FROM trivia_questions WHERE id != ? ORDER BY RANDOM() LIMIT 1",
        )
        .bind(question_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| "Database error")?;

        if let Some(question) = next_question {
            state["current_question"] = serde_json::json!({
                "id": question.0,
                "question": question.1,
                "options": serde_json::from_str::<Vec<String>>(&question.2).unwrap_or_default(),
                "category": question.4
            });
            state["answered"] = serde_json::json!([]);
        } else {
            game.status = "finished".to_string();

            // Safe immutable borrow of scores here
            let scores = state["scores"]
                .as_object()
                .ok_or("Invalid scores")?;

            let p1_score = scores
                .get(&game.player1_username)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let p2_score = if let Some(p2) = &game.player2_username {
                scores.get(p2).and_then(|v| v.as_i64()).unwrap_or(0)
            } else {
                0
            };

            game.winner = if p1_score > p2_score {
                Some(game.player1_username.clone())
            } else if p2_score > p1_score {
                game.player2_username.clone()
            } else {
                Some("draw".to_string())
            };
        }
    }

    game.game_state = state.to_string();
    Ok(())
}

fn check_tictactoe_win(board: &[serde_json::Value], symbol: &str) -> bool {
    // Check rows, columns, and diagonals
    for i in 0..3 {
        // Check row
        if (0..3).all(|j| board[i][j].as_str() == Some(symbol)) {
            return true;
        }
        // Check column
        if (0..3).all(|j| board[j][i].as_str() == Some(symbol)) {
            return true;
        }
    }
    
    // Check diagonals
    if (0..3).all(|i| board[i][i].as_str() == Some(symbol)) {
        return true;
    }
    if (0..3).all(|i| board[i][2-i].as_str() == Some(symbol)) {
        return true;
    }
    
    false
}

fn check_tictactoe_draw(board: &[serde_json::Value]) -> bool {
    board.iter().all(|row| {
        row.as_array().unwrap().iter().all(|cell| {
            !cell.as_str().unwrap_or("").is_empty()
        })
    })
}


#[tokio::main]
async fn main() {
    // Initialize database
    dotenv().ok();
    env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set in .env file");
    let database_url = "sqlite:./db/chat.db";
    let pool = SqlitePool::connect(database_url)
        .await
        .expect("Failed to connect to database");

    // Create polls tables
    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS polls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER NOT NULL,
            creator_username TEXT NOT NULL,
            question TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT,
            is_active BOOLEAN DEFAULT 1,
            allow_multiple_choices BOOLEAN DEFAULT 0,
            FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS poll_options (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            poll_id INTEGER NOT NULL,
            option_text TEXT NOT NULL,
            option_order INTEGER NOT NULL,
            FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS poll_votes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            poll_id INTEGER NOT NULL,
            option_id INTEGER NOT NULL,
            username TEXT NOT NULL,
            voted_at TEXT NOT NULL,
            UNIQUE(poll_id, option_id, username),
            FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE,
            FOREIGN KEY (option_id) REFERENCES poll_options(id) ON DELETE CASCADE
        )"
    ).execute(&pool).await;

    let _ = sqlx::query(
        "CREATE TABLE IF NOT EXISTS highlights (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_username TEXT NOT NULL,
            target_type TEXT NOT NULL,
            target_id INTEGER,
            target_name TEXT NOT NULL,
            highlight_type TEXT NOT NULL,
            summary TEXT NOT NULL,
            key_topics TEXT NOT NULL,
            message_count INTEGER NOT NULL,
            participant_count INTEGER NOT NULL,
            start_date TEXT NOT NULL,
            end_date TEXT NOT NULL,
            created_at TEXT NOT NULL
        )"
    ).execute(&pool).await;
    initialize_games(&pool).await;

    async fn initialize_games(pool: &SqlitePool) {
    create_game_tables(pool).await;
}



    // Ensure uploads directory exists
    if let Err(e) = std::fs::create_dir_all("./db/uploads") {
        eprintln!("Warning: failed to create uploads dir: {:?}", e);
    }

    // Create a broadcast channel for messages
    let (tx, _rx) = broadcast::channel::<ChatMessage>(100);
    let users: Users = Arc::new(Mutex::new(HashMap::new()));
    
    // Clone for use in filters
    let users_filter = warp::any().map(move || users.clone());
    let tx_filter = warp::any().map(move || tx.clone());
    let pool_filter = warp::any().map({
        let pool = pool.clone();
        move || pool.clone()
    });

    let ai_assistant = warp::path!("ai" / "assistant")
    .and(warp::post())
    .and(warp::body::json::<AIAssistantRequest>())
    .and(warp::header::<String>("authorization"))
    .and(pool_filter.clone())
    .and_then(ai_assistant_handler);

    // Serve static files
    let static_files = warp::path::end()
        .and(warp::fs::file("./static/index.html"))
        .or(warp::path("static").and(warp::fs::dir("./static")));
    
    // Serve uploaded images
    let uploads_files = warp::path("uploads").and(warp::fs::dir("./db/uploads"));

    // Registration endpoint
    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(pool_filter.clone())
        .and_then(handle_register);

    // Login endpoint
    let login = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and(pool_filter.clone())
        .and_then(handle_login);

    // Users list endpoint
    let users_list = warp::path("users")
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(pool_filter.clone())
        .and_then(handle_users_list);

    // Poll endpoints
    let create_poll = warp::path!("polls" / "create")
        .and(warp::post())
        .and(warp::body::json::<CreatePollRequest>())
        .and(warp::header::<String>("authorization"))
        .and(pool_filter.clone())
        .and_then(create_poll_handler);

    let vote_poll = warp::path!("polls" / "vote")
        .and(warp::post())
        .and(warp::body::json::<VotePollRequest>())
        .and(warp::header::<String>("authorization"))
        .and(pool_filter.clone())
        .and_then(vote_poll_handler);

    let get_poll = warp::path!("polls" / i64)
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(pool_filter.clone())
        .and_then(get_poll_handler);

    // WebSocket route
    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(warp::query::<HashMap<String, String>>())
        .and(users_filter)
        .and(tx_filter)
        .and(pool_filter.clone())
        .map(|ws: warp::ws::Ws, params: HashMap<String, String>, users, tx, pool| {
            ws.on_upgrade(move |socket| handle_websocket(socket, users, tx, params, pool))
        });

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // Group routes
    let group_routes = groups::extended_routes(pool.clone());

    // Add this route for debugging




    // Add these routes to your existing routes
let generate_highlights = warp::path!("highlights" / "generate")
    .and(warp::post())
    .and(warp::body::json::<HighlightRequest>())
    .and(warp::header::<String>("authorization"))
    .and(pool_filter.clone())
    .and_then(generate_highlights_handler);

let get_highlights = warp::path!("highlights")
    .and(warp::get())
    .and(warp::query::<HashMap<String, String>>())
    .and(warp::header::<String>("authorization"))
    .and(pool_filter.clone())
    .and_then(get_highlights_handler);

    let routes = static_files
        .or(uploads_files)
        .or(register)
        .or(login)
        .or(users_list)
        .or(create_poll)
        .or(vote_poll)
        .or(get_poll)
        .or(websocket)
        .or(group_routes)
        .or(generate_highlights)
        .or(get_highlights)
        .or(ai_assistant)
        //.or(debug_messages) 
        .with(cors);

    println!("Personal Messenger App starting on http://0.0.0.0:3030");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030))
        .await;
}



// Poll handlers
async fn create_poll_handler(
    request: CreatePollRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid or expired token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // Validate that user is member of the group
    let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
        .bind(request.group_id)
        .bind(&username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| warp::reject::reject())?;

    if member_check.is_none() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "Not a member of this group".to_string(),
            }),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }

    // Validate poll data
    if request.question.trim().is_empty() || request.options.len() < 2 || request.options.len() > 10 {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "Invalid poll data".to_string(),
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let created_at = get_current_time();
    let allow_multiple = request.allow_multiple_choices.unwrap_or(false);

    // Create poll
    let poll_id = sqlx::query(
        "INSERT INTO polls (group_id, creator_username, question, created_at, expires_at, allow_multiple_choices) 
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(request.group_id)
    .bind(&username)
    .bind(&request.question)
    .bind(&created_at)
    .bind(&request.expires_at)
    .bind(allow_multiple)
    .execute(&pool)
    .await
    .map_err(|_| warp::reject::reject())?
    .last_insert_rowid();

    // Create poll options
    for (index, option_text) in request.options.iter().enumerate() {
        if !option_text.trim().is_empty() {
            sqlx::query("INSERT INTO poll_options (poll_id, option_text, option_order) VALUES (?, ?, ?)")
                .bind(poll_id)
                .bind(option_text.trim())
                .bind(index as i32)
                .execute(&pool)
                .await
                .map_err(|_| warp::reject::reject())?;
        }
    }

    // Get the created poll with options
    let poll = get_poll_by_id(&pool, poll_id, &username).await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&poll),
        warp::http::StatusCode::CREATED,
    ))
}

async fn vote_poll_handler(
    request: VotePollRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid or expired token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // Get poll info
    let poll_row = sqlx::query("SELECT group_id, allow_multiple_choices, is_active FROM polls WHERE id = ?")
        .bind(request.poll_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| warp::reject::reject())?;

    let poll_row = match poll_row {
        Some(row) => row,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Poll not found".to_string(),
                }),
                warp::http::StatusCode::NOT_FOUND,
            ));
        }
    };

    let group_id: i64 = poll_row.get("group_id");
    let allow_multiple: bool = poll_row.get("allow_multiple_choices");
    let is_active: bool = poll_row.get("is_active");

    if !is_active {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "Poll is not active".to_string(),
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Check if user is member of the group
    let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
        .bind(group_id)
        .bind(&username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| warp::reject::reject())?;

    if member_check.is_none() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "Not a member of this group".to_string(),
            }),
            warp::http::StatusCode::FORBIDDEN,
        ));
    }

    // Validate vote options
    if request.option_ids.is_empty() || (!allow_multiple && request.option_ids.len() > 1) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "Invalid vote options".to_string(),
            }),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Remove existing votes if not allowing multiple choices
    if !allow_multiple {
        sqlx::query("DELETE FROM poll_votes WHERE poll_id = ? AND username = ?")
            .bind(request.poll_id)
            .bind(&username)
            .execute(&pool)
            .await
            .map_err(|_| warp::reject::reject())?;
    }

    let voted_at = get_current_time();

    // Add new votes
    for option_id in request.option_ids {
        // Verify option belongs to this poll
        let option_check = sqlx::query("SELECT 1 FROM poll_options WHERE id = ? AND poll_id = ?")
            .bind(option_id)
            .bind(request.poll_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| warp::reject::reject())?;

        if option_check.is_some() {
            // Insert vote (ignore if duplicate due to UNIQUE constraint)
            let _ = sqlx::query("INSERT OR IGNORE INTO poll_votes (poll_id, option_id, username, voted_at) VALUES (?, ?, ?, ?)")
                .bind(request.poll_id)
                .bind(option_id)
                .bind(&username)
                .bind(&voted_at)
                .execute(&pool)
                .await;
        }
    }

    // Get updated poll
    let poll = get_poll_by_id(&pool, request.poll_id, &username).await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&poll),
        warp::http::StatusCode::OK,
    ))
}

async fn get_poll_handler(
    poll_id: i64,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid or expired token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let poll = get_poll_by_id(&pool, poll_id, &username).await?;

    Ok(warp::reply::with_status(
        warp::reply::json(&poll),
        warp::http::StatusCode::OK,
    ))
}

// Helper function to get poll with options and vote counts
async fn get_poll_by_id(pool: &SqlitePool, poll_id: i64, current_username: &str) -> Result<Poll, warp::Rejection> {
    // Get poll info
    let poll_row = sqlx::query(
        "SELECT id, group_id, creator_username, question, created_at, expires_at, is_active, allow_multiple_choices 
         FROM polls WHERE id = ?"
    )
    .bind(poll_id)
    .fetch_one(pool)
    .await
    .map_err(|_| warp::reject::reject())?;

    // Get poll options with vote counts
    let options_rows = sqlx::query(
        "SELECT po.id, po.option_text, 
                COUNT(pv.id) as vote_count,
                CASE WHEN user_votes.option_id IS NOT NULL THEN 1 ELSE 0 END as voted_by_current_user
         FROM poll_options po
         LEFT JOIN poll_votes pv ON po.id = pv.option_id
         LEFT JOIN poll_votes user_votes ON po.id = user_votes.option_id AND user_votes.username = ?
         WHERE po.poll_id = ?
         GROUP BY po.id, po.option_text, po.option_order
         ORDER BY po.option_order"
    )
    .bind(current_username)
    .bind(poll_id)
    .fetch_all(pool)
    .await
    .map_err(|_| warp::reject::reject())?;

    let mut options = Vec::new();
    let mut total_votes = 0;

    for row in options_rows {
        let vote_count: i64 = row.get("vote_count");
        total_votes += vote_count;

        options.push(PollOption {
            id: row.get("id"),
            option_text: row.get("option_text"),
            vote_count,
            voted_by_current_user: row.get::<i64, _>("voted_by_current_user") == 1,
        });
    }

    Ok(Poll {
        id: poll_row.get("id"),
        group_id: poll_row.get("group_id"),
        creator_username: poll_row.get("creator_username"),
        question: poll_row.get("question"),
        created_at: poll_row.get("created_at"),
        expires_at: poll_row.get("expires_at"),
        is_active: poll_row.get("is_active"),
        allow_multiple_choices: poll_row.get("allow_multiple_choices"),
        options,
        total_votes,
    })
}

fn extract_username_from_auth(auth_header: String) -> Result<String, jsonwebtoken::errors::Error> {
    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
    };

    verify_jwt(token)
}

async fn handle_register(
    request: RegisterRequest,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    // Validate input
    if request.username.trim().is_empty() || request.password.len() < 6 {
        let error = ErrorResponse {
            error: "Username cannot be empty and password must be at least 6 characters".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Hash password
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(request.password.as_bytes(), &salt)
        .map_err(|_| warp::reject::reject())?
        .to_string();

    // Insert user into database
    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
        .bind(&request.username)
        .bind(&password_hash)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => {
            let response = AuthResponse {
                message: "User registered successfully".to_string(),
                token: None,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.message().contains("UNIQUE constraint failed") => {
            let error = ErrorResponse {
                error: "Username already exists".to_string(),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&error),
                warp::http::StatusCode::CONFLICT,
            ))
        }
        Err(_) => {
            let error = ErrorResponse {
                error: "Internal server error".to_string(),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&error),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn handle_login(
    request: LoginRequest,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    // Get user from database
    let row = sqlx::query("SELECT username, password_hash FROM users WHERE username = ?")
        .bind(&request.username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| warp::reject::reject())?;

    match row {
        Some(row) => {
            let stored_hash: String = row.get("password_hash");
            
            // Verify password
            let argon2 = Argon2::default();
            let parsed_hash = PasswordHash::new(&stored_hash)
                .map_err(|_| warp::reject::reject())?;
            
            if argon2.verify_password(request.password.as_bytes(), &parsed_hash).is_ok() {
                // Generate JWT
                let expiration = Utc::now()
                    .checked_add_signed(Duration::hours(24))
                    .expect("valid timestamp")
                    .timestamp() as usize;

                let claims = Claims {
                    sub: request.username.clone(),
                    exp: expiration,
                };

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(JWT_SECRET),
                )
                .map_err(|_| warp::reject::reject())?;

                let response = AuthResponse {
                    message: "Login successful".to_string(),
                    token: Some(token),
                };

                Ok(warp::reply::with_status(
                    warp::reply::json(&response),
                    warp::http::StatusCode::OK,
                ))
            } else {
                let error = ErrorResponse {
                    error: "Invalid credentials".to_string(),
                };
                Ok(warp::reply::with_status(
                    warp::reply::json(&error),
                    warp::http::StatusCode::UNAUTHORIZED,
                ))
            }
        }
        None => {
            let error = ErrorResponse {
                error: "Invalid credentials".to_string(),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&error),
                warp::http::StatusCode::UNAUTHORIZED,
            ))
        }
    }
}

async fn handle_users_list(
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    // Extract token from Authorization header
    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                error: "Invalid authorization header".to_string(),
            }),
            warp::http::StatusCode::UNAUTHORIZED,
        ));
    };


    // Add this line near the top of main.rs, just before the verify_jwt function:

pub fn verify_jwt(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation,
    )?;
    Ok(token_data.claims.sub)
}



    // Verify JWT token
    let current_username = match verify_jwt(token) {
        Ok(username) => username,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid or expired token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // Get all users except the current user
    let rows = sqlx::query("SELECT username FROM users WHERE username != ? ORDER BY username")
        .bind(&current_username)
        .fetch_all(&pool)
        .await
        .map_err(|_| warp::reject::reject())?;

    let users: Vec<String> = rows
        .into_iter()
        .map(|row| row.get("username"))
        .collect();

    let response = UserListResponse { users };
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK,
    ))
}

fn verify_jwt(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation,
    )?;
    Ok(token_data.claims.sub)
}

async fn store_message(pool: &SqlitePool, sender_username: &str, receiver_username: &str, message: &str, timestamp: &str) -> Result<i64, sqlx::Error> {
    let _ = sqlx::query("INSERT INTO messages (sender_username, receiver_username, message, timestamp) VALUES (?, ?, ?, ?)")
        .bind(sender_username)
        .bind(receiver_username)
        .bind(message)
        .bind(timestamp)
        .execute(pool)
        .await?;
    Ok(sqlx::query_scalar("SELECT last_insert_rowid()").fetch_one(pool).await?)
}

async fn get_conversation_messages(pool: &SqlitePool, user1: &str, user2: &str, limit: i32) -> Vec<ChatMessage> {
    let rows = sqlx::query(
        "SELECT id, sender_username, receiver_username, message, timestamp FROM messages \n         WHERE (sender_username = ? AND receiver_username = ?) \n            OR (sender_username = ? AND receiver_username = ?) \n         ORDER BY id DESC LIMIT ?"
    )
    .bind(user1)
    .bind(user2)
    .bind(user2)
    .bind(user1)
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut messages: Vec<ChatMessage> = Vec::new();
    for row in rows {
        let message_id: i64 = row.get("id");
        let reactions_rows = sqlx::query("SELECT username, emoji FROM message_reactions WHERE message_id = ?")
            .bind(message_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

        let mut reactions_map: HashMap<String, String> = HashMap::new();
        for reaction_row in reactions_rows {
            reactions_map.insert(reaction_row.get("username"), reaction_row.get("emoji"));
        }

        messages.push(ChatMessage {
            id: message_id,
            group_id: row.try_get("group_id").ok(),
            sender_username: row.get("sender_username"),
            receiver_username: row.get("receiver_username"),
            message: row.get("message"),
            timestamp: row.get("timestamp"),
            reactions: if reactions_map.is_empty() { None } else { Some(reactions_map) },
        });
    }

    // Reverse to get chronological order (oldest first)
    messages.reverse();
    messages
}

async fn handle_websocket(
    websocket: WebSocket,
    users: Users,
    tx: broadcast::Sender<ChatMessage>,
    params: HashMap<String, String>,
    pool: SqlitePool,
) {
    let (mut ws_tx, mut ws_rx) = websocket.split();
    let mut rx = tx.subscribe();

    // Extract and verify JWT token
    let username = match params.get("token") {
        Some(token) => match verify_jwt(token) {
            Ok(username) => username,
            Err(_) => {
                let _ = ws_tx.send(Message::text(r#"{"error": "Invalid or expired token"}"#)).await;
                return;
            }
        },
        None => {
            let _ = ws_tx.send(Message::text(r#"{"error": "Authentication required"}"#)).await;
            return;
        }
    };

    println!("DEBUG: WebSocket connected for user: {}", username);

    let connection_id = format!(
        "{}_{}",
        username,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() % 10000
    );

    // Add user to users map
    {
        let mut users_lock = users.lock().await;
        users_lock.insert(connection_id.clone(), User {
            username: username.clone(),
            sender: tx.clone(),
        });
        println!("DEBUG: Added user to connections. Total connections: {}", users_lock.len());
    }

    let tx_clone = tx.clone();
    let username_clone = username.clone();
    let ws_tx_clone = Arc::new(Mutex::new(ws_tx));

    // Clone pool and users for incoming task
    let pool_incoming = pool.clone();
    let users_incoming = users.clone();
    let ws_tx_for_incoming = ws_tx_clone.clone();

    let incoming_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    // TEXT messages
                    if let Ok(text) = msg.to_str() {
                        println!("DEBUG: Received WebSocket message: {}", text);
                        match serde_json::from_str::<IncomingMessage>(text) {
                            Ok(incoming_msg) => {
                                println!("DEBUG: Successfully parsed message type: {}", incoming_msg.message_type);
                                match incoming_msg.message_type.as_str() {
                                    "get_conversation" => {
                                        if let Some(receiver_username) = incoming_msg.receiver_username {
                                            println!("DEBUG: Getting conversation history for: {}", receiver_username);
                                            let messages = get_conversation_messages(&pool_incoming, &username_clone, &receiver_username, 50).await;
                                            
                                            let history_response = ConversationHistoryResponse {
                                                message_type: "conversation_history".to_string(),
                                                conversation_with: receiver_username.clone(),
                                                messages,
                                            };
                                            
                                            if let Ok(json) = serde_json::to_string(&history_response) {
                                                let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                                                let _ = ws_tx_lock.send(Message::text(json)).await;
                                                println!("DEBUG: Sent conversation history for: {}", receiver_username);
                                            }
                                        }
                                    }

                                    "get_group_conversation" => {
                                        if let Some(group_id) = incoming_msg.group_id {
                                            println!("DEBUG: Getting group conversation history for group: {}", group_id);
                                            let messages = get_group_conversation_messages(&pool_incoming, group_id, 50).await;
                                            
                                            let history_response = GroupHistoryResponse {
                                                message_type: "group_conversation_history".to_string(),
                                                group_id,
                                                messages,
                                            };
                                            
                                            if let Ok(json) = serde_json::to_string(&history_response) {
                                                let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                                                let _ = ws_tx_lock.send(Message::text(json)).await;
                                                println!("DEBUG: Sent group conversation history for group: {}", group_id);
                                            }
                                        }
                                    }

                                    "chat_message" => {
                                        println!("DEBUG: Processing private chat message");
                                        if let (Some(message_text), Some(receiver_username)) =
                                            (incoming_msg.message, incoming_msg.receiver_username)
                                        {
                                            let message = ChatMessage {
                                                id: 0,
                                                sender_username: username_clone.clone(),
                                                receiver_username: receiver_username.clone(),
                                                group_id: None,
                                                message: message_text.clone(),
                                                timestamp: get_current_time(),
                                                reactions: None,
                                            };

                                            let message_id = store_message(
                                                &pool_incoming,
                                                &message.sender_username,
                                                &message.receiver_username,
                                                &message.message,
                                                &message.timestamp
                                            ).await.unwrap_or(0);

                                            let mut message_with_id = message.clone();
                                            message_with_id.id = message_id;
                                            let _ = tx_clone.send(message_with_id);
                                            println!("DEBUG: Sent private message via broadcast");
                                        }
                                    }

                                    "group_message" => {
                                        println!("DEBUG: Received group message request");
                                        if let (Some(group_id), Some(message_text)) = (incoming_msg.group_id, incoming_msg.message) {
                                            println!("DEBUG: Group ID: {}, Message: {}", group_id, message_text);
                                            let timestamp = get_current_time();

                                            // Store message in DB
                                            let message_id = store_group_message(
                                                &pool_incoming,
                                                group_id,
                                                &username_clone,
                                                &message_text,
                                                &timestamp
                                            ).await.unwrap_or(0);
                                            
                                            println!("DEBUG: Stored group message with ID: {}", message_id);

                                            // Prepare chat message
                                            let chat_msg = ChatMessage {
                                                id: message_id,
                                                sender_username: username_clone.clone(),
                                                receiver_username: "".to_string(),
                                                group_id: Some(group_id),
                                                message: message_text,
                                                timestamp,
                                                reactions: None,
                                            };

                                            // Just send via broadcast channel - don't manually send to individual users
                                            println!("DEBUG: Sending group message via broadcast channel");
                                            let _ = tx_clone.send(chat_msg);
                                        } else {
                                            println!("DEBUG: Missing group_id or message in group_message request");
                                        }
                                    }

                                    "create_poll" => {
                                        println!("DEBUG: Creating poll in group");
                                        println!("DEBUG: group_id: {:?}", incoming_msg.group_id);
                                        println!("DEBUG: poll_question: {:?}", incoming_msg.poll_question);
                                        println!("DEBUG: poll_options: {:?}", incoming_msg.poll_options);
                                        
                                        if let (Some(group_id), Some(question), Some(options)) = 
                                            (incoming_msg.group_id, incoming_msg.poll_question, incoming_msg.poll_options) {
                                            
                                            println!("DEBUG: Poll creation data validated");
                                            
                                            // Validate user is member of group
                                            let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
                                                .bind(group_id)
                                                .bind(&username_clone)
                                                .fetch_optional(&pool_incoming)
                                                .await
                                                .unwrap_or(None);
                                                
                                            if member_check.is_none() {
                                                println!("DEBUG: User {} not member of group {}", username_clone, group_id);
                                                continue;
                                            }
                                            
                                            println!("DEBUG: User is member of group");
                                            
                                            if question.trim().is_empty() || options.len() < 2 || options.len() > 10 {
                                                println!("DEBUG: Invalid poll data - question: '{}', options count: {}", question, options.len());
                                                continue;
                                            }
                                            
                                            println!("DEBUG: Poll data is valid");
                                            
                                            let created_at = get_current_time();
                                            let allow_multiple = incoming_msg.poll_allow_multiple.unwrap_or(false);
                                            
                                            // Create poll
                                            let poll_result = sqlx::query(
                                                "INSERT INTO polls (group_id, creator_username, question, created_at, expires_at, allow_multiple_choices) 
                                                 VALUES (?, ?, ?, ?, ?, ?)"
                                            )
                                            .bind(group_id)
                                            .bind(&username_clone)
                                            .bind(&question)
                                            .bind(&created_at)
                                            .bind(&incoming_msg.poll_expires_at)
                                            .bind(allow_multiple)
                                            .execute(&pool_incoming)
                                            .await;
                                            
                                            match poll_result {
                                                Ok(result) => {
                                                    let poll_id = result.last_insert_rowid();
                                                    println!("DEBUG: Poll created with ID: {}", poll_id);
                                                    
                                                    // Create poll options
                                                    for (index, option_text) in options.iter().enumerate() {
                                                        if !option_text.trim().is_empty() {
                                                            let option_result = sqlx::query("INSERT INTO poll_options (poll_id, option_text, option_order) VALUES (?, ?, ?)")
                                                                .bind(poll_id)
                                                                .bind(option_text.trim())
                                                                .bind(index as i32)
                                                                .execute(&pool_incoming)
                                                                .await;
                                                            
                                                            match option_result {
                                                                Ok(_) => println!("DEBUG: Created option: {}", option_text),
                                                                Err(e) => println!("DEBUG: Failed to create option: {:?}", e),
                                                            }
                                                        }
                                                    }
                                                    
                                                    // Store the poll creation as a group message
                                                    let poll_message_text = format!("📊 Poll created: {}", question);
                                                    let message_result = store_group_message(
                                                        &pool_incoming,
                                                        group_id,
                                                        &username_clone,
                                                        &poll_message_text,
                                                        &created_at
                                                    ).await;
                                                    
                                                    match message_result {
                                                        Ok(message_id) => {
                                                            println!("DEBUG: Stored group message with ID: {}", message_id);
                                                            
                                                            // Broadcast poll creation as a group message
                                                            let poll_message = ChatMessage {
                                                                id: poll_id, // Use poll_id for poll identification
                                                                sender_username: username_clone.clone(),
                                                                receiver_username: "".to_string(),
                                                                group_id: Some(group_id),
                                                                message: poll_message_text,
                                                                timestamp: created_at,
                                                                reactions: None,
                                                            };
                                                            
                                                            println!("DEBUG: Broadcasting poll creation to group {}", group_id);
                                                            match tx_clone.send(poll_message) {
                                                                Ok(_) => println!("DEBUG: Poll message broadcast successfully"),
                                                                Err(e) => println!("DEBUG: Failed to broadcast poll message: {:?}", e),
                                                            }
                                                        }
                                                        Err(e) => println!("DEBUG: Failed to store group message: {:?}", e),
                                                    }
                                                }
                                                Err(e) => println!("DEBUG: Failed to create poll in database: {:?}", e),
                                            }
                                        } else {
                                            println!("DEBUG: Missing required poll data");
                                        }
                                    }

                                    "vote_poll" => {
                                        println!("DEBUG: Voting on poll");
                                        if let (Some(poll_id), Some(option_ids)) = (incoming_msg.poll_id, incoming_msg.poll_option_ids) {
                                            
                                            // Get poll info and validate
                                            let poll_check = sqlx::query(
                                                "SELECT p.group_id, p.allow_multiple_choices, p.is_active, p.question, gm.username as member_check
                                                 FROM polls p
                                                 LEFT JOIN group_members gm ON p.group_id = gm.group_id AND gm.username = ?
                                                 WHERE p.id = ?"
                                            )
                                            .bind(&username_clone)
                                            .bind(poll_id)
                                            .fetch_optional(&pool_incoming)
                                            .await
                                            .unwrap_or(None);
                                            
                                            if let Some(poll_row) = poll_check {
                                                let group_id: i64 = poll_row.get("group_id");
                                                let allow_multiple: bool = poll_row.get("allow_multiple_choices");
                                                let is_active: bool = poll_row.get("is_active");
                                                let question: String = poll_row.get("question");
                                                let member_check: Option<String> = poll_row.try_get("member_check").ok().flatten();
                                                
                                                if member_check.is_none() {
                                                    println!("DEBUG: User not member of group for voting");
                                                    continue;
                                                }
                                                
                                                if !is_active {
                                                    println!("DEBUG: Poll not active");
                                                    continue;
                                                }
                                                
                                                if option_ids.is_empty() || (!allow_multiple && option_ids.len() > 1) {
                                                    println!("DEBUG: Invalid vote options");
                                                    continue;
                                                }
                                                
                                                // Remove existing votes if not allowing multiple choices
                                                if !allow_multiple {
                                                    let _ = sqlx::query("DELETE FROM poll_votes WHERE poll_id = ? AND username = ?")
                                                        .bind(poll_id)
                                                        .bind(&username_clone)
                                                        .execute(&pool_incoming)
                                                        .await;
                                                }
                                                
                                                let voted_at = get_current_time();
                                                
                                                // Add new votes
                                                for option_id in option_ids {
                                                    // Verify option belongs to this poll
                                                    let option_check = sqlx::query("SELECT 1 FROM poll_options WHERE id = ? AND poll_id = ?")
                                                        .bind(option_id)
                                                        .bind(poll_id)
                                                        .fetch_optional(&pool_incoming)
                                                        .await
                                                        .unwrap_or(None);
                                                    
                                                    if option_check.is_some() {
                                                        let _ = sqlx::query("INSERT OR IGNORE INTO poll_votes (poll_id, option_id, username, voted_at) VALUES (?, ?, ?, ?)")
                                                            .bind(poll_id)
                                                            .bind(option_id)
                                                            .bind(&username_clone)
                                                            .bind(&voted_at)
                                                            .execute(&pool_incoming)
                                                            .await;
                                                    }
                                                }
                                                
                                                println!("DEBUG: Vote recorded, broadcasting update");
                                                
                                                // Store the vote update as a group message
                                                let vote_message_text = format!("📊 Poll updated: {} voted on \"{}\"", username_clone, question);
                                                let message_id = store_group_message(
                                                    &pool_incoming,
                                                    group_id,
                                                    &username_clone,
                                                    &vote_message_text,
                                                    &voted_at
                                                ).await.unwrap_or(0);
                                                
                                                // Broadcast poll update
                                                let update_message = ChatMessage {
                                                    id: poll_id, // Use poll_id for consistency
                                                    sender_username: username_clone.clone(),
                                                    receiver_username: "".to_string(),
                                                    group_id: Some(group_id),
                                                    message: vote_message_text,
                                                    timestamp: voted_at,
                                                    reactions: None,
                                                };
                                                
                                                println!("DEBUG: Broadcasting poll vote update to group {}", group_id);
                                                let _ = tx_clone.send(update_message);
                                            }
                                        }
                                    }

                                    "get_poll_details" => {
                                        println!("DEBUG: Getting poll details");
                                        if let Some(poll_id) = incoming_msg.poll_id {
                                            // Get full poll details and send back to requesting user
                                            let poll_details = get_poll_details(&pool_incoming, poll_id, &username_clone).await;
                                            
                                            if let Ok(poll_data) = poll_details {
                                                let response = serde_json::json!({
                                                    "type": "poll_details",
                                                    "poll": poll_data
                                                });
                                                
                                                if let Ok(json) = serde_json::to_string(&response) {
                                                    let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                                                    let _ = ws_tx_lock.send(Message::text(json)).await;
                                                }
                                            }
                                        }
                                    }

                                    "create_game" => {
    println!("DEBUG: Creating game");
    if let Some(game_type) = &incoming_msg.game_type {
        println!("DEBUG: Game type: {}", game_type);
        
        let (conversation_type, conversation_id, target) = if let Some(group_id) = incoming_msg.group_id {
            println!("DEBUG: Creating game in group {}", group_id);
            
            // Verify user is member of group
            let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
                .bind(group_id)
                .bind(&username_clone)
                .fetch_optional(&pool_incoming)
                .await
                .unwrap_or(None);
                
            if member_check.is_none() {
                println!("DEBUG: User {} not member of group {}", username_clone, group_id);
                continue;
            }
            
            ("group".to_string(), Some(group_id), None)
        } else if let Some(target_username) = &incoming_msg.target_username {
            println!("DEBUG: Creating private game with {}", target_username);
            ("private".to_string(), None, Some(target_username.as_str()))
        } else {
            // For cases where no explicit target is provided
            println!("DEBUG: No valid target for game");
            continue;
        };

        // For private games without explicit target, use current conversation
        let actual_target = if conversation_type == "private" && target.is_none() {
            // This should be handled by frontend, but adding safety
            None
        } else {
            target
        };

        match create_game(&pool_incoming, game_type, &username_clone, actual_target, &conversation_type, conversation_id).await {
            Ok(game) => {
                println!("DEBUG: Game created successfully: {:?}", game);
                
                let game_icon = match game_type.as_str() {
                    "chess" => "♟️ Chess",
                    "tictactoe" => "⭕ Tic-Tac-Toe", 
                    "trivia" => "🧠 Trivia",
                    _ => "Game"
                };
                
                let game_message = if game.player2_username.is_some() {
                    format!("🎮 {} game started! Game ID: {}", game_icon, game.id)
                } else {
                    format!("🎮 {} game created! Waiting for players. Game ID: {}", game_icon, game.id)
                };

                let timestamp = get_current_time();
                
                if conversation_type == "group" && conversation_id.is_some() {
                    let message_id = store_group_message(&pool_incoming, conversation_id.unwrap(), &username_clone, &game_message, &timestamp).await.unwrap_or(0);
                    
                    let chat_msg = ChatMessage {
                        id: message_id,
                        sender_username: username_clone.clone(),
                        receiver_username: "".to_string(),
                        group_id: conversation_id,
                        message: game_message,
                        timestamp,
                        reactions: None,
                    };
                    let _ = tx_clone.send(chat_msg);
                } else if let Some(target_user) = actual_target {
                    let message_id = store_message(&pool_incoming, &username_clone, target_user, &game_message, &timestamp).await.unwrap_or(0);
                    
                    let chat_msg = ChatMessage {
                        id: message_id,
                        sender_username: username_clone.clone(),
                        receiver_username: target_user.to_string(),
                        group_id: None,
                        message: game_message,
                        timestamp,
                        reactions: None,
                    };
                    let _ = tx_clone.send(chat_msg);
                }
                
                // Send game creation confirmation
                let game_response = serde_json::json!({
                    "type": "game_created",
                    "game": game
                });

                if let Ok(json) = serde_json::to_string(&game_response) {
                    let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                    let _ = ws_tx_lock.send(Message::text(json)).await;
                }
            }
            Err(e) => {
                println!("DEBUG: Failed to create game: {:?}", e);
                
                let error_response = serde_json::json!({
                    "type": "game_error",
                    "error": format!("Failed to create game: {:?}", e)
                });

                if let Ok(json) = serde_json::to_string(&error_response) {
                    let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                    let _ = ws_tx_lock.send(Message::text(json)).await;
                }
            }
        }
    } else {
        println!("DEBUG: No game type specified");
    }
}
"join_game" => {
    println!("DEBUG: Joining game");
    if let Some(game_id) = incoming_msg.game_id {
        println!("DEBUG: Attempting to join game {}", game_id);
        
        // Check if game exists and is waiting for players
        let game_check = sqlx::query(
            "SELECT id, player1_username, player2_username, status, conversation_type, conversation_id 
             FROM games WHERE id = ?"
        )
        .bind(game_id)
        .fetch_optional(&pool_incoming)
        .await
        .unwrap_or(None);

        if let Some(game_row) = game_check {
            let player1: String = game_row.get("player1_username");
            let player2: Option<String> = game_row.get("player2_username");
            let status: String = game_row.get("status");
            let conv_type: String = game_row.get("conversation_type");
            let conv_id: Option<i64> = game_row.get("conversation_id");

            if player1 == username_clone {
                println!("DEBUG: Player trying to join their own game");
                continue;
            }

            if player2.is_some() {
                println!("DEBUG: Game already has two players");
                continue;
            }

            if status != "waiting" {
                println!("DEBUG: Game is not waiting for players");
                continue;
            }

            // For group games, verify user is member
            if conv_type == "group" && conv_id.is_some() {
                let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
                    .bind(conv_id.unwrap())
                    .bind(&username_clone)
                    .fetch_optional(&pool_incoming)
                    .await
                    .unwrap_or(None);
                    
                if member_check.is_none() {
                    println!("DEBUG: User {} not member of group for game", username_clone);
                    continue;
                }
            }

            let result = sqlx::query(
                "UPDATE games SET player2_username = ?, status = 'active' WHERE id = ? AND player2_username IS NULL"
            )
            .bind(&username_clone)
            .bind(game_id)
            .execute(&pool_incoming)
            .await;

            if result.is_ok() && result.unwrap().rows_affected() > 0 {
                println!("DEBUG: Successfully joined game {}", game_id);
                
                let join_message = format!("🎮 {} joined the game! Game is now active.", username_clone);
                let timestamp = get_current_time();

                if conv_type == "group" && conv_id.is_some() {
                    let message_id = store_group_message(&pool_incoming, conv_id.unwrap(), &username_clone, &join_message, &timestamp).await.unwrap_or(0);
                    
                    let chat_msg = ChatMessage {
                        id: message_id,
                        sender_username: username_clone.clone(),
                        receiver_username: "".to_string(),
                        group_id: conv_id,
                        message: join_message,
                        timestamp,
                        reactions: None,
                    };
                    let _ = tx_clone.send(chat_msg);
                } else {
                    // For private games, send to both players
                    let message_id = store_message(&pool_incoming, &username_clone, &player1, &join_message, &timestamp).await.unwrap_or(0);
                    
                    let chat_msg = ChatMessage {
                        id: message_id,
                        sender_username: username_clone.clone(),
                        receiver_username: player1.clone(),
                        group_id: None,
                        message: join_message,
                        timestamp,
                        reactions: None,
                    };
                    let _ = tx_clone.send(chat_msg);
                }

                // Send updated game state to both players
                if let Ok(updated_game_row) = sqlx::query(
                    "SELECT id, game_type, player1_username, player2_username, game_state, current_turn, status, winner, created_at, conversation_type, conversation_id
                     FROM games WHERE id = ?"
                )
                .bind(game_id)
                .fetch_one(&pool_incoming)
                .await {
                    
                    let updated_game = Game {
                        id: updated_game_row.get("id"),
                        game_type: updated_game_row.get("game_type"),
                        player1_username: updated_game_row.get("player1_username"),
                        player2_username: updated_game_row.get("player2_username"),
                        game_state: updated_game_row.get("game_state"),
                        current_turn: updated_game_row.get("current_turn"),
                        status: updated_game_row.get("status"),
                        winner: updated_game_row.get("winner"),
                        created_at: updated_game_row.get("created_at"),
                        conversation_type: updated_game_row.get("conversation_type"),
                        conversation_id: updated_game_row.get("conversation_id"),
                    };

                    let game_update = serde_json::json!({
                        "type": "game_joined",
                        "game": updated_game
                    });

                    if let Ok(json) = serde_json::to_string(&game_update) {
                        let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                        let _ = ws_tx_lock.send(Message::text(json)).await;
                    }
                }
            } else {
                println!("DEBUG: Failed to join game {}", game_id);
            }
        } else {
            println!("DEBUG: Game {} not found", game_id);
        }
    }
}

"game_move" => {
    println!("DEBUG: Processing game move");
    if let (Some(game_id), Some(move_data)) = (incoming_msg.game_id, &incoming_msg.game_move) {
        println!("DEBUG: Game ID: {}, Move: {}", game_id, move_data);
        
        match process_game_move(&pool_incoming, game_id, &username_clone, move_data).await {
            Ok(updated_game) => {
                println!("DEBUG: Game move processed successfully");
                
                let game_icon = match updated_game.game_type.as_str() {
                    "chess" => "♟️ Chess",
                    "tictactoe" => "⭕ Tic-Tac-Toe", 
                    "trivia" => "🧠 Trivia",
                    _ => "Game"
                };
                
                let move_message = if updated_game.status == "finished" {
                    if let Some(ref winner) = updated_game.winner {
                        if winner == "draw" {
                            format!("🎮 {} game #{} ended in a draw!", game_icon, game_id)
                        } else {
                            format!("🎮 {} game #{} finished! 🏆 {} wins!", game_icon, game_id, winner)
                        }
                    } else {
                        format!("🎮 {} game #{} finished!", game_icon, game_id)
                    }
                } else {
                    format!("🎮 {} made a move in {} game #{}", username_clone, game_icon, game_id)
                };

                let timestamp = get_current_time();
                
                if updated_game.conversation_type == "group" && updated_game.conversation_id.is_some() {
                    let message_id = store_group_message(&pool_incoming, updated_game.conversation_id.unwrap(), &username_clone, &move_message, &timestamp).await.unwrap_or(0);
                    
                    let chat_msg = ChatMessage {
                        id: message_id,
                        sender_username: username_clone.clone(),
                        receiver_username: "".to_string(),
                        group_id: updated_game.conversation_id,
                        message: move_message,
                        timestamp,
                        reactions: None,
                    };
                    let _ = tx_clone.send(chat_msg);
                } else {
                    // For private games, send to the other player
                    let other_player = if updated_game.player1_username == username_clone {
                        updated_game.player2_username.as_ref().unwrap_or(&updated_game.player1_username)
                    } else {
                        &updated_game.player1_username
                    };
                    
                    let message_id = store_message(&pool_incoming, &username_clone, other_player, &move_message, &timestamp).await.unwrap_or(0);
                    
                    let chat_msg = ChatMessage {
                        id: message_id,
                        sender_username: username_clone.clone(),
                        receiver_username: other_player.clone(),
                        group_id: None,
                        message: move_message,
                        timestamp,
                        reactions: None,
                    };
                    let _ = tx_clone.send(chat_msg);
                }

                // Send game state update
                let game_update = serde_json::json!({
                    "type": "game_update",
                    "game": updated_game
                });

                if let Ok(json) = serde_json::to_string(&game_update) {
                    let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                    let _ = ws_tx_lock.send(Message::text(json)).await;
                }
            }
            Err(e) => {
                println!("DEBUG: Game move failed: {}", e);
                
                let error_response = serde_json::json!({
                    "type": "game_error",
                    "error": e
                });

                if let Ok(json) = serde_json::to_string(&error_response) {
                    let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                    let _ = ws_tx_lock.send(Message::text(json)).await;
                }
            }
        }
    } else {
        println!("DEBUG: Invalid game move request");
    }
}

"get_game_state" => {
    println!("DEBUG: Getting game state");
    if let Some(game_id) = incoming_msg.game_id {
        println!("DEBUG: Requesting state for game {}", game_id);
        
        let game_row = sqlx::query(
            "SELECT id, game_type, player1_username, player2_username, game_state, current_turn, status, winner, created_at, conversation_type, conversation_id
             FROM games WHERE id = ?"
        )
        .bind(game_id)
        .fetch_optional(&pool_incoming)
        .await
        .unwrap_or(None);

        if let Some(row) = game_row {
            // Verify user is involved in this game
            let player1: String = row.get("player1_username");
            let player2: Option<String> = row.get("player2_username");
            let conv_type: String = row.get("conversation_type");
            let conv_id: Option<i64> = row.get("conversation_id");

            let mut can_access = false;

            // Check if user is a player
            if player1 == username_clone || player2.as_ref() == Some(&username_clone) {
                can_access = true;
            }

            // For group games, check if user is a member
            if conv_type == "group" && conv_id.is_some() {
                let member_check = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
                    .bind(conv_id.unwrap())
                    .bind(&username_clone)
                    .fetch_optional(&pool_incoming)
                    .await
                    .unwrap_or(None);
                    
                if member_check.is_some() {
                    can_access = true;
                }
            }

            if can_access {
                let game = Game {
                    id: row.get("id"),
                    game_type: row.get("game_type"),
                    player1_username: row.get("player1_username"),
                    player2_username: row.get("player2_username"),
                    game_state: row.get("game_state"),
                    current_turn: row.get("current_turn"),
                    status: row.get("status"),
                    winner: row.get("winner"),
                    created_at: row.get("created_at"),
                    conversation_type: row.get("conversation_type"),
                    conversation_id: row.get("conversation_id"),
                };

                let game_response = serde_json::json!({
                    "type": "game_state",
                    "game": game
                });

                if let Ok(json) = serde_json::to_string(&game_response) {
                    let mut ws_tx_lock = ws_tx_for_incoming.lock().await;
                    let _ = ws_tx_lock.send(Message::text(json)).await;
                }
            } else {
                println!("DEBUG: User {} cannot access game {}", username_clone, game_id);
            }
        } else {
            println!("DEBUG: Game {} not found", game_id);
        }
    }
}

                                    // Other message types
                                    _ => {
                                        println!("DEBUG: Unhandled message type: {}", incoming_msg.message_type);
                                    }
                                }
                            }
                            Err(parse_error) => {
                                println!("DEBUG: Failed to parse WebSocket message as JSON: {:?}", parse_error);
                                println!("DEBUG: Raw message was: {}", text);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("DEBUG: WebSocket error: {:?}", e);
                    break;
                }
            }
        }
        println!("DEBUG: WebSocket incoming task ended for user: {}", username_clone);
    });

    // Clone pool and users for outgoing task
    let pool_outgoing = pool.clone();
    let username_outgoing = username.clone();
    let ws_tx_outgoing = ws_tx_clone.clone();

    let outgoing_task = tokio::spawn(async move {
        println!("DEBUG: Started outgoing task for user: {}", username_outgoing);
        while let Ok(msg) = rx.recv().await {
            println!("DEBUG: Received broadcast message for user {}: {:?}", username_outgoing, msg);
            let mut send_to_user = false;

            // Check if it's a group message first
            if let Some(group_id) = msg.group_id {
                println!("DEBUG: Checking if {} is member of group {}", username_outgoing, group_id);
                let row = sqlx::query("SELECT 1 FROM group_members WHERE group_id = ? AND username = ?")
                    .bind(group_id)
                    .bind(&username_outgoing)
                    .fetch_optional(&pool_outgoing)
                    .await
                    .unwrap_or(None);
                if row.is_some() { 
                    send_to_user = true; 
                    println!("DEBUG: User {} is member of group {} - sending group message", username_outgoing, group_id);
                } else {
                    println!("DEBUG: User {} is NOT member of group {}", username_outgoing, group_id);
                }
            }
            // Only check for 1:1 message if it's NOT a group message (group_id is None)
            else if msg.group_id.is_none() && (msg.sender_username == username_outgoing || msg.receiver_username == username_outgoing) {
                send_to_user = true;
                println!("DEBUG: Message is for private chat with {}", username_outgoing);
            }

            if send_to_user {
                if let Ok(json) = serde_json::to_string(&msg) {
                    println!("DEBUG: Sending message to {}: {}", username_outgoing, json);
                    let mut ws_tx_lock = ws_tx_outgoing.lock().await;
                    if ws_tx_lock.send(Message::text(json)).await.is_err() {
                        println!("DEBUG: Failed to send WebSocket message to {}", username_outgoing);
                        break;
                    } else {
                        println!("DEBUG: Successfully sent WebSocket message to {}", username_outgoing);
                    }
                }
            } else {
                println!("DEBUG: Message not for user {}: sender={}, receiver={}, group_id={:?}", 
                    username_outgoing, msg.sender_username, msg.receiver_username, msg.group_id);
            }
        }
        println!("DEBUG: WebSocket outgoing task ended for user: {}", username_outgoing);
    });

    tokio::select! {
        _ = incoming_task => {
            println!("DEBUG: Incoming task completed for {}", username);
        },
        _ = outgoing_task => {
            println!("DEBUG: Outgoing task completed for {}", username);
        },
    }

    // Remove user from users map
    {
        let mut users_lock = users.lock().await;
        users_lock.remove(&connection_id);
        println!("DEBUG: Removed user {} from connections. Remaining: {}", username, users_lock.len());
    }
}

// Helper function to get poll details
async fn get_poll_details(
    pool: &SqlitePool, 
    poll_id: i64, 
    current_username: &str
) -> Result<serde_json::Value, sqlx::Error> {
    // Get poll info
    let poll_row = sqlx::query(
        "SELECT id, group_id, creator_username, question, created_at, expires_at, is_active, allow_multiple_choices 
         FROM polls WHERE id = ?"
    )
    .bind(poll_id)
    .fetch_one(pool)
    .await?;

    // Get poll options with vote counts
    let options_rows = sqlx::query(
        "SELECT po.id, po.option_text, 
                COUNT(pv.id) as vote_count,
                CASE WHEN user_votes.option_id IS NOT NULL THEN 1 ELSE 0 END as voted_by_current_user
         FROM poll_options po
         LEFT JOIN poll_votes pv ON po.id = pv.option_id
         LEFT JOIN poll_votes user_votes ON po.id = user_votes.option_id AND user_votes.username = ?
         WHERE po.poll_id = ?
         GROUP BY po.id, po.option_text, po.option_order
         ORDER BY po.option_order"
    )
    .bind(current_username)
    .bind(poll_id)
    .fetch_all(pool)
    .await?;

    let mut options = Vec::new();
    let mut total_votes = 0i64;

    for row in options_rows {
        let vote_count: i64 = row.get("vote_count");
        total_votes += vote_count;

        options.push(serde_json::json!({
            "id": row.get::<i64, _>("id"),
            "option_text": row.get::<String, _>("option_text"),
            "vote_count": vote_count,
            "voted_by_current_user": row.get::<i64, _>("voted_by_current_user") == 1
        }));
    }

    Ok(serde_json::json!({
        "id": poll_row.get::<i64, _>("id"),
        "group_id": poll_row.get::<i64, _>("group_id"),
        "creator_username": poll_row.get::<String, _>("creator_username"),
        "question": poll_row.get::<String, _>("question"),
        "created_at": poll_row.get::<String, _>("created_at"),
        "expires_at": poll_row.get::<Option<String>, _>("expires_at"),
        "is_active": poll_row.get::<bool, _>("is_active"),
        "allow_multiple_choices": poll_row.get::<bool, _>("allow_multiple_choices"),
        "options": options,
        "total_votes": total_votes
    }))
}

fn get_current_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

async fn get_group_conversation_messages(pool: &SqlitePool, group_id: i64, limit: i32) -> Vec<ChatMessage> {
    let rows = sqlx::query(
        "SELECT id, sender_username, message, timestamp FROM group_messages 
         WHERE group_id = ? ORDER BY id DESC LIMIT ?"
    )
    .bind(group_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut messages: Vec<ChatMessage> = Vec::new();
    for row in rows {
        messages.push(ChatMessage {
            id: row.get("id"),
            group_id: Some(group_id),
            sender_username: row.get("sender_username"),
            receiver_username: "".to_string(), // Empty for group messages
            message: row.get("message"),
            timestamp: row.get("timestamp"),
            reactions: None, // Could be enhanced later
        });
    }

    // Reverse to get chronological order (oldest first)
    messages.reverse();
    messages
}

async fn store_group_message(
    pool: &SqlitePool,
    group_id: i64,
    sender_username: &str,
    message: &str,
    timestamp: &str,
) -> Result<i64, sqlx::Error> {
    println!("DEBUG: Storing group message in database");
    let rec = sqlx::query(
        "INSERT INTO group_messages (group_id, sender_username, message, timestamp)
         VALUES (?, ?, ?, ?)"
    )
    .bind(group_id)
    .bind(sender_username)
    .bind(message)
    .bind(timestamp)
    .execute(pool)
    .await?;

    let message_id = rec.last_insert_rowid();
    println!("DEBUG: Group message stored with ID: {}", message_id);
    Ok(message_id)
}


async fn generate_highlights_handler(
    request: HighlightRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid or expired token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    // We'll ignore date ranges now and just use "recent" for everything
    let start_date = "recent".to_string();
    let end_date = "recent".to_string();

    // Handle different target types
    let highlights = match request.target_type.as_str() {
        "personal" => {
    if let Some(specific_user) = request.specific_user {
        // Generate highlights for specific user only
        generate_specific_user_highlights(&pool, &username, &specific_user, &request.highlight_type).await
            .map_err(|_| warp::reject::reject())?
    } else {
        // Generate for all personal chats
        generate_personal_highlights(&pool, &username, &request.highlight_type, &start_date, &end_date).await
            .map_err(|_| warp::reject::reject())?
    }
}
        "group" => {
            if let Some(group_id) = request.target_id {
                // Generate for specific group - we'll need to create this function
                vec![generate_single_group_highlight(&pool, group_id, &username, &request.highlight_type).await
                    .map_err(|_| warp::reject::reject())?]
            } else {
                // Generate for all groups
                generate_all_group_highlights(&pool, &username, &request.highlight_type, &start_date, &end_date).await
                    .map_err(|_| warp::reject::reject())?
            }
        }
        "all" | _ => {
            // Generate both personal and group highlights
            let mut all_highlights = Vec::new();
            
            let personal = generate_personal_highlights(&pool, &username, &request.highlight_type, &start_date, &end_date).await
                .map_err(|_| warp::reject::reject())?;
            all_highlights.extend(personal);
            
            let groups = generate_all_group_highlights(&pool, &username, &request.highlight_type, &start_date, &end_date).await
                .map_err(|_| warp::reject::reject())?;
            all_highlights.extend(groups);
            
            all_highlights
        }
    };
    
    let highlight_count = highlights.len();
    Ok(warp::reply::with_status(
        warp::reply::json(&HighlightResponse {
            highlights,
            period: format!("Recent {} messages", if highlight_count == 0 { "0" } else { "500" }),
            total_messages: highlight_count as i64,
        }),
        warp::http::StatusCode::OK,
    ))
}

async fn get_highlights_handler(
    query_params: HashMap<String, String>,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid or expired token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let limit: i32 = query_params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(10);
    let rows = sqlx::query("SELECT * FROM highlights WHERE user_username = ? ORDER BY created_at DESC LIMIT ?")
        .bind(&username).bind(limit).fetch_all(&pool).await.map_err(|_| warp::reject::reject())?;

    let highlights: Vec<Highlight> = rows.into_iter().map(|row| {
        let key_topics_json: String = row.get("key_topics");
        let key_topics: Vec<String> = serde_json::from_str(&key_topics_json).unwrap_or_default();

        Highlight {
            id: row.get("id"),
            user_username: row.get("user_username"),
            target_type: row.get("target_type"),
            target_id: row.get("target_id"),
            target_name: row.get("target_name"),
            highlight_type: row.get("highlight_type"),
            summary: row.get("summary"),
            key_topics,
            message_count: row.get("message_count"),
            participant_count: row.get("participant_count"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            created_at: row.get("created_at"),
        }
    }).collect();

    Ok(warp::reply::with_status(
        warp::reply::json(&HighlightResponse {
            highlights,
            period: format!("Last {} highlights", limit),
            total_messages: 0,
        }),
        warp::http::StatusCode::OK,
    ))
}

fn get_daily_range() -> (String, String) {
    let now = chrono::Utc::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap();
    (start.format("%Y-%m-%d %H:%M:%S").to_string(), end.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn get_weekly_range() -> (String, String) {
    let now = chrono::Utc::now();
    let days_since_monday = now.weekday().num_days_from_monday();
    let start_of_week = now.date_naive() - chrono::Duration::days(days_since_monday as i64);
    let end_of_week = start_of_week + chrono::Duration::days(6);
    (start_of_week.and_hms_opt(0, 0, 0).unwrap().format("%Y-%m-%d %H:%M:%S").to_string(), end_of_week.and_hms_opt(23, 59, 59).unwrap().format("%Y-%m-%d %H:%M:%S").to_string())
}

async fn generate_personal_highlights(
    pool: &SqlitePool,
    username: &str,
    highlight_type: &str,
    _start_date: &str,  // We'll ignore these parameters now
    _end_date: &str,
) -> Result<Vec<Highlight>, sqlx::Error> {
    // Get last 500 messages for this user
    let conversations = sqlx::query(
        "SELECT 
            CASE WHEN sender_username = ? THEN receiver_username ELSE sender_username END as other_user,
            COUNT(*) as message_count 
         FROM (
             SELECT sender_username, receiver_username 
             FROM messages 
             WHERE sender_username = ? OR receiver_username = ? 
             ORDER BY id DESC 
             LIMIT 500
         ) recent_messages
         GROUP BY other_user 
         HAVING message_count > 0 
         ORDER BY message_count DESC"
    )
    .bind(username).bind(username).bind(username)
    .fetch_all(pool).await?;

    let mut highlights = Vec::new();
    for conv_row in conversations {
        let other_user: String = conv_row.get("other_user");
        let msg_count: i64 = conv_row.get("message_count");

        // Get actual messages for this conversation from the last 500
        let messages = sqlx::query(
            "SELECT sender_username, message, timestamp 
             FROM (
                 SELECT sender_username, receiver_username, message, timestamp
                 FROM messages 
                 WHERE ((sender_username = ? AND receiver_username = ?) 
                        OR (sender_username = ? AND receiver_username = ?))
                 ORDER BY id DESC 
                 LIMIT 100
             ) conversation_messages
             ORDER BY timestamp ASC"
        )
        .bind(username).bind(&other_user)
        .bind(&other_user).bind(username)
        .fetch_all(pool).await?;

        if !messages.is_empty() {
            let (summary, key_topics) = generate_rule_based_summary(&messages, &other_user, highlight_type).await;
            let now = get_current_time();

            let highlight_id = sqlx::query(
                "INSERT INTO highlights (user_username, target_type, target_id, target_name, highlight_type, summary, key_topics, message_count, participant_count, start_date, end_date, created_at) 
                 VALUES (?, 'personal', NULL, ?, ?, ?, ?, ?, 2, ?, ?, ?)"
            )
            .bind(username).bind(&other_user).bind(highlight_type).bind(&summary)
            .bind(&serde_json::to_string(&key_topics).unwrap()).bind(msg_count)
            .bind("recent").bind("recent").bind(&now)
            .execute(pool).await?.last_insert_rowid();

            highlights.push(Highlight {
                id: highlight_id,
                user_username: username.to_string(),
                target_type: "personal".to_string(),
                target_id: None,
                target_name: other_user,
                highlight_type: highlight_type.to_string(),
                summary,
                key_topics,
                message_count: msg_count,
                participant_count: 2,
                start_date: "recent".to_string(),
                end_date: "recent".to_string(),
                created_at: now,
            });
        }
    }
    Ok(highlights)
}

async fn generate_rule_based_summary(
    messages: &[sqlx::sqlite::SqliteRow],
    target_name: &str,
    period: &str,
) -> (String, Vec<String>) {
    if messages.is_empty() {
        return (format!("No activity in {} during this {} period.", target_name, period), vec![]);
    }

    let message_count = messages.len();
    let mut all_text = String::new();
    let mut participants = std::collections::HashSet::new();

    for row in messages {
        let sender: String = row.get("sender_username");
        let message: String = row.get("message");
        participants.insert(sender);
        all_text.push_str(&format!("{} ", message));
    }

    let key_topics = extract_enhanced_topics(&all_text);
    let activity_level = if message_count < 5 { "light" } else if message_count < 20 { "moderate" } else { "active" };

    let summary = format!(
        "During this {} period, {} had {} messages with {} discussion involving {} participant{}.",
        period, target_name, message_count, activity_level, participants.len(),
        if participants.len() == 1 { "" } else { "s" }
    );

    (summary, key_topics)
}

fn extract_enhanced_topics(text: &str) -> Vec<String> {
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    
    let words: Vec<String> = text.split_whitespace()
        .map(|w| clean_word(w))
        .filter(|w| w.len() > 2 && !is_stop_word(w))
        .collect();
    
    for word in &words {
        *word_freq.entry(word.clone()).or_insert(0) += 1;
    }
    
    let mut words_sorted: Vec<_> = word_freq.iter().collect();
    words_sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    words_sorted.iter().take(5).filter(|(_, count)| **count >= 2).map(|(word, _)| word.to_string()).collect()
}

fn clean_word(word: &str) -> String {
    word.chars().filter(|c| c.is_alphabetic()).collect::<String>().to_lowercase()
}

fn is_stop_word(word: &str) -> bool {
    matches!(word, "the" | "and" | "or" | "but" | "in" | "on" | "at" | "to" | "for" | "of" | "with" | "by" | "from" | "this" | "that" | "you" | "he" | "she" | "it" | "we" | "they" | "me" | "him" | "her" | "us" | "them" | "my" | "your" | "his"  | "its" | "our" | "their" | "am" | "is" | "are" | "was" | "were" | "be" | "been" | "being" | "have" | "has" | "had" | "do" | "does" | "did" | "will" | "would" | "could" | "should" | "may" | "might" | "must" | "can" | "shall")
}


async fn generate_group_highlight(
    pool: &SqlitePool,
    group_id: i64,
    username: &str,
    highlight_type: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Highlight, sqlx::Error> {
    let group_row = sqlx::query("SELECT name FROM groups WHERE id = ?").bind(group_id).fetch_one(pool).await?;
    let group_name: String = group_row.get("name");

    let messages = sqlx::query("SELECT sender_username, message, timestamp FROM group_messages WHERE group_id = ? AND timestamp BETWEEN ? AND ? ORDER BY timestamp ASC")
        .bind(group_id).bind(start_date).bind(end_date).fetch_all(pool).await?;

    let participants: std::collections::HashSet<String> = messages.iter().map(|row| row.get::<String, _>("sender_username")).collect();
    let message_count = messages.len() as i64;
    let participant_count = participants.len() as i64;

    let (summary, key_topics) = generate_rule_based_summary(&messages, &group_name, highlight_type).await;

    let highlight_id = sqlx::query("INSERT INTO highlights (user_username, target_type, target_id, target_name, highlight_type, summary, key_topics, message_count, participant_count, start_date, end_date, created_at) VALUES (?, 'group', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(username).bind(group_id).bind(&group_name).bind(highlight_type).bind(&summary).bind(&serde_json::to_string(&key_topics).unwrap()).bind(message_count).bind(participant_count).bind(start_date).bind(end_date).bind(&get_current_time())
        .execute(pool).await?.last_insert_rowid();

    Ok(Highlight {
        id: highlight_id,
        user_username: username.to_string(),
        target_type: "group".to_string(),
        target_id: Some(group_id),
        target_name: group_name,
        highlight_type: highlight_type.to_string(),
        summary,
        key_topics,
        message_count,
        participant_count,
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        created_at: get_current_time(),
    })
}

async fn generate_all_group_highlights(
    pool: &SqlitePool,
    username: &str,
    highlight_type: &str,
    _start_date: &str,
    _end_date: &str,
) -> Result<Vec<Highlight>, sqlx::Error> {
    let groups = sqlx::query("SELECT g.id, g.name FROM groups g INNER JOIN group_members gm ON g.id = gm.group_id WHERE gm.username = ?")
        .bind(username).fetch_all(pool).await?;

    let mut highlights = Vec::new();
    for group_row in groups {
        let group_id: i64 = group_row.get("id");
        let group_name: String = group_row.get("name");
        
        // Get last 200 messages for this group
        let messages = sqlx::query(
            "SELECT sender_username, message, timestamp 
             FROM group_messages 
             WHERE group_id = ? 
             ORDER BY id DESC 
             LIMIT 200"
        )
        .bind(group_id).fetch_all(pool).await?;

        if !messages.is_empty() {
            let participants: std::collections::HashSet<String> = messages.iter()
                .map(|row| row.get::<String, _>("sender_username")).collect();
            let message_count = messages.len() as i64;
            let participant_count = participants.len() as i64;

            let (summary, key_topics) = generate_rule_based_summary(&messages, &group_name, highlight_type).await;
            let now = get_current_time();

            let highlight_id = sqlx::query(
                "INSERT INTO highlights (user_username, target_type, target_id, target_name, highlight_type, summary, key_topics, message_count, participant_count, start_date, end_date, created_at) 
                 VALUES (?, 'group', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(username).bind(group_id).bind(&group_name).bind(highlight_type)
            .bind(&summary).bind(&serde_json::to_string(&key_topics).unwrap())
            .bind(message_count).bind(participant_count)
            .bind("recent").bind("recent").bind(&now)
            .execute(pool).await?.last_insert_rowid();

            highlights.push(Highlight {
                id: highlight_id,
                user_username: username.to_string(),
                target_type: "group".to_string(),
                target_id: Some(group_id),
                target_name: group_name,
                highlight_type: highlight_type.to_string(),
                summary,
                key_topics,
                message_count,
                participant_count,
                start_date: "recent".to_string(),
                end_date: "recent".to_string(),
                created_at: now,
            });
        }
    }
    Ok(highlights)
}

async fn generate_single_group_highlight(
    pool: &SqlitePool,
    group_id: i64,
    username: &str,
    highlight_type: &str,
) -> Result<Highlight, sqlx::Error> {
    let group_row = sqlx::query("SELECT name FROM groups WHERE id = ?").bind(group_id).fetch_one(pool).await?;
    let group_name: String = group_row.get("name");
    
    // Get last 200 messages for this specific group
    let messages = sqlx::query(
        "SELECT sender_username, message, timestamp 
         FROM group_messages 
         WHERE group_id = ? 
         ORDER BY id DESC 
         LIMIT 200"
    )
    .bind(group_id).fetch_all(pool).await?;

    let participants: std::collections::HashSet<String> = messages.iter()
        .map(|row| row.get::<String, _>("sender_username")).collect();
    let message_count = messages.len() as i64;
    let participant_count = participants.len() as i64;

    let (summary, key_topics) = generate_rule_based_summary(&messages, &group_name, highlight_type).await;
    let now = get_current_time();

    let highlight_id = sqlx::query(
        "INSERT INTO highlights (user_username, target_type, target_id, target_name, highlight_type, summary, key_topics, message_count, participant_count, start_date, end_date, created_at) 
         VALUES (?, 'group', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(username).bind(group_id).bind(&group_name).bind(highlight_type)
    .bind(&summary).bind(&serde_json::to_string(&key_topics).unwrap())
    .bind(message_count).bind(participant_count)
    .bind("recent").bind("recent").bind(&now)
    .execute(pool).await?.last_insert_rowid();

    Ok(Highlight {
        id: highlight_id,
        user_username: username.to_string(),
        target_type: "group".to_string(),
        target_id: Some(group_id),
        target_name: group_name,
        highlight_type: highlight_type.to_string(),
        summary,
        key_topics,
        message_count,
        participant_count,
        start_date: "recent".to_string(),
        end_date: "recent".to_string(),
        created_at: now,
    })
}

async fn debug_messages_handler(
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ErrorResponse {
                    error: "Invalid token".to_string(),
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let messages = sqlx::query("SELECT sender_username, receiver_username, message, timestamp FROM messages WHERE sender_username = ? OR receiver_username = ? ORDER BY id DESC LIMIT 10")
        .bind(&username).bind(&username).fetch_all(&pool).await.map_err(|_| warp::reject::reject())?;

    let debug_info = serde_json::json!({
        "user": username,
        "recent_messages": messages.iter().map(|row| {
            serde_json::json!({
                "sender": row.get::<String, _>("sender_username"),
                "receiver": row.get::<String, _>("receiver_username"),
                "message": row.get::<String, _>("message"),
                "timestamp": row.get::<String, _>("timestamp")
            })
        }).collect::<Vec<_>>(),
        "total_count": messages.len()
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&debug_info),
        warp::http::StatusCode::OK,
    ))
}

async fn generate_specific_user_highlights(
    pool: &SqlitePool,
    username: &str,
    target_user: &str,
    highlight_type: &str,
) -> Result<Vec<Highlight>, sqlx::Error> {
    // Get last 200 messages between these two users specifically
    let messages = sqlx::query(
        "SELECT sender_username, message, timestamp 
         FROM messages 
         WHERE ((sender_username = ? AND receiver_username = ?) 
                OR (sender_username = ? AND receiver_username = ?))
         ORDER BY id DESC 
         LIMIT 200"
    )
    .bind(username).bind(target_user)
    .bind(target_user).bind(username)
    .fetch_all(pool).await?;

    if messages.is_empty() {
        return Ok(vec![]);
    }

    let message_count = messages.len() as i64;
    let (summary, key_topics) = generate_rule_based_summary(&messages, target_user, highlight_type).await;
    let now = get_current_time();

    let highlight_id = sqlx::query(
        "INSERT INTO highlights (user_username, target_type, target_id, target_name, highlight_type, summary, key_topics, message_count, participant_count, start_date, end_date, created_at) 
         VALUES (?, 'personal', NULL, ?, ?, ?, ?, ?, 2, ?, ?, ?)"
    )
    .bind(username).bind(target_user).bind(highlight_type).bind(&summary)
    .bind(&serde_json::to_string(&key_topics).unwrap()).bind(message_count)
    .bind("recent").bind("recent").bind(&now)
    .execute(pool).await?.last_insert_rowid();

    Ok(vec![Highlight {
        id: highlight_id,
        user_username: username.to_string(),
        target_type: "personal".to_string(),
        target_id: None,
        target_name: target_user.to_string(),
        highlight_type: highlight_type.to_string(),
        summary,
        key_topics,
        message_count,
        participant_count: 2,
        start_date: "recent".to_string(),
        end_date: "recent".to_string(),
        created_at: now,
    }])
}

async fn ai_assistant_handler(
    request: AIAssistantRequest,
    auth_header: String,
    pool: SqlitePool,
) -> Result<impl Reply, warp::Rejection> {
    let username = match extract_username_from_auth(auth_header) {
        Ok(u) => u,
        Err(_) => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&AIAssistantResponse {
                    response: "Authentication required".to_string(),
                    query_type: "error".to_string(),
                    success: false,
                }),
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let response = process_ai_query_with_gemini(&pool, &username, &request).await;
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK,
    ))
}

async fn process_ai_query_with_gemini(
    pool: &SqlitePool,
    username: &str,
    request: &AIAssistantRequest,
) -> AIAssistantResponse {
    let query_lower = request.query.to_lowercase();
    
    // Handle simple queries locally first
    if query_lower.contains("help") || query_lower.contains("what can you do") {
        return AIAssistantResponse {
            response: "I can help you with:\n\n• Get conversation summaries (e.g., 'summarize my chat with admin1')\n• Show recent activity across all your chats\n• List your most active groups\n• Answer questions about your messaging history\n\nJust ask me in natural language!".to_string(),
            query_type: "help".to_string(),
            success: true,
        };
    }

    // Get API key from environment
    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            return AIAssistantResponse {
                response: "API configuration error. Please contact administrator.".to_string(),
                query_type: "error".to_string(),
                success: false,
            };
        }
    };

    // For complex queries, gather context and use Gemini
    let context = gather_user_context(pool, username, request).await;
    
    match call_gemini_api(&request.query, &context, &api_key).await {
        Ok(response) => AIAssistantResponse {
            response,
            query_type: "gemini_response".to_string(),
            success: true,
        },
        Err(error) => {
            println!("Gemini API error: {}", error);
            // Fallback to local processing
            fallback_local_response(pool, username, request).await
        }
    }
}


async fn call_gemini_api(user_query: &str, context: &str, api_key: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let system_prompt = format!(
        "You are an AI assistant that analyzes conversations. The user is asking: '{}'

Here is the conversation data:
{}

Based on the actual messages shown above, provide a natural summary of what was discussed. Focus on:
- Main topics and subjects that were talked about
- Key points or decisions made
- The general nature of the conversation

Be specific about the actual content discussed, not just metadata like message counts. If they talked about tariffs, mention tariffs. If they discussed work projects, mention the projects. Keep the summary conversational and informative.",
        user_query, context
    );

    let gemini_request = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: system_prompt,
            }],
        }],
        generation_config: GeminiGenerationConfig {
            temperature: 0.7,
            top_k: 40,
            top_p: 0.95,
            max_output_tokens: 1024,
        },
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&gemini_request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API error: {}", error_text));
    }

    let gemini_response: GeminiResponse = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(candidate) = gemini_response.candidates.into_iter().next() {
        if let Some(part) = candidate.content.parts.into_iter().next() {
            return Ok(part.text);
        }
    }

    Err("No response from Gemini".to_string())
}


async fn gather_user_context(
    pool: &SqlitePool,
    username: &str,
    request: &AIAssistantRequest,
) -> String {
    let mut context = format!("User: {}\n\n", username);
    
    // Get recent conversations
    let recent_conversations = sqlx::query(
        "SELECT 
            CASE WHEN sender_username = ? THEN receiver_username ELSE sender_username END as other_user,
            COUNT(*) as message_count
         FROM (
             SELECT sender_username, receiver_username
             FROM messages 
             WHERE sender_username = ? OR receiver_username = ? 
             ORDER BY id DESC 
             LIMIT 50
         ) recent_messages
         GROUP BY other_user 
         ORDER BY message_count DESC
         LIMIT 5"
    )
    .bind(username).bind(username).bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if !recent_conversations.is_empty() {
        context.push_str("Recent Conversations:\n");
        for conv in recent_conversations {
            let other_user: String = conv.get("other_user");
            let count: i64 = conv.get("message_count");
            context.push_str(&format!("- {}: {} messages\n", other_user, count));
        }
        context.push('\n');
    }

    // If query mentions specific person, get their conversation
    if let Some(target) = extract_target_from_query(&request.query) {
        if let Some(conversation_summary) = get_specific_conversation_context(pool, username, &target).await {
            context.push_str(&format!("Conversation with {}:\n{}\n", target, conversation_summary));
        }
    }

    context
}

async fn get_specific_conversation_context(
    pool: &SqlitePool,
    username: &str,
    target: &str,
) -> Option<String> {
    let personal_messages = sqlx::query(
        "SELECT sender_username, message, timestamp 
         FROM messages 
         WHERE ((sender_username = ? AND receiver_username = ?) 
                OR (sender_username = ? AND receiver_username = ?))
         ORDER BY id DESC 
         LIMIT 30"
    )
    .bind(username).bind(target)
    .bind(target).bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if !personal_messages.is_empty() {
        let mut conversation_content = String::new();
        conversation_content.push_str(&format!("Conversation between {} and {}:\n\n", username, target));
        
        // Include actual message content for analysis
        for msg in personal_messages.iter().rev() { // Reverse to show chronological order
            let sender: String = msg.get("sender_username");
            let message: String = msg.get("message");
            let timestamp: String = msg.get("timestamp");
            
            conversation_content.push_str(&format!("[{}] {}: {}\n", timestamp, sender, message));
        }
        
        return Some(conversation_content);
    }

    None
}

async fn fallback_local_response(
    pool: &SqlitePool,
    username: &str,
    request: &AIAssistantRequest,
) -> AIAssistantResponse {
    let query_lower = request.query.to_lowercase();
    
    if query_lower.contains("summary") || query_lower.contains("summarize") {
        if let Some(target) = extract_target_from_query(&request.query) {
            return generate_conversation_summary(pool, username, &target).await;
        }
    }
    
    if query_lower.contains("recent") || query_lower.contains("activity") {
        return get_recent_activity_summary(pool, username).await;
    }
    
    AIAssistantResponse {
        response: "I'm having trouble with that request. Try asking for conversation summaries or recent activity.".to_string(),
        query_type: "fallback".to_string(),
        success: false,
    }
}


// Add these missing functions to your main.rs (you can add them at the end before the closing brace)

fn extract_target_from_query(query: &str) -> Option<String> {
    let words: Vec<&str> = query.split_whitespace().collect();
    
    // Look for patterns like "summarize my chat with admin1" or "summary of admin1"
    for (i, word) in words.iter().enumerate() {
        if word.to_lowercase() == "with" && i + 1 < words.len() {
            return Some(words[i + 1].to_string());
        }
        if word.to_lowercase() == "of" && i + 1 < words.len() {
            return Some(words[i + 1].to_string());
        }
    }
    
    // Look for the last word that might be a username
    if let Some(last_word) = words.last() {
        if last_word.len() > 2 && !matches!(last_word.to_lowercase().as_str(), "chat" | "conversation" | "messages") {
            return Some(last_word.to_string());
        }
    }
    
    None
}

async fn generate_conversation_summary(
    pool: &SqlitePool,
    username: &str,
    target_name: &str,
) -> AIAssistantResponse {
    // First check if it's a personal conversation
    let personal_messages = sqlx::query(
        "SELECT sender_username, message, timestamp 
         FROM messages 
         WHERE ((sender_username = ? AND receiver_username = ?) 
                OR (sender_username = ? AND receiver_username = ?))
         ORDER BY id DESC 
         LIMIT 50"
    )
    .bind(username).bind(target_name)
    .bind(target_name).bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if !personal_messages.is_empty() {
        let (summary, _topics) = generate_rule_based_summary(&personal_messages, target_name, "recent").await;
        return AIAssistantResponse {
            response: format!("📊 Conversation Summary with {}\n\n{}", target_name, summary),
            query_type: "conversation_summary".to_string(),
            success: true,
        };
    }

    // Check if it's a group
    let group_check = sqlx::query(
        "SELECT g.id FROM groups g 
         INNER JOIN group_members gm ON g.id = gm.group_id 
         WHERE gm.username = ? AND g.name = ?"
    )
    .bind(username).bind(target_name)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if let Some(group_row) = group_check {
        let group_id: i64 = group_row.get("id");
        let group_messages = sqlx::query(
            "SELECT sender_username, message, timestamp 
             FROM group_messages 
             WHERE group_id = ? 
             ORDER BY id DESC 
             LIMIT 50"
        )
        .bind(group_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        if !group_messages.is_empty() {
            let (summary, _topics) = generate_rule_based_summary(&group_messages, target_name, "recent").await;
            return AIAssistantResponse {
                response: format!("👥 Group Summary: {}\n\n{}", target_name, summary),
                query_type: "group_summary".to_string(),
                success: true,
            };
        }
    }

    AIAssistantResponse {
        response: format!("I couldn't find any conversation with '{}'. Make sure the name is spelled correctly and you have messages with this person or group.", target_name),
        query_type: "not_found".to_string(),
        success: false,
    }
}

async fn get_recent_activity_summary(
    pool: &SqlitePool,
    username: &str,
) -> AIAssistantResponse {
    let recent_conversations = sqlx::query(
        "SELECT 
            CASE WHEN sender_username = ? THEN receiver_username ELSE sender_username END as other_user,
            COUNT(*) as message_count,
            MAX(timestamp) as last_message
         FROM (
             SELECT sender_username, receiver_username, timestamp
             FROM messages 
             WHERE sender_username = ? OR receiver_username = ? 
             ORDER BY id DESC 
             LIMIT 100
         ) recent_messages
         GROUP BY other_user 
         ORDER BY message_count DESC
         LIMIT 5"
    )
    .bind(username).bind(username).bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let recent_groups = sqlx::query(
        "SELECT g.name, COUNT(*) as message_count, MAX(gm.timestamp) as last_message
         FROM group_messages gm
         INNER JOIN groups g ON g.id = gm.group_id
         INNER JOIN group_members gmem ON gmem.group_id = g.id AND gmem.username = ?
         WHERE gm.timestamp > datetime('now', '-7 days')
         GROUP BY g.id, g.name
         ORDER BY message_count DESC
         LIMIT 3"
    )
    .bind(username)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut response = "📈 Recent Activity Summary\n\n".to_string();
    
    // Check if empty BEFORE iterating
    let has_conversations = !recent_conversations.is_empty();
    let has_groups = !recent_groups.is_empty();
    
    if has_conversations {
        response.push_str("**Most Active Personal Chats:**\n");
        for conv in &recent_conversations {  // Use & to borrow instead of move
            let other_user: String = conv.get("other_user");
            let count: i64 = conv.get("message_count");
            response.push_str(&format!("• {}: {} messages\n", other_user, count));
        }
        response.push('\n');
    }

    if has_groups {
        response.push_str("**Most Active Groups:**\n");
        for group in &recent_groups {  // Use & to borrow instead of move
            let group_name: String = group.get("name");
            let count: i64 = group.get("message_count");
            response.push_str(&format!("• {}: {} messages\n", group_name, count));
        }
    }

    if !has_conversations && !has_groups {
        response = "You haven't had much recent activity. Start a conversation to see your activity summary!".to_string();
    }

    AIAssistantResponse {
        response,
        query_type: "activity_summary".to_string(),
        success: true,
    }
}


// Then add .or(ai_assistant) to your routes before .with(cors)

async fn store_reaction(pool: &SqlitePool, message_id: i64, username: &str, emoji: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO message_reactions (message_id, username, emoji, timestamp) VALUES (?, ?, ?, ?)")
        .bind(message_id)
        .bind(username)
        .bind(emoji)
        .bind(get_current_time())
        .execute(pool)
        .await?;
    Ok(())
}