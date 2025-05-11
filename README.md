# Vynn - A Tailored Neovim Experience for Writers

A modern, keyboard-driven text editor that combines the power of Neovim with the rich features of modern document processors.

## Overview

Neovim for Writers is an innovative text editor designed for writers who value efficiency and customization. It provides the core functionality of Google Docs while allowing users to bind everything to custom shortcuts and extend the editor via plugins.

## Key Features

- **Vim-like Interface**
  - Modal editing (Normal, Insert, Command modes)
  - Command palette with custom commands
  - Fully customizable keybindings
  - Distraction-free, keyboard-centric interface

- **Document Management**
  - Project-based organization
  - Version control and undo history
  - Real-time collaboration capabilities
  - Multiple format export options

- **AI Integration**
  - Chatbox for large text edits
  - Inline AI editing with context-aware suggestions
  - Fine-tuned AI assistant that learns your writing style
  - Smart auto-complete and grammar enhancement

## Tech Stack

### Frontend
- SvelteKit
- Bootstrap
- Typescript

### Backend
- Rust + Axum
- PostgreSQL
- Sqlx

## Getting Started

1. Clone
2. Create .env File
3. install docker and docker-compsoe
4. ensure docker daemon is running
5. psql -h localhost -p 5431 -U <db_user> -d <db_name>
6. run migration \i <migration_path>
7 npm install frontend
8. cargo build backend w db up

## Contributing

[Contribution guidelines to be added]

## License

[License information to be added]
