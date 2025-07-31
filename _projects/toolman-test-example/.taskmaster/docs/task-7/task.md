# Task 7: Chat UI Implementation

## Overview
This task involves developing a comprehensive chat interface with real-time messaging capabilities, room management, and responsive design that works seamlessly across mobile and desktop devices.

## Technical Implementation Guide

### 1. Project Structure

```
src/
├── components/
│   ├── chat/
│   │   ├── ChatLayout.tsx
│   │   ├── Sidebar/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── RoomList.tsx
│   │   │   ├── UserInfo.tsx
│   │   │   └── RoomItem.tsx
│   │   ├── ChatRoom/
│   │   │   ├── ChatRoom.tsx
│   │   │   ├── MessageList.tsx
│   │   │   ├── MessageItem.tsx
│   │   │   ├── MessageInput.tsx
│   │   │   └── TypingIndicator.tsx
│   │   └── Modals/
│   │       ├── CreateRoomModal.tsx
│   │       ├── RoomSettingsModal.tsx
│   │       └── InviteUsersModal.tsx
│   ├── common/
│   │   ├── ThemeToggle.tsx
│   │   ├── Avatar.tsx
│   │   └── LoadingSpinner.tsx
│   └── layout/
│       └── ResponsiveContainer.tsx
├── hooks/
│   ├── useSocket.ts
│   ├── useChat.ts
│   ├── useTheme.ts
│   └── useMediaQuery.ts
├── contexts/
│   ├── SocketContext.tsx
│   └── ChatContext.tsx
├── styles/
│   ├── themes/
│   │   ├── light.css
│   │   └── dark.css
│   └── components/
│       └── chat.module.css
└── utils/
    ├── socketClient.ts
    ├── messageFormatter.ts
    └── dateHelpers.ts
```

### 2. Main Chat Layout Components

#### ChatLayout Component
```typescript
// src/components/chat/ChatLayout.tsx
import React, { useState } from 'react';
import { Sidebar } from './Sidebar/Sidebar';
import { ChatRoom } from './ChatRoom/ChatRoom';
import { useMediaQuery } from '../../hooks/useMediaQuery';
import styles from '../../styles/components/chat.module.css';

export const ChatLayout: React.FC = () => {
  const [selectedRoomId, setSelectedRoomId] = useState<string | null>(null);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const isMobile = useMediaQuery('(max-width: 768px)');

  return (
    <div className={styles.chatLayout}>
      <Sidebar 
        isOpen={isSidebarOpen}
        onToggle={() => setIsSidebarOpen(!isSidebarOpen)}
        onRoomSelect={setSelectedRoomId}
        selectedRoomId={selectedRoomId}
        isMobile={isMobile}
      />
      <ChatRoom 
        roomId={selectedRoomId}
        onMenuClick={() => setIsSidebarOpen(true)}
        isMobile={isMobile}
      />
    </div>
  );
};
```

#### Sidebar Implementation
```typescript
// src/components/chat/Sidebar/Sidebar.tsx
import React from 'react';
import { RoomList } from './RoomList';
import { UserInfo } from './UserInfo';
import { ThemeToggle } from '../../common/ThemeToggle';
import styles from '../../../styles/components/chat.module.css';

interface SidebarProps {
  isOpen: boolean;
  onToggle: () => void;
  onRoomSelect: (roomId: string) => void;
  selectedRoomId: string | null;
  isMobile: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  isOpen,
  onToggle,
  onRoomSelect,
  selectedRoomId,
  isMobile
}) => {
  const sidebarClass = `${styles.sidebar} ${
    isOpen ? styles.sidebarOpen : styles.sidebarClosed
  } ${isMobile ? styles.sidebarMobile : ''}`;

  return (
    <>
      {isMobile && isOpen && (
        <div className={styles.overlay} onClick={onToggle} />
      )}
      <aside className={sidebarClass}>
        <div className={styles.sidebarHeader}>
          <UserInfo />
          <ThemeToggle />
        </div>
        <RoomList 
          onRoomSelect={(roomId) => {
            onRoomSelect(roomId);
            if (isMobile) onToggle();
          }}
          selectedRoomId={selectedRoomId}
        />
      </aside>
    </>
  );
};
```

### 3. Socket.io Client Integration

#### Socket Context Provider
```typescript
// src/contexts/SocketContext.tsx
import React, { createContext, useContext, useEffect, useState } from 'react';
import { io, Socket } from 'socket.io-client';
import { useAuth } from './AuthContext';

interface SocketContextValue {
  socket: Socket | null;
  isConnected: boolean;
}

const SocketContext = createContext<SocketContextValue>({
  socket: null,
  isConnected: false,
});

export const SocketProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { user, isAuthenticated } = useAuth();
  const [socket, setSocket] = useState<Socket | null>(null);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    if (!isAuthenticated || !user) return;

    const token = localStorage.getItem('accessToken');
    const newSocket = io(process.env.REACT_APP_SOCKET_URL || 'http://localhost:3001', {
      auth: { token },
      reconnection: true,
      reconnectionAttempts: 5,
      reconnectionDelay: 1000,
      transports: ['websocket', 'polling'],
    });

    // Connection event handlers
    newSocket.on('connect', () => {
      setIsConnected(true);
      console.log('Socket connected:', newSocket.id);
    });

    newSocket.on('disconnect', (reason) => {
      setIsConnected(false);
      console.log('Socket disconnected:', reason);
    });

    newSocket.on('connect_error', (error) => {
      console.error('Connection error:', error.message);
    });

    // Reconnection events
    newSocket.on('reconnect', (attemptNumber) => {
      console.log('Reconnected after', attemptNumber, 'attempts');
    });

    newSocket.on('reconnect_error', (error) => {
      console.error('Reconnection error:', error);
    });

    setSocket(newSocket);

    return () => {
      newSocket.disconnect();
    };
  }, [isAuthenticated, user]);

  return (
    <SocketContext.Provider value={{ socket, isConnected }}>
      {children}
    </SocketContext.Provider>
  );
};

export const useSocket = () => useContext(SocketContext);
```

#### Chat Hook for Real-time Events
```typescript
// src/hooks/useChat.ts
import { useEffect, useState, useCallback } from 'react';
import { useSocket } from '../contexts/SocketContext';

interface Message {
  id: string;
  content: string;
  userId: string;
  username: string;
  roomId: string;
  timestamp: Date;
  status: 'sending' | 'sent' | 'delivered' | 'read';
}

interface TypingUser {
  userId: string;
  username: string;
}

export const useChat = (roomId: string | null) => {
  const { socket, isConnected } = useSocket();
  const [messages, setMessages] = useState<Message[]>([]);
  const [typingUsers, setTypingUsers] = useState<TypingUser[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Join room
  useEffect(() => {
    if (!socket || !isConnected || !roomId) return;

    socket.emit('room:join', { roomId });

    return () => {
      socket.emit('room:leave', { roomId });
    };
  }, [socket, isConnected, roomId]);

  // Handle incoming messages
  useEffect(() => {
    if (!socket) return;

    const handleNewMessage = (message: Message) => {
      setMessages(prev => [...prev, message]);
    };

    const handleMessageStatus = ({ messageId, status }: { messageId: string; status: string }) => {
      setMessages(prev => 
        prev.map(msg => 
          msg.id === messageId ? { ...msg, status: status as Message['status'] } : msg
        )
      );
    };

    const handleTyping = ({ userId, username, isTyping }: any) => {
      setTypingUsers(prev => {
        if (isTyping) {
          return prev.find(u => u.userId === userId) 
            ? prev 
            : [...prev, { userId, username }];
        } else {
          return prev.filter(u => u.userId !== userId);
        }
      });
    };

    socket.on('message:new', handleNewMessage);
    socket.on('message:status', handleMessageStatus);
    socket.on('user:typing', handleTyping);

    return () => {
      socket.off('message:new', handleNewMessage);
      socket.off('message:status', handleMessageStatus);
      socket.off('user:typing', handleTyping);
    };
  }, [socket]);

  // Send message
  const sendMessage = useCallback((content: string) => {
    if (!socket || !isConnected || !roomId) return;

    const tempId = `temp-${Date.now()}`;
    const message: Message = {
      id: tempId,
      content,
      userId: 'current-user', // Get from auth context
      username: 'Current User', // Get from auth context
      roomId,
      timestamp: new Date(),
      status: 'sending',
    };

    setMessages(prev => [...prev, message]);

    socket.emit('message:send', {
      roomId,
      content,
      tempId,
    }, (response: { success: boolean; messageId?: string; error?: string }) => {
      if (response.success && response.messageId) {
        setMessages(prev => 
          prev.map(msg => 
            msg.id === tempId 
              ? { ...msg, id: response.messageId!, status: 'sent' } 
              : msg
          )
        );
      } else {
        console.error('Failed to send message:', response.error);
        setMessages(prev => 
          prev.map(msg => 
            msg.id === tempId ? { ...msg, status: 'sent' } : msg
          )
        );
      }
    });
  }, [socket, isConnected, roomId]);

  // Emit typing status
  const setTyping = useCallback((isTyping: boolean) => {
    if (!socket || !roomId) return;
    socket.emit('user:typing', { roomId, isTyping });
  }, [socket, roomId]);

  return {
    messages,
    typingUsers,
    isLoading,
    sendMessage,
    setTyping,
  };
};
```

### 4. Message Components

#### Message List Component
```typescript
// src/components/chat/ChatRoom/MessageList.tsx
import React, { useRef, useEffect } from 'react';
import { MessageItem } from './MessageItem';
import { TypingIndicator } from './TypingIndicator';
import styles from '../../../styles/components/chat.module.css';

interface MessageListProps {
  messages: Array<{
    id: string;
    content: string;
    userId: string;
    username: string;
    timestamp: Date;
    status: string;
  }>;
  typingUsers: Array<{ userId: string; username: string }>;
  currentUserId: string;
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  typingUsers,
  currentUserId,
}) => {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Auto-scroll to bottom on new messages
    if (listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight;
    }
  }, [messages]);

  return (
    <div className={styles.messageList} ref={listRef}>
      {messages.map((message, index) => (
        <MessageItem
          key={message.id}
          message={message}
          isOwn={message.userId === currentUserId}
          showAvatar={
            index === 0 || messages[index - 1].userId !== message.userId
          }
        />
      ))}
      {typingUsers.length > 0 && (
        <TypingIndicator users={typingUsers} />
      )}
    </div>
  );
};
```

#### Message Input Component
```typescript
// src/components/chat/ChatRoom/MessageInput.tsx
import React, { useState, useRef, useEffect } from 'react';
import { Send, Smile } from 'lucide-react';
import EmojiPicker from 'emoji-picker-react';
import styles from '../../../styles/components/chat.module.css';

interface MessageInputProps {
  onSendMessage: (content: string) => void;
  onTyping: (isTyping: boolean) => void;
  disabled?: boolean;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSendMessage,
  onTyping,
  disabled = false,
}) => {
  const [message, setMessage] = useState('');
  const [showEmoji, setShowEmoji] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const typingTimeoutRef = useRef<NodeJS.Timeout>();
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (message.trim() && !disabled) {
      onSendMessage(message.trim());
      setMessage('');
      setIsTyping(false);
      onTyping(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value);

    // Handle typing indicator
    if (!isTyping) {
      setIsTyping(true);
      onTyping(true);
    }

    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    // Set new timeout
    typingTimeoutRef.current = setTimeout(() => {
      setIsTyping(false);
      onTyping(false);
    }, 1000);
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
      if (isTyping) {
        onTyping(false);
      }
    };
  }, [isTyping, onTyping]);

  return (
    <form className={styles.messageInput} onSubmit={handleSubmit}>
      <button
        type="button"
        className={styles.emojiButton}
        onClick={() => setShowEmoji(!showEmoji)}
      >
        <Smile size={20} />
      </button>
      
      {showEmoji && (
        <div className={styles.emojiPicker}>
          <EmojiPicker
            onEmojiClick={(emoji) => {
              setMessage(prev => prev + emoji.emoji);
              setShowEmoji(false);
              inputRef.current?.focus();
            }}
          />
        </div>
      )}

      <textarea
        ref={inputRef}
        value={message}
        onChange={handleChange}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmit(e);
          }
        }}
        placeholder="Type a message..."
        className={styles.messageTextarea}
        disabled={disabled}
        rows={1}
      />

      <button
        type="submit"
        className={styles.sendButton}
        disabled={!message.trim() || disabled}
      >
        <Send size={20} />
      </button>
    </form>
  );
};
```

### 5. Responsive Design Implementation

#### CSS Variables for Theming
```css
/* src/styles/themes/light.css */
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f5f7fa;
  --bg-tertiary: #e9ecef;
  --text-primary: #2c3e50;
  --text-secondary: #7f8c8d;
  --text-tertiary: #95a5a6;
  --border-color: #dee2e6;
  --accent-primary: #3498db;
  --accent-hover: #2980b9;
  --success: #27ae60;
  --warning: #f39c12;
  --danger: #e74c3c;
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.1);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1);
}

/* src/styles/themes/dark.css */
[data-theme="dark"] {
  --bg-primary: #1a1a1a;
  --bg-secondary: #2d2d2d;
  --bg-tertiary: #3a3a3a;
  --text-primary: #e0e0e0;
  --text-secondary: #b0b0b0;
  --text-tertiary: #888888;
  --border-color: #404040;
  --accent-primary: #5dade2;
  --accent-hover: #3498db;
  --success: #58d68d;
  --warning: #f4d03f;
  --danger: #ec7063;
  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
  --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.3);
  --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.3);
}
```

#### Responsive Layout Styles
```css
/* src/styles/components/chat.module.css */
.chatLayout {
  display: flex;
  height: 100vh;
  background-color: var(--bg-primary);
  position: relative;
}

/* Sidebar Styles */
.sidebar {
  width: 320px;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  transition: transform 0.3s ease;
}

.sidebarClosed {
  transform: translateX(-100%);
}

.sidebarOpen {
  transform: translateX(0);
}

/* Mobile Sidebar */
@media (max-width: 768px) {
  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    height: 100vh;
    z-index: 100;
    width: 80%;
    max-width: 320px;
  }

  .sidebarMobile {
    box-shadow: var(--shadow-lg);
  }

  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 99;
  }
}

/* Chat Room Styles */
.chatRoom {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.chatHeader {
  height: 60px;
  background-color: var(--bg-primary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 1rem;
  gap: 1rem;
}

.messageList {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
  scroll-behavior: smooth;
}

/* Message Styles */
.messageItem {
  display: flex;
  gap: 0.75rem;
  margin-bottom: 1rem;
  align-items: flex-start;
}

.messageItemOwn {
  flex-direction: row-reverse;
}

.messageBubble {
  max-width: 70%;
  padding: 0.75rem 1rem;
  border-radius: 1rem;
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  word-wrap: break-word;
}

.messageItemOwn .messageBubble {
  background-color: var(--accent-primary);
  color: white;
}

/* Input Styles */
.messageInput {
  display: flex;
  align-items: flex-end;
  gap: 0.5rem;
  padding: 1rem;
  border-top: 1px solid var(--border-color);
  background-color: var(--bg-primary);
}

.messageTextarea {
  flex: 1;
  min-height: 40px;
  max-height: 120px;
  padding: 0.5rem 1rem;
  border: 1px solid var(--border-color);
  border-radius: 1.5rem;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  resize: none;
  font-family: inherit;
  font-size: 0.95rem;
  line-height: 1.5;
}

/* Touch-friendly mobile adjustments */
@media (max-width: 768px) {
  .messageInput {
    padding: 0.75rem;
  }

  .sendButton,
  .emojiButton {
    width: 44px;
    height: 44px;
    min-width: 44px;
  }

  .messageBubble {
    font-size: 1rem;
  }
}
```

### 6. Theme System Implementation

```typescript
// src/hooks/useTheme.ts
import { useState, useEffect } from 'react';

type Theme = 'light' | 'dark';

export const useTheme = () => {
  const [theme, setTheme] = useState<Theme>(() => {
    const saved = localStorage.getItem('theme') as Theme;
    return saved || 'light';
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };

  return { theme, toggleTheme };
};
```

### 7. Performance Optimizations

#### Message Virtualization
```typescript
// src/components/chat/ChatRoom/VirtualizedMessageList.tsx
import React from 'react';
import { VariableSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { MessageItem } from './MessageItem';

interface VirtualizedMessageListProps {
  messages: Message[];
  currentUserId: string;
}

export const VirtualizedMessageList: React.FC<VirtualizedMessageListProps> = ({
  messages,
  currentUserId,
}) => {
  const getItemSize = (index: number) => {
    // Calculate dynamic height based on message content
    const message = messages[index];
    const baseHeight = 60;
    const contentLines = Math.ceil(message.content.length / 50);
    return baseHeight + (contentLines - 1) * 20;
  };

  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const message = messages[index];
    const showAvatar = index === 0 || messages[index - 1].userId !== message.userId;

    return (
      <div style={style}>
        <MessageItem
          message={message}
          isOwn={message.userId === currentUserId}
          showAvatar={showAvatar}
        />
      </div>
    );
  };

  return (
    <AutoSizer>
      {({ height, width }) => (
        <List
          height={height}
          itemCount={messages.length}
          itemSize={getItemSize}
          width={width}
          overscanCount={5}
        >
          {Row}
        </List>
      )}
    </AutoSizer>
  );
};
```

#### Message Caching and Optimistic Updates
```typescript
// src/utils/messageCache.ts
interface CachedMessage {
  id: string;
  content: string;
  timestamp: number;
  status: string;
}

class MessageCache {
  private cache: Map<string, CachedMessage[]> = new Map();
  private maxCacheSize = 1000;

  addMessage(roomId: string, message: CachedMessage) {
    const roomMessages = this.cache.get(roomId) || [];
    roomMessages.push(message);
    
    // Limit cache size
    if (roomMessages.length > this.maxCacheSize) {
      roomMessages.shift();
    }
    
    this.cache.set(roomId, roomMessages);
  }

  getMessages(roomId: string): CachedMessage[] {
    return this.cache.get(roomId) || [];
  }

  updateMessageStatus(roomId: string, messageId: string, status: string) {
    const roomMessages = this.cache.get(roomId);
    if (!roomMessages) return;

    const message = roomMessages.find(m => m.id === messageId);
    if (message) {
      message.status = status;
    }
  }

  clearRoom(roomId: string) {
    this.cache.delete(roomId);
  }

  clearAll() {
    this.cache.clear();
  }
}

export const messageCache = new MessageCache();
```

### 8. Accessibility Features

```typescript
// src/components/chat/ChatRoom/AccessibleMessageList.tsx
import React from 'react';
import { MessageItem } from './MessageItem';
import styles from '../../../styles/components/chat.module.css';

export const AccessibleMessageList: React.FC<MessageListProps> = ({
  messages,
  typingUsers,
  currentUserId,
}) => {
  return (
    <div 
      className={styles.messageList}
      role="log"
      aria-label="Chat messages"
      aria-live="polite"
      aria-relevant="additions"
    >
      <h2 className="sr-only">Message History</h2>
      {messages.map((message, index) => (
        <div
          key={message.id}
          role="article"
          aria-label={`Message from ${message.username}`}
        >
          <MessageItem
            message={message}
            isOwn={message.userId === currentUserId}
            showAvatar={
              index === 0 || messages[index - 1].userId !== message.userId
            }
          />
        </div>
      ))}
      {typingUsers.length > 0 && (
        <div aria-live="polite" aria-atomic="true">
          <TypingIndicator users={typingUsers} />
        </div>
      )}
    </div>
  );
};
```

## Testing Strategy

### Unit Tests
- Test individual components in isolation
- Mock Socket.io connections
- Test theme switching functionality
- Verify message formatting and display

### Integration Tests
- Test real-time message flow
- Verify room switching behavior
- Test typing indicators
- Validate reconnection logic

### E2E Tests
- Complete chat flow from login to messaging
- Multi-user scenarios
- Mobile and desktop responsive behavior
- Theme persistence across sessions

### Accessibility Tests
- Screen reader compatibility
- Keyboard navigation
- Focus management
- ARIA attributes validation