CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       username VARCHAR(255) NOT NULL UNIQUE,
                       email VARCHAR(255) NOT NULL UNIQUE,
                       password_hash VARCHAR(255) NOT NULL,
                       created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                       last_seen TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE messages (
                          id UUID NOT NULL PRIMARY KEY,
                          sender_id UUID NOT NULL REFERENCES users(id),
                          receiver_id UUID NOT NULL REFERENCES users(id),
                          content TEXT NOT NULL,
                          content_type JSONB NOT NULL,
                          created_at TIMESTAMP WITH TIME ZONE NOT NULL,
                          read_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_messages_sender ON messages(sender_id);
CREATE INDEX idx_messages_receiver ON messages(receiver_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);