-- Migration Script can be run using the cli argument 'migrate'
-- In psql run \i /path/to/your/migration_script.sql
-- This will wipe the database

DROP TABLE IF EXISTS user_keybindings CASCADE;
DROP TABLE IF EXISTS commands CASCADE;

DROP TABLE IF EXISTS document_permissions CASCADE;
DROP TABLE IF EXISTS document_projects CASCADE;
DROP TABLE IF EXISTS documents CASCADE;
DROP TABLE IF EXISTS project_permissions CASCADE;
DROP TABLE IF EXISTS projects CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TABLE IF EXISTS user_profile_images;

-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL
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
    user_id INT REFERENCES users(id) ON DELETE CASCADE
);

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

-- Insert default users
INSERT INTO users(name,email,password) 
VALUES('Christian','CFdefence@gmail.com','$argon2id$v=19$m=19456,t=2,p=1$kNRxgrDUnkl79WdlNuLXOw$v+gZeEyNvLQNvw2Q3l6T7HQOerrVSbRfOnp/Cx1xadk')
ON CONFLICT (email) DO NOTHING;

INSERT INTO users(name,email,password) 
VALUES('Marko','MarkoP@gmail.com','MarkosPassword')
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
(1, 'Bold Selected', 'Bolds The Selected Text', 'Ctrl, B'),
(2, 'Italic Selected', 'Italics The Selected Text', 'Ctrl, I'),
(3, 'Underline Selected', 'Underline The Selected Text', 'Ctrl, U');

-- Give User 1 Some Custom Keybindings
INSERT INTO user_keybindings(user_id, command_id, keybinding)
VALUES
(1, 1, 'Ctrl H'), -- Bind 'Bold Selected' to Ctrl H
(1, 2, 'Ctrl E'); -- Bind 'Italic Selected' to Ctrl E

-- Set sequence values to match the highest IDs
SELECT setval('users_id_seq', (SELECT MAX(id) FROM users));
SELECT setval('projects_id_seq', (SELECT MAX(id) FROM projects));
SELECT setval('documents_id_seq', (SELECT MAX(id) FROM documents));

-- Update sequence after adding new documents
SELECT setval('documents_id_seq', (SELECT MAX(id) FROM documents));