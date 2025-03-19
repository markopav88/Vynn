# Neovim for Writers

## Overview

A **highly customizable, keyboard-driven writing environment** inspired by Neovim, tailored for writers.  
The app will provide **all the core functionality of Google Docs** but allow users to **bind everything to custom shortcuts** and **extend the editor via plugins**.

It will feature **AI-powered editing**, **fine-tuned assistance that mimics the user’s style**, and **real-time collaboration (maybe)**, all within a **distraction-free, keyboard-centric interface**.

---

## AI Integration

- **Chatbox for large text edits** – Rewrite, summarize, or refine entire sections.
- **Highlight function for targeted edits** – AI will have full document context and provide intelligent suggestions.

---

## Key Features

- **Fully bindable actions** – Every function (bold, italics, track changes, AI edits) can be mapped to custom shortcuts.
- **Vim-like modal editing** – Normal, Insert, and Visual modes for efficient text navigation.
- **Command Palette (`:` commands)** – Execute functions like `:format`, `:synonym`, or `:replace "word" with "alternative"`.
- **Version control & undo history** – Git-like tracking of every document change.
- **Account functionality** – User authentication, document storage, and management.
- **Export to PDF?** – Ability to export documents in multiple formats.
- **Must** be able to dynamically change keybindings
- **Create** Projects of files and allow for seamless switching between documents.

---

## AI Features

- **Chatbox for long-form edits** – Rewrite, summarize, or refine entire sections.
- **Inline AI Editing** – Highlight a sentence/paragraph and issue a command (`:fix grammar`, `:simplify`, `:make poetic`).
- **Fine-tuned AI Assistant** – Learns the user's writing style and provides suggestions that sound like them.
- **Smart Auto-Complete** – Predicts upcoming sentences based on the writer’s style.
- **Grammar & Style Enhancements** – AI-powered real-time feedback, integrated directly into the editor.

---

## Tech Stack

### **Frontend (SvelteKit + Tailwind CSS + WASM)**

- **SvelteKit** → Reactive, fast UI.
- **Tailwind CSS** → Minimalist, distraction-free styling.

### **Backend (Rust + Axum + Redis + Postgres/SQLx)**

- **Axum** → Fast async Rust web framework.
- **Tokio & WebSockets** → Real-time document sync.
- **Redis** → Pub/Sub for live collaboration.
- **PostgreSQL/SQLx** → Document storage and versioning.
- **Tonic (gRPC)** → AI model interactions.

### **AI Integration**
- **Fine-tuned GPT Model** (via OpenAI API or local LLaMA/Claude model).  
- **LangChain or Custom AI Pipeline** for personalized writing suggestions.  
- **Sentence Embeddings** to learn user writing patterns.  
