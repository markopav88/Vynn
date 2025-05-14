### Live @ [Vynn.app](https://vynn.app)

# Vynn - A Tailored Neovim Experience for Writers

A minimal, keyboard-driven document storage and processing solution that combines the speed and power of Neovim with the simplicity of modern writing tools and intelligent RAG agents to create a fast, customizable workspace for writers, researchers, and note takers.

## Demo

## Overview

Vynn is an innovative text editor designed for writers who value efficiency and customization. It provides the core functionality of modern document storage and processing solutions, such as Google Drive, while allowing users to customize their environment through custom keybindings and styles.

Vynn employs its 'project-based' environment to make full use of RAG (Retrieval-Augmented Generation) to develop and suggest informative and grounded document changes and insights to users.

Vynn utilizes multiple fine-tuned AI Agents to leverage document-diff generation and input sanitization tools, providing users with intelligent AI decisions and reactions.

## Key Features

- **Vim-like Interface**
  - Modal editing (Normal, Insert, Command modes)
  - Command palette with custom commands
  - Fully customizable keybindings
  - Distraction-free, keyboard-centric interface
  - Relative searching and find and replace features
  - Vim-like rapid navigation keybindings

- **Document Management**
  - Project-based organization
  - Real-time collaboration capabilities
  - Multiple format export options
  - Sharing capabilities and permissions
  - Document filtering functionality
  - Document management

- **AI Integration**
  - Chatbox for large text edits and document suggestions
  - RAG top-k relevant document semantic search algorithm for identifying relevant context
  - Agentic diff-rendering for in-line changes based on AI recommendations
  - Agentic AI response filtering and processing
  - Fine-tuned AI assistant that learns your writing style based on previous chats
  - AI commands for writing assistance
      - :grammar -> Provides insightful grammar improvements on selected text or entire document
      - :spellcheck -> Corrects spelling mistakes for a selected chunk of text or the entire document
      - :shrink -> Condenses the selected text or the entire document
      - :expand -> Expands the selected text or the entire document
      - :summarize -> Summarizes the selected text or the entire document
      - :rephrase -> Rephrases the selected text or the entire document
      - :rewriteas [style] -> Rewrites the selected text or the entire document in a specified style

## Tech Stack

### Frontend
- Node
- Vite
- SvelteKit
- Bootstrap
- Typescript

### Backend
- Rust + Axum
- PostgreSQL
- Sqlx
- rust-langchain
- tower

## Getting Started

1. Clone the git repository
2. Create .env File in backend/
    - POSTGRES_USER = {your postgres username}
    - POSTGRES_PASSWORD = {your postgres db password}
    - POSTGRES_DB = {your postgres db name}
    - DATABASE_URL = {your postgres db url}
    - API_BASE_URL = {your backend API URL - ex: http://localhost:2000}
    - FRONTEND_URL = {your frontend URL - ex: http://localhost:2001}
    - BIND_ADDRESS = {backend port address - ex: 0.0.0.0:2000}
    - OPENAI_API_KEY = {your open ai API key}
4. Install docker and docker-compose
5. Ensure Docker daemon is running
6. psql -h localhost -p 5431 -U <db_user> -d <db_name>
7. Run migration script from inside db \i migrations/01_migration_script.sql
8. npm install in frontend/
9. Cargo build backend with the database running

## API and Storage Limits

The application supports per-user limits and tracking:

- Each non-paid user is limited to 3 projects and 10 documents by default
- Each non-paid user is limited to 10 AI requests

## License

MIT

[Kept alive with](https://keepalive.dashdashhard.com/)
