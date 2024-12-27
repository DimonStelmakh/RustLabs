use bcrypt::{hash, verify, DEFAULT_COST};
use crate::realtime_messenger::models::User;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    DatabaseError(sqlx::Error),
    HashingError(bcrypt::BcryptError),
}

pub struct Auth {
    db_pool: PgPool,
}

impl Auth {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn register_user(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User, AuthError> {
        let password_hash = hash(password.as_bytes(), DEFAULT_COST)
            .map_err(AuthError::HashingError)?;

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at, last_seen)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING *
            "#,
            Uuid::new_v4(),
            username,
            email,
            password_hash,
        )
            .fetch_one(&self.db_pool)
            .await
            .map_err(AuthError::DatabaseError)?;

        Ok(user)
    }

    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> Result<User, AuthError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            email
        )
            .fetch_optional(&self.db_pool)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)?;

        if !verify(password.as_bytes(), &user.password_hash)
            .map_err(AuthError::HashingError)? {
            return Err(AuthError::InvalidCredentials);
        }

        sqlx::query!(
            "UPDATE users SET last_seen = NOW() WHERE id = $1",
            user.id
        )
            .execute(&self.db_pool)
            .await
            .map_err(AuthError::DatabaseError)?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AuthError> {
        sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
            .fetch_optional(&self.db_pool)
            .await
            .map_err(AuthError::DatabaseError)?
            .ok_or(AuthError::InvalidCredentials)
    }
}