-- Migration Script can be run using the cli argument 'migrate'
-- In psql run \i /path/to/your/migration_script.sql
-- This will wipe the database

DROP TABLE IF EXISTS document_projects;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS users;

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
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    user_id INT REFERENCES users(id) ON DELETE CASCADE
);

-- Create junction table for many-to-many relationship
CREATE TABLE document_projects (
    document_id INT REFERENCES documents(id) ON DELETE CASCADE,
    project_id INT REFERENCES projects(id) ON DELETE CASCADE,
    PRIMARY KEY (document_id, project_id)
);

-- Dummy User
INSERT INTO users(name,email,password) VALUES('Christian','CFdefence@gmail.com','MyPassword');