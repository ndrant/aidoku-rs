# Architecture

## Goal

This repository contains a production-grade Aidoku Source extension capable of supporting multiple manga websites through a shared architecture.

The codebase must remain scalable.

Adding a new website should require creating only a new module inside:

src/sites/

without modifying existing site implementations.

---

# High Level Flow

User

↓

Aidoku

↓

Source Router

↓

Selected Website

↓

Network Layer

↓

Parser

↓

Models

↓

Aidoku Objects

↓

Reader

---

# Layer Responsibilities

Source

Responsible for exposing Aidoku APIs.

Never parse HTML here.

Never make direct HTTP requests here.

Only delegate.

---

Router

Chooses which website implementation to use.

No business logic.

---

Sites

Contains all website-specific implementations.

Each site is completely isolated.

No cross-dependency.

---

Network

Responsible for:

- HTTP
- Retry
- Timeout
- Headers
- Cookies
- Compression

Never parse HTML.

---

Parser

Responsible for converting:

HTML

↓

Models

Never perform HTTP requests.

---

Models

Pure Rust structures.

No network.

No parsing.

No Aidoku logic.

---

Utils

Reusable helper functions.

No business logic.

---

Error

Shared error types.

Every module returns Result<T, AidokuError>.

---

Future Expansion

The architecture must support adding:

Komiku

WestManga

MangaDex

Asura

Batoto

without modifying existing implementations.