# 💬 Personal Messenger - Rust WebSocket Chat Application

A modern, secure personal messaging application built with Rust and WebSockets, featuring WhatsApp-style private conversations, real-time communication, and persistent message history.

## 🌟 Features

### 🔐 Authentication & Security
- **Secure User Registration** - Create accounts with username/password
- **JWT Authentication** - Industry-standard token-based authentication
- **Password Security** - Argon2 hashing with salt for maximum security
- **Session Management** - Persistent login across browser sessions
- **Input Validation** - Server-side validation and SQL injection protection

### 💬 Personal Messaging
- **Private Conversations** - One-on-one messaging between users
- **Contact List** - Sidebar showing all available users to chat with
- **Real-time Communication** - Instant message delivery via WebSockets
- **Message History** - Persistent conversation history across sessions
- **Conversation Selection** - Click contacts to start/continue private chats

### 🎨 Modern Interface
- **WhatsApp-style UI** - Clean, professional messenger interface
- **Message Bubbles** - Sent messages (blue, right) and received (white, left)
- **Responsive Design** - Works perfectly on desktop and mobile devices
- **Contact Sidebar** - Easy navigation with active conversation highlighting
- **Welcome Screen** - Informative interface when no conversation is selected

### 🚀 Technical Excellence
- **High Performance** - Built with Rust for maximum speed and reliability
- **WebSocket Protocol** - Real-time bidirectional communication
- **SQLite Database** - Efficient local data storage
- **Cross-Platform** - Runs on Windows, macOS, and Linux
- **Network Sharing** - Access from multiple devices on the same network

## 🛠️ Installation & Setup

### Prerequisites
- **Rust** (latest stable version)
- **Cargo** (comes with Rust)
- **SQLite3** (for database operations)

### Quick Start

1. **Extract the project**
   ```bash
   unzip rust_personal_messenger_v3.0.zip
   cd chat_app
   ```

2. **Install Rust** (if not already installed)
   
   **Windows (PowerShell):**
   ```powershell
   winget install --id Rustlang.Rust.MSVC
   # OR download from https://rustup.rs/
   ```
   
   **macOS/Linux:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **Install Build Tools** (Windows only)
   - Download Visual Studio Build Tools from Microsoft
   - Select "Desktop development with C++" workload during installation

4. **Run the application**
   ```bash
   cargo run
   ```

5. **Access the messenger**
   - Open your browser and go to: `http://localhost:3030`
   - Or use your local IP: `http://127.0.0.1:3030`

## 🎯 How to Use

### Getting Started
1. **Register** a new account with username and password (minimum 6 characters)
2. **Login** with your credentials
3. **Select a contact** from the sidebar to start a conversation
4. **Send messages** using the input field at the bottom
5. **Switch conversations** by clicking different contacts

### Features Guide

#### Private Conversations
- Each conversation is completely private between two users
- Only you and your conversation partner can see the messages
- Message history is preserved across login sessions

#### Contact Management
- All registered users appear in your contact list
- Click any contact to start or continue a conversation
- Active conversations are highlighted in blue

#### Message Interface
- **Sent messages** appear as blue bubbles on the right
- **Received messages** appear as white bubbles on the left
- **Timestamps** show when each message was sent
- **Conversation headers** display your chat partner's name

## 🌐 Network Access

### Local Network Sharing
To access the messenger from other devices on your network:

1. **Find your IP address**
   ```bash
   # Windows
   ipconfig
   
   # macOS/Linux
   ifconfig
   ```

2. **Configure firewall** (Windows)
   - Allow the application through Windows Firewall
   - Create inbound rule for port 3030

3. **Access from other devices**
   - Use your IP address: `http://[YOUR_IP]:3030`
   - Example: `http://192.168.1.100:3030`

## 🏗️ Architecture

### Backend (Rust)
- **Warp Framework** - High-performance web server
- **Tokio** - Asynchronous runtime for concurrent connections
- **SQLx** - Type-safe SQL database operations
- **Argon2** - Secure password hashing
- **JWT** - Stateless authentication tokens

### Frontend (Web)
- **Vanilla JavaScript** - No framework dependencies
- **WebSocket API** - Real-time communication
- **Responsive CSS** - Mobile-first design
- **Local Storage** - Session persistence

### Database (SQLite)
```sql
-- Users table for authentication
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);

-- Messages table for conversation history
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender_username TEXT NOT NULL,
    receiver_username TEXT NOT NULL,
    message TEXT NOT NULL,
    timestamp TEXT NOT NULL
);
```

## 🔧 Configuration

### Environment Variables
- `RUST_LOG` - Set logging level (e.g., `debug`, `info`)
- Database file location: `./db/chat.db`
- Server port: `3030` (configurable in source)

### Security Settings
- JWT secret key (change in production)
- Password requirements (minimum 6 characters)
- Session timeout (24 hours default)

## 🧪 Testing

The application has been thoroughly tested with:
- ✅ User registration and authentication
- ✅ Private message sending and receiving
- ✅ Conversation history persistence
- ✅ Contact list management
- ✅ Real-time message delivery
- ✅ Cross-browser compatibility
- ✅ Mobile responsiveness
- ✅ Network access from multiple devices

## 🚀 Use Cases

Perfect for:
- **Personal messaging** between friends and family
- **Team communication** in small organizations
- **Customer support** chat systems
- **Dating applications** with private messaging
- **Social media** direct message features
- **Educational projects** learning WebSocket and Rust
- **Prototype development** for chat applications

## 📱 Browser Compatibility

- ✅ Chrome/Chromium (recommended)
- ✅ Firefox
- ✅ Safari
- ✅ Edge
- ✅ Mobile browsers (iOS Safari, Chrome Mobile)

## 🔒 Security Features

- **Password Hashing** - Argon2 with salt
- **JWT Tokens** - Secure, stateless authentication
- **Input Validation** - Prevents injection attacks
- **CORS Protection** - Configurable cross-origin policies
- **Session Management** - Automatic token expiration

## 📊 Performance

- **Concurrent Users** - Supports hundreds of simultaneous connections
- **Message Latency** - Sub-millisecond message delivery
- **Memory Usage** - Efficient Rust memory management
- **Database Performance** - Optimized SQLite queries
- **Network Efficiency** - Minimal bandwidth usage

## 🛠️ Development

### Project Structure
```
chat_app/
├── src/
│   └── main.rs          # Rust backend server
├── static/
│   ├── index.html       # Frontend interface
│   ├── style.css        # Styling and layout
│   └── script.js        # Frontend logic
├── db/
│   └── chat.db          # SQLite database
├── Cargo.toml           # Rust dependencies
├── README.md            # This file
└── CHANGELOG.md         # Version history
```

### Building from Source
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

## 📝 License

This project is open source and available under the MIT License.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## 📞 Support

For questions, issues, or feature requests, please create an issue in the project repository.

---

**Built with ❤️ using Rust and modern web technologies**

