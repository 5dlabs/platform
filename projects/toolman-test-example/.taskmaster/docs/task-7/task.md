# Task 7: Chat UI Implementation

## Overview
Develop a modern, responsive chat interface with real-time messaging capabilities, room management, and theme support. This task creates the primary user interface for the chat application, integrating with Socket.io for real-time updates and providing an intuitive user experience across all devices.

## Technical Architecture

### UI Framework
- **Component Library**: Material-UI or custom styled-components
- **State Management**: React Context + useReducer for chat state
- **Real-time**: Socket.io client integration
- **Styling**: CSS-in-JS with theme support
- **Icons**: Material Icons or React Icons
- **Animations**: Framer Motion for smooth transitions

### Component Structure
```
ChatApp/
├── Layout/
│   ├── Sidebar/
│   │   ├── UserProfile
│   │   ├── RoomList
│   │   └── CreateRoomButton
│   └── MainChat/
│       ├── ChatHeader
│       ├── MessageList
│       ├── MessageInput
│       └── TypingIndicator
├── Messages/
│   ├── MessageBubble
│   ├── MessageStatus
│   └── MessageActions
└── Modals/
    ├── CreateRoomModal
    ├── RoomSettingsModal
    └── UserProfileModal
```

## Implementation Details

### 1. Main Layout Component

```typescript
// frontend/src/components/chat/ChatLayout.tsx
import React, { useState, useEffect } from 'react';
import { Box, Drawer, useTheme, useMediaQuery } from '@mui/material';
import { Sidebar } from './Sidebar';
import { ChatRoom } from './ChatRoom';
import { useSocket } from '../../hooks/useSocket';
import { useChatState } from '../../contexts/ChatContext';
import { Room } from '../../types/chat';

interface ChatLayoutProps {
  initialRoomId?: string;
}

export const ChatLayout: React.FC<ChatLayoutProps> = ({ initialRoomId }) => {
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const [sidebarOpen, setSidebarOpen] = useState(!isMobile);
  const [selectedRoom, setSelectedRoom] = useState<Room | null>(null);
  
  const { socket, isConnected } = useSocket();
  const { rooms, messages, dispatch } = useChatState();

  useEffect(() => {
    if (initialRoomId && rooms.length > 0) {
      const room = rooms.find(r => r.id === initialRoomId);
      if (room) {
        setSelectedRoom(room);
        socket?.emit('join-room', room.id);
      }
    }
  }, [initialRoomId, rooms, socket]);

  const handleRoomSelect = (room: Room) => {
    setSelectedRoom(room);
    if (isMobile) {
      setSidebarOpen(false);
    }
    
    // Join room via Socket.io
    socket?.emit('join-room', room.id, (response) => {
      if (response.success) {
        dispatch({ type: 'SET_ACTIVE_ROOM', payload: room.id });
      }
    });
  };

  const drawerWidth = 320;

  return (
    <Box sx={{ display: 'flex', height: '100vh', bgcolor: 'background.default' }}>
      <Drawer
        variant={isMobile ? 'temporary' : 'permanent'}
        open={sidebarOpen}
        onClose={() => setSidebarOpen(false)}
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
          },
        }}
      >
        <Sidebar
          rooms={rooms}
          selectedRoom={selectedRoom}
          onRoomSelect={handleRoomSelect}
          connectionStatus={isConnected}
        />
      </Drawer>
      
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          display: 'flex',
          flexDirection: 'column',
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          ml: { sm: `${drawerWidth}px` },
        }}
      >
        {selectedRoom ? (
          <ChatRoom
            room={selectedRoom}
            messages={messages[selectedRoom.id] || []}
            onMenuClick={() => setSidebarOpen(true)}
            isMobile={isMobile}
          />
        ) : (
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              color: 'text.secondary',
            }}
          >
            Select a room to start chatting
          </Box>
        )}
      </Box>
    </Box>
  );
};
```

### 2. Sidebar Component

```typescript
// frontend/src/components/chat/Sidebar/index.tsx
import React, { useState } from 'react';
import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemAvatar,
  ListItemText,
  Avatar,
  Typography,
  IconButton,
  Divider,
  Badge,
  Tooltip,
  Chip,
} from '@mui/material';
import {
  Add as AddIcon,
  Settings as SettingsIcon,
  Brightness4 as DarkModeIcon,
  Brightness7 as LightModeIcon,
} from '@mui/icons-material';
import { useAuth } from '../../../contexts/AuthContext';
import { useTheme } from '../../../contexts/ThemeContext';
import { CreateRoomModal } from '../Modals/CreateRoomModal';
import { Room } from '../../../types/chat';
import { formatDistanceToNow } from 'date-fns';

interface SidebarProps {
  rooms: Room[];
  selectedRoom: Room | null;
  onRoomSelect: (room: Room) => void;
  connectionStatus: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  rooms,
  selectedRoom,
  onRoomSelect,
  connectionStatus,
}) => {
  const { user } = useAuth();
  const { theme, toggleTheme } = useTheme();
  const [createRoomOpen, setCreateRoomOpen] = useState(false);

  const sortedRooms = [...rooms].sort((a, b) => {
    // Sort by last activity
    const aTime = new Date(a.lastActivity || a.createdAt).getTime();
    const bTime = new Date(b.lastActivity || b.createdAt).getTime();
    return bTime - aTime;
  });

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* User Profile Section */}
      <Box sx={{ p: 2, bgcolor: 'primary.main', color: 'primary.contrastText' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
          <Avatar
            src={user?.avatarUrl}
            alt={user?.username}
            sx={{ mr: 2 }}
          />
          <Box sx={{ flexGrow: 1 }}>
            <Typography variant="subtitle1" fontWeight="bold">
              {user?.username}
            </Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
              <Box
                sx={{
                  width: 8,
                  height: 8,
                  borderRadius: '50%',
                  bgcolor: connectionStatus ? 'success.main' : 'error.main',
                }}
              />
              <Typography variant="caption">
                {connectionStatus ? 'Connected' : 'Connecting...'}
              </Typography>
            </Box>
          </Box>
          <IconButton
            color="inherit"
            onClick={toggleTheme}
            size="small"
          >
            {theme === 'dark' ? <LightModeIcon /> : <DarkModeIcon />}
          </IconButton>
        </Box>
      </Box>

      <Divider />

      {/* Room Actions */}
      <Box sx={{ p: 2 }}>
        <Button
          fullWidth
          variant="outlined"
          startIcon={<AddIcon />}
          onClick={() => setCreateRoomOpen(true)}
        >
          Create Room
        </Button>
      </Box>

      <Divider />

      {/* Room List */}
      <Box sx={{ flexGrow: 1, overflow: 'auto' }}>
        <List sx={{ py: 0 }}>
          {sortedRooms.map((room) => (
            <ListItem key={room.id} disablePadding>
              <ListItemButton
                selected={selectedRoom?.id === room.id}
                onClick={() => onRoomSelect(room)}
              >
                <ListItemAvatar>
                  <Badge
                    badgeContent={room.unreadCount}
                    color="primary"
                    invisible={!room.unreadCount}
                  >
                    <Avatar>{room.name[0].toUpperCase()}</Avatar>
                  </Badge>
                </ListItemAvatar>
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                      <Typography variant="subtitle1">{room.name}</Typography>
                      {room.isPrivate && (
                        <Chip label="Private" size="small" />
                      )}
                    </Box>
                  }
                  secondary={
                    <Typography variant="caption" color="text.secondary">
                      {room.lastMessage
                        ? room.lastMessage.content.substring(0, 30) + '...'
                        : 'No messages yet'}
                      {room.lastActivity && (
                        <> · {formatDistanceToNow(new Date(room.lastActivity), { addSuffix: true })}</>
                      )}
                    </Typography>
                  }
                />
                {room.typingUsers && room.typingUsers.length > 0 && (
                  <Typography variant="caption" color="primary">
                    typing...
                  </Typography>
                )}
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Box>

      {/* Modals */}
      <CreateRoomModal
        open={createRoomOpen}
        onClose={() => setCreateRoomOpen(false)}
      />
    </Box>
  );
};
```

### 3. Chat Room Component

```typescript
// frontend/src/components/chat/ChatRoom/index.tsx
import React, { useState, useRef, useEffect } from 'react';
import {
  Box,
  AppBar,
  Toolbar,
  Typography,
  IconButton,
  Avatar,
  AvatarGroup,
  Tooltip,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Info as InfoIcon,
  MoreVert as MoreVertIcon,
} from '@mui/icons-material';
import { MessageList } from '../Messages/MessageList';
import { MessageInput } from '../Messages/MessageInput';
import { TypingIndicator } from '../Messages/TypingIndicator';
import { Room, Message } from '../../../types/chat';
import { useSocket } from '../../../hooks/useSocket';

interface ChatRoomProps {
  room: Room;
  messages: Message[];
  onMenuClick: () => void;
  isMobile: boolean;
}

export const ChatRoom: React.FC<ChatRoomProps> = ({
  room,
  messages,
  onMenuClick,
  isMobile,
}) => {
  const { socket } = useSocket();
  const [isTyping, setIsTyping] = useState(false);
  const typingTimeoutRef = useRef<NodeJS.Timeout>();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Scroll to bottom on new messages
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSendMessage = (content: string, type: 'text' | 'image' | 'file' = 'text') => {
    if (!socket || !content.trim()) return;

    socket.emit('send-message', {
      roomId: room.id,
      content,
      messageType: type,
    }, (response) => {
      if (!response.success) {
        console.error('Failed to send message:', response.error);
        // Show error toast
      }
    });

    // Stop typing indicator
    handleStopTyping();
  };

  const handleTyping = () => {
    if (!isTyping) {
      setIsTyping(true);
      socket?.emit('typing-start', room.id);
    }

    // Clear existing timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    // Set new timeout to stop typing
    typingTimeoutRef.current = setTimeout(() => {
      handleStopTyping();
    }, 2000);
  };

  const handleStopTyping = () => {
    if (isTyping) {
      setIsTyping(false);
      socket?.emit('typing-stop', room.id);
    }
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Chat Header */}
      <AppBar position="static" color="default" elevation={1}>
        <Toolbar>
          {isMobile && (
            <IconButton edge="start" onClick={onMenuClick} sx={{ mr: 2 }}>
              <MenuIcon />
            </IconButton>
          )}
          
          <Box sx={{ display: 'flex', alignItems: 'center', flexGrow: 1 }}>
            <Avatar sx={{ mr: 2 }}>{room.name[0].toUpperCase()}</Avatar>
            <Box>
              <Typography variant="h6">{room.name}</Typography>
              <Typography variant="caption" color="text.secondary">
                {room.members?.length || 0} members
                {room.typingUsers && room.typingUsers.length > 0 && (
                  <> · {room.typingUsers.map(u => u.username).join(', ')} typing...</>
                )}
              </Typography>
            </Box>
          </Box>

          <AvatarGroup max={4} sx={{ mr: 2 }}>
            {room.members?.map((member) => (
              <Tooltip key={member.id} title={member.username}>
                <Avatar src={member.avatarUrl} alt={member.username}>
                  {member.username[0].toUpperCase()}
                </Avatar>
              </Tooltip>
            ))}
          </AvatarGroup>

          <IconButton>
            <InfoIcon />
          </IconButton>
          <IconButton>
            <MoreVertIcon />
          </IconButton>
        </Toolbar>
      </AppBar>

      {/* Message List */}
      <MessageList
        messages={messages}
        currentUserId={userId}
        room={room}
      />
      <div ref={messagesEndRef} />

      {/* Typing Indicator */}
      {room.typingUsers && room.typingUsers.length > 0 && (
        <TypingIndicator users={room.typingUsers} />
      )}

      {/* Message Input */}
      <MessageInput
        onSendMessage={handleSendMessage}
        onTyping={handleTyping}
        onStopTyping={handleStopTyping}
      />
    </Box>
  );
};
```

### 4. Message Components

```typescript
// frontend/src/components/chat/Messages/MessageBubble.tsx
import React, { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  Avatar,
  IconButton,
  Menu,
  MenuItem,
  Tooltip,
  Chip,
} from '@mui/material';
import {
  MoreVert as MoreIcon,
  Done as SentIcon,
  DoneAll as DeliveredIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Reply as ReplyIcon,
} from '@mui/icons-material';
import { format, formatDistanceToNow } from 'date-fns';
import { Message } from '../../../types/chat';
import { useSocket } from '../../../hooks/useSocket';
import { useAuth } from '../../../contexts/AuthContext';

interface MessageBubbleProps {
  message: Message;
  isOwn: boolean;
  showAvatar: boolean;
  onEdit?: (message: Message) => void;
  onReply?: (message: Message) => void;
}

export const MessageBubble: React.FC<MessageBubbleProps> = ({
  message,
  isOwn,
  showAvatar,
  onEdit,
  onReply,
}) => {
  const { socket } = useSocket();
  const { user } = useAuth();
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [showFullTime, setShowFullTime] = useState(false);

  const handleDelete = () => {
    socket?.emit('delete-message', {
      roomId: message.roomId,
      messageId: message.id,
    });
    setAnchorEl(null);
  };

  const getStatusIcon = () => {
    if (!isOwn) return null;
    
    if (message.readBy && message.readBy.length > 0) {
      return (
        <Tooltip title="Read">
          <DoneAllIcon sx={{ fontSize: 16, color: 'primary.main' }} />
        </Tooltip>
      );
    }
    
    return (
      <Tooltip title="Sent">
        <SentIcon sx={{ fontSize: 16, color: 'text.secondary' }} />
      </Tooltip>
    );
  };

  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: isOwn ? 'flex-end' : 'flex-start',
        mb: 1,
        px: 2,
      }}
    >
      {!isOwn && showAvatar && (
        <Avatar
          src={message.user.avatarUrl}
          alt={message.user.username}
          sx={{ mr: 1, width: 32, height: 32 }}
        >
          {message.user.username[0].toUpperCase()}
        </Avatar>
      )}
      
      {!isOwn && !showAvatar && <Box sx={{ width: 40 }} />}

      <Box sx={{ maxWidth: '70%' }}>
        {!isOwn && showAvatar && (
          <Typography variant="caption" color="text.secondary" sx={{ ml: 1 }}>
            {message.user.username}
          </Typography>
        )}
        
        <Paper
          elevation={1}
          sx={{
            p: 1.5,
            bgcolor: isOwn ? 'primary.main' : 'background.paper',
            color: isOwn ? 'primary.contrastText' : 'text.primary',
            borderRadius: 2,
            borderTopLeftRadius: isOwn ? 16 : 4,
            borderTopRightRadius: isOwn ? 4 : 16,
            position: 'relative',
            '&:hover .message-actions': {
              opacity: 1,
            },
          }}
        >
          <Box
            className="message-actions"
            sx={{
              position: 'absolute',
              top: -8,
              right: isOwn ? 'auto' : 8,
              left: isOwn ? 8 : 'auto',
              opacity: 0,
              transition: 'opacity 0.2s',
            }}
          >
            <IconButton
              size="small"
              onClick={(e) => setAnchorEl(e.currentTarget)}
              sx={{
                bgcolor: 'background.paper',
                boxShadow: 1,
                '&:hover': { bgcolor: 'background.paper' },
              }}
            >
              <MoreIcon fontSize="small" />
            </IconButton>
          </Box>

          {message.replyTo && (
            <Box
              sx={{
                p: 1,
                mb: 1,
                borderLeft: 3,
                borderColor: isOwn ? 'primary.light' : 'primary.main',
                bgcolor: isOwn ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.05)',
                borderRadius: 1,
              }}
            >
              <Typography variant="caption" fontWeight="bold">
                {message.replyTo.user.username}
              </Typography>
              <Typography variant="caption" display="block">
                {message.replyTo.content.substring(0, 50)}...
              </Typography>
            </Box>
          )}

          <Typography variant="body2" sx={{ wordBreak: 'break-word' }}>
            {message.content}
          </Typography>
          
          {message.isEdited && (
            <Typography variant="caption" sx={{ fontStyle: 'italic', opacity: 0.7 }}>
              (edited)
            </Typography>
          )}

          <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5, mt: 0.5 }}>
            <Typography
              variant="caption"
              sx={{ opacity: 0.7, cursor: 'pointer' }}
              onClick={() => setShowFullTime(!showFullTime)}
            >
              {showFullTime
                ? format(new Date(message.createdAt), 'PPpp')
                : formatDistanceToNow(new Date(message.createdAt), { addSuffix: true })}
            </Typography>
            {getStatusIcon()}
          </Box>
        </Paper>
      </Box>

      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={() => setAnchorEl(null)}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: isOwn ? 'left' : 'right',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: isOwn ? 'left' : 'right',
        }}
      >
        <MenuItem onClick={() => { onReply?.(message); setAnchorEl(null); }}>
          <ReplyIcon fontSize="small" sx={{ mr: 1 }} /> Reply
        </MenuItem>
        {isOwn && (
          <MenuItem onClick={() => { onEdit?.(message); setAnchorEl(null); }}>
            <EditIcon fontSize="small" sx={{ mr: 1 }} /> Edit
          </MenuItem>
        )}
        {(isOwn || user?.role === 'admin') && (
          <MenuItem onClick={handleDelete}>
            <DeleteIcon fontSize="small" sx={{ mr: 1 }} /> Delete
          </MenuItem>
        )}
      </Menu>
    </Box>
  );
};
```

### 5. Theme Support

```typescript
// frontend/src/contexts/ThemeContext.tsx
import React, { createContext, useContext, useState, useEffect } from 'react';
import { createTheme, ThemeProvider as MuiThemeProvider } from '@mui/material/styles';
import { CssBaseline } from '@mui/material';

type ThemeMode = 'light' | 'dark';

interface ThemeContextType {
  theme: ThemeMode;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [theme, setTheme] = useState<ThemeMode>(() => {
    const saved = localStorage.getItem('theme');
    return (saved as ThemeMode) || 'light';
  });

  useEffect(() => {
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme((prev) => (prev === 'light' ? 'dark' : 'light'));
  };

  const muiTheme = createTheme({
    palette: {
      mode: theme,
      primary: {
        main: theme === 'light' ? '#1976d2' : '#90caf9',
      },
      secondary: {
        main: theme === 'light' ? '#dc004e' : '#f48fb1',
      },
      background: {
        default: theme === 'light' ? '#f5f5f5' : '#121212',
        paper: theme === 'light' ? '#ffffff' : '#1e1e1e',
      },
    },
    typography: {
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Arial, sans-serif',
    },
    shape: {
      borderRadius: 8,
    },
    components: {
      MuiButton: {
        styleOverrides: {
          root: {
            textTransform: 'none',
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            backgroundImage: 'none',
          },
        },
      },
    },
  });

  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      <MuiThemeProvider theme={muiTheme}>
        <CssBaseline />
        {children}
      </MuiThemeProvider>
    </ThemeContext.Provider>
  );
};
```

### 6. Responsive Design Utilities

```typescript
// frontend/src/hooks/useResponsive.ts
import { useTheme } from '@mui/material/styles';
import { useMediaQuery } from '@mui/material';

export const useResponsive = () => {
  const theme = useTheme();
  
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'));
  const isTablet = useMediaQuery(theme.breakpoints.between('sm', 'md'));
  const isDesktop = useMediaQuery(theme.breakpoints.up('md'));
  const isLargeDesktop = useMediaQuery(theme.breakpoints.up('lg'));

  return {
    isMobile,
    isTablet,
    isDesktop,
    isLargeDesktop,
    breakpoint: {
      xs: isMobile,
      sm: isTablet,
      md: isDesktop,
      lg: isLargeDesktop,
    },
  };
};
```

## Socket.io Integration

### Event Handlers
```typescript
// New message received
socket.on('new-message', (message) => {
  dispatch({ type: 'ADD_MESSAGE', payload: message });
  
  // Show notification if not in current room
  if (message.roomId !== activeRoomId) {
    showNotification(message);
  }
});

// Typing indicators
socket.on('user-typing', ({ roomId, userId, username }) => {
  dispatch({ 
    type: 'UPDATE_TYPING', 
    payload: { roomId, userId, username, isTyping: true } 
  });
});

// User presence
socket.on('user-online', ({ userId }) => {
  dispatch({ type: 'UPDATE_USER_STATUS', payload: { userId, isOnline: true } });
});
```

## Performance Optimizations

### Message List Virtualization
- Use react-window for large message lists
- Lazy load older messages on scroll
- Implement message pagination
- Cache rendered messages

### Image Optimization
- Lazy load images in messages
- Compress images before upload
- Use progressive loading
- Implement image preview thumbnails

### State Management
- Memoize expensive computations
- Use React.memo for components
- Implement proper key strategies
- Optimize re-renders

## Accessibility Features

### Keyboard Navigation
- Tab through UI elements
- Enter to send messages
- Escape to close modals
- Arrow keys for message navigation

### Screen Reader Support
- Proper ARIA labels
- Live regions for new messages
- Role attributes
- Descriptive alt text

### Visual Accessibility
- High contrast mode support
- Focus indicators
- Color blind friendly palette
- Adjustable font sizes