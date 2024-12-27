use chrono::DateTime;
use chrono::Utc;
use super::models::{Message, MessageType, User};
use sqlx::PgPool;
use uuid::Uuid;
use std::path::PathBuf;

pub struct Storage {
    db_pool: PgPool,
    file_storage_path: PathBuf,
}

#[derive(Debug)]
pub enum StorageError {
    Database(sqlx::Error),
    FileSystem(std::io::Error),
    NotFound,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub content_type: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

impl Storage {
    pub fn new(db_pool: PgPool, file_storage_path: PathBuf) -> Self {
        // Ensure the base storage directory exists
        if let Err(e) = std::fs::create_dir_all(&file_storage_path) {
            println!("Warning: Could not create storage directory at {:?}: {}", file_storage_path, e);
        }

        println!("Initialized storage with path: {:?}", file_storage_path);

        Self {
            db_pool,
            file_storage_path,
        }
    }

    pub async fn save_message(&self, message: &Message) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            INSERT INTO messages
            (id, sender_id, receiver_id, content, content_type, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            message.id,
            message.sender_id,
            message.receiver_id,
            message.content,
            serde_json::to_value(&message.content_type).unwrap(),
            message.created_at,
        )
            .execute(&self.db_pool)
            .await
            .map_err(StorageError::Database)?;

        Ok(())
    }

    pub async fn get_user_messages(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, StorageError> {
        let db_messages = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            sender_id as "sender_id!",
            receiver_id as "receiver_id!",
            content,
            content_type as "content_type!: serde_json::Value",
            created_at as "created_at!",
            read_at
        FROM messages
        WHERE sender_id = $1 OR receiver_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset,
    )
            .fetch_all(&self.db_pool)
            .await
            .map_err(StorageError::Database)?;

        let messages = db_messages
            .into_iter()
            .filter_map(|row| {
                let content_type = serde_json::from_value(row.content_type).ok()?;
                Some(Message {
                    id: row.id,
                    sender_id: row.sender_id,
                    receiver_id: row.receiver_id,
                    content: row.content,
                    content_type,
                    created_at: row.created_at,
                    read_at: row.read_at,
                })
            })
            .collect();

        Ok(messages)
    }

    pub async fn get_users(&self) -> Result<Vec<User>, StorageError> {
        let users = sqlx::query_as!(
        User,
        r#"
        SELECT
            id as "id!",
            username as "username!",
            email as "email!",
            password_hash as "password_hash!",
            created_at as "created_at!",
            last_seen as "last_seen!"
        FROM users
        ORDER BY username
        "#,
    )
            .fetch_all(&self.db_pool)
            .await
            .map_err(StorageError::Database)?;

        Ok(users)
    }

    pub async fn get_conversation_messages(
        &self,
        user_id: Uuid,
        other_user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Message>, StorageError> {
        let db_messages = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            sender_id as "sender_id!",
            receiver_id as "receiver_id!",
            content,
            content_type as "content_type!: serde_json::Value",
            created_at as "created_at!",
            read_at
        FROM messages
        WHERE (sender_id = $1 AND receiver_id = $2)
           OR (sender_id = $2 AND receiver_id = $1)
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
        user_id,
        other_user_id,
        limit,
        offset,
    )
            .fetch_all(&self.db_pool)
            .await
            .map_err(StorageError::Database)?;

        let messages = db_messages
            .into_iter()
            .filter_map(|row| {
                let content_type = serde_json::from_value(row.content_type).ok()?;
                Some(Message {
                    id: row.id,
                    sender_id: row.sender_id,
                    receiver_id: row.receiver_id,
                    content: row.content,
                    content_type,
                    created_at: row.created_at,
                    read_at: row.read_at,
                })
            })
            .collect();

        Ok(messages)
    }

    pub async fn save_file(
        &self,
        user_id: Uuid,
        filename: String,
        data: Vec<u8>,
    ) -> Result<PathBuf, StorageError> {
        let user_dir = self.file_storage_path.join(user_id.to_string());
        println!("Creating user directory at: {:?}", user_dir);

        // Create user directory
        tokio::fs::create_dir_all(&user_dir).await.map_err(|e| {
            println!("Failed to create user directory: {}", e);
            StorageError::FileSystem(e)
        })?;

        let file_path = user_dir.join(&filename);
        println!("Writing file to: {:?}", file_path);

        // Write file using tokio's async file operations
        tokio::fs::write(&file_path, data).await.map_err(|e| {
            println!("Failed to write file: {}", e);
            StorageError::FileSystem(e)
        })?;

        println!("Successfully wrote file at: {:?}", file_path);
        Ok(file_path)
    }

    // Add this method to get base storage path
    pub fn get_base_path(&self) -> PathBuf {
        self.file_storage_path.clone()
    }

    // Update get_file_path to use tokio
    pub async fn get_file_path(&self, user_id: Uuid, filename: &str) -> PathBuf {
        let user_dir = self.file_storage_path.join(user_id.to_string());
        // Ensure directory exists
        let _ = tokio::fs::create_dir_all(&user_dir).await;
        user_dir.join(filename)
    }

    pub async fn mark_messages_as_read(
        &self,
        user_id: Uuid,
        message_ids: &[Uuid],
    ) -> Result<(), StorageError> {
        sqlx::query!(
            r#"
            UPDATE messages
            SET read_at = NOW()
            WHERE id = ANY($1) AND receiver_id = $2
            "#,
            message_ids,
            user_id,
        )
            .execute(&self.db_pool)
            .await
            .map_err(StorageError::Database)?;

        Ok(())
    }

    pub async fn get_unread_messages_count(
        &self,
        user_id: Uuid,
    ) -> Result<i64, StorageError> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM messages
            WHERE receiver_id = $1 AND read_at IS NULL
            "#,
            user_id,
        )
            .fetch_one(&self.db_pool)
            .await
            .map_err(StorageError::Database)?;

        Ok(result.count.unwrap_or(0))
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self {
            db_pool: self.db_pool.clone(),
            file_storage_path: self.file_storage_path.clone(),
        }
    }
}