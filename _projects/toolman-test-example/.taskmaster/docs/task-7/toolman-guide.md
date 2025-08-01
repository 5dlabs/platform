# Toolman Guide for Task 7: Chat UI Implementation

## Overview

This guide provides comprehensive instructions for using the selected tools to implement Task 7, which focuses on building a complete chat user interface with real-time messaging, room management, responsive design, and theme switching capabilities.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create the chat UI component directory structure

**When to Use**: 
- At the beginning to organize chat components
- When creating layout components
- For organizing chat-specific utilities and styles

**How to Use**:
```
# Create chat UI structure
create_directory /chat-application/frontend/src/components/chat
create_directory /chat-application/frontend/src/components/chat/room
create_directory /chat-application/frontend/src/components/chat/sidebar
create_directory /chat-application/frontend/src/components/chat/messages
create_directory /chat-application/frontend/src/components/chat/modals
create_directory /chat-application/frontend/src/components/layout
create_directory /chat-application/frontend/src/styles/themes
```

**Parameters**:
- `path`: Directory path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create chat components, Socket.io integration, and theme configurations

**When to Use**: 
- To create chat room components
- To implement Socket.io client setup
- To create message components
- To implement responsive layouts

**How to Use**:
```
# Create main chat layout
write_file /chat-application/frontend/src/components/chat/ChatLayout.tsx <layout-content>

# Create room component
write_file /chat-application/frontend/src/components/chat/room/ChatRoom.tsx <room-content>

# Create message list component
write_file /chat-application/frontend/src/components/chat/messages/MessageList.tsx <messages-content>

# Create Socket.io hook
write_file /chat-application/frontend/src/hooks/useSocket.ts <socket-hook>

# Create theme context
write_file /chat-application/frontend/src/contexts/ThemeContext.tsx <theme-content>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review existing components and Socket.io setup

**When to Use**: 
- To check AuthContext from Task 6
- To review existing component structure
- To understand current styling approach

**How to Use**:
```
# Read AuthContext
read_file /chat-application/frontend/src/contexts/AuthContext.tsx

# Check existing components
read_file /chat-application/frontend/src/App.tsx

# Review package.json
read_file /chat-application/frontend/package.json
```

**Parameters**:
- `path`: File to read
- `head`/`tail`: Optional line limits

### 4. **edit_file** (Local - filesystem)
**Purpose**: Update existing files to integrate chat UI

**When to Use**: 
- To add Socket.io client dependencies
- To update routing with chat pages
- To add theme switching
- To modify global styles

**How to Use**:
```
# Add Socket.io client
edit_file /chat-application/frontend/package.json
# Add: socket.io-client, @types/socket.io-client

# Update App routing
edit_file /chat-application/frontend/src/App.tsx
# Add chat routes and layout

# Add CSS framework
edit_file /chat-application/frontend/package.json
# Add: tailwindcss or styled-components
```

**Parameters**:
- `old_string`: Exact text to replace
- `new_string`: New text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify chat UI structure creation

**When to Use**: 
- After creating component structure
- To confirm file organization
- Before testing implementation

**How to Use**:
```
# Verify chat components
list_directory /chat-application/frontend/src/components/chat

# Check hooks
list_directory /chat-application/frontend/src/hooks
```

**Parameters**:
- `path`: Directory to list

## Implementation Flow

1. **Directory Structure Phase**
   - Use `create_directory` to build chat UI structure
   - Organize by feature (room, sidebar, messages)
   - Create theme and layout directories

2. **Socket.io Client Setup Phase**
   - Use `write_file` to create useSocket hook
   - Implement connection management
   - Add authentication to socket connection
   - Handle reconnection logic

3. **Layout Components Phase**
   - Create ChatLayout.tsx with responsive grid
   - Implement Sidebar.tsx with room list
   - Create ChatHeader.tsx with room info
   - Build responsive navigation

4. **Chat Room Implementation**
   - Create ChatRoom.tsx main container
   - Implement MessageList.tsx with virtualization
   - Create MessageItem.tsx with status indicators
   - Build MessageInput.tsx with typing indicators

5. **Real-time Features Phase**
   - Integrate Socket.io event handlers
   - Implement typing indicators
   - Add read receipts UI
   - Create online user indicators

6. **Theme Implementation Phase**
   - Create ThemeContext for dark/light modes
   - Implement theme toggle component
   - Apply theme variables throughout UI

## Best Practices

1. **Component Organization**: Keep components small and focused
2. **Performance**: Use React.memo for message components
3. **Responsive Design**: Mobile-first approach
4. **Accessibility**: Include ARIA labels
5. **State Management**: Minimize re-renders
6. **Error Handling**: Show connection status

## Task-Specific Implementation Details

### Socket.io Hook Pattern
```typescript
// useSocket.ts
import { useEffect, useRef } from 'react';
import { io, Socket } from 'socket.io-client';
import { useAuth } from './useAuth';

export const useSocket = () => {
  const socketRef = useRef<Socket | null>(null);
  const { user } = useAuth();

  useEffect(() => {
    if (!user) return;

    socketRef.current = io(process.env.REACT_APP_SERVER_URL, {
      auth: {
        token: localStorage.getItem('accessToken')
      }
    });

    const socket = socketRef.current;

    socket.on('connect', () => {
      console.log('Connected to server');
    });

    socket.on('error', (error) => {
      console.error('Socket error:', error);
    });

    return () => {
      socket.disconnect();
    };
  }, [user]);

  return socketRef.current;
};
```

### Message List Pattern
```typescript
// MessageList.tsx
import { useEffect, useRef } from 'react';
import { Message } from '../../types';

export const MessageList: React.FC<{ messages: Message[] }> = ({ messages }) => {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Auto-scroll to bottom on new messages
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="flex-1 overflow-y-auto p-4">
      {messages.map((message) => (
        <MessageItem key={message.id} message={message} />
      ))}
      <div ref={bottomRef} />
    </div>
  );
};
```

### Responsive Layout Pattern
```typescript
// ChatLayout.tsx
export const ChatLayout: React.FC = () => {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="flex h-screen">
      {/* Sidebar - hidden on mobile, visible on desktop */}
      <div className={`
        ${sidebarOpen ? 'block' : 'hidden'}
        md:block w-64 bg-gray-100 dark:bg-gray-800
      `}>
        <Sidebar />
      </div>

      {/* Main chat area */}
      <div className="flex-1 flex flex-col">
        <ChatHeader onMenuClick={() => setSidebarOpen(!sidebarOpen)} />
        <ChatRoom />
      </div>
    </div>
  );
};
```

### Theme Context Pattern
```typescript
// ThemeContext.tsx
export const ThemeProvider: React.FC = ({ children }) => {
  const [theme, setTheme] = useState<'light' | 'dark'>('light');

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};
```

## Troubleshooting

- **Socket Connection**: Check CORS and authentication
- **Message Ordering**: Ensure proper timestamp handling
- **Performance**: Implement virtual scrolling for long message lists
- **Mobile Layout**: Test on various screen sizes
- **Theme Switching**: Persist preference in localStorage

## Testing Approach

1. **Component Tests**:
   - Test message rendering
   - Test user interactions
   - Test responsive breakpoints

2. **Integration Tests**:
   - Test Socket.io connection
   - Test real-time updates
   - Test theme switching

3. **E2E Tests**:
   - Test complete chat flow
   - Test mobile interactions
   - Test offline behavior