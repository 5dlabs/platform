# Task 7: Chat UI Implementation - Autonomous AI Agent Prompt

## Objective
You are tasked with implementing a comprehensive chat user interface for a real-time messaging application. The UI must be responsive, accessible, and integrate seamlessly with Socket.io for real-time communication.

## Context
You are building the frontend chat interface for an existing chat application that already has:
- User authentication system (Task 5)
- Socket.io server implementation (Task 6)
- Backend API endpoints for room and message management
- Database models for users, rooms, and messages

## Implementation Requirements

### 1. Component Architecture
Create a modular React component structure:
- **ChatLayout**: Main container managing sidebar and chat room state
- **Sidebar**: Contains room list, user info, and theme toggle
- **ChatRoom**: Displays messages and input area for selected room
- **Message Components**: Individual message bubbles with status indicators
- **Modal Components**: For room creation, settings, and user invites

### 2. Socket.io Integration
Implement real-time features:
- Establish authenticated Socket.io connection using JWT tokens
- Handle connection lifecycle (connect, disconnect, reconnect)
- Listen for and emit real-time events:
  - `message:new` - Incoming messages
  - `message:send` - Outgoing messages
  - `message:status` - Delivery/read receipts
  - `user:typing` - Typing indicators
  - `room:join` / `room:leave` - Room management
  - `user:presence` - Online/offline status

### 3. Responsive Design Requirements
Create layouts that adapt to different screen sizes:
- **Desktop (>768px)**:
  - Fixed sidebar (320px width)
  - Chat room takes remaining space
  - Persistent room list visibility
  
- **Mobile (≤768px)**:
  - Collapsible sidebar with overlay
  - Full-screen chat room
  - Touch-friendly controls (44px min touch targets)
  - Swipe gestures for sidebar toggle

### 4. Theme System
Implement dark/light theme switching:
- Use CSS custom properties for theme values
- Store theme preference in localStorage
- Apply theme to all UI components
- Smooth transitions between themes
- System preference detection as default

### 5. Message Features
Implement rich messaging capabilities:
- Markdown support for formatting
- Emoji picker integration
- Message status indicators (sending, sent, delivered, read)
- Timestamp display with relative times
- User avatars and usernames
- Message grouping by sender
- Auto-scroll to latest messages
- Typing indicators with multiple users

### 6. Performance Optimizations
Ensure smooth performance:
- Virtualize long message lists (>100 messages)
- Implement message pagination
- Cache messages in memory
- Optimistic UI updates for sent messages
- Debounce typing indicators
- Lazy load room history
- Use React.memo for message components

### 7. Accessibility Standards
Meet WCAG 2.1 AA compliance:
- Semantic HTML structure
- ARIA labels and roles
- Keyboard navigation support
- Focus management in modals
- Screen reader announcements
- High contrast mode support
- Reduced motion preferences

## Technical Specifications

### Dependencies
```json
{
  "socket.io-client": "^4.7.0",
  "react": "^18.2.0",
  "react-window": "^1.8.10",
  "react-window-infinite-loader": "^1.0.9",
  "emoji-picker-react": "^4.5.0",
  "lucide-react": "^0.263.0",
  "react-markdown": "^9.0.0",
  "date-fns": "^3.0.0"
}
```

### File Structure
```
src/
├── components/chat/      # Chat-specific components
├── hooks/               # Custom React hooks
├── contexts/            # React contexts
├── styles/              # CSS modules and themes
├── utils/               # Helper functions
└── types/               # TypeScript interfaces
```

### API Integration Points
- `GET /api/rooms` - Fetch user's rooms
- `GET /api/rooms/:id/messages` - Fetch room messages
- `POST /api/rooms` - Create new room
- `PUT /api/rooms/:id` - Update room settings
- `POST /api/rooms/:id/invite` - Invite users to room

## Success Criteria
1. Real-time messaging works reliably with <100ms latency
2. UI remains responsive with 1000+ messages
3. Mobile layout functions smoothly on all devices
4. Theme switching persists across sessions
5. All interactive elements are keyboard accessible
6. Messages display correctly with proper formatting
7. Reconnection handles gracefully without data loss

## Testing Requirements
- Unit tests for all components (>80% coverage)
- Integration tests for Socket.io events
- E2E tests for complete chat flows
- Performance tests for message rendering
- Accessibility audit passing
- Cross-browser compatibility (Chrome, Firefox, Safari, Edge)
- Mobile device testing (iOS, Android)

## Error Handling
- Display connection status to users
- Queue messages during disconnection
- Retry failed message sends
- Show error states for failed operations
- Graceful degradation without WebSocket support
- Handle rate limiting appropriately

## Security Considerations
- Sanitize user-generated content
- Validate all incoming Socket.io events
- Implement XSS protection for messages
- Use secure WebSocket connections (WSS)
- Handle authentication failures properly
- Prevent message spoofing

Execute this implementation following React best practices, ensuring code reusability, and maintaining a clean component hierarchy. The final implementation should provide a smooth, responsive chat experience across all devices.