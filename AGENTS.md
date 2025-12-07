# Project Context: Audio Recorder & Transcriber

## Overview
This project is a full-stack web application built with **Rust** and **Leptos**, designed to run on **Cloudflare Workers**. It features a client-side audio recorder that performs **in-browser speech-to-text transcription** using the OpenAI Whisper model, powered by the **Candle** machine learning framework (compiled to WebAssembly).

## Tech Stack
-   **Language:** Rust (2024 edition)
-   **Framework:** Leptos (Signal-based UI framework)
-   **Platform:** Cloudflare Workers (Edge runtime)
-   **ML Framework:** Candle (by Hugging Face) for WASM-based inference
-   **Deployment:** Wrangler
-   **Build Tool:** `just`

## Architecture
The application utilizes a **Isomorphic/Universal** architecture where Rust code runs both on the server (Cloudflare Workers) and the client (WASM).

1.  **Server-Side (Cloudflare Worker):**
    -   Serves the initial HTML shell.
    -   Provides "Server Functions" (Leptos RPC) to fetch model weights/config from an R2 bucket (or other storage).
    -   Handles static asset serving.

2.  **Client-Side (WASM):**
    -   Hydrates the UI into a single-page application (SPA).
    -   **Media Recorder:** Uses `web-sys` APIs to capture audio from the microphone.
    -   **Inference:** Runs the Whisper model locally in the browser to ensure privacy and low latency.
    -   **Web Worker:** Heavy ML computation is offloaded to a dedicated Web Worker (`src/webworker.rs`) to keep the UI thread responsive.

## Key Directory Structure

### `/src`
-   **`lib.rs`**: The library entry point. Handles conditional compilation for hydration (client) vs. SSR (server). Registers server functions.
-   **`main.rs`**: Entry point for local development (Axum-based) or other server environments.
-   **`webworker.rs`**: Dedicated entry point for the Web Worker thread that runs the Candle/Whisper model.
-   **`app.rs`**: Root application component and routing logic.

### `/src/components`
-   **`media_recorder.rs`**: The primary UI component. Handles:
    -   Microphone access permissions.
    -   Recording state (start/stop).
    -   Visualizing audio chunks.
    -   Communicating with the Web Worker to send audio data and receive transcription segments.

### `/src/whisper`
Contains the logic for the machine learning model.
-   **`model.rs`**: Loading and initialization of the Whisper model using Candle.
-   **`audio.rs`**: Audio preprocessing utilities (resampling, normalization).
-   **`languages.rs`**: Language detection/selection utilities.

## Development & Build

The project uses a `justfile` to manage tasks.

-   **`just dev`**: Starts the local development server (usually with `cargo leptos watch`).
-   **`just deploy`**: Deploys the worker using `wrangler deploy`.
-   **`just build`**: Compiles the WASM binaries.

## Important Notes for AI Agents
-   **Async/Await**: Heavy use of Rust's async features.
-   **Leptos Syntax**:
    -   **Signals**: Uses `let (read, write) = signal(initial_value);` from `leptos::prelude::*`.
    -   **Resources**: Uses `LocalResource::new(...)` for async data loading.
-   **WASM Constraints**: Code in `src/whisper` and `src/webworker.rs` must be WASM-compatible (no heavy I/O blocking, specific threading model).
-   **Communication**: The UI communicates with the model worker via `post_message` (or similar message passing).
