-- Migration Script can be run using the cli argument 'migrate'
-- In psql run \i /path/to/your/migration_script.sql
-- This will wipe the database

DROP TABLE IF EXISTS user_keybindings CASCADE;
DROP TABLE IF EXISTS commands CASCADE;
DROP TABLE IF EXISTS user_preferences CASCADE;
DROP TABLE IF EXISTS default_preferences CASCADE;
DROP TABLE IF EXISTS user_backgrounds CASCADE;

DROP TABLE IF EXISTS document_permissions CASCADE;
DROP TABLE IF EXISTS document_projects CASCADE;
DROP TABLE IF EXISTS documents CASCADE;
DROP TABLE IF EXISTS project_permissions CASCADE;
DROP TABLE IF EXISTS projects CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS user_profile_images CASCADE;
DROP TABLE IF EXISTS writing_assistant_messages CASCADE;
DROP TABLE IF EXISTS writing_assistant_sessions CASCADE;
DROP TYPE IF EXISTS message_role_enum CASCADE;

CREATE EXTENSION IF NOT EXISTS vector; -- Use PGVECTOR

-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    ai_credits INT NOT NULL DEFAULT 10,
    storage_bytes BIGINT NOT NULL DEFAULT 0,
    max_projects INT NOT NULL DEFAULT 3,
    max_documents INT NOT NULL DEFAULT 10
);

-- Create projects table
CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) DEFAULT 'Untitled Project' NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_starred BOOLEAN DEFAULT FALSE,
    is_trashed BOOLEAN DEFAULT FALSE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE
);

-- Create documents table
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) DEFAULT 'Untitled Document' NOT NULL,
    content TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_starred BOOLEAN DEFAULT FALSE,
    is_trashed BOOLEAN DEFAULT FALSE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    embedding vector(1536),
    embedding_updated_at TIMESTAMP WITH TIME ZONE
);

-- Create vector index for similarity search
CREATE INDEX document_embedding_idx ON documents USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Create junction table for many-to-many relationship
CREATE TABLE document_projects (
    document_id INT REFERENCES documents(id) ON DELETE CASCADE,
    project_id INT REFERENCES projects(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, project_id)
);

-- Create document_permissions table for role-based access
CREATE TABLE document_permissions (
    document_id INT REFERENCES documents(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('viewer', 'editor', 'owner')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (document_id, user_id)
);

-- Create an index for faster lookups
CREATE INDEX idx_document_permissions_user_id ON document_permissions(user_id);

-- Create project_permissions table for role-based access
CREATE TABLE project_permissions (
    project_id INT REFERENCES projects(id) ON DELETE CASCADE,
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('viewer', 'editor', 'owner')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, user_id)
);

-- Create an index for faster lookups
CREATE INDEX idx_project_permissions_user_id ON project_permissions(user_id);

-- Create commands table for holding all valid commands
CREATE TABLE commands (
    command_id SERIAL PRIMARY KEY,
    command_name VARCHAR(100) NOT NULL,
    command_description VARCHAR(150) NOT NULL,
    default_keybinding VARCHAR(50) NOT NULL
);

-- Create user_keybindings table for holding custom keybindings users set
CREATE TABLE user_keybindings (
    user_id INT NOT NULL,
    command_id INT NOT NULL,
    keybinding VARCHAR(50) NOT NULL,
    PRIMARY KEY (user_id, command_id),
    FOREIGN KEY (command_id) REFERENCES commands(command_id)
);

-- Create table for user profile images
CREATE TABLE user_profile_images (
    user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    image_data BYTEA NOT NULL,
    content_type VARCHAR(255) NOT NULL DEFAULT 'image/jpeg'
);

-- Create tables for user preferences
CREATE TABLE default_preferences (
    preference_id SERIAL PRIMARY KEY,
    preference_name VARCHAR(100) NOT NULL UNIQUE,
    preference_value VARCHAR(100) NOT NULL,
    preference_description VARCHAR(255) NOT NULL
);

CREATE TABLE user_preferences (
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    preference_id INT REFERENCES default_preferences(preference_id) ON DELETE CASCADE,
    preference_value VARCHAR(100) NOT NULL,
    PRIMARY KEY (user_id, preference_id)
);

CREATE TABLE user_backgrounds (
    user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    image_data BYTEA NOT NULL,
    content_type VARCHAR(255) NOT NULL DEFAULT 'image/jpeg'
);

-- Insert default users
INSERT INTO users(name,email,password, ai_credits)
VALUES('Christian','CFdefence@gmail.com','$argon2id$v=19$m=19456,t=2,p=1$kNRxgrDUnkl79WdlNuLXOw$v+gZeEyNvLQNvw2Q3l6T7HQOerrVSbRfOnp/Cx1xadk', 999) --MyPassword--
ON CONFLICT (email) DO NOTHING;

INSERT INTO users(name,email,password, ai_credits)
VALUES('Marko','MarkoP@gmail.com','$argon2id$v=19$m=19456,t=2,p=1$6hDoev817tzWWUfs6z/6LA$n1mPBXmxmIw915nVg9nHc3YXba0OjgRw7Yx6q85UCTE', 999) --MarkosPassword--
ON CONFLICT (email) DO NOTHING;

-- Create a default project for user 1
INSERT INTO projects (id, name, user_id) 
VALUES (1, 'Default Project', 1)
ON CONFLICT (id) DO NOTHING;

-- Create a second project for testing
INSERT INTO projects (id, name, user_id) 
VALUES (2, 'Test Project', 1)
ON CONFLICT (id) DO NOTHING;

-- Ensure user 1 has owner permission for the default project
INSERT INTO project_permissions(project_id, user_id, role)
VALUES(1, 1, 'owner')
ON CONFLICT (project_id, user_id) DO UPDATE SET role = 'owner';

-- Ensure user 1 has owner permission for the test project
INSERT INTO project_permissions(project_id, user_id, role)
VALUES(2, 1, 'owner')
ON CONFLICT (project_id, user_id) DO UPDATE SET role = 'owner';

-- Create test documents owned by user 1
INSERT INTO documents(id, name, content, user_id)
VALUES(1, 'Test Document 1', 'Test content for document 1', 1)
ON CONFLICT (id) DO NOTHING;

INSERT INTO documents(id, name, content, user_id)
VALUES(2, 'Test Document 2', 'Test content for document 2', 1)
ON CONFLICT (id) DO NOTHING;

-- Ensure user 1 has owner permission for document 1
INSERT INTO document_permissions(document_id, user_id, role)
VALUES(1, 1, 'owner')
ON CONFLICT (document_id, user_id) DO UPDATE SET role = 'owner';

-- Ensure user 1 has owner permission for document 2
INSERT INTO document_permissions(document_id, user_id, role)
VALUES(2, 1, 'owner')
ON CONFLICT (document_id, user_id) DO UPDATE SET role = 'owner';

-- Add user 2 as editor for document 1
INSERT INTO document_permissions(document_id, user_id, role)
VALUES(1, 2, 'editor')
ON CONFLICT (document_id, user_id) DO UPDATE SET role = 'editor';

-- Add document 1 to project 1
INSERT INTO document_projects(document_id, project_id)
VALUES(1, 1)
ON CONFLICT DO NOTHING;

-- Create additional documents for the project
INSERT INTO documents(id, name, content, user_id)
VALUES
(3, 'Project Overview', '# Project Overview\n\nThis document provides a high-level overview of our project goals and timeline.', 1),
(4, 'Technical Specifications', '# Technical Specifications\n\nDetailed technical requirements and implementation details.', 1),
(5, 'Meeting Notes', '# Meeting Notes\n\nNotes from our project planning meetings and discussions.', 1),
(6, 'Research Findings', '# Research Findings\n\nSummary of research conducted for this project.', 1)
ON CONFLICT (id) DO NOTHING;

-- Add permissions for user 1 on these documents
INSERT INTO document_permissions(document_id, user_id, role)
VALUES
(3, 1, 'owner'),
(4, 1, 'owner'),
(5, 1, 'owner'),
(6, 1, 'owner')
ON CONFLICT (document_id, user_id) DO UPDATE SET role = 'owner';

-- Add these documents to the same project as document 1 (project 1)
INSERT INTO document_projects(document_id, project_id)
VALUES
(3, 1),
(4, 1),
(5, 1),
(6, 1)
ON CONFLICT DO NOTHING;

-- Insert Default Commands
INSERT INTO commands(command_id, command_name, command_description, default_keybinding)
VALUES 
(1, 'bold', 'Action: bold', 'Ctrl+B'),
(2, 'italic', 'Action: italic', 'Ctrl+I'),
(3, 'underline', 'Action: underline', 'Ctrl+U'),
(4, 'openColorPicker', 'Action: open Color Picker', 'Ctrl+F'),
(5, 'moveLeft', 'Action: move Left', 'H'),
(6, 'moveRight', 'Action: move Right', 'L'),
(7, 'moveUp', 'Action: move Up', 'K'),
(8, 'moveDown', 'Action: move Down', 'J'),
(9, 'switchToDocument1', 'Action: switch To Document1', 'Ctrl+1'),
(10, 'switchToDocument2', 'Action: switch To Document2', 'Ctrl+2'),
(11, 'switchToDocument3', 'Action: switch To Document3', 'Ctrl+3'),
(12, 'switchToDocument4', 'Action: switch To Document4', 'Ctrl+4'),
(13, 'switchToDocument5', 'Action: switch To Document5', 'Ctrl+5'),
(14, 'switchToDocument6', 'Action: switch To Document6', 'Ctrl+6'),
(15, 'switchToDocument7', 'Action: switch To Document7', 'Ctrl+7'),
(16, 'switchToDocument8', 'Action: switch To Document8', 'Ctrl+8'),
(17, 'switchToDocument9', 'Action: switch To Document9', 'Ctrl+9'),
(18, 'enterInsertMode', 'Action: enter Insert Mode', 'I'),
(19, 'moveToStartOfLine', 'Action: move To Start Of Line', '0'),
(20, 'moveToEndOfLine', 'Action: move To End Of Line', 'Shift+$'),
(21, 'moveToEndOfDocument', 'Action: move To End Of Document', 'Shift+G'),
(22, 'moveToStartOfDocument', 'Action: move To Start Of Document', 'g'),
(23, 'toggleCommandSheet', 'Action: toggle Command Sheet', 'Ctrl+/'),
(24, 'findNextMatch', 'Action: find Next Match', 'N'),
(25, 'findPreviousMatch', 'Action: find Previous Match', 'M'),
(26, 'deleteSelectedText', 'Action: delete Selected Text', 'X'),
(27, 'yankText', 'Action: yank Text', 'Y'),
(28, 'deleteLine', 'Action: delete Line', 'D'),
(29, 'pasteText', 'Action: paste Text', 'P'),
(30, 'toggleChatAssistant', 'Action: toggle AI Chat Window', 'Alt+C')
ON CONFLICT (command_id) DO UPDATE SET
    command_name = EXCLUDED.command_name,
    command_description = EXCLUDED.command_description,
    default_keybinding = EXCLUDED.default_keybinding;

-- Give User 1 Some Custom Keybindings
INSERT INTO user_keybindings(user_id, command_id, keybinding)
VALUES
(1, 1, 'Ctrl H'), -- Bind 'Bold Selected' to Ctrl H
(1, 2, 'Ctrl E'); -- Bind 'Italic Selected' to Ctrl E

-- Set sequence values to match the highest IDs
SELECT setval('users_id_seq', (SELECT MAX(id) FROM users));
SELECT setval('projects_id_seq', (SELECT MAX(id) FROM projects));
SELECT setval('documents_id_seq', (SELECT MAX(id) FROM documents));
SELECT setval('commands_command_id_seq', (SELECT MAX(command_id) FROM commands));

-- Update sequence after adding new documents
SELECT setval('documents_id_seq', (SELECT MAX(id) FROM documents));

-- Create enum type for message roles
CREATE TYPE message_role_enum AS ENUM ('user', 'assistant');

-- Create tables for AI writing assistant functionality

-- Writing assistant sessions table
CREATE TABLE IF NOT EXISTS writing_assistant_sessions (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    document_id INT REFERENCES documents(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL DEFAULT 'New Writing Session',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Writing assistant messages table
CREATE TABLE IF NOT EXISTS writing_assistant_messages (
    id SERIAL PRIMARY KEY,
    session_id INT NOT NULL REFERENCES writing_assistant_sessions(id) ON DELETE CASCADE,
    role message_role_enum NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    embedding vector(1536)
);

-- Indexes for faster queries
CREATE INDEX IF NOT EXISTS idx_writing_messages_session_id ON writing_assistant_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_writing_sessions_user_id ON writing_assistant_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_writing_sessions_document_id ON writing_assistant_sessions(document_id);

-- Set initial sequence values for writing assistant tables
SELECT setval('writing_assistant_sessions_id_seq', 1, false);
SELECT setval('writing_assistant_messages_id_seq', 1, false);

-- Insert default color preferences
INSERT INTO default_preferences(preference_id, preference_name, preference_value, preference_description)
VALUES 
(1, 'primary_color', '#0A1721', 'Default primary color for text and UI elements'),
(2, 'secondary_color', '#10b981', 'Default secondary color for UI elements'),
(3, 'primary_accent_color', '#10b981', 'Default primary accent color for UI elements'),
(4, 'secondary_accent_color', '#808080', 'Default secondary accent color for UI elements'),
(5, 'primary_text_color', '#10b981', 'Default primary text color for UI elements'),
(6, 'secondary_text_color', '#FFFFFF', 'Default secondary text color for UI elements'),
(7, 'editor_background_opacity', '0.2', 'Default opacity for the editor background')
ON CONFLICT (preference_id) DO UPDATE SET
    preference_name = EXCLUDED.preference_name,
    preference_value = EXCLUDED.preference_value,
    preference_description = EXCLUDED.preference_description;

-- Set sequence values to match the highest IDs after adding preferences
SELECT setval('users_id_seq', (SELECT MAX(id) FROM users));
SELECT setval('projects_id_seq', (SELECT MAX(id) FROM projects));
SELECT setval('documents_id_seq', (SELECT MAX(id) FROM documents));
SELECT setval('commands_command_id_seq', (SELECT MAX(command_id) FROM commands));
SELECT setval('default_preferences_preference_id_seq', (SELECT MAX(preference_id) FROM default_preferences));

-- Update the storage_bytes column for existing users based on their document content
UPDATE users
SET storage_bytes = (
    SELECT COALESCE(SUM(LENGTH(COALESCE(d.content, ''))), 0)
    FROM documents d
    JOIN document_permissions dp ON d.id = dp.document_id
    WHERE dp.user_id = users.id AND dp.role = 'owner'
);