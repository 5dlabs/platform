# Task 7: Chat UI Implementation - AI Agent Prompt

You are a senior frontend developer tasked with building a modern, responsive chat interface with real-time capabilities. Your implementation must provide an intuitive user experience across all devices while integrating seamlessly with Socket.io for real-time updates.

## Primary Objectives

1. **Layout Components**: Create a flexible layout with sidebar navigation and main chat area that works on desktop and mobile.

2. **Real-time Integration**: Connect Socket.io client with UI components for instant message updates, typing indicators, and presence.

3. **Message Components**: Build message display with proper styling, timestamps, status indicators, and user information.

4. **Responsive Design**: Ensure the interface works perfectly on mobile, tablet, and desktop with appropriate breakpoints.

5. **Theme Support**: Implement dark/light theme switching with consistent styling across all components.

## Required Actions

### Phase 1: Project Setup (10 minutes)
1. Install UI dependencies:
   ```bash
   npm install @mui/material @emotion/react @emotion/styled
   npm install socket.io-client date-fns
   npm install react-window react-intersection-observer
   npm install framer-motion @mui/icons-material
   ```

2. Create component structure:
   ```
   components/
   ├── chat/
   │   ├── ChatLayout.tsx
   │   ├── Sidebar/
   │   ├── ChatRoom/
   │   ├── Messages/
   │   └── Modals/
   ```

3. Set up theme context for dark/light mode

### Phase 2: Layout Implementation (20 minutes)
1. **Main Layout**:
   - Flexible container with sidebar and chat area
   - Responsive drawer for mobile
   - Proper spacing and scrolling

2. **Sidebar Component**:
   - User profile section
   - Room list with search
   - Create room button
   - Theme toggle
   - Connection status

3. **Mobile Adaptations**:
   - Collapsible sidebar
   - Full-screen chat on mobile
   - Touch-friendly interactions

### Phase 3: Chat Room Components (25 minutes)
1. **Chat Header**:
   - Room name and avatar
   - Member list
   - Room actions menu
   - Mobile menu button

2. **Message List**:
   - Scrollable message container
   - Auto-scroll to bottom
   - Load more on scroll up
   - Date separators

3. **Message Input**:
   - Text input with emoji support
   - File upload button
   - Send button
   - Character counter
   - Typing indicator trigger

### Phase 4: Message Components (20 minutes)
1. **Message Bubble**:
   - Different styles for own/others
   - Avatar display logic
   - Timestamp formatting
   - Read receipts
   - Edit/delete actions

2. **Message Types**:
   - Text messages
   - Image messages with preview
   - File attachments
   - System messages
   - Reply indicators

3. **Status Indicators**:
   - Sent/delivered/read states
   - Typing indicators
   - Online/offline badges

### Phase 5: Socket.io Integration (15 minutes)
1. **Connection Management**:
   - Initialize socket with auth
   - Handle reconnection
   - Show connection status
   - Error handling

2. **Event Listeners**:
   - New messages
   - Typing events
   - Presence updates
   - Read receipts
   - Room updates

3. **Emit Events**:
   - Send messages
   - Typing start/stop
   - Mark as read
   - Join/leave rooms

### Phase 6: Responsive & Theme (10 minutes)
1. **Responsive Breakpoints**:
   - Mobile: < 768px
   - Tablet: 768px - 1024px
   - Desktop: > 1024px
   - Adjust layouts accordingly

2. **Theme Implementation**:
   - Light/dark color schemes
   - Consistent spacing
   - Typography scales
   - Component variants

## Implementation Details

### Mobile-First Approach
```typescript
// Design for mobile first, then enhance for larger screens
const styles = {
  container: {
    padding: theme.spacing(1), // Mobile
    [theme.breakpoints.up('md')]: {
      padding: theme.spacing(3), // Desktop
    },
  },
};
```

### Socket.io Event Pattern
```typescript
// Listen for events
useEffect(() => {
  socket.on('new-message', handleNewMessage);
  socket.on('user-typing', handleTypingUpdate);
  
  return () => {
    socket.off('new-message');
    socket.off('user-typing');
  };
}, []);

// Emit with acknowledgment
const sendMessage = (content: string) => {
  socket.emit('send-message', 
    { roomId, content }, 
    (response) => {
      if (!response.success) {
        // Handle error
      }
    }
  );
};
```

### Responsive Sidebar Pattern
```typescript
const drawerVariant = isMobile ? 'temporary' : 'permanent';
const drawerProps = isMobile ? {
  open: sidebarOpen,
  onClose: () => setSidebarOpen(false),
} : {};
```

## UI/UX Requirements

### Visual Design
- Clean, modern interface
- Consistent spacing (8px grid)
- Clear visual hierarchy
- Smooth transitions
- Loading states

### Interaction Design
- Instant feedback for actions
- Optimistic updates
- Error recovery
- Offline handling
- Keyboard shortcuts

### Mobile Considerations
- Touch targets minimum 44px
- Swipe gestures
- Virtual keyboard handling
- Landscape orientation
- Safe area insets

## Performance Optimization

### Message List
```typescript
// Use virtualization for long lists
import { FixedSizeList } from 'react-window';

// Lazy load images
const LazyImage = ({ src, alt }) => {
  const { ref, inView } = useInView({
    triggerOnce: true,
    rootMargin: '100px',
  });
  
  return (
    <div ref={ref}>
      {inView && <img src={src} alt={alt} />}
    </div>
  );
};
```

### State Updates
```typescript
// Batch updates
const handleMessages = useCallback((newMessages) => {
  setBatch(() => {
    // Update multiple state values
  });
}, []);

// Memoize expensive computations
const sortedRooms = useMemo(() => 
  rooms.sort((a, b) => b.lastActivity - a.lastActivity),
  [rooms]
);
```

## Accessibility Requirements

### Keyboard Navigation
- Tab order logical
- Focus indicators visible
- Escape closes modals
- Enter sends messages
- Arrow keys navigate

### Screen Reader Support
```typescript
// Announce new messages
<div role="log" aria-live="polite" aria-label="Chat messages">
  {messages.map(msg => (
    <div role="article" aria-label={`Message from ${msg.user.name}`}>
      {msg.content}
    </div>
  ))}
</div>
```

### Color Contrast
- WCAG AA compliance
- 4.5:1 for normal text
- 3:1 for large text
- Don't rely only on color

## Testing Checklist

### Component Tests
```typescript
describe('ChatLayout', () => {
  test('renders sidebar and chat area');
  test('toggles sidebar on mobile');
  test('shows selected room');
  test('handles empty state');
});

describe('MessageBubble', () => {
  test('shows own messages on right');
  test('shows others messages on left');
  test('displays read receipts');
  test('formats timestamps correctly');
});
```

### Integration Tests
- Send and receive messages
- Real-time updates work
- Typing indicators show/hide
- Theme switching persists
- Responsive layouts adapt

## Error Handling

### Connection Errors
```typescript
const ConnectionStatus = () => {
  if (!isConnected) {
    return (
      <Alert severity="warning">
        Connection lost. Trying to reconnect...
        <Button onClick={reconnect}>Retry Now</Button>
      </Alert>
    );
  }
};
```

### Message Failures
```typescript
// Retry failed messages
const retryMessage = async (tempId: string) => {
  const message = failedMessages.get(tempId);
  if (message) {
    await sendMessage(message.content);
    removeFailedMessage(tempId);
  }
};
```

## Theme Configuration

### Color Schemes
```typescript
const lightTheme = {
  primary: '#1976d2',
  background: '#ffffff',
  surface: '#f5f5f5',
  text: '#000000',
};

const darkTheme = {
  primary: '#90caf9',
  background: '#121212',
  surface: '#1e1e1e',
  text: '#ffffff',
};
```

### Component Theming
```typescript
// Consistent theming across components
const MessageBubble = styled(Paper)(({ theme, isOwn }) => ({
  backgroundColor: isOwn 
    ? theme.palette.primary.main 
    : theme.palette.background.paper,
  color: isOwn 
    ? theme.palette.primary.contrastText 
    : theme.palette.text.primary,
}));
```

## Final Deliverables

Before marking complete:
- [ ] Layout responsive on all devices
- [ ] Socket.io fully integrated
- [ ] Messages display correctly
- [ ] Typing indicators working
- [ ] Theme switching functional
- [ ] Performance optimized
- [ ] Accessibility standards met
- [ ] Error handling comprehensive
- [ ] Loading states smooth
- [ ] Animations performant

Execute this task systematically, ensuring the chat interface provides a delightful user experience with real-time capabilities and works flawlessly across all devices.