# Autonomous Agent Prompt: Chat UI Implementation

You are tasked with developing a comprehensive chat interface with real-time messaging capabilities, responsive design, and theme support using React and Socket.io.

## Objective
Build a modern chat UI that integrates with Socket.io for real-time communication, supports multiple rooms, shows typing indicators and online status, and works seamlessly on both desktop and mobile devices.

## Detailed Requirements

### 1. Socket.io Client Setup
Create a custom hook for Socket.io:
- Connect with JWT authentication
- Handle connection/disconnection events
- Implement reconnection logic
- Provide methods: joinRoom, leaveRoom, sendMessage
- Manage typing indicators
- Clean up on unmount

### 2. Main Chat Layout
Build the primary chat interface:
- Sidebar for room list (collapsible on mobile)
- Main chat area with messages
- Responsive flex layout
- Mobile hamburger menu
- Desktop: sidebar always visible
- Mobile: overlay sidebar with backdrop

### 3. Sidebar Component
Implement sidebar with:
- User profile section with avatar
- Online/offline status indicator
- Room search functionality
- Room list with unread counts
- Create room button
- Theme toggle button
- Logout option
- Active room highlighting

### 4. Chat Room Component
Create the main chat area:
- Room header with name and member count
- Message list with scroll management
- Typing indicators below messages
- Message input at bottom
- Auto-scroll to new messages
- Load messages on room change
- Show online user count

### 5. Message Components
Build message display:
- Message bubbles (own vs others)
- User avatar and name
- Timestamp formatting
- Date separators
- Read receipts
- Message status indicators
- Support for emojis
- Markdown rendering (optional)

### 6. Message Input
Implement rich input:
- Textarea with auto-resize
- Send on Enter (Shift+Enter for newline)
- Emoji picker integration
- Typing indicator triggers
- Character limit display
- Send button with disabled state
- Focus management

### 7. Real-time Features
Handle Socket.io events:
- `message-received`: Add to message list
- `user-typing`: Show indicator
- `user-stopped-typing`: Hide indicator
- `user-online`: Update status
- `user-offline`: Update status
- `room-joined`: Update UI
- Error handling for failed events

### 8. Responsive Design
Mobile-first approach:
- Breakpoint: 768px (md)
- Mobile: Full-screen sidebar
- Desktop: Fixed sidebar
- Touch-friendly tap targets
- Swipe gestures (optional)
- Proper viewport meta tag

### 9. Theme Support
Dark/light mode toggle:
- CSS variables or Tailwind dark:
- System preference detection
- LocalStorage persistence
- Smooth transitions
- Toggle in sidebar
- Apply to all components

### 10. Create Room Modal
Room creation interface:
- Room name input
- Description (optional)
- Public/private toggle
- Member invite (future)
- Validation
- Loading state
- Success redirect

## Expected Deliverables

1. useSocket custom hook
2. ChatLayout main component
3. Sidebar with room list
4. ChatRoom component
5. MessageList and MessageBubble
6. MessageInput with emoji
7. TypingIndicator component
8. CreateRoomModal
9. ThemeToggle component
10. Responsive CSS/Tailwind classes

## Component Structure

```
components/chat/
├── ChatLayout.tsx       // Main container
├── Sidebar.tsx         // Room list sidebar
├── ChatRoom.tsx        // Active chat view
├── MessageList.tsx     // Message container
├── MessageBubble.tsx   // Individual message
├── MessageInput.tsx    // Input with features
├── TypingIndicator.tsx // "User is typing..."
├── CreateRoomModal.tsx // Room creation
└── EmojiPicker.tsx     // Emoji selection
```

## State Management

Local component state for:
- Active room
- Messages per room (cached)
- Typing users per room
- Online users set
- Sidebar open/closed
- Theme preference

## Socket Event Handling

```typescript
// Incoming events
socket.on('message-received', handleNewMessage)
socket.on('user-typing', handleTypingStart)
socket.on('user-stopped-typing', handleTypingStop)
socket.on('user-online', handleUserOnline)
socket.on('user-offline', handleUserOffline)

// Outgoing events
socket.emit('join-room', roomId)
socket.emit('leave-room', roomId)
socket.emit('send-message', { roomId, content })
socket.emit('typing-start', roomId)
socket.emit('typing-stop', roomId)
```

## Styling Guidelines

Use Tailwind CSS:
- Consistent spacing: p-4, m-2, etc.
- Color scheme: indigo primary
- Gray scales for backgrounds
- Hover states on interactive elements
- Focus rings for accessibility
- Dark mode classes throughout

## Performance Optimizations

- Virtualize long message lists
- Debounce typing indicators
- Cache messages by room
- Lazy load older messages
- Memoize expensive renders
- Optimize re-renders

## Accessibility Requirements

- Proper ARIA labels
- Keyboard navigation
- Focus management
- Screen reader support
- High contrast mode
- Reduced motion support

## Testing Scenarios

1. Send and receive messages
2. Switch between rooms
3. Toggle theme
4. Resize window
5. Create new room
6. Show typing indicators
7. Handle disconnection
8. Test on mobile devices

Begin by setting up the Socket.io client hook, then build the main layout components, followed by the message handling and real-time features.