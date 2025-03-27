-- Migration Script can be run using the cli argument 'migrate'
-- In psql run \i /path/to/your/migration_script.sql
-- This will wipe the database

DROP TABLE IF EXISTS document_permissions CASCADE;
DROP TABLE IF EXISTS document_projects CASCADE;
DROP TABLE IF EXISTS documents CASCADE;
DROP TABLE IF EXISTS project_permissions CASCADE;
DROP TABLE IF EXISTS projects CASCADE;
DROP TABLE IF EXISTS users CASCADE;

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
    user_id INT REFERENCES users(id) ON DELETE CASCADE
);

-- Create documents table
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) DEFAULT 'Untitled Document' NOT NULL,
    content TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
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

-- Insert default users
INSERT INTO users(name,email,password) 
VALUES('Christian','CFdefence@gmail.com','MyPassword')
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
ON CONFLICT (document_id, project_id) DO NOTHING;

-- Add document 2 to project 2
INSERT INTO document_projects(document_id, project_id)
VALUES(2, 2)
ON CONFLICT (document_id, project_id) DO NOTHING;

-- Set sequence values to match the highest IDs
SELECT setval('users_id_seq', (SELECT MAX(id) FROM users));
SELECT setval('projects_id_seq', (SELECT MAX(id) FROM projects));
SELECT setval('documents_id_seq', (SELECT MAX(id) FROM documents));
