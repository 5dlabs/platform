# Task 7: Chat UI Implementation

## Overview
Develop a comprehensive chat interface with real-time messaging capabilities, room management, responsive design for mobile and desktop, and theme support. Integrate Socket.io client for real-time communication.

## Technical Implementation Guide

### Phase 1: Socket.io Client Integration

#### Socket Hook Implementation
```typescript
// frontend/src/hooks/useSocket.ts
import { useEffect, useState, useCallback, useRef } from 'react';
import { io, Socket } from 'socket.io-client';
import { useAuth } from '../contexts/AuthContext';

interface UseSocketReturn {
  socket: Socket | null;
  isConnected: boolean;
  joinRoom: (roomId: string) => void;
  leaveRoom: (roomId: string) => void;
  sendMessage: (roomId: string, content: string) => void;
  startTyping: (roomId: string) => void;
  stopTyping: (roomId: string) => void;
}

export const useSocket = (): UseSocketReturn => {
  const { isAuthenticated } = useAuth();
  const [socket, setSocket] = useState<Socket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const socketRef = useRef<Socket | null>(null);

  useEffect(() => {
    if (!isAuthenticated) return;

    const token = localStorage.getItem('accessToken');
    if (!token) return;

    const newSocket = io(process.env.REACT_APP_SOCKET_URL || 'http://localhost:3001', {
      auth: { token },
      transports: ['websocket', 'polling'],
      reconnection: true,
      reconnectionAttempts: 5,
      reconnectionDelay: 1000,
      reconnectionDelayMax: 5000,
    });

    socketRef.current = newSocket;

    newSocket.on('connect', () => {
      setIsConnected(true);
      console.log('Socket connected:', newSocket.id);
    });

    newSocket.on('disconnect', (reason) => {
      setIsConnected(false);
      console.log('Socket disconnected:', reason);
    });

    newSocket.on('connect_error', (error) => {
      console.error('Socket connection error:', error.message);
    });

    setSocket(newSocket);

    return () => {
      newSocket.removeAllListeners();
      newSocket.disconnect();
    };
  }, [isAuthenticated]);

  const joinRoom = useCallback((roomId: string) => {
    socket?.emit('join-room', roomId);
  }, [socket]);

  const leaveRoom = useCallback((roomId: string) => {
    socket?.emit('leave-room', roomId);
  }, [socket]);

  const sendMessage = useCallback((roomId: string, content: string) => {
    socket?.emit('send-message', { roomId, content });
  }, [socket]);

  const startTyping = useCallback((roomId: string) => {
    socket?.emit('typing-start', roomId);
  }, [socket]);

  const stopTyping = useCallback((roomId: string) => {
    socket?.emit('typing-stop', roomId);
  }, [socket]);

  return {
    socket,
    isConnected,
    joinRoom,
    leaveRoom,
    sendMessage,
    startTyping,
    stopTyping,
  };
};
```

### Phase 2: Main Chat Layout

#### Chat Layout Component
```typescript
// frontend/src/components/chat/ChatLayout.tsx
import React, { useState, useEffect } from 'react';
import { Sidebar } from './Sidebar';
import { ChatRoom } from './ChatRoom';
import { useSocket } from '../../hooks/useSocket';
import { useAuth } from '../../contexts/AuthContext';
import { Room, Message } from '../../types';
import { api } from '../../services/api';

export const ChatLayout: React.FC = () => {
  const { user } = useAuth();
  const { socket, isConnected } = useSocket();
  const [rooms, setRooms] = useState<Room[]>([]);
  const [activeRoom, setActiveRoom] = useState<Room | null>(null);
  const [messages, setMessages] = useState<Record<string, Message[]>>({});
  const [typingUsers, setTypingUsers] = useState<Record<string, string[]>>({});
  const [onlineUsers, setOnlineUsers] = useState<Set<string>>(new Set());
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [isMobile, setIsMobile] = useState(false);

  // Detect mobile screen
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth < 768);
      if (window.innerWidth < 768) {
        setIsSidebarOpen(false);
      }
    };

    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, []);

  // Load rooms on mount
  useEffect(() => {
    loadRooms();
  }, []);

  // Socket event listeners
  useEffect(() => {
    if (!socket) return;

    socket.on('message-received', (message: Message) => {
      setMessages(prev => ({
        ...prev,
        [message.roomId]: [...(prev[message.roomId] || []), message]
      }));
    });

    socket.on('user-typing', ({ roomId, userId, username }) => {
      if (userId !== user?.id) {
        setTypingUsers(prev => ({
          ...prev,
          [roomId]: [...(prev[roomId] || []).filter(id => id !== userId), userId]
        }));
      }
    });

    socket.on('user-stopped-typing', ({ roomId, userId }) => {
      setTypingUsers(prev => ({
        ...prev,
        [roomId]: (prev[roomId] || []).filter(id => id !== userId)
      }));
    });

    socket.on('user-online', ({ userId }) => {
      setOnlineUsers(prev => new Set([...prev, userId]));
    });

    socket.on('user-offline', ({ userId }) => {
      setOnlineUsers(prev => {
        const newSet = new Set(prev);
        newSet.delete(userId);
        return newSet;
      });
    });

    return () => {
      socket.off('message-received');
      socket.off('user-typing');
      socket.off('user-stopped-typing');
      socket.off('user-online');
      socket.off('user-offline');
    };
  }, [socket, user]);

  const loadRooms = async () => {
    try {
      const response = await api.get('/api/rooms');
      setRooms(response.data.data);
    } catch (error) {
      console.error('Failed to load rooms:', error);
    }
  };

  const handleRoomSelect = async (room: Room) => {
    setActiveRoom(room);
    
    if (isMobile) {
      setIsSidebarOpen(false);
    }

    // Load messages if not cached
    if (!messages[room.id]) {
      try {
        const response = await api.get(`/api/rooms/${room.id}/messages`);
        setMessages(prev => ({
          ...prev,
          [room.id]: response.data.data
        }));
      } catch (error) {
        console.error('Failed to load messages:', error);
      }
    }
  };

  const toggleSidebar = () => {
    setIsSidebarOpen(!isSidebarOpen);
  };

  return (
    <div className="flex h-screen bg-gray-100 dark:bg-gray-900">
      {/* Sidebar */}
      <div className={`${
        isSidebarOpen ? 'translate-x-0' : '-translate-x-full'
      } fixed md:relative md:translate-x-0 z-30 w-64 h-full transition-transform duration-300`}>
        <Sidebar
          rooms={rooms}
          activeRoom={activeRoom}
          onRoomSelect={handleRoomSelect}
          onlineUsers={onlineUsers}
          onClose={() => setIsSidebarOpen(false)}
        />
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        {/* Mobile Header */}
        {isMobile && (
          <div className="bg-white dark:bg-gray-800 border-b dark:border-gray-700 p-4 flex items-center">
            <button
              onClick={toggleSidebar}
              className="mr-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
            <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
              {activeRoom?.name || 'Select a room'}
            </h1>
          </div>
        )}

        {/* Chat Room */}
        {activeRoom ? (
          <ChatRoom
            room={activeRoom}
            messages={messages[activeRoom.id] || []}
            typingUsers={typingUsers[activeRoom.id] || []}
            onlineUsers={onlineUsers}
          />
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <p className="text-gray-500 dark:text-gray-400">
              Select a room to start chatting
            </p>
          </div>
        )}
      </div>

      {/* Mobile sidebar overlay */}
      {isMobile && isSidebarOpen && (
        <div
          className="fixed inset-0 bg-black bg-opacity-50 z-20"
          onClick={() => setIsSidebarOpen(false)}
        />
      )}
    </div>
  );
};
```

### Phase 3: Sidebar Component

```typescript
// frontend/src/components/chat/Sidebar.tsx
import React, { useState } from 'react';
import { Room } from '../../types';
import { useAuth } from '../../contexts/AuthContext';
import { CreateRoomModal } from './CreateRoomModal';
import { ThemeToggle } from '../common/ThemeToggle';

interface SidebarProps {
  rooms: Room[];
  activeRoom: Room | null;
  onRoomSelect: (room: Room) => void;
  onlineUsers: Set<string>;
  onClose: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  rooms,
  activeRoom,
  onRoomSelect,
  onlineUsers,
  onClose
}) => {
  const { user, logout } = useAuth();
  const [showCreateRoom, setShowCreateRoom] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  const filteredRooms = rooms.filter(room =>
    room.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="h-full bg-white dark:bg-gray-800 border-r dark:border-gray-700 flex flex-col">
      {/* User Header */}
      <div className="p-4 border-b dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center">
            <img
              src={user?.avatarUrl || `https://ui-avatars.com/api/?name=${user?.username}`}
              alt={user?.username}
              className="w-10 h-10 rounded-full mr-3"
            />
            <div>
              <p className="font-semibold text-gray-900 dark:text-white">
                {user?.username}
              </p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                {onlineUsers.has(user?.id || '') ? 'Online' : 'Offline'}
              </p>
            </div>
          </div>
          <ThemeToggle />
        </div>

        {/* Search */}
        <div className="relative">
          <input
            type="text"
            placeholder="Search rooms..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-3 py-2 border dark:border-gray-600 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white"
          />
        </div>
      </div>

      {/* Room List */}
      <div className="flex-1 overflow-y-auto">
        <div className="p-2">
          <div className="flex items-center justify-between mb-2 px-2">
            <h3 className="text-sm font-semibold text-gray-600 dark:text-gray-400 uppercase">
              Rooms
            </h3>
            <button
              onClick={() => setShowCreateRoom(true)}
              className="text-indigo-600 hover:text-indigo-700 dark:text-indigo-400"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
              </svg>
            </button>
          </div>

          {filteredRooms.map(room => (
            <button
              key={room.id}
              onClick={() => onRoomSelect(room)}
              className={`w-full text-left px-3 py-2 rounded-lg mb-1 transition-colors ${
                activeRoom?.id === room.id
                  ? 'bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300'
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300'
              }`}
            >
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium">{room.name}</p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    {room.memberCount} members
                  </p>
                </div>
                {room.unreadCount > 0 && (
                  <span className="bg-indigo-600 text-white text-xs rounded-full px-2 py-1">
                    {room.unreadCount}
                  </span>
                )}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Bottom Actions */}
      <div className="p-4 border-t dark:border-gray-700">
        <button
          onClick={logout}
          className="w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
          Logout
        </button>
      </div>

      {/* Create Room Modal */}
      {showCreateRoom && (
        <CreateRoomModal
          onClose={() => setShowCreateRoom(false)}
          onRoomCreated={(room) => {
            setShowCreateRoom(false);
            onRoomSelect(room);
          }}
        />
      )}
    </div>
  );
};
```

### Phase 4: Chat Room Component

```typescript
// frontend/src/components/chat/ChatRoom.tsx
import React, { useEffect, useRef, useState } from 'react';
import { Room, Message } from '../../types';
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';
import { TypingIndicator } from './TypingIndicator';
import { useSocket } from '../../hooks/useSocket';
import { useAuth } from '../../contexts/AuthContext';

interface ChatRoomProps {
  room: Room;
  messages: Message[];
  typingUsers: string[];
  onlineUsers: Set<string>;
}

export const ChatRoom: React.FC<ChatRoomProps> = ({
  room,
  messages,
  typingUsers,
  onlineUsers
}) => {
  const { user } = useAuth();
  const { socket, joinRoom, leaveRoom, sendMessage, startTyping, stopTyping } = useSocket();
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (socket && room) {
      joinRoom(room.id);
      return () => {
        leaveRoom(room.id);
      };
    }
  }, [socket, room, joinRoom, leaveRoom]);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleSendMessage = async (content: string) => {
    if (!content.trim()) return;
    sendMessage(room.id, content);
  };

  const handleTyping = () => {
    startTyping(room.id);
  };

  const handleStopTyping = () => {
    stopTyping(room.id);
  };

  return (
    <div className="flex-1 flex flex-col bg-white dark:bg-gray-800">
      {/* Room Header */}
      <div className="px-6 py-4 border-b dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              {room.name}
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {room.memberCount} members, {onlineUsers.size} online
            </p>
          </div>
          <button className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200">
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
            </svg>
          </button>
        </div>
      </div>

      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        {isLoading ? (
          <div className="flex justify-center items-center h-full">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
          </div>
        ) : (
          <>
            <MessageList 
              messages={messages} 
              currentUserId={user?.id || ''} 
              onlineUsers={onlineUsers}
            />
            {typingUsers.length > 0 && (
              <TypingIndicator users={typingUsers} />
            )}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Message Input */}
      <MessageInput
        onSendMessage={handleSendMessage}
        onTyping={handleTyping}
        onStopTyping={handleStopTyping}
      />
    </div>
  );
};
```

### Phase 5: Message Components

#### Message List Component
```typescript
// frontend/src/components/chat/MessageList.tsx
import React from 'react';
import { Message } from '../../types';
import { MessageBubble } from './MessageBubble';
import { format, isToday, isYesterday } from 'date-fns';

interface MessageListProps {
  messages: Message[];
  currentUserId: string;
  onlineUsers: Set<string>;
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  currentUserId,
  onlineUsers
}) => {
  const renderDateSeparator = (date: Date) => {
    let dateText = '';
    if (isToday(date)) {
      dateText = 'Today';
    } else if (isYesterday(date)) {
      dateText = 'Yesterday';
    } else {
      dateText = format(date, 'MMMM d, yyyy');
    }

    return (
      <div className="flex items-center my-4">
        <div className="flex-1 border-t dark:border-gray-700"></div>
        <span className="px-3 text-xs text-gray-500 dark:text-gray-400">
          {dateText}
        </span>
        <div className="flex-1 border-t dark:border-gray-700"></div>
      </div>
    );
  };

  let lastDate: Date | null = null;

  return (
    <div className="space-y-4">
      {messages.map((message, index) => {
        const messageDate = new Date(message.createdAt);
        const showDateSeparator = !lastDate || 
          format(lastDate, 'yyyy-MM-dd') !== format(messageDate, 'yyyy-MM-dd');
        
        lastDate = messageDate;

        return (
          <React.Fragment key={message.id}>
            {showDateSeparator && renderDateSeparator(messageDate)}
            <MessageBubble
              message={message}
              isOwn={message.userId === currentUserId}
              isOnline={onlineUsers.has(message.userId)}
              showAvatar={
                index === 0 || 
                messages[index - 1].userId !== message.userId ||
                showDateSeparator
              }
            />
          </React.Fragment>
        );
      })}
    </div>
  );
};
```

#### Message Input Component
```typescript
// frontend/src/components/chat/MessageInput.tsx
import React, { useState, useRef, useEffect } from 'react';
import { EmojiPicker } from './EmojiPicker';

interface MessageInputProps {
  onSendMessage: (content: string) => void;
  onTyping: () => void;
  onStopTyping: () => void;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSendMessage,
  onTyping,
  onStopTyping
}) => {
  const [message, setMessage] = useState('');
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const typingTimeoutRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    };
  }, []);

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setMessage(e.target.value);
    
    // Handle typing indicator
    onTyping();
    
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
    
    typingTimeoutRef.current = setTimeout(() => {
      onStopTyping();
    }, 1000);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (message.trim()) {
      onSendMessage(message);
      setMessage('');
      onStopTyping();
      if (typingTimeoutRef.current) {
        clearTimeout(typingTimeoutRef.current);
      }
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  const handleEmojiSelect = (emoji: string) => {
    setMessage(prev => prev + emoji);
    setShowEmojiPicker(false);
    inputRef.current?.focus();
  };

  return (
    <form onSubmit={handleSubmit} className="px-6 py-4 border-t dark:border-gray-700">
      <div className="flex items-end space-x-2">
        <div className="flex-1 relative">
          <textarea
            ref={inputRef}
            value={message}
            onChange={handleChange}
            onKeyPress={handleKeyPress}
            placeholder="Type a message..."
            rows={1}
            className="w-full px-4 py-2 pr-10 border dark:border-gray-600 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white"
            style={{ minHeight: '40px', maxHeight: '120px' }}
          />
          
          {/* Emoji Button */}
          <button
            type="button"
            onClick={() => setShowEmojiPicker(!showEmojiPicker)}
            className="absolute right-2 bottom-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            😊
          </button>
          
          {showEmojiPicker && (
            <div className="absolute bottom-12 right-0">
              <EmojiPicker onSelect={handleEmojiSelect} />
            </div>
          )}
        </div>
        
        <button
          type="submit"
          disabled={!message.trim()}
          className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
          </svg>
        </button>
      </div>
    </form>
  );
};
```

### Phase 6: Theme Toggle Implementation

```typescript
// frontend/src/components/common/ThemeToggle.tsx
import React, { useEffect, useState } from 'react';

export const ThemeToggle: React.FC = () => {
  const [isDark, setIsDark] = useState(() => {
    const saved = localStorage.getItem('theme');
    return saved === 'dark' || (!saved && window.matchMedia('(prefers-color-scheme: dark)').matches);
  });

  useEffect(() => {
    const root = document.documentElement;
    if (isDark) {
      root.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      root.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }, [isDark]);

  return (
    <button
      onClick={() => setIsDark(!isDark)}
      className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
      aria-label="Toggle theme"
    >
      {isDark ? (
        <svg className="w-5 h-5 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
      ) : (
        <svg className="w-5 h-5 text-gray-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
        </svg>
      )}
    </button>
  );
};
```

### Phase 7: Responsive Design Utilities

```css
/* frontend/src/styles/chat.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer components {
  /* Custom scrollbar for chat */
  .chat-scrollbar {
    @apply scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600 scrollbar-track-transparent;
  }

  /* Message animations */
  .message-enter {
    @apply transform translate-y-4 opacity-0;
  }
  
  .message-enter-active {
    @apply transform translate-y-0 opacity-100 transition-all duration-300;
  }

  /* Mobile-specific styles */
  @media (max-width: 767px) {
    .sidebar-mobile {
      @apply fixed inset-y-0 left-0 w-64 z-50;
    }
  }

  /* Desktop-specific styles */
  @media (min-width: 768px) {
    .sidebar-desktop {
      @apply relative w-64;
    }
  }
}

/* Smooth theme transitions */
* {
  @apply transition-colors duration-200;
}
```

## Success Metrics

- Real-time messages delivered instantly
- Typing indicators show/hide correctly
- Responsive design works on all devices
- Theme toggle persists preference
- Socket reconnection handles gracefully
- Message status indicators accurate
- Smooth animations and transitions
- Accessibility standards met