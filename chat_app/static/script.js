// ==========================
// Global Variables
// ==========================
let socket = null;
let currentUser = null;
let authToken = null;
let currentConversation = null;
let currentGroup = null;
let contacts = [];
let memberGroups = []; // Groups user is a member of
let availableGroups = []; // Groups user can join
let reactionPickerTimeout = null;
let pendingFile = null;
let gameModal, closeGameModal, gameContainer, gameBoard, gameInfo, gameControls;
let currentGame = null;

// DOM element variables (will be set after DOM loads)
let authContainer, messengerContainer, loginForm, registerForm;
let showRegisterLink, showLoginLink, loginBtn, registerBtn, logoutBtn;
let currentUserSpan, contactsList, groupsList, availableGroupsList, welcomeScreen, chatHeader;
let messagesContainer, messageInputArea, messagesDiv, messageInput, sendBtn;
let chatWithUsername, imageInput, imageBtn;
let createGroupBtn, createGroupModal, closeGroupModal, submitGroupBtn;
let groupNameInput, groupDescInput, groupMembersInput;

// Group management elements
let groupMenu, groupMenuBtn, groupMenuDropdown;
let addMembersBtn, viewMembersBtn, editGroupBtn, leaveGroupBtn;
let addMembersModal, closeAddMembersModal, newMembersInput, submitAddMembersBtn;
let viewMembersModal, closeViewMembersModal, membersList;

// Poll elements
let createPollModal, closePollModal, submitPollBtn;
let pollQuestionInput, pollOptionsContainer, addPollOptionBtn;
let allowMultipleCheckbox, pollExpiresInput;
let activePollsContainer;

// AI Assistant variables
let aiAssistantOpen = false;
let aiAssistantWidget, aiAssistantToggle, aiAssistantChat, aiAssistantClose;
let aiMessagesContainer, aiMessageInput, aiSendBtn;

// ==========================
// Helper Functions
// ==========================
function showAuthInterface() {
    authContainer.style.display = 'flex';
    messengerContainer.style.display = 'none';
}

function showMessengerInterface() {
    authContainer.style.display = 'none';
    messengerContainer.style.display = 'flex';
    currentUserSpan.textContent = currentUser;
    connectWebSocket();
}

function showLoginForm() {
    loginForm.style.display = 'block';
    registerForm.style.display = 'none';
    clearErrors();
}

function showRegisterForm() {
    loginForm.style.display = 'none';
    registerForm.style.display = 'block';
    clearErrors();
}

function clearErrors() {
    document.getElementById('login-error').style.display = 'none';
    document.getElementById('register-error').style.display = 'none';
}

function showError(element, message) {
    element.textContent = message;
    element.style.display = 'block';
}

function getCurrentTime() {
    return new Date().toTimeString().split(' ')[0];
}

function clearMessages() {
    messagesDiv.innerHTML = '';
}

function clearContacts() {
    contactsList.innerHTML = '';
    contacts = [];
}

function clearGroups() {
    groupsList.innerHTML = '';
    if (availableGroupsList) availableGroupsList.innerHTML = '';
    memberGroups = [];
    availableGroups = [];
}

function scrollToBottom() {
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
}

// Initialize AI Assistant after DOM loads
function initializeAIAssistant() {
    aiAssistantWidget = document.getElementById('ai-assistant-widget');
    aiAssistantToggle = document.getElementById('ai-assistant-toggle');
    aiAssistantChat = document.getElementById('ai-assistant-chat');
    aiAssistantClose = document.getElementById('ai-assistant-close');
    aiMessagesContainer = document.getElementById('ai-assistant-messages');
    aiMessageInput = document.getElementById('ai-message-input');
    aiSendBtn = document.getElementById('ai-send-btn');

    if (!aiAssistantWidget) return;

    // Event listeners
    aiAssistantToggle.addEventListener('click', toggleAIAssistant);
    aiAssistantClose.addEventListener('click', closeAIAssistant);
    aiSendBtn.addEventListener('click', sendAIMessage);
    
    aiMessageInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendAIMessage();
        }
    });

    // Auto-resize textarea
    aiMessageInput.addEventListener('input', function() {
        this.style.height = 'auto';
        this.style.height = Math.min(this.scrollHeight, 80) + 'px';
    });

    // Show widget if user is logged in (check both current session and localStorage)
    if (currentUser && authToken) {
        console.log('Showing AI Assistant for logged in user');
        aiAssistantWidget.style.display = 'block';
    }
}

function toggleAIAssistant() {
    aiAssistantOpen = !aiAssistantOpen;
    if (aiAssistantOpen) {
        aiAssistantChat.classList.add('open');
        aiMessageInput.focus();
    } else {
        aiAssistantChat.classList.remove('open');
    }
}

function closeAIAssistant() {
    aiAssistantOpen = false;
    aiAssistantChat.classList.remove('open');
}

function showAIAssistant() {
    if ((currentUser && authToken) || (localStorage.getItem('currentUser') && localStorage.getItem('authToken'))) {
        console.log('Showing AI Assistant');
        if (aiAssistantWidget) {
            aiAssistantWidget.style.display = 'block';
        }
    }
}

function hideAIAssistant() {
    aiAssistantWidget.style.display = 'none';
    closeAIAssistant();
}

async function sendAIMessage() {
    const message = aiMessageInput.value.trim();
    if (!message || !authToken) return;

    // Add user message to chat
    addAIMessage(message, 'user');
    aiMessageInput.value = '';
    aiMessageInput.style.height = 'auto';

    // Show typing indicator
    showAITyping();

    try {
        const response = await fetch('/ai/assistant', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({
                query: message,
                context_type: currentGroup ? 'group' : (currentConversation ? 'conversation' : 'general'),
                target_name: currentGroup ? currentGroup.name : currentConversation
            })
        });

        const data = await response.json();
        
        // Remove typing indicator
        hideAITyping();

        if (response.ok && data.success) {
            addAIMessage(data.response, 'bot');
        } else {
            addAIMessage(data.response || 'Sorry, I encountered an error. Please try again.', 'bot');
        }

    } catch (error) {
        hideAITyping();
        addAIMessage('Sorry, I\'m having trouble connecting. Please try again later.', 'bot');
        console.error('AI Assistant error:', error);
    }
}

function sendQuickMessage(message) {
    aiMessageInput.value = message;
    sendAIMessage();
}

function addAIMessage(message, sender) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `ai-message ${sender}`;
    
    const content = document.createElement('div');
    content.innerHTML = formatAIMessage(message);
    
    const time = document.createElement('div');
    time.className = 'ai-message-time';
    time.textContent = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    
    messageDiv.appendChild(content);
    messageDiv.appendChild(time);
    
    aiMessagesContainer.appendChild(messageDiv);
    aiMessagesContainer.scrollTop = aiMessagesContainer.scrollHeight;
}

function formatAIMessage(message) {
    // Format markdown-like text
    return message
        .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
        .replace(/\*(.*?)\*/g, '<em>$1</em>')
        .replace(/\n\n/g, '<br><br>')
        .replace(/\n/g, '<br>')
        .replace(/â€¢/g, '•');
}

function showAITyping() {
    const typingDiv = document.createElement('div');
    typingDiv.className = 'ai-typing-indicator';
    typingDiv.id = 'ai-typing-indicator';
    
    typingDiv.innerHTML = `
        <div class="ai-typing-dots">
            <div class="ai-typing-dot"></div>
            <div class="ai-typing-dot"></div>
            <div class="ai-typing-dot"></div>
        </div>
        <span>AI is thinking...</span>
    `;
    
    aiMessagesContainer.appendChild(typingDiv);
    aiMessagesContainer.scrollTop = aiMessagesContainer.scrollHeight;
}

function hideAITyping() {
    const typingIndicator = document.getElementById('ai-typing-indicator');
    if (typingIndicator) {
        typingIndicator.remove();
    }
}

// Call this when user logs in
function onUserLogin() {
    showAIAssistant();
}

// Call this when user logs out  
function onUserLogout() {
    hideAIAssistant();
}

// Initialize game elements - FIXED
function initializeGameElements() {
    console.log('Initializing game elements...');
    
    // Try to initialize game modal elements multiple times if needed
    const tryInitialize = () => {
        gameModal = document.getElementById('game-modal');
        closeGameModal = document.getElementById('close-game-modal');
        gameContainer = document.getElementById('game-container');
        gameBoard = document.getElementById('game-board');
        gameInfo = document.getElementById('game-info');
        gameControls = document.getElementById('game-controls');
        
        if (!gameModal) {
            console.warn('Game modal not found, will retry when needed');
            return false;
        }
        
        // Add close event listener
        if (closeGameModal) {
            closeGameModal.addEventListener('click', () => {
                gameModal.style.display = 'none';
                currentGame = null;
            });
        }
        
        // Close modal on outside click
        gameModal.addEventListener('click', (e) => {
            if (e.target === gameModal) {
                gameModal.style.display = 'none';
                currentGame = null;
            }
        });
        
        console.log('Game elements initialized successfully');
        return true;
    };
    
    // Try to initialize immediately
    if (!tryInitialize()) {
        // If it fails, we'll try again when displayGame is called
        console.log('Game elements initialization deferred');
    }
}

// Game creation function - UPDATED
function createGame(gameType) {
    console.log('Creating game of type:', gameType);
    console.log('Current conversation:', currentConversation);
    console.log('Current group:', currentGroup);
    console.log('Socket state:', socket ? socket.readyState : 'no socket');

    if (!socket || socket.readyState !== WebSocket.OPEN) {
        console.error('WebSocket not connected');
        alert('Connection error. Please try again.');
        return;
    }

    const gameMessage = {
        type: 'create_game',
        game_type: gameType
    };

    if (currentGroup) {
        gameMessage.group_id = currentGroup.id;
        console.log('Creating game in group:', currentGroup.id);
    } else if (currentConversation) {
        gameMessage.target_username = currentConversation;
        console.log('Creating game with user:', currentConversation);
    } else {
        alert('Please select a conversation or group first');
        return;
    }

    console.log('Sending game creation message:', gameMessage);
    socket.send(JSON.stringify(gameMessage));
    
    // Show loading indicator
    showNotification(`Creating ${getGameName(gameType)} game...`, 'info');
}

// Display game message in chat - IMPROVED FOR GAME JOINING
function displayGameMessage(message, historical = false) {
    if (message.message && message.message.includes('🎮')) {
        const existingMessage = document.querySelector(`[data-message-id="${message.id}"]`);
        if (existingMessage && message.id) {
            return;
        }

        const msgDiv = document.createElement('div');
        msgDiv.className = `message ${message.sender_username === currentUser ? 'sent' : 'received'} game-message`;
        msgDiv.dataset.messageId = message.id;
        if (historical) msgDiv.classList.add('historical');

        const header = document.createElement('div');
        header.className = 'message-header';
        header.textContent = `${message.sender_username} • ${message.timestamp}`;

        const content = document.createElement('div');
        content.className = 'message-content';
        content.textContent = message.message;

        const gameButton = document.createElement('button');
        gameButton.className = 'game-view-btn';
        
        if (message.message.includes('created') || message.message.includes('started')) {
            // For newly created games, show appropriate button based on who created it
            if (message.sender_username === currentUser) {
                gameButton.textContent = 'View Game';
            } else {
                gameButton.textContent = 'Join Game';
            }
            const gameId = extractGameId(message.message);
            gameButton.addEventListener('click', () => {
                if (message.sender_username === currentUser) {
                    loadGame(gameId);
                } else {
                    joinGame(gameId);
                }
            });
        } else if (message.message.includes('made a move')) {
            gameButton.textContent = 'View Game';
            const gameId = extractGameId(message.message);
            gameButton.addEventListener('click', () => loadGame(gameId));
        }

        msgDiv.appendChild(header);
        msgDiv.appendChild(content);
        msgDiv.appendChild(gameButton);
        
        messagesDiv.appendChild(msgDiv);
        if (!historical) scrollToBottom();
    } else {
        displayMessage(message, historical);
    }
}

function extractGameId(message) {
    const match = message.match(/Game ID: (\d+)|game #(\d+)/);
    return match ? parseInt(match[1] || match[2]) : null;
}

function joinGame(gameId) {
    if (!socket || socket.readyState !== WebSocket.OPEN) {
        alert('Connection error. Please try again.');
        return;
    }

    console.log('Joining game:', gameId);
    socket.send(JSON.stringify({
        type: 'join_game',
        game_id: gameId
    }));
    
    showNotification('Joining game...', 'info');
}

function loadGame(gameId) {
    if (!socket || socket.readyState !== WebSocket.OPEN) {
        alert('Connection error. Please try again.');
        return;
    }

    console.log('Loading game:', gameId);
    socket.send(JSON.stringify({
        type: 'get_game_state',
        game_id: gameId
    }));
}

// COMPLETELY NEW SAFE DISPLAY GAME FUNCTION
function displayGame(game) {
    console.log('=== displayGame called ===');
    console.log('Game data:', game);
    
    // Get elements directly from DOM every time
    const modal = document.getElementById('game-modal');
    const info = document.getElementById('game-info');
    const board = document.getElementById('game-board');
    
    console.log('DOM elements found:');
    console.log('modal:', modal);
    console.log('info:', info);
    console.log('board:', board);
    
    if (!modal) {
        console.error('CRITICAL: game-modal element not found');
        alert('Game modal not found. Check HTML structure.');
        return;
    }
    
    if (!info) {
        console.error('CRITICAL: game-info element not found');
        alert('Game info element not found. Check HTML structure.');
        return;
    }
    
    if (!board) {
        console.error('CRITICAL: game-board element not found');
        alert('Game board element not found. Check HTML structure.');
        return;
    }
    
    // Set global variables
    gameModal = modal;
    gameInfo = info;
    gameBoard = board;
    currentGame = game;
    
    console.log('About to set modal style...');
    
    try {
        modal.style.display = 'block';
        console.log('Modal display set to block');
    } catch (error) {
        console.error('Error setting modal style:', error);
        alert('Error showing game modal: ' + error.message);
        return;
    }
    
    console.log('About to set game info...');
    
    try {
        info.innerHTML = `
            <div class="game-header">
                <h3>${getGameIcon(game.game_type)} ${getGameName(game.game_type)}</h3>
                <div class="game-players">
                    <span class="player player1">${game.player1_username}</span>
                    ${game.player2_username ? 
                        `<span class="vs">vs</span><span class="player player2">${game.player2_username}</span>` :
                        '<span class="waiting">Waiting for player...</span>'
                    }
                </div>
                <div class="game-status">${getGameStatus(game)}</div>
            </div>
        `;
        console.log('Game info set successfully');
    } catch (error) {
        console.error('Error setting game info:', error);
        alert('Error setting game info: ' + error.message);
        return;
    }

    console.log('About to display game board...');
    
    try {
        // Display game board based on type
        switch (game.game_type) {
            case 'chess':
                console.log('Displaying chess board');
                displayChessBoard(game);
                break;
            case 'tictactoe':
                console.log('Displaying tic-tac-toe board');
                displayTicTacToeBoard(game);
                break;
            case 'trivia':
                console.log('Displaying trivia game');
                displayTriviaGame(game);
                break;
            default:
                console.warn('Unknown game type:', game.game_type);
        }
        console.log('Game board displayed successfully');
    } catch (error) {
        console.error('Error displaying game board:', error);
        alert('Error displaying game board: ' + error.message);
        return;
    }
    
    console.log('=== displayGame completed successfully ===');
}

function getGameIcon(gameType) {
    switch (gameType) {
        case 'chess': return '♟️';
        case 'tictactoe': return '⭕';
        case 'trivia': return '🧠';
        default: return '🎮';
    }
}

function getGameName(gameType) {
    switch (gameType) {
        case 'chess': return 'Chess';
        case 'tictactoe': return 'Tic-Tac-Toe';
        case 'trivia': return 'Trivia';
        default: return 'Game';
    }
}

function getGameStatus(game) {
    if (game.status === 'finished') {
        if (game.winner === 'draw') {
            return "🤝 Game ended in a draw";
        } else if (game.winner === currentUser) {
            return "🏆 You won!";
        } else {
            return `🏆 ${game.winner} wins!`;
        }
    } else if (game.status === 'waiting') {
        // Improve waiting status messaging
        if (game.player1_username === currentUser) {
            return "⏳ Waiting for another player to join. Share the game ID with someone!";
        } else {
            return "⏳ Waiting for another player to join";
        }
    } else if (game.game_type === 'trivia') {
        // Special handling for trivia games
        const gameState = JSON.parse(game.game_state);
        const answered = gameState.answered || [];
        
        if (answered.includes(currentUser)) {
            if (answered.length === 1) {
                const otherPlayer = game.player1_username === currentUser ? game.player2_username : game.player1_username;
                return `⏳ Waiting for ${otherPlayer} to answer`;
            } else {
                return "⏳ All players answered - processing results...";
            }
        } else {
            return "🎯 Your turn to answer the question!";
        }
    } else if (game.current_turn === currentUser) {
        return "🎯 Your turn - make your move!";
    } else {
        return `⏳ ${game.current_turn}'s turn`;
    }
}

// Chess board display
function displayChessBoard(game) {
    const gameState = JSON.parse(game.game_state);
    const board = gameState.board;
    
    gameBoard.innerHTML = '<div class="chess-board"></div>';
    const chessBoard = gameBoard.querySelector('.chess-board');
    
    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const square = document.createElement('div');
            square.className = `chess-square ${(row + col) % 2 === 0 ? 'light' : 'dark'}`;
            square.dataset.row = row;
            square.dataset.col = col;
            
            const piece = board[row][col];
            if (piece !== '.') {
                square.textContent = getChessPieceSymbol(piece);
                square.classList.add('has-piece');
            }
            
            if (game.current_turn === currentUser && game.status === 'active') {
                square.addEventListener('click', () => handleChessSquareClick(row, col));
            }
            
            chessBoard.appendChild(square);
        }
    }
}

function getChessPieceSymbol(piece) {
    const symbols = {
        'K': '♔', 'Q': '♕', 'R': '♖', 'B': '♗', 'N': '♘', 'P': '♙',
        'k': '♚', 'q': '♛', 'r': '♜', 'b': '♝', 'n': '♞', 'p': '♟'
    };
    return symbols[piece] || piece;
}

let selectedChessSquare = null;

function handleChessSquareClick(row, col) {
    if (!selectedChessSquare) {
        // Select piece
        const square = document.querySelector(`[data-row="${row}"][data-col="${col}"]`);
        if (square.classList.contains('has-piece')) {
            selectedChessSquare = { row, col };
            square.classList.add('selected');
        }
    } else {
        // Make move
        const moveData = {
            from: [selectedChessSquare.row, selectedChessSquare.col],
            to: [row, col]
        };
        
        makeGameMove(JSON.stringify(moveData));
        
        // Clear selection
        document.querySelector('.selected')?.classList.remove('selected');
        selectedChessSquare = null;
    }
}

// Tic-tac-toe board display - IMPROVED FOR WAITING STATE
function displayTicTacToeBoard(game) {
    const gameState = JSON.parse(game.game_state);
    const board = gameState.board;
    
    gameBoard.innerHTML = '<div class="tictactoe-board"></div>';
    const tictactoeBoard = gameBoard.querySelector('.tictactoe-board');
    
    // If game is waiting for a second player, show a message
    if (game.status === 'waiting') {
        const waitingMessage = document.createElement('div');
        waitingMessage.className = 'waiting-for-player';
        waitingMessage.innerHTML = `
            <div style="text-align: center; padding: 20px; background: #f8f9fa; border-radius: 8px; margin-bottom: 20px;">
                <h4>🎮 Game ID: ${game.id}</h4>
                <p>Share this Game ID with someone so they can join!</p>
                ${game.player1_username !== currentUser ? '<p><strong>Click "Join Game" to start playing!</strong></p>' : ''}
            </div>
        `;
        gameBoard.insertBefore(waitingMessage, tictactoeBoard);
    }
    
    for (let row = 0; row < 3; row++) {
        for (let col = 0; col < 3; col++) {
            const square = document.createElement('div');
            square.className = 'tictactoe-square';
            square.dataset.row = row;
            square.dataset.col = col;
            square.textContent = board[row][col];
            
            // Only allow clicks if:
            // 1. Game is active (not waiting)
            // 2. It's current user's turn  
            // 3. Square is empty
            if (game.status === 'active' && game.current_turn === currentUser && !board[row][col]) {
                square.addEventListener('click', () => handleTicTacToeClick(row, col));
                square.classList.add('clickable');
            } else if (game.status === 'waiting') {
                // Show that squares are not clickable yet
                square.classList.add('waiting');
            }
            
            tictactoeBoard.appendChild(square);
        }
    }
}

function handleTicTacToeClick(row, col) {
    const moveData = { row, col };
    makeGameMove(JSON.stringify(moveData));
}

// Trivia game display - ENHANCED WITH BETTER STATE HANDLING
function displayTriviaGame(game) {
    const gameState = JSON.parse(game.game_state);
    const question = gameState.current_question;
    const scores = gameState.scores || {};
    const answered = gameState.answered || [];
    
    console.log('Displaying trivia game state:', gameState);
    console.log('Players answered:', answered);
    console.log('Current scores:', scores);
    
    gameBoard.innerHTML = `
        <div class="trivia-game">
            <div class="trivia-scores">
                <h4>Scores:</h4>
                <div class="score-list">
                    <div class="score-item">
                        <span>${game.player1_username}:</span>
                        <span>${scores[game.player1_username] || 0}</span>
                    </div>
                    ${game.player2_username ? `
                        <div class="score-item">
                            <span>${game.player2_username}:</span>
                            <span>${scores[game.player2_username] || 0}</span>
                        </div>
                    ` : ''}
                </div>
            </div>
            
            <div class="trivia-question">
                <div class="question-category">${question ? question.category : 'Loading...'}</div>
                <div class="question-text">${question ? question.question : 'Loading question...'}</div>
                
                ${!question ? 
                    '<div class="loading-notice">Loading next question...</div>' :
                    answered.includes(currentUser) ? 
                        `<div class="answered-notice">
                            <p><strong>You have answered this question!</strong></p>
                            ${answered.length < 2 ? 
                                `<p>Waiting for ${answered.length === 1 && game.player2_username ? 
                                    (currentUser === game.player1_username ? game.player2_username : game.player1_username) + ' to answer...' : 
                                    'other players to answer...'}</p>` :
                                '<p>All players have answered! Processing results...</p>'
                            }
                            <div class="answered-players">
                                <p>Answered: ${answered.join(', ')}</p>
                            </div>
                            ${answered.length >= 2 ? 
                                '<button class="next-question-btn" onclick="requestNextQuestion()">Continue to Next Question</button>' : ''
                            }
                        </div>` : 
                        `<div class="question-options">
                            ${question.options.map((option, index) => `
                                <button class="trivia-option" 
                                        data-answer="${index}">
                                    ${option}
                                </button>
                            `).join('')}
                        </div>
                        <div class="answer-prompt">Choose your answer:</div>`
                }
            </div>
        </div>
    `;
    
    // Add click handlers for options only if user hasn't answered and question exists
    if (question && !answered.includes(currentUser) && game.status === 'active') {
        const options = gameBoard.querySelectorAll('.trivia-option');
        options.forEach(option => {
            option.addEventListener('click', () => {
                const answer = parseInt(option.dataset.answer);
                console.log('Trivia answer selected:', answer);
                handleTriviaAnswer(answer);
            });
        });
    }
}

// Add function to request next question
function requestNextQuestion() {
    if (!currentGame || !socket || socket.readyState !== WebSocket.OPEN) {
        console.error('Cannot request next question: no game or connection');
        return;
    }
    
    console.log('Requesting next question for game:', currentGame.id);
    
    socket.send(JSON.stringify({
        type: 'game_move',
        game_id: currentGame.id,
        game_move: JSON.stringify({
            action: 'next_question',
            player: currentUser
        })
    }));
}

function handleTriviaAnswer(answer) {
    if (!currentGame) {
        console.error('No current game set');
        return;
    }
    
    console.log('=== TRIVIA ANSWER WORKAROUND ===');
    console.log('Using chat-based trivia answer system');
    console.log('Answer selected:', answer);
    
    // Check if user has already answered
    const gameState = JSON.parse(currentGame.game_state);
    const answered = gameState.answered || [];
    
    if (answered.includes(currentUser)) {
        console.log('User has already answered this question');
        alert('You have already answered this question!');
        return;
    }
    
    // Instead of using game_move, send as a special chat message
    // This bypasses the turn validation completely
    if (socket && socket.readyState === WebSocket.OPEN) {
        const triviaAnswerMessage = {
            type: 'chat_message',
            receiver_username: currentConversation,
            group_id: currentGroup ? currentGroup.id : null,
            message: `🧠 TRIVIA_ANSWER:${currentGame.id}:${answer}:${currentUser}:${Date.now()}`,
            timestamp: getCurrentTime()
        };
        
        console.log('Sending trivia answer via chat message:', triviaAnswerMessage);
        socket.send(JSON.stringify(triviaAnswerMessage));
        
        // Disable answer options immediately
        const options = gameBoard.querySelectorAll('.trivia-option');
        options.forEach(option => {
            option.disabled = true;
            option.style.opacity = '0.5';
            option.style.cursor = 'not-allowed';
        });
        
        // Show immediate feedback
        const answerPrompt = gameBoard.querySelector('.answer-prompt');
        if (answerPrompt) {
            answerPrompt.innerHTML = `<div style="color: #28a745; font-weight: bold;">Answer ${answer + 1} submitted! Waiting for other players...</div>`;
        }
        
        // Update local game state to show user has answered
        const updatedGameState = JSON.parse(currentGame.game_state);
        updatedGameState.answered = updatedGameState.answered || [];
        if (!updatedGameState.answered.includes(currentUser)) {
            updatedGameState.answered.push(currentUser);
        }
        currentGame.game_state = JSON.stringify(updatedGameState);
        
        console.log('Updated local game state:', updatedGameState);
        console.log('=== END TRIVIA ANSWER WORKAROUND ===');
        
        // Refresh the display to show updated state
        displayTriviaGame(currentGame);
    } else {
        console.error('WebSocket not connected');
        alert('Connection error. Please try again.');
    }
}

// Make a move in the current game
function makeGameMove(moveData) {
    if (!currentGame || !socket || socket.readyState !== WebSocket.OPEN) {
        return;
    }

    socket.send(JSON.stringify({
        type: 'game_move',
        game_id: currentGame.id,
        game_move: moveData
    }));
    
    // Disable game interactions temporarily
    disableGameInteractions();
    
    // Re-enable after a short delay (will be overridden by game update)
    setTimeout(enableGameInteractions, 2000);
}

function disableGameInteractions() {
    if (gameBoard) {
        gameBoard.style.pointerEvents = 'none';
        gameBoard.style.opacity = '0.7';
    }
}

function enableGameInteractions() {
    if (gameBoard) {
        gameBoard.style.pointerEvents = 'auto';
        gameBoard.style.opacity = '1';
    }
}

// Handle game-specific WebSocket messages - FIXED
function handleGameWebSocketMessage(data) {
    console.log('Handling game WebSocket message:', data);
    
    switch (data.type) {
        case 'game_state':
            console.log('Calling displayGame from game_state');
            displayGame(data.game);
            break;
        case 'game_update':
            if (currentGame && currentGame.id === data.game.id) {
                console.log('Calling displayGame from game_update');
                displayGame(data.game);
            }
            // Also show a notification if game finished
            if (data.game.status === 'finished') {
                showGameFinishedNotification(data.game);
            }
            break;
        case 'game_created':
            console.log('Game created:', data.game);
            // Optionally auto-open the game
            if (data.game.player1_username === currentUser) {
                console.log('Auto-opening game for creator');
                setTimeout(() => {
                    console.log('Calling displayGame from game_created timeout');
                    displayGame(data.game);
                }, 500);
            }
            break;
        case 'game_joined':
            console.log('Game joined:', data.game);
            // Auto-open the game for the joining player
            setTimeout(() => {
                console.log('Calling displayGame from game_joined timeout');
                displayGame(data.game);
            }, 500);
            break;
        case 'game_error':
            console.error('Game error:', data.error);
            alert('Game error: ' + data.error);
            break;
    }
}

function showGameFinishedNotification(game) {
    console.log('=== GAME FINISHED NOTIFICATION DEBUG ===');
    console.log('Game:', game);
    console.log('Current user:', currentUser);
    console.log('Player1:', game.player1_username);
    console.log('Player2:', game.player2_username);
    console.log('Winner:', game.winner);
    console.log('Game status:', game.status);
    
    // Check if current user is part of this game
    const isPlayer1 = game.player1_username === currentUser;
    const isPlayer2 = game.player2_username === currentUser;
    const isMyGame = isPlayer1 || isPlayer2;
    
    console.log('Is my game:', isMyGame);
    console.log('Am I player1:', isPlayer1);
    console.log('Am I player2:', isPlayer2);
    
    if (!isMyGame) {
        console.log('Not my game, skipping notification');
        return;
    }
    
    if (game.winner === 'draw') {
        console.log('Showing draw notification');
        showNotification('Game ended in a draw! 🤝', 'info');
    } else if (game.winner === currentUser) {
        console.log('Showing win notification');
        showNotification('Congratulations! You won! 🏆', 'success');
    } else {
        console.log('Showing lose notification');
        const opponent = isPlayer1 ? game.player2_username : game.player1_username;
        showNotification(`Game finished. ${opponent} won! Better luck next time.`, 'info');
    }
    
    console.log('=== END GAME FINISHED NOTIFICATION DEBUG ===');
}

function showNotification(message, type = 'info') {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    
    // Style the notification
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: ${type === 'success' ? '#28a745' : type === 'error' ? '#dc3545' : '#17a2b8'};
        color: white;
        padding: 12px 20px;
        border-radius: 8px;
        z-index: 10000;
        font-weight: 500;
        box-shadow: 0 4px 12px rgba(0,0,0,0.2);
        animation: slideInRight 0.3s ease-out;
    `;
    
    document.body.appendChild(notification);
    
    // Auto-remove after 5 seconds
    setTimeout(() => {
        notification.style.animation = 'slideOutRight 0.3s ease-out';
        setTimeout(() => {
            if (document.body.contains(notification)) {
                document.body.removeChild(notification);
            }
        }, 300);
    }, 5000);
}

// ==========================
// Event Handlers
// ==========================
function setupEventListeners() {
    showRegisterLink.addEventListener('click', (e) => { e.preventDefault(); showRegisterForm(); });
    showLoginLink.addEventListener('click', (e) => { e.preventDefault(); showLoginForm(); });
    loginBtn.addEventListener('click', handleLogin);
    registerBtn.addEventListener('click', handleRegister);
    logoutBtn.addEventListener('click', handleLogout);
    sendBtn.addEventListener('click', sendMessage);

    messageInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); }
    });

    imageBtn.addEventListener('click', () => imageInput.click());
    imageInput.addEventListener('change', sendImage);

    document.getElementById('login-password').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') handleLogin();
    });
    document.getElementById('register-password').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') handleRegister();
    });

    // Group creation modal event listeners
    if (createGroupBtn && createGroupModal) {
        createGroupBtn.addEventListener('click', () => { 
            createGroupModal.style.display = 'block'; 
        });
    }

    if (closeGroupModal) {
        closeGroupModal.addEventListener('click', () => { 
            createGroupModal.style.display = 'none'; 
        });
    }

    if (submitGroupBtn) {
        submitGroupBtn.addEventListener('click', createGroup);
    }

    // Group management event listeners
    if (groupMenuBtn) {
        groupMenuBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            groupMenuDropdown.classList.toggle('show');
        });
    }

    // Close dropdown when clicking outside
    document.addEventListener('click', () => {
        if (groupMenuDropdown) {
            groupMenuDropdown.classList.remove('show');
        }
    });

    // Group menu item listeners
    if (addMembersBtn) {
        addMembersBtn.addEventListener('click', () => {
            groupMenuDropdown.classList.remove('show');
            addMembersModal.style.display = 'block';
        });
    }

    if (viewMembersBtn) {
        viewMembersBtn.addEventListener('click', () => {
            groupMenuDropdown.classList.remove('show');
            showGroupMembers();
        });
    }

    if (editGroupBtn) {
        editGroupBtn.addEventListener('click', () => {
            groupMenuDropdown.classList.remove('show');
            editGroup();
        });
    }

    if (leaveGroupBtn) {
        leaveGroupBtn.addEventListener('click', () => {
            groupMenuDropdown.classList.remove('show');
            leaveGroup();
        });
    }

    // Poll menu item listener
    const createPollMenuBtn = document.getElementById('create-poll-menu-btn');
    if (createPollMenuBtn) {
        createPollMenuBtn.addEventListener('click', () => {
            groupMenuDropdown.classList.remove('show');
            showCreatePollModal();
        });
    }

    // Add members modal listeners
    if (closeAddMembersModal) {
        closeAddMembersModal.addEventListener('click', () => {
            addMembersModal.style.display = 'none';
        });
    }

    if (submitAddMembersBtn) {
        submitAddMembersBtn.addEventListener('click', addMembersToGroup);
    }

    // View members modal listeners
    if (closeViewMembersModal) {
        closeViewMembersModal.addEventListener('click', () => {
            viewMembersModal.style.display = 'none';
        });
    }

    // Poll creation event listeners
    if (addPollOptionBtn) {
        addPollOptionBtn.addEventListener('click', addPollOption);
    }
    
    if (closePollModal) {
        closePollModal.addEventListener('click', () => {
            createPollModal.style.display = 'none';
            resetPollForm();
        });
    }
    
    if (submitPollBtn) {
        submitPollBtn.addEventListener('click', createPoll);
    }

    // Close modals when clicking outside
    window.addEventListener('click', (e) => {
        if (e.target === createGroupModal) createGroupModal.style.display = 'none';
        if (e.target === addMembersModal) addMembersModal.style.display = 'none';
        if (e.target === viewMembersModal) viewMembersModal.style.display = 'none';
        if (e.target === createPollModal) {
            createPollModal.style.display = 'none';
            resetPollForm();
        }
    });
}

// ==========================
// Poll Functions
// ==========================
function showCreatePollModal() {
    if (!currentGroup) {
        alert('No group selected');
        return;
    }
    
    resetPollForm();
    createPollModal.style.display = 'block';
}

function resetPollForm() {
    pollQuestionInput.value = '';
    allowMultipleCheckbox.checked = false;
    pollExpiresInput.value = '';
    
    // Reset poll options to default 2 options
    pollOptionsContainer.innerHTML = `
        <div class="poll-option-input">
            <input type="text" placeholder="Option 1" required>
            <button type="button" class="remove-option-btn" onclick="removePollOption(this)">✕</button>
        </div>
        <div class="poll-option-input">
            <input type="text" placeholder="Option 2" required>
            <button type="button" class="remove-option-btn" onclick="removePollOption(this)">✕</button>
        </div>
    `;
}

function addPollOption() {
    const optionCount = pollOptionsContainer.children.length;
    if (optionCount >= 10) {
        alert('Maximum 10 options allowed');
        return;
    }
    
    const optionDiv = document.createElement('div');
    optionDiv.className = 'poll-option-input';
    optionDiv.innerHTML = `
        <input type="text" placeholder="Option ${optionCount + 1}" required>
        <button type="button" class="remove-option-btn" onclick="removePollOption(this)">✕</button>
    `;
    
    pollOptionsContainer.appendChild(optionDiv);
}

function removePollOption(button) {
    const optionDiv = button.parentElement;
    if (pollOptionsContainer.children.length > 2) {
        optionDiv.remove();
        
        // Update placeholders
        Array.from(pollOptionsContainer.children).forEach((div, index) => {
            const input = div.querySelector('input');
            input.placeholder = `Option ${index + 1}`;
        });
    } else {
        alert('Minimum 2 options required');
    }
}

function createPoll() {
    if (!currentGroup) {
        alert('No group selected');
        return;
    }
    
    const question = pollQuestionInput.value.trim();
    if (!question) {
        alert('Poll question is required');
        return;
    }
    
    const options = Array.from(pollOptionsContainer.children)
        .map(div => div.querySelector('input').value.trim())
        .filter(option => option.length > 0);
    
    if (options.length < 2) {
        alert('At least 2 options are required');
        return;
    }
    
    const allowMultiple = allowMultipleCheckbox.checked;
    const expiresAt = pollExpiresInput.value || null;
    
    if (socket && socket.readyState === WebSocket.OPEN) {
        const pollMessage = {
            type: 'create_poll',
            group_id: currentGroup.id,
            poll_question: question,
            poll_options: options,
            poll_allow_multiple: allowMultiple,
            poll_expires_at: expiresAt,
            receiver_username: null,
            message: null,
            timestamp: null,
            message_id: null,
            emoji: null,
            poll_id: null,
            poll_option_ids: null
        };
        
        console.log('Sending poll creation message:', pollMessage);
        socket.send(JSON.stringify(pollMessage));
        
        createPollModal.style.display = 'none';
        resetPollForm();
        
        console.log('Poll creation message sent successfully');
    } else {
        alert('Connection error. Please try again.');
        console.error('WebSocket not connected. ReadyState:', socket ? socket.readyState : 'socket is null');
    }
}

function displayPollMessage(message, historical = false) {
    if (message.message && message.message.includes('📊 Poll')) {
        const existingMessage = document.querySelector(`[data-message-id="${message.id}"]`);
        if (existingMessage && message.id) {
            console.log('Duplicate poll message prevented (DOM check):', message.id);
            return;
        }

        const msgDiv = document.createElement('div');
        msgDiv.className = `message ${message.sender_username === currentUser ? 'sent' : 'received'} poll-message`;
        msgDiv.dataset.messageId = message.id;
        if (historical) msgDiv.classList.add('historical');

        const header = document.createElement('div');
        header.className = 'message-header';
        header.textContent = `${message.sender_username} • ${message.timestamp}`;

        const content = document.createElement('div');
        content.className = 'message-content';
        content.textContent = message.message;

        const pollButton = document.createElement('button');
        pollButton.className = 'poll-view-btn';
        pollButton.textContent = 'View Poll';
        pollButton.addEventListener('click', () => loadPollDetails(message.id));

        msgDiv.appendChild(header);
        msgDiv.appendChild(content);
        msgDiv.appendChild(pollButton);
        
        messagesDiv.appendChild(msgDiv);
        if (!historical) scrollToBottom();
    } else {
        displayMessage(message, historical);
    }
}

function loadPollDetails(pollId) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({
            type: 'get_poll_details',
            poll_id: pollId
        }));
    }
}

function displayPollDetails(pollData) {
    const pollModal = document.createElement('div');
    pollModal.className = 'poll-modal';
    
    const totalVotes = pollData.total_votes || 0;
    
    pollModal.innerHTML = `
        <div class="poll-modal-content">
            <span class="close-poll-details">&times;</span>
            <h3>${pollData.question}</h3>
            <p>Created by: ${pollData.creator_username} at ${pollData.created_at}</p>
            ${pollData.allow_multiple_choices ? '<p><em>Multiple choices allowed</em></p>' : ''}
            <div class="poll-options">
                ${pollData.options.map(option => `
                    <div class="poll-option" data-option-id="${option.id}">
                        <label>
                            <input type="${pollData.allow_multiple_choices ? 'checkbox' : 'radio'}" 
                                   name="poll-vote" 
                                   value="${option.id}"
                                   ${option.voted_by_current_user ? 'checked' : ''}>
                            ${option.option_text}
                        </label>
                        <div class="vote-count">${option.vote_count} votes</div>
                        <div class="vote-bar">
                            <div class="vote-progress" style="width: ${totalVotes > 0 ? (option.vote_count / totalVotes * 100) : 0}%"></div>
                        </div>
                    </div>
                `).join('')}
            </div>
            <p>Total votes: ${totalVotes}</p>
            <button class="vote-poll-btn" onclick="submitVote(${pollData.id}, ${pollData.allow_multiple_choices})">Submit Vote</button>
        </div>
    `;
    
    document.body.appendChild(pollModal);
    pollModal.style.display = 'block';
    
    pollModal.querySelector('.close-poll-details').addEventListener('click', () => {
        pollModal.remove();
    });
    
    pollModal.addEventListener('click', (e) => {
        if (e.target === pollModal) {
            pollModal.remove();
        }
    });
}

function submitVote(pollId, allowMultiple) {
    const checkedInputs = document.querySelectorAll('input[name="poll-vote"]:checked');
    const optionIds = Array.from(checkedInputs).map(input => parseInt(input.value));
    
    if (optionIds.length === 0) {
        alert('Please select at least one option');
        return;
    }
    
    if (!allowMultiple && optionIds.length > 1) {
        alert('Only one option allowed');
        return;
    }
    
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({
            type: 'vote_poll',
            poll_id: pollId,
            poll_option_ids: optionIds
        }));
        
        const pollModal = document.querySelector('.poll-modal');
        if (pollModal) {
            pollModal.remove();
        }
    }
}

// ==========================
// Auth Functions
// ==========================
async function handleLogin() {
    const username = document.getElementById('login-username').value.trim();
    const password = document.getElementById('login-password').value;
    const errorDiv = document.getElementById('login-error');

    if (!username || !password) { showError(errorDiv, 'Please fill in all fields'); return; }

    try {
        const response = await fetch('/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });
        const data = await response.json();

        if (response.ok) {
            authToken = data.token;
            currentUser = username;
            localStorage.setItem('authToken', authToken);
            localStorage.setItem('currentUser', currentUser);
            showMessengerInterface();
            loadContacts();
            loadGroups();
            showAIAssistant();
        } else {
            showError(errorDiv, data.error || 'Login failed');
        }
    } catch {
        showError(errorDiv, 'Network error. Please try again.');
    }
}

async function handleRegister() {
    const username = document.getElementById('register-username').value.trim();
    const password = document.getElementById('register-password').value;
    const errorDiv = document.getElementById('register-error');

    if (!username || !password) { showError(errorDiv, 'Please fill in all fields'); return; }
    if (password.length < 6) { showError(errorDiv, 'Password must be at least 6 characters'); return; }

    try {
        const response = await fetch('/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });
        const data = await response.json();

        if (response.ok) {
            document.getElementById('login-username').value = username;
            document.getElementById('login-password').value = password;
            showLoginForm();
            handleLogin();
        } else {
            showError(errorDiv, data.error || 'Registration failed');
        }
    } catch {
        showError(errorDiv, 'Network error. Please try again.');
    }
}

function handleLogout() {
    localStorage.removeItem('authToken');
    localStorage.removeItem('currentUser');
    authToken = null;
    currentUser = null;
    currentConversation = null;
    currentGroup = null;

    if (socket) { socket.close(); socket = null; }

    showAuthInterface();
    clearMessages();
    clearContacts();
    clearGroups();
    hideAIAssistant();
}

// ==========================
// Contacts
// ==========================
async function loadContacts() {
    try {
        console.log('Loading contacts...');
        const response = await fetch('/users', { headers: { 'Authorization': `Bearer ${authToken}` } });
        console.log('Contacts response status:', response.status);
        
        if (response.ok) {
            const data = await response.json();
            console.log('Contacts data received:', data);
            contacts = data.users || [];
            console.log('Contacts array now contains:', contacts);
            displayContacts();
        } else {
            console.error('Failed to load contacts, status:', response.status);
            const errorText = await response.text();
            console.error('Contacts error response:', errorText);
        }
    } catch (err) { 
        console.error('Failed to load contacts:', err); 
    }
}

function displayContacts() {
    contactsList.innerHTML = '';
    contacts.forEach(username => {
        const contactItem = document.createElement('div');
        contactItem.className = 'contact-item';
        
        contactItem.innerHTML = `
            <div class="contact-info">
                <span class="contact-name">${username}</span>
            </div>
            <button class="contact-highlight-btn" title="Get chat highlights" onclick="event.stopPropagation(); generateChatHighlight('${username}', 'personal')">✨</button>
        `;
        
        contactItem.addEventListener('click', () => selectContact(username, contactItem));
        contactsList.appendChild(contactItem);
    });
}

function selectContact(username, element) {
    console.log('Selecting contact:', username);
    
    document.querySelectorAll('.contact-item').forEach(item => item.classList.remove('active'));
    document.querySelectorAll('.group-item').forEach(item => item.classList.remove('active'));
    element.classList.add('active');

    currentConversation = username;
    currentGroup = null;
    chatWithUsername.textContent = username;
    chatHeader.style.display = 'block';
    welcomeScreen.style.display = 'none';
    messagesContainer.style.display = 'block';
    messageInputArea.style.display = 'block';
    messageInput.disabled = false;
    sendBtn.disabled = false;

    // Hide group menu for private chats but SHOW game buttons
    if (groupMenu) groupMenu.style.display = 'none';
    
    // Show game buttons for conversations
    const gameButtons = document.getElementById('game-buttons');
    if (gameButtons) {
        gameButtons.style.display = 'flex';
        console.log('Game buttons should now be visible for contact');
    }

    clearMessages();
    loadConversation(username);
}

// ==========================
// Groups
// ==========================
async function loadGroups() {
    try {
        console.log('Loading groups...');
        const response = await fetch('/groups', {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        console.log('Groups response status:', response.status);
        
        if (response.ok) {
            const data = await response.json();
            console.log('Groups data received:', data);
            
            memberGroups = data.member_groups || [];
            availableGroups = data.available_groups || [];
            
            displayGroups();
        } else {
            console.error('Failed to load groups, status:', response.status);
            const errorText = await response.text();
            console.error('Groups error response:', errorText);
        }
    } catch (err) {
        console.error('Failed to load groups:', err);
    }
}

function displayGroups() {
    groupsList.innerHTML = '';
    if (availableGroupsList) availableGroupsList.innerHTML = '';
    
    memberGroups.forEach(group => {
        const groupItem = document.createElement('div');
        groupItem.className = 'group-item';
        
        groupItem.innerHTML = `
            <div class="contact-info">
                <span class="contact-name">${group.name}</span>
            </div>
            <button class="contact-highlight-btn" title="Get group highlights" onclick="event.stopPropagation(); generateChatHighlight('${group.name}', 'group', ${group.id})">✨</button>
        `;
        
        groupItem.addEventListener('click', () => selectGroup(group, groupItem));
        groupsList.appendChild(groupItem);
    });

    if (availableGroupsList) {
        availableGroups.forEach(group => {
            const groupItem = document.createElement('div');
            groupItem.className = 'available-group-item';
            
            const groupName = document.createElement('span');
            groupName.className = 'group-name';
            groupName.textContent = group.name;
            
            const joinBtn = document.createElement('button');
            joinBtn.className = 'join-group-btn';
            joinBtn.textContent = 'Join';
            joinBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                joinGroup(group.id);
            });
            
            groupItem.appendChild(groupName);
            groupItem.appendChild(joinBtn);
            availableGroupsList.appendChild(groupItem);
        });
    }
}

function selectGroup(group, element) {
    console.log('Selecting group:', group.name);
    
    document.querySelectorAll('.contact-item').forEach(item => item.classList.remove('active'));
    document.querySelectorAll('.group-item').forEach(item => item.classList.remove('active'));
    element.classList.add('active');

    currentGroup = group;
    currentConversation = null;
    chatWithUsername.textContent = `Group: ${group.name}`;
    chatHeader.style.display = 'block';
    welcomeScreen.style.display = 'none';
    messagesContainer.style.display = 'block';
    messageInputArea.style.display = 'block';
    messageInput.disabled = false;
    sendBtn.disabled = false;

    // Show group menu for group chats AND game buttons
    if (groupMenu) groupMenu.style.display = 'block';
    
    const gameButtons = document.getElementById('game-buttons');
    if (gameButtons) {
        gameButtons.style.display = 'flex';
        console.log('Game buttons should now be visible for group');
    }

    clearMessages();
    loadGroupConversation(group.id);
}

function loadGroupConversation(groupId) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        console.log('Requesting group conversation history for:', groupId);
        socket.send(JSON.stringify({ type: 'get_group_conversation', group_id: groupId }));
    }
}

async function joinGroup(groupId) {
    try {
        console.log(`Attempting to join group ${groupId}`);
        const response = await fetch('/groups/join', {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ 
                group_id: groupId, 
                username: currentUser 
            })
        });
        
        const data = await response.json();
        console.log('Join group response:', data);
        
        if (response.ok) {
            if (data.status === 'joined') {
                alert('Successfully joined the group!');
                loadGroups();
            } else if (data.status === 'already_member') {
                alert('You are already a member of this group');
            } else {
                alert('Unexpected response: ' + data.status);
            }
        } else {
            alert('Failed to join group: ' + (data.error || 'Unknown error'));
        }
        
    } catch (err) {
        console.error('Error joining group:', err);
        alert('Failed to join group: ' + err.message);
    }
}

async function createGroup() {
    const name = groupNameInput.value.trim();
    const description = groupDescInput.value.trim();
    const members = groupMembersInput.value.split(',').map(u => u.trim()).filter(u => u);

    if (!name) { alert("Group name is required"); return; }

    console.log('Sending group creation request:', { name, description, members });

    try {
        const response = await fetch('/groups', {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ name, description, members })
        });

        console.log('Response status:', response.status);
        
        const responseText = await response.text();
        console.log('Raw response:', responseText);

        if (!responseText) {
            console.error('Empty response from server');
            alert('Server returned empty response');
            return;
        }

        let data;
        try {
            data = JSON.parse(responseText);
        } catch (jsonError) {
            console.error('JSON parsing error:', jsonError);
            alert('Server returned invalid response: ' + responseText.substring(0, 100));
            return;
        }

        if (response.ok) {
            console.log('Group created successfully:', data);
            loadGroups();
            groupNameInput.value = '';
            groupDescInput.value = '';
            groupMembersInput.value = '';
            createGroupModal.style.display = 'none';
        } else {
            console.error('Group creation failed:', data);
            alert(data.error || "Failed to create group");
        }
    } catch (err) {
        console.error("Network error creating group:", err);
        alert("Network error creating group: " + err.message);
    }
}

// ==========================
// Group Management Functions
// ==========================
async function showGroupMembers() {
    if (!currentGroup) return;
    
    try {
        membersList.innerHTML = '';
        
        if (currentGroup.members && currentGroup.members.length > 0) {
            currentGroup.members.forEach(member => {
                const memberDiv = document.createElement('div');
                memberDiv.style.padding = '10px';
                memberDiv.style.borderBottom = '1px solid #eee';
                memberDiv.textContent = member;
                membersList.appendChild(memberDiv);
            });
        } else {
            membersList.innerHTML = '<div style="padding: 20px; text-align: center; color: #666;">No members found</div>';
        }
        
        viewMembersModal.style.display = 'block';
    } catch (err) {
        console.error('Error showing group members:', err);
        alert('Failed to load group members');
    }
}

async function addMembersToGroup() {
    if (!currentGroup) {
        alert('No group selected');
        return;
    }
    
    const newMembers = newMembersInput.value.split(',').map(u => u.trim()).filter(u => u);
    
    if (newMembers.length === 0) {
        alert('Please enter at least one username');
        return;
    }
    
    try {
        let successCount = 0;
        let errorCount = 0;
        
        for (const username of newMembers) {
            console.log(`Adding user: ${username}`);
            const response = await fetch('/groups/join', {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${authToken}`
                },
                body: JSON.stringify({ 
                    group_id: currentGroup.id, 
                    username: username 
                })
            });
            
            if (response.ok) {
                successCount++;
                console.log(`Successfully added ${username}`);
            } else {
                errorCount++;
                console.error(`Failed to add member ${username}`, await response.text());
            }
        }
        
        if (successCount > 0 && errorCount === 0) {
            alert(`Successfully added ${successCount} member(s) to the group`);
        } else if (successCount > 0 && errorCount > 0) {
            alert(`Added ${successCount} member(s) successfully, but failed to add ${errorCount} member(s)`);
        } else {
            alert('Failed to add any members');
            return;
        }
        
        newMembersInput.value = '';
        addMembersModal.style.display = 'none';
        await loadGroups();
        
    } catch (err) {
        console.error('Error adding members:', err);
        alert('Failed to add members: ' + err.message);
    }
}

async function editGroup() {
    if (!currentGroup) return;
    
    const newName = prompt('Enter new group name:', currentGroup.name);
    if (!newName || newName.trim() === '') return;
    
    const newDescription = prompt('Enter new description:', currentGroup.description || '');
    
    try {
        const response = await fetch('/groups/update', {
            method: 'PUT',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ 
                group_id: currentGroup.id,
                name: newName.trim(),
                description: newDescription ? newDescription.trim() : null
            })
        });
        
        if (response.ok) {
            alert('Group updated successfully');
            loadGroups();
            chatWithUsername.textContent = `Group: ${newName.trim()}`;
        } else {
            const errorData = await response.json();
            alert('Failed to update group: ' + (errorData.error || 'Unknown error'));
        }
        
    } catch (err) {
        console.error('Error updating group:', err);
        alert('Failed to update group: ' + err.message);
    }
}

async function leaveGroup() {
    if (!currentGroup) return;
    
    if (!confirm(`Are you sure you want to leave "${currentGroup.name}"?`)) {
        return;
    }
    
    try {
        const response = await fetch('/groups/leave', {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ 
                group_id: currentGroup.id, 
                username: currentUser 
            })
        });
        
        if (response.ok) {
            alert('Left group successfully');
            
            currentGroup = null;
            chatHeader.style.display = 'none';
            welcomeScreen.style.display = 'block';
            messagesContainer.style.display = 'none';
            messageInputArea.style.display = 'none';
            
            loadGroups();
        } else {
            const errorData = await response.json();
            alert('Failed to leave group: ' + (errorData.error || 'Unknown error'));
        }
        
    } catch (err) {
        console.error('Error leaving group:', err);
        alert('Failed to leave group: ' + err.message);
    }
}

// ==========================
// WebSocket & Messaging
// ==========================
function connectWebSocket() {
    if (socket) socket.close();

    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/ws?token=${authToken}`;
    socket = new WebSocket(wsUrl);

    socket.binaryType = 'arraybuffer';

    socket.onopen = () => console.log('Connected to server');

    socket.onmessage = async (event) => {
        try {
            if (typeof event.data === 'string') {
                const data = JSON.parse(event.data);
                console.log('Received WebSocket message:', data);
                
                // Handle game-specific messages first
                if (data.type === 'game_state' || data.type === 'game_update' || data.type === 'game_created' || data.type === 'game_joined') {
                    handleGameWebSocketMessage(data);
                } else if (data.type === 'game_error') {
                    console.error('Game error:', data.error);
                    alert('Game error: ' + data.error);
                    
                    // Re-enable trivia options if this was a trivia game error
                    const options = gameBoard ? gameBoard.querySelectorAll('.trivia-option') : [];
                    if (options.length > 0) {
                        options.forEach(option => {
                            option.disabled = false;
                            option.style.opacity = '1';
                            option.style.cursor = 'pointer';
                        });
                        
                        // Reset the answer prompt
                        const answerPrompt = gameBoard.querySelector('.answer-prompt');
                        if (answerPrompt) {
                            answerPrompt.innerHTML = 'Choose your answer:';
                        }
                    }
                } else if (data.message && data.message.includes('🎮') && data.message.includes('made a move')) {
                    // This is a move notification message - request updated game state
                    console.log('Move notification received, requesting game state update');
                    displayGameMessage(data);
                    
                    // Extract game ID and request current game state
                    const gameId = extractGameId(data.message);
                    if (gameId && socket && socket.readyState === WebSocket.OPEN) {
                        console.log('Requesting updated game state for game ID:', gameId);
                        socket.send(JSON.stringify({
                            type: 'get_game_state',
                            game_id: gameId
                        }));
                    }
                } else if (data.message && data.message.includes('🎮')) {
                    // Other game messages (like game created)
                    displayGameMessage(data);
                } else if (data.type === 'conversation_history') {
                    displayConversationHistory(data);
                } else if (data.type === 'group_conversation_history') {
                    displayConversationHistory(data);
                } else if (data.type === 'poll_details') {
                    displayPollDetails(data.poll);
                } else if (data.sender_username && data.receiver_username && isMessageForCurrentConversation(data)) {
                    displayMessage(data);
                } else if (data.group_id && currentGroup && data.group_id === currentGroup.id) {
                    console.log('Displaying group message for current group:', data);
                    if (data.message && data.message.includes('📊 Poll')) {
                        console.log('This is a poll message, displaying with poll styling');
                        displayPollMessage(data);
                    } else {
                        console.log('This is a regular group message');
                        displayMessage(data);
                    }
                } else if (data.group_id) {
                    console.log('Received message for different group:', data.group_id, 'current group:', currentGroup ? currentGroup.id : 'none');
                } else if (data.type === 'file_ready' && pendingFile) {
                    const arrayBuffer = await pendingFile.arrayBuffer();
                    socket.send(arrayBuffer);
                    pendingFile = null;
                } else {
                    console.log('Message not handled:', data);
                }
            } else {
                console.log('Binary message received (not displayed in UI)');
            }
        } catch (err) {
            console.error('Error parsing WebSocket message:', err);
        }
    };

    socket.onclose = () => setTimeout(() => { if (authToken) connectWebSocket(); }, 3000);
    socket.onerror = (err) => console.error('WebSocket error:', err);
}

function isMessageForCurrentConversation(message) {
    if (!currentConversation) return false;
    return (message.sender_username === currentUser && message.receiver_username === currentConversation) ||
           (message.sender_username === currentConversation && message.receiver_username === currentUser);
}

function loadConversation(username) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        console.log('Requesting conversation history for:', username);
        socket.send(JSON.stringify({ type: 'get_conversation', receiver_username: username }));
    }
}

function displayConversationHistory(data) {
    if (data.conversation_with) {
        if (data.conversation_with !== currentConversation) {
            console.log('History for different conversation, ignoring');
            return;
        }
    }
    
    if (data.group_id) {
        if (!currentGroup || data.group_id !== currentGroup.id) {
            console.log('History for different group, ignoring');
            return;
        }
    }

    console.log('Displaying conversation history:', data);
    clearMessages();
    
    if (data.messages && data.messages.length > 0) {
        addHistorySeparator('--- Conversation History ---');
        data.messages.forEach(msg => {
            if (msg.message && msg.message.includes('🎮')) {
                displayGameMessage(msg, true);
            } else if (msg.message && msg.message.includes('📊 Poll')) {
                displayPollMessage(msg, true);
            } else {
                displayMessage(msg, true);
            }
        });
        addHistorySeparator('--- End of History ---');
        console.log(`Displayed ${data.messages.length} historical messages`);
    } else {
        console.log('No messages in history');
    }
    
    scrollToBottom();
}

function addHistorySeparator(text) {
    const sep = document.createElement('div');
    sep.className = 'history-separator';
    sep.textContent = text;
    messagesDiv.appendChild(sep);
}

function displayMessage(message, historical = false) {
    const existingMessage = document.querySelector(`[data-message-id="${message.id}"]`);
    if (existingMessage && message.id) {
        console.log('Duplicate message prevented (DOM check):', message.id);
        return;
    }

    const msgDiv = document.createElement('div');
    msgDiv.className = `message ${message.sender_username === currentUser ? 'sent' : 'received'}`;
    msgDiv.dataset.messageId = message.id;
    if (historical) msgDiv.classList.add('historical');

    const header = document.createElement('div');
    header.className = 'message-header';
    header.textContent = `${message.sender_username} • ${message.timestamp}`;

    const content = document.createElement('div');
    content.className = 'message-content';

    if (message.file_url) {
        const img = document.createElement('img');
        img.src = message.file_url;
        img.style.maxWidth = '200px';
        img.style.maxHeight = '200px';
        img.style.display = 'block';
        content.appendChild(img);
    } else {
        content.textContent = message.message;
    }

    const reactionsDiv = document.createElement('div');
    reactionsDiv.className = 'message-reactions';
    if (message.reactions) {
        for (const emoji in message.reactions) {
            const span = document.createElement('span');
            span.className = 'reaction-emoji';
            span.textContent = message.reactions[emoji];
            reactionsDiv.appendChild(span);
        }
    }

    const reactionButton = document.createElement('button');
    reactionButton.className = 'reaction-button';
    reactionButton.textContent = '😊';
    reactionButton.addEventListener('click', (e) => {
        e.stopPropagation();
        showReactionPicker(msgDiv, message.id);
    });

    msgDiv.appendChild(header);
    msgDiv.appendChild(content);
    msgDiv.appendChild(reactionsDiv);
    msgDiv.appendChild(reactionButton);

    messagesDiv.appendChild(msgDiv);
    if (!historical) scrollToBottom();
}

function sendMessage() {
    const text = messageInput.value.trim();
    if (!text || !socket || socket.readyState !== WebSocket.OPEN) return;

    if (currentConversation) {
        const msg = { type: 'chat_message', receiver_username: currentConversation, message: text, timestamp: getCurrentTime() };
        socket.send(JSON.stringify(msg));
    } else if (currentGroup) {
        const msg = { type: 'group_message', group_id: currentGroup.id, message: text, timestamp: getCurrentTime() };
        socket.send(JSON.stringify(msg));
    }

    messageInput.value = '';
}

function sendImage() {
    const file = imageInput.files[0];
    if (!file || !socket || socket.readyState !== WebSocket.OPEN) return;

    pendingFile = file;

    const metaMessage = {
        type: 'file_meta',
        message: file.name
    };

    if (currentConversation) metaMessage.receiver_username = currentConversation;
    else if (currentGroup) metaMessage.group_id = currentGroup.id;

    socket.send(JSON.stringify(metaMessage));

    imageInput.value = '';

    const fakeUrl = URL.createObjectURL(file);
    displayMessage({
        id: Date.now(),
        sender_username: currentUser,
        receiver_username: currentConversation || null,
        group_id: currentGroup ? currentGroup.id : null,
        message: '',
        file_url: fakeUrl,
        timestamp: getCurrentTime()
    });
}

function showReactionPicker(messageDiv, messageId) {
    clearTimeout(reactionPickerTimeout);
    const existingPicker = document.querySelector('.reaction-picker');
    if (existingPicker) existingPicker.remove();

    const picker = document.createElement('div');
    picker.className = 'reaction-picker';
    picker.innerHTML = `
        <span class="emoji" data-emoji="👍">👍</span>
        <span class="emoji" data-emoji="❤️">❤️</span>
        <span class="emoji" data-emoji="😂">😂</span>
        <span class="emoji" data-emoji="😢">😢</span>
        <span class="emoji" data-emoji="😡">😡</span>
    `;
    picker.addEventListener('click', (e) => {
        if (e.target.classList.contains('emoji')) {
            sendReaction(messageId, e.target.dataset.emoji);
            picker.remove();
        }
    });

    messageDiv.appendChild(picker);
    const rect = messageDiv.getBoundingClientRect();
    picker.style.top = `${rect.top - picker.offsetHeight - 5}px`;
    picker.style.left = `${rect.left}px`;
    picker.style.display = 'flex';
}

function hideReactionPicker() {
    reactionPickerTimeout = setTimeout(() => {
        const picker = document.querySelector('.reaction-picker');
        if (picker) picker.remove();
    }, 2000);
}

function sendReaction(messageId, emoji) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({ type: 'reaction', message_id: messageId, reaction: emoji }));
    }
}

// ==========================
// Highlights Functions
// ==========================
let highlightsModal, closeHighlightsModal, highlightsList, generateHighlightsBtn;
let highlightsTypeSelect, highlightsTargetSelect, highlightsNavBtn;

async function loadHighlights() {
    try {
        console.log('Loading highlights...');
        const params = new URLSearchParams({
            limit: '20'
        });
        
        const response = await fetch(`/highlights?${params}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        console.log('Load response status:', response.status);
        
        if (response.ok) {
            const data = await response.json();
            console.log('Loaded highlights data:', data);
            displayHighlights(data.highlights);
        } else {
            console.error('Failed to load highlights, status:', response.status);
            highlightsList.innerHTML = '<div class="highlight-error">Failed to load highlights</div>';
        }
    } catch (error) {
        console.error('Load highlights error:', error);
        highlightsList.innerHTML = '<div class="highlight-error">Error loading highlights</div>';
    }
}

async function generateHighlights() {
    const generateBtn = generateHighlightsBtn;
    const originalText = generateBtn.textContent;
    
    try {
        generateBtn.textContent = 'Generating...';
        generateBtn.disabled = true;
        
        const requestData = {
            type: highlightsTypeSelect.value,
            target_type: highlightsTargetSelect.value,
            date_range: "auto"
        };
        
        console.log('Sending request:', requestData);
        
        const response = await fetch('/highlights/generate', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify(requestData)
        });
        
        console.log('Response status:', response.status);
        
        if (response.ok) {
            const data = await response.json();
            console.log('Response data:', data);
            
            const highlights = Array.isArray(data) ? data : 
                              data.highlights ? data.highlights : [data];
            
            console.log('Highlights to display:', highlights);
            displayHighlights(highlights);
            alert('Highlights generated successfully!');
        } else {
            const errorData = await response.json();
            console.error('Error response:', errorData);
            alert(errorData.error || 'Failed to generate highlights');
        }
    } catch (error) {
        console.error('Exception:', error);
        alert('Error generating highlights');
    } finally {
        generateBtn.textContent = originalText;
        generateBtn.disabled = false;
    }
}

function displayHighlights(highlights) {
    if (!highlights || highlights.length === 0) {
        highlightsList.innerHTML = `
            <div class="no-highlights">
                <h3>No highlights available</h3>
                <p>Generate some highlights to see summaries of your conversations!</p>
            </div>
        `;
        return;
    }
    
    highlightsList.innerHTML = '';
    
    highlights.forEach(highlight => {
        const highlightDiv = document.createElement('div');
        highlightDiv.className = 'highlight-item';
        
        const targetIcon = highlight.target_type === 'group' ? '👥' : '💬';
        const typeIcon = highlight.highlight_type === 'daily' ? '📅' : '📆';
        
        highlightDiv.innerHTML = `
            <div class="highlight-header">
                <div class="highlight-title">
                    <span class="highlight-icon">${targetIcon}</span>
                    <span class="highlight-name">${highlight.target_name}</span>
                    <span class="highlight-type-badge">${typeIcon} ${highlight.highlight_type}</span>
                </div>
                <div class="highlight-stats">
                    <span class="message-count">${highlight.message_count} messages</span>
                    <span class="participant-count">${highlight.participant_count} participants</span>
                </div>
            </div>
            
            <div class="highlight-summary">
                ${highlight.summary}
            </div>
            
            ${highlight.key_topics && highlight.key_topics.length > 0 ? `
                <div class="highlight-topics">
                    <div class="topics-label">Key Topics:</div>
                    <div class="topics-list">
                        ${highlight.key_topics.map(topic => `<span class="topic-tag">${topic}</span>`).join('')}
                    </div>
                </div>
            ` : ''}
            
            <div class="highlight-footer">
                <span class="highlight-period">${highlight.start_date} to ${highlight.end_date}</span>
                <span class="highlight-created">Generated: ${highlight.created_at}</span>
            </div>
        `;
        
        highlightsList.appendChild(highlightDiv);
    });
}

async function generateChatHighlight(chatName, chatType, groupId = null) {
    try {
        const requestData = {
            type: 'recent',
            target_type: chatType,
            date_range: "auto"
        };
        
        if (groupId) {
            requestData.target_id = groupId;
        }
        
        if (chatType === 'personal') {
            requestData.specific_user = chatName;
        }
        
        const response = await fetch('/highlights/generate', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify(requestData)
        });
        
        if (response.ok) {
            const data = await response.json();
            const highlights = Array.isArray(data) ? data : 
                              data.highlights ? data.highlights : [data];
            
            const chatHighlights = highlights.filter(h => 
                h.target_name === chatName || (groupId && h.target_id === groupId)
            );
            
            if (chatHighlights.length > 0) {
                showChatHighlightModal(chatHighlights[0], chatName, chatType);
            } else {
                alert(`No recent activity found for ${chatName}`);
            }
        } else {
            const errorData = await response.json();
            alert(errorData.error || 'Failed to generate highlights');
        }
    } catch (error) {
        console.error('Error generating chat highlights:', error);
        alert('Error generating highlights');
    }
}

function showChatHighlightModal(highlight, chatName, chatType) {
    const modal = document.createElement('div');
    modal.className = 'modal';
    modal.style.display = 'block';
    
    const icon = chatType === 'group' ? '👥' : '💬';
    
    modal.innerHTML = `
        <div class="modal-content" style="max-width: 600px;">
            <span class="close" onclick="this.parentElement.parentElement.remove()">&times;</span>
            <h2>${icon} Highlights for ${chatName}</h2>
            
            <div class="highlight-item">
                <div class="highlight-summary" style="margin: 20px 0; font-size: 16px; line-height: 1.6;">
                    ${highlight.summary}
                </div>
                
                ${highlight.key_topics && highlight.key_topics.length > 0 ? `
                    <div class="highlight-topics" style="margin: 20px 0;">
                        <div class="topics-label" style="font-weight: bold; margin-bottom: 10px;">Key Topics:</div>
                        <div class="topics-list">
                            ${highlight.key_topics.map(topic => `<span class="topic-tag">${topic}</span>`).join('')}
                        </div>
                    </div>
                ` : ''}
                
                <div class="highlight-stats" style="margin-top: 20px; padding-top: 15px; border-top: 1px solid #eee; font-size: 14px; color: #666;">
                    <strong>${highlight.message_count}</strong> messages • <strong>${highlight.participant_count}</strong> participants
                </div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            modal.remove();
        }
    });
}

// ==========================
// Init
// ==========================
window.addEventListener('DOMContentLoaded', () => {
    authContainer = document.getElementById('auth-container');
    messengerContainer = document.getElementById('messenger-container');
    loginForm = document.getElementById('login-form');
    registerForm = document.getElementById('register-form');
    showRegisterLink = document.getElementById('show-register');
    showLoginLink = document.getElementById('show-login');
    loginBtn = document.getElementById('login-btn');
    registerBtn = document.getElementById('register-btn');
    logoutBtn = document.getElementById('logout-btn');
    currentUserSpan = document.getElementById('current-user');
    contactsList = document.getElementById('contacts-list');
    groupsList = document.getElementById('groups-list');
    availableGroupsList = document.getElementById('available-groups-list');
    welcomeScreen = document.getElementById('welcome-screen');
    chatHeader = document.getElementById('chat-header');
    messagesContainer = document.getElementById('messages-container');
    messageInputArea = document.getElementById('message-input-area');
    messagesDiv = document.getElementById('messages');
    messageInput = document.getElementById('message-input');
    sendBtn = document.getElementById('send-btn');
    chatWithUsername = document.getElementById('chat-with-username');
    imageInput = document.getElementById('image-input');
    imageBtn = document.getElementById('image-btn');
    
    createGroupBtn = document.getElementById('create-group-btn');
    createGroupModal = document.getElementById('create-group-modal');
    closeGroupModal = document.getElementById('close-group-modal');
    submitGroupBtn = document.getElementById('submit-group-btn');
    groupNameInput = document.getElementById('group-name');
    groupDescInput = document.getElementById('group-description');
    groupMembersInput = document.getElementById('group-members');
    
    groupMenu = document.getElementById('group-menu');
    groupMenuBtn = document.getElementById('group-menu-btn');
    groupMenuDropdown = document.getElementById('group-menu-dropdown');
    addMembersBtn = document.getElementById('add-members-btn');
    viewMembersBtn = document.getElementById('view-members-btn');
    editGroupBtn = document.getElementById('edit-group-btn');
    leaveGroupBtn = document.getElementById('leave-group-btn');
    
    addMembersModal = document.getElementById('add-members-modal');
    closeAddMembersModal = document.getElementById('close-add-members-modal');
    newMembersInput = document.getElementById('new-members-input');
    submitAddMembersBtn = document.getElementById('submit-add-members-btn');
    
    viewMembersModal = document.getElementById('view-members-modal');
    closeViewMembersModal = document.getElementById('close-view-members-modal');
    membersList = document.getElementById('members-list');
    
    // Poll elements
    createPollModal = document.getElementById('create-poll-modal');
    closePollModal = document.getElementById('close-poll-modal');
    submitPollBtn = document.getElementById('submit-poll-btn');
    pollQuestionInput = document.getElementById('poll-question');
    pollOptionsContainer = document.getElementById('poll-options-container');
    addPollOptionBtn = document.getElementById('add-poll-option-btn');
    allowMultipleCheckbox = document.getElementById('allow-multiple-choices');
    pollExpiresInput = document.getElementById('poll-expires');
    activePollsContainer = document.getElementById('active-polls-container');
    
    // Highlights elements
    highlightsModal = document.getElementById('highlights-modal');
    closeHighlightsModal = document.getElementById('close-highlights-modal');
    highlightsList = document.getElementById('highlights-list');
    generateHighlightsBtn = document.getElementById('generate-highlights-btn');
    highlightsTypeSelect = document.getElementById('highlights-type');
    highlightsTargetSelect = document.getElementById('highlights-target');
    highlightsNavBtn = document.getElementById('highlights-nav-btn');
    
    console.log('DOM loaded, all elements found');
    console.log('Groups list element:', groupsList);
    console.log('Available groups list element:', availableGroupsList);
    
    setupEventListeners();
    
    // Add highlights event listeners
    if (highlightsNavBtn) {
        highlightsNavBtn.addEventListener('click', () => {
            highlightsModal.style.display = 'block';
            loadHighlights();
        });
    }

    if (closeHighlightsModal) {
        closeHighlightsModal.addEventListener('click', () => {
            highlightsModal.style.display = 'none';
        });
    }

    if (generateHighlightsBtn) {
        generateHighlightsBtn.addEventListener('click', generateHighlights);
    }

    const savedToken = localStorage.getItem('authToken');
    const savedUser = localStorage.getItem('currentUser');

    if (savedToken && savedUser) {
        authToken = savedToken;
        currentUser = savedUser;
        showMessengerInterface();
        loadContacts();
        loadGroups();
    } else {
        showAuthInterface();
        showLoginForm();
    }

    initializeAIAssistant();
    initializeGameElements();
    
    // Ensure game modal close handler is set up
    const gameModalElement = document.getElementById('game-modal');
    const closeGameModalElement = document.getElementById('close-game-modal');
    
    if (gameModalElement && closeGameModalElement) {
        closeGameModalElement.addEventListener('click', () => {
            gameModalElement.style.display = 'none';
            currentGame = null;
        });
        
        // Close on outside click
        gameModalElement.addEventListener('click', (e) => {
            if (e.target === gameModalElement) {
                gameModalElement.style.display = 'none';
                currentGame = null;
            }
        });
    }
    
    // Close game modal on escape key
    document.addEventListener('keydown', function(event) {
        if (event.key === 'Escape') {
            const modal = gameModal || document.getElementById('game-modal');
            if (modal && modal.style.display === 'block') {
                modal.style.display = 'none';
                currentGame = null;
            }
        }
    });
    
    console.log('Game system initialized successfully');
});

// Make functions globally available for onclick handlers
window.removePollOption = removePollOption;
window.submitVote = submitVote;
window.createGame = createGame;
window.joinGame = joinGame;
window.loadGame = loadGame;
window.extractGameId = extractGameId;
window.handleGameWebSocketMessage = handleGameWebSocketMessage;
window.displayGameMessage = displayGameMessage;
window.sendQuickMessage = sendQuickMessage;
window.generateChatHighlight = generateChatHighlight;
window.requestNextQuestion = requestNextQuestion;

console.log('Game system integration loaded');