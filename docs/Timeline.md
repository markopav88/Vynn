# Neovim for Writers – Project Timeline  

## Phase 1: Research & Planning 
- Define: **core features & tech stack**  
- Prototype: **Flesh out project skeleton**
- **Deploy: Proposal Document (1-2 pages)**

## Phase 2: Flesh Out Editor 
- Build: **core editor with intuitive UI**  
- Implement: **Typing features and seamless utility**
- Implement: **Database Connection and Utility**
- Deploy: Text Editor and Database Integration

## Phase 3: Account Functionality and Vim Functionality 
- Build: Account Functionality and Vim Functions
- Deploy: **stable version & finalize documentation**  

## Phase 4: Customizable Binding and Additional Fun Features
- Build: Customizable Binding UI
- Deploy: Other Fun Features
- Implement: Addition Project Requirements

## Phase 5: AI Integration  
- Integrate: **AI-assisted editing & auto-complete**  
- Fine-tune: **AI responses**  
- Implement: **real-time collaborative AI features**  

## Summary of work for Mar 12, 2025
    -Built a Rust-based backend web server built with the Axum framework connected to a PostgreSQL database containerized inside of a Docker container. The application features a modular structure with separate controllers, models, and database connections. It implements a RESTful API with endpoints to check server status (/api/hello), test database connectivity (/api/test-db), and perform CRUD operations on user data (/api/users). The server uses asynchronous programming with Tokio for handling concurrent requests and SQLx for type-safe database interactions.So far progress demonstrates proper separation of concerns with route handlers, database queries, and model definitions organized in their respective modules.