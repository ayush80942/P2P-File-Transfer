# 📁 P2P File Transfer

A lightweight real-time peer-to-peer (P2P) file transfer application using WebSockets, built with **Rust (Axum)** on the backend and **Next.js** on the frontend — structured as a **mono-repo**.

---

## ⚙️ Features

- 📡 WebSocket-based real-time file streaming between peers  
- 🔌 Peer registration and routing using unique `connectionId`  
- 🔁 Direct binary data transfer (files) without server-side storage  
- 🌐 Next.js UI for easy sharing between browsers  
- 🧩 Mono-repo structure with separate `/Backend` and `/Frontend` folders

---

## 📁 Folder Structure

```
.
├── Backend/       # Axum server with WebSocket logic
├── Frontend/      # Next.js frontend UI
├── .DS_Store      # macOS system file (ignored)
└── README.md
```

---

## 🚀 Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/your-username/p2p-file-transfer.git
cd p2p-file-transfer
```

---

### 2. Run the Backend (Rust + Axum)

> Ensure [Rust](https://rustup.rs/) is installed.

```bash
cd Backend
cargo run
```

> ✅ Backend runs at `ws://localhost:8000/ws`

---

### 3. Run the Frontend (Next.js)

> Requires Node.js v18+

```bash
cd ../Frontend
npm install
npm run dev
```

> ✅ Frontend runs at `http://localhost:3000`

---

## 🔧 How It Works

- Each WebSocket connection is assigned a unique ID using `Uuid`.
- Clients send a `register` message to claim a `connectionId`.
- Messages with `target_id` are forwarded to the appropriate peer.
- Supports:
  - `Text`: JSON-based signaling for registration & session control
  - `Binary`: File data sent directly between browsers

---

## 📸 Demo (Local)

1. Open two browser tabs at `http://localhost:3000`
2. Share the session code between them
3. Select or drag a file to start transferring

---

## 📄 License

This project is licensed under the MIT License.

---

## ✨ Built With

- [Axum](https://docs.rs/axum/latest/axum/)
- [Next.js](https://nextjs.org/)
- [Tokio](https://tokio.rs/)
- [WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
- [UUID](https://docs.rs/uuid/)
