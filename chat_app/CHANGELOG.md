# Changelog

## Version 3.0.0 - Personal Messenger Transformation

### 🎉 Major Release: Complete Application Transformation

This release completely transforms the Rust chat application from a group chat room into a modern personal messenger application, similar to WhatsApp or Facebook Messenger.

### ✨ New Features

#### Personal Messaging System
- **Private Conversations**: One-on-one messaging between users instead of group chat
- **Contact List**: Sidebar showing all available users to chat with
- **Conversation Selection**: Click on contacts to start/continue private conversations
- **Message Filtering**: Only shows messages relevant to the current conversation

#### Modern Messenger Interface
- **WhatsApp-style UI**: Complete redesign with sidebar and chat area layout
- **Message Bubbles**: Sent messages appear on right (blue), received on left (white)
- **Contact Sidebar**: Clean contact list with active state highlighting
- **Chat Headers**: Shows current conversation partner name
- **Welcome Screen**: Informative welcome when no conversation is selected

#### Enhanced User Experience
- **Responsive Design**: Works perfectly on desktop and mobile devices
- **Professional Styling**: Modern gradient backgrounds and clean typography
- **Intuitive Navigation**: Easy contact selection and conversation switching
- **Visual Feedback**: Clear indication of active conversations and message states

### 🔧 Technical Improvements

#### Database Schema Updates
- **Messages Table**: Redesigned with `sender_username` and `receiver_username` fields
- **Private Message Storage**: Each message linked to specific sender-receiver pair
- **Conversation History**: Efficient retrieval of conversation-specific messages

#### Backend Enhancements
- **Contact API**: New `/users` endpoint to fetch available contacts
- **Private Message Routing**: WebSocket messages filtered by conversation participants
- **Conversation History**: `get_conversation` protocol for loading chat history
- **Message Filtering**: Only broadcasts messages to intended recipients

#### Frontend Architecture
- **Complete Redesign**: New HTML structure for messenger-style layout
- **Contact Management**: Dynamic contact list loading and selection
- **Conversation State**: Proper state management for active conversations
- **Message Protocol**: Enhanced WebSocket communication for private messaging

### 🛠️ Breaking Changes
- **Database Schema**: Messages table structure completely changed
- **UI Layout**: Entirely new interface design
- **Message Protocol**: Updated WebSocket message format
- **User Experience**: Fundamentally different from group chat to personal messaging

### 🔄 Migration Notes
- Existing group chat messages will not be compatible with new schema
- Users will need to create new conversations in the personal messenger format
- All authentication and user management remains unchanged

### 📱 Supported Features
- ✅ User registration and login
- ✅ JWT-based authentication
- ✅ Private one-on-one conversations
- ✅ Real-time message delivery
- ✅ Conversation history persistence
- ✅ Contact list management
- ✅ Responsive mobile design
- ✅ Message timestamps
- ✅ Secure password hashing

### 🎯 Use Cases
Perfect for:
- Personal messaging applications
- Private team communication
- Customer support chat systems
- Dating app messaging
- Social media direct messages
- Any scenario requiring private conversations

---

## Version 2.1.0 - Chat History Persistence

### 🎉 Major Features
- **Chat History Persistence**: Messages are now stored in the database and displayed when users log back in
- **Historical Message Display**: Clear visual separation between historical and new messages
- **Seamless Experience**: Users can see their conversation history across login sessions

### ✨ New Features
- **Message Storage**: All chat messages (including system messages) are automatically saved to the database
- **History Retrieval**: Last 100 messages are loaded and displayed when a user logs in
- **Visual Indicators**: Historical messages have distinct styling and clear separators
- **Chronological Order**: Messages are displayed in proper chronological order (oldest first)
- **Real-time Integration**: New messages appear seamlessly after historical messages

### 🔧 Technical Changes

#### Backend (main.rs)
- Added `messages` table to SQLite database schema
- Implemented `store_message()` function to save all chat messages
- Added `get_recent_messages()` function to retrieve historical messages
- Enhanced WebSocket handler to send historical messages upon connection
- Added `HistoryMessage` struct for structured history delivery
- Modified message handling to store messages in database alongside broadcasting

#### Frontend (script.js)
- Added `displayHistoricalMessages()` function to handle history display
- Enhanced message handling to distinguish between history and real-time messages
- Added visual separators for chat history sections
- Implemented proper scrolling behavior (history doesn't auto-scroll, new messages do)
- Added historical message styling with reduced opacity

#### Database Schema
```sql
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    message TEXT NOT NULL,
    timestamp TEXT NOT NULL
);
```

#### Frontend (style.css)
- Added `.message.historical` styling for visual distinction
- Enhanced message header opacity for historical messages

### 🎨 User Experience Improvements
- **Clear History Markers**: "--- Chat History ---" and "--- End of History ---" separators
- **Visual Distinction**: Historical messages have slightly reduced opacity
- **Smooth Transitions**: Seamless flow from historical to real-time messages
- **Preserved Context**: Users can see conversation context when returning to chat

### 🧪 Testing
- Verified message storage in database
- Confirmed historical message retrieval and display
- Tested visual separation and styling
- Validated chronological message ordering
- Confirmed real-time message integration after history
- Tested across logout/login cycles

### 📊 Performance
- Efficient database queries with proper ordering
- Limited to last 100 messages for optimal performance
- Fast message retrieval with no noticeable delays
- Minimal impact on real-time messaging performance

---

## Version 2.0.0 - Login System Implementation

### 🎉 Major Features
- **Complete Login System**: Users can now register accounts and log in with username/password
- **Secure Authentication**: JWT-based authentication with password hashing using Argon2
- **Persistent Sessions**: Login state is maintained across browser sessions
- **Database Integration**: SQLite database for user storage

### ✨ New Features
- **User Registration**: Create new accounts with username and password validation
- **User Login**: Authenticate with existing credentials
- **Automatic Session Management**: Tokens are stored locally and validated on connection
- **Secure Password Storage**: Passwords are hashed using industry-standard Argon2
- **Real-time Authentication**: WebSocket connections require valid JWT tokens
- **Logout Functionality**: Clean session termination and token removal

### 🔧 Technical Changes

#### Backend (main.rs)
- Added SQLite database integration with `sqlx`
- Implemented user registration endpoint (`/register`)
- Implemented user login endpoint (`/login`)
- Added JWT token generation and validation
- Enhanced WebSocket handler to require authentication
- Added Argon2 password hashing for security
- Removed username change functionality (now uses authenticated identity)

#### Frontend (HTML/CSS/JS)
- **New Login/Registration Interface**: Clean, modern authentication forms
- **Responsive Design**: Mobile-friendly login and chat screens
- **Session Management**: Automatic token storage and validation
- **Error Handling**: User-friendly error messages for authentication failures
- **Auto-login**: Seamless transition from registration to login
- **Logout Button**: Easy session termination

#### Database
- Created `users` table with secure schema
- Username uniqueness constraints
- Secure password hash storage

### 🛡️ Security Features
- **Password Requirements**: Minimum 6 characters
- **Secure Hashing**: Argon2 with salt for password storage
- **JWT Authentication**: Stateless token-based authentication
- **Input Validation**: Server-side validation for all user inputs
- **SQL Injection Protection**: Parameterized queries with sqlx

### 🧪 Testing
- Verified user registration and login flows
- Confirmed secure password hashing and verification
- Tested JWT token generation and validation
- Validated WebSocket authentication requirements
- Confirmed session persistence across browser restarts

---

## Version 1.1.1 - Username Update Trigger Fix

### 🐛 Bug Fixes
- **Refined username change trigger**: Username updates are now sent only when the user presses Enter in the username input field, preventing a flood of intermediate name change notifications.

### 🔧 Technical Changes

#### Frontend (script.js)
- Removed `input` event listener for username input field.
- Modified `keypress` event listener for username input field to trigger `sendUsernameUpdate()` only on `Enter` keypress.

### 🧪 Testing
- Verified that intermediate username changes (on keystroke) no longer trigger notifications.
- Confirmed that a single, correct username change notification is displayed upon pressing Enter.

---

## Version 1.1.0 - Username Fix Update

### 🐛 Bug Fixes
- **Fixed username not reflecting in chat messages**: Users can now change their username and all subsequent messages will display the updated username instead of the original auto-generated user ID.

### ✨ New Features
- **Real-time username updates**: Username changes are immediately synchronized between frontend and backend
- **Username change notifications**: System automatically announces when users change their names
- **Enhanced message protocol**: Added support for different message types (chat messages vs. username updates)

### 🔧 Technical Changes

#### Frontend (script.js)
- Added message type system to distinguish between chat messages and username updates
- Implemented automatic username synchronization when input field changes
- Added initial username sync on connection establishment
- Enhanced message sending to include message type

#### Backend (main.rs)
- Added `IncomingMessage` struct to handle different message types
- Created `User` struct to track username and connection for each user
- Implemented username update message handling
- Added system notifications for username changes
- Maintained backward compatibility with legacy message format

### 🧪 Testing
- Verified username changes are immediately reflected in messages
- Confirmed system notifications work correctly
- Tested backward compatibility with existing message format
- Validated real-time synchronization between multiple clients

---

## Version 1.0.0 - Initial Release

### ✨ Features
- Real-time chat messaging using WebSockets
- Beautiful responsive web interface
- Auto-generated user IDs
- Join/leave notifications
- Message timestamps
- Cross-platform compatibility
- Mobile-friendly design

