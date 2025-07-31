# Task 7: Chat UI Implementation - Toolman Usage Guide

## Overview
This guide explains how to effectively use Toolman for implementing the chat UI with Socket.io integration. Toolman provides intelligent assistance for component development, real-time features, responsive design, and testing.

## Getting Started

### 1. Initial Setup
```bash
# Install dependencies
toolman run "npm install socket.io-client react-window emoji-picker-react lucide-react"

# Set up component structure
toolman generate component-structure --path src/components/chat

# Create base styles
toolman generate styles --themes light,dark --path src/styles
```

### 2. Component Development Workflow

#### Creating React Components
```bash
# Generate a new component with TypeScript
toolman generate component ChatLayout --typescript --hooks --styles

# Add Socket.io integration to component
toolman enhance ChatLayout --add-socket --real-time

# Generate child components
toolman generate component-tree Sidebar RoomList MessageList MessageInput
```

#### Component Templates
Toolman provides templates for common patterns:

```bash
# Real-time component with Socket.io
toolman template real-time-component MessageList

# Responsive layout component
toolman template responsive-layout ChatLayout

# Form component with validation
toolman template form-input MessageInput
```

### 3. Socket.io Integration Patterns

#### Setting Up Socket Context
```bash
# Generate Socket.io context with authentication
toolman generate socket-context --auth jwt --events message,typing,presence

# Create custom hooks for Socket.io
toolman generate hook useSocket --socket-client
toolman generate hook useChat --socket-events
```

#### Event Handler Generation
```bash
# Generate event handlers for real-time features
toolman socket add-handler message:new --component MessageList
toolman socket add-handler user:typing --component TypingIndicator
toolman socket add-emitter message:send --component MessageInput
```

#### Connection Management
```bash
# Set up reconnection logic
toolman socket configure --reconnection auto --max-attempts 5

# Add connection status monitoring
toolman socket add-status-indicator --component ChatHeader
```

### 4. Responsive Design Implementation

#### Breakpoint Management
```bash
# Generate responsive utilities
toolman responsive create-breakpoints --mobile 768 --tablet 1024

# Create responsive hook
toolman generate hook useMediaQuery --responsive

# Apply responsive styles
toolman style apply-responsive ChatLayout --breakpoints mobile,tablet,desktop
```

#### Mobile-Specific Features
```bash
# Add touch gestures
toolman mobile add-swipe-gesture Sidebar --direction right

# Optimize for mobile keyboards
toolman mobile handle-keyboard MessageInput --avoid-overlap

# Add mobile-friendly touch targets
toolman mobile optimize-touch-targets --min-size 44
```

### 5. Theme System Implementation

#### CSS Variables Setup
```bash
# Generate theme files with CSS variables
toolman theme create --names light,dark --css-vars

# Create theme context and hook
toolman generate theme-context --persistent localStorage

# Apply theme to components
toolman theme apply-to-components --all
```

#### Theme Testing
```bash
# Test theme switching
toolman test theme-switch --visual-regression

# Validate contrast ratios
toolman a11y check-contrast --themes all
```

### 6. Accessibility Implementation

#### ARIA Attributes
```bash
# Add ARIA labels automatically
toolman a11y add-aria-labels --components all

# Set up live regions for messages
toolman a11y configure-live-region MessageList --politeness polite

# Add keyboard navigation
toolman a11y add-keyboard-nav RoomList --arrow-keys
```

#### Screen Reader Testing
```bash
# Test with screen readers
toolman a11y test-screen-reader --nvda --jaws

# Generate accessibility report
toolman a11y generate-report --wcag-level AA
```

### 7. Performance Optimization

#### Message List Virtualization
```bash
# Implement virtual scrolling
toolman optimize virtualize MessageList --library react-window

# Add lazy loading for images
toolman optimize lazy-load MessageItem --images

# Implement message caching
toolman optimize add-cache useChat --strategy lru --max-size 1000
```

#### Bundle Optimization
```bash
# Analyze bundle size
toolman analyze bundle --visualize

# Code split components
toolman optimize code-split ChatRoom Modals

# Enable tree shaking
toolman build configure --tree-shake --minimize
```

### 8. Testing Strategies

#### Unit Testing
```bash
# Generate unit tests for components
toolman test generate --unit ChatLayout MessageItem

# Test Socket.io integration
toolman test socket-events --mock

# Run tests with coverage
toolman test run --coverage --threshold 80
```

#### Integration Testing
```bash
# Test real-time message flow
toolman test integration message-flow --e2e

# Test responsive behavior
toolman test responsive --viewports mobile,tablet,desktop

# Test theme switching
toolman test theme-integration --all-themes
```

#### E2E Testing
```bash
# Generate Cypress tests
toolman e2e generate chat-flow --cypress

# Test multi-user scenarios
toolman e2e multi-user --users 3 --actions send,type,join

# Run accessibility tests
toolman e2e a11y --axe-core
```

## Advanced Features

### 9. Real-time Performance Monitoring
```bash
# Monitor Socket.io latency
toolman monitor socket-latency --threshold 100ms

# Track render performance
toolman monitor react-renders --identify-unnecessary

# Monitor memory usage
toolman monitor memory-leaks --duration 30m
```

### 10. Debugging Tools

#### Socket.io Debugging
```bash
# Enable Socket.io debug mode
toolman debug socket --verbose

# Log all events
toolman debug socket-events --log-file socket.log

# Monitor connection state
toolman debug connection-state --real-time
```

#### Component Debugging
```bash
# Enable React DevTools profiler
toolman debug react-profiler --record

# Track component re-renders
toolman debug renders MessageList --highlight

# Debug responsive breakpoints
toolman debug responsive --show-breakpoints
```

## Best Practices

### 11. Code Quality

#### Linting and Formatting
```bash
# Run ESLint with React rules
toolman lint --fix --react-hooks

# Format with Prettier
toolman format --prettier

# Check TypeScript types
toolman typecheck --strict
```

#### Code Review Assistance
```bash
# Get AI code review
toolman review ChatLayout --security --performance

# Check for accessibility issues
toolman review --a11y

# Suggest optimizations
toolman suggest optimizations MessageList
```

### 12. Documentation Generation

#### Component Documentation
```bash
# Generate component docs
toolman docs generate --components --storybook

# Create usage examples
toolman docs examples MessageInput --interactive

# Generate API documentation
toolman docs api hooks/useSocket --markdown
```

### 13. Continuous Integration

#### CI Pipeline Setup
```bash
# Generate CI configuration
toolman ci generate --github-actions

# Add test stages
toolman ci add-stage test lint build

# Configure deployment
toolman ci add-deployment --preview --production
```

## Troubleshooting

### Common Issues

#### Socket.io Connection Problems
```bash
# Diagnose connection issues
toolman diagnose socket-connection

# Test with different transports
toolman test socket-transports --websocket --polling

# Check CORS configuration
toolman check cors --socket-server
```

#### Performance Issues
```bash
# Profile slow renders
toolman profile MessageList --flame-graph

# Find memory leaks
toolman detect memory-leaks --heap-snapshot

# Optimize re-renders
toolman optimize prevent-renders --memo --callback
```

#### Mobile-Specific Issues
```bash
# Test on real devices
toolman test mobile --devices "iPhone 12,Samsung S21"

# Debug touch events
toolman debug touch-events --visualize

# Check viewport issues
toolman check viewport --mobile-keyboards
```

## Integration with External Tools

### 14. Design Systems
```bash
# Import design tokens
toolman import design-tokens --figma

# Sync with design system
toolman sync design-system --components

# Generate theme from design
toolman generate theme-from-design --figma-file
```

### 15. Analytics and Monitoring
```bash
# Add performance tracking
toolman analytics add-performance --web-vitals

# Track user interactions
toolman analytics track-events --chat-actions

# Monitor errors
toolman monitor errors --sentry
```

## Conclusion

Toolman streamlines the chat UI implementation process by providing:
- Automated component generation with best practices
- Built-in Socket.io integration patterns
- Responsive design utilities
- Accessibility compliance tools
- Performance optimization features
- Comprehensive testing capabilities

Use these commands throughout your development process to ensure a high-quality, performant, and accessible chat interface.