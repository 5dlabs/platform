# Task 6: Create Basic Frontend Interface - Acceptance Criteria

## Overview
This document defines acceptance criteria for the frontend interface implementation. The frontend should provide a complete user interface for authentication and task management using vanilla JavaScript.

## File Structure Criteria

### ✓ Public Directory Created
- **Requirement**: Public directory exists with required files
- **Verification**:
  ```bash
  ls -la public/
  ```
- **Expected Files**:
  - index.html
  - styles.css
  - app.js

### ✓ Static Files Served
- **Requirement**: Express serves static files from public directory
- **Verification**:
  ```bash
  curl http://localhost:3000/index.html
  ```
- **Expected**: HTML content returned

## HTML Structure Criteria

### ✓ Complete HTML Document
- **Requirement**: Valid HTML5 structure
- **Verification**: Open http://localhost:3000 in browser
- **Expected Elements**:
  - DOCTYPE declaration
  - Meta viewport tag for responsive design
  - All required sections (auth, tasks)
  - Forms with proper inputs
  - Modal for editing

### ✓ Semantic HTML
- **Requirement**: Proper use of semantic elements
- **Verification**: Inspect HTML structure
- **Expected**:
  - `<header>`, `<main>`, `<section>` used appropriately
  - Form labels associated with inputs
  - Proper heading hierarchy

### ✓ Accessibility Features
- **Requirement**: Basic accessibility support
- **Expected**:
  - Labels for all form inputs
  - Autocomplete attributes
  - Type attributes on inputs
  - Button text is descriptive

## CSS Styling Criteria

### ✓ Responsive Design
- **Requirement**: Works on mobile, tablet, desktop
- **Test**:
  - Mobile: 320px - 768px width
  - Tablet: 768px - 1024px width
  - Desktop: 1024px+ width
- **Expected**: Layout adapts appropriately

### ✓ Visual Hierarchy
- **Requirement**: Clear visual organization
- **Expected**:
  - Headers stand out
  - Forms are clearly defined
  - Buttons have hover states
  - Active states visible

### ✓ Color Scheme
- **Requirement**: Consistent use of colors
- **Expected**:
  - Primary color for main actions
  - Error color for errors/delete
  - Success color for confirmations
  - Consistent throughout app

### ✓ Loading States
- **Requirement**: Visual feedback during async operations
- **Test**: Perform any API call
- **Expected**: Loading spinner covers screen

## JavaScript Functionality Criteria

### ✓ No Frameworks Used
- **Requirement**: Vanilla JavaScript only
- **Verification**: Check app.js source
- **Expected**: No React, Vue, Angular, jQuery, etc.

### ✓ API Integration
- **Requirement**: All API endpoints integrated
- **Test Each**:
  - POST /auth/register
  - POST /auth/login
  - GET /auth/me
  - GET /api/tasks
  - POST /api/tasks
  - PUT /api/tasks/:id
  - DELETE /api/tasks/:id

### ✓ State Management
- **Requirement**: Application state properly managed
- **Expected**:
  - User state maintained
  - Tasks array updated
  - Filter state persists
  - Loading state prevents duplicate requests

## Authentication Criteria

### ✓ Registration Flow
- **Requirement**: Users can register
- **Test**:
  1. Click Register tab
  2. Enter email and password
  3. Submit form
- **Expected**:
  - Success message shown
  - Redirected to tasks view
  - Token stored in localStorage

### ✓ Login Flow
- **Requirement**: Users can login
- **Test**:
  1. Enter credentials
  2. Submit form
- **Expected**:
  - Success message
  - Tasks section shown
  - User email displayed

### ✓ Tab Switching
- **Requirement**: Can switch between login/register
- **Test**: Click tabs
- **Expected**:
  - Active tab highlighted
  - Correct form shown
  - No page reload

### ✓ Token Persistence
- **Requirement**: Session persists on refresh
- **Test**:
  1. Login
  2. Refresh page
- **Expected**: Still logged in

### ✓ Logout Functionality
- **Requirement**: Can logout
- **Test**: Click logout button
- **Expected**:
  - Tokens cleared from localStorage
  - Redirected to login
  - Success message shown

### ✓ Auto-Authentication Check
- **Requirement**: Checks auth on load
- **Test**: Refresh with valid token
- **Expected**: Automatically shows tasks

## Task Management Criteria

### ✓ Create Task
- **Requirement**: Can create new tasks
- **Test**:
  1. Enter title (required)
  2. Enter description (optional)
  3. Submit form
- **Expected**:
  - Task appears in list
  - Form clears
  - Success message

### ✓ List Tasks
- **Requirement**: Shows all user tasks
- **Test**: Load tasks section
- **Expected**:
  - All tasks displayed
  - Proper formatting
  - Completed tasks styled differently

### ✓ Edit Task
- **Requirement**: Can edit existing tasks
- **Test**:
  1. Click Edit button
  2. Change fields
  3. Save changes
- **Expected**:
  - Modal opens with current data
  - Changes saved
  - List updates

### ✓ Delete Task
- **Requirement**: Can delete tasks
- **Test**:
  1. Click Delete button
  2. Confirm deletion
- **Expected**:
  - Confirmation prompt
  - Task removed from list
  - Success message

### ✓ Task Filtering
- **Requirement**: Can filter by status
- **Test**: Click All/Active/Completed
- **Expected**:
  - List updates immediately
  - Active filter highlighted
  - Correct tasks shown

### ✓ Empty State
- **Requirement**: Shows message when no tasks
- **Test**: Delete all tasks or filter to empty
- **Expected**: "No tasks yet" message

## Form Validation Criteria

### ✓ Client-Side Validation
- **Requirement**: Basic HTML5 validation
- **Test Cases**:
  - Email: required, email type
  - Password: required, minlength="8"
  - Title: required, maxlength="255"
  - Description: optional, maxlength="1000"

### ✓ Validation Feedback
- **Requirement**: Browser shows validation errors
- **Test**: Submit invalid form
- **Expected**: Browser validation messages

## Error Handling Criteria

### ✓ API Error Display
- **Requirement**: Shows API errors to user
- **Test**: Trigger various errors
- **Expected**:
  - Error message displayed
  - Auto-hides after 5 seconds
  - Can manually close

### ✓ Network Error Handling
- **Requirement**: Handles connection issues
- **Test**: Stop server and try action
- **Expected**: "Network error" message

### ✓ Auth Error Handling
- **Requirement**: Handles auth failures
- **Test**: Use expired token
- **Expected**: Redirect to login

## UI Feedback Criteria

### ✓ Success Messages
- **Requirement**: Shows success feedback
- **Test**: Complete any action
- **Expected**:
  - Success message shown
  - Auto-hides after 3 seconds
  - Green color

### ✓ Loading States
- **Requirement**: Shows loading during requests
- **Test**: Perform any API call
- **Expected**:
  - Loading overlay shown
  - Prevents duplicate submissions
  - Hides after request

### ✓ Interactive Elements
- **Requirement**: Clear interaction feedback
- **Expected**:
  - Buttons have hover states
  - Active states visible
  - Disabled during loading

## Security Criteria

### ✓ XSS Prevention
- **Requirement**: User input is escaped
- **Test**: Create task with HTML/script
  ```
  Title: <script>alert('XSS')</script>
  Description: <img src=x onerror=alert('XSS')>
  ```
- **Expected**: HTML displayed as text, not executed

### ✓ Token Storage
- **Requirement**: Tokens stored in localStorage
- **Verification**:
  ```javascript
  localStorage.getItem('accessToken')
  localStorage.getItem('refreshToken')
  ```
- **Expected**: Tokens present after login

### ✓ Authorization Headers
- **Requirement**: Token sent with API requests
- **Verification**: Check network tab
- **Expected**: Authorization: Bearer TOKEN

## Modal Functionality Criteria

### ✓ Edit Modal Opens
- **Requirement**: Modal shows for editing
- **Test**: Click Edit on any task
- **Expected**:
  - Modal appears
  - Current values populated
  - Can modify all fields

### ✓ Modal Closes
- **Requirement**: Multiple ways to close
- **Test Each**:
  - Click X button
  - Click Cancel button
  - Click outside modal
  - Press Escape key
- **Expected**: Modal closes without saving

### ✓ Modal Saves
- **Requirement**: Can save changes
- **Test**: Edit and click Save
- **Expected**:
  - Changes saved
  - Modal closes
  - List updates

## Responsive Design Criteria

### ✓ Mobile Layout
- **Requirement**: Usable on mobile
- **Test**: Width < 768px
- **Expected**:
  - Single column layout
  - Buttons remain clickable
  - Forms fit screen

### ✓ Tablet Layout
- **Requirement**: Optimized for tablets
- **Test**: Width 768px - 1024px
- **Expected**: Appropriate spacing

### ✓ Desktop Layout
- **Requirement**: Takes advantage of space
- **Test**: Width > 1024px
- **Expected**: Centered content, max-width applied

## Performance Criteria

### ✓ Fast Initial Load
- **Requirement**: Page loads quickly
- **Expected**: < 1 second on localhost

### ✓ Efficient Updates
- **Requirement**: DOM updates efficiently
- **Test**: Add many tasks
- **Expected**: No lag when filtering/updating

## Test Summary Checklist

- [ ] All files created (index.html, styles.css, app.js)
- [ ] Static files served by Express
- [ ] HTML structure complete and semantic
- [ ] CSS provides responsive design
- [ ] No frameworks used (vanilla JS only)
- [ ] Registration flow works
- [ ] Login flow works
- [ ] Logout clears session
- [ ] Token persists on refresh
- [ ] Tasks can be created
- [ ] Tasks list displays correctly
- [ ] Tasks can be edited via modal
- [ ] Tasks can be deleted with confirmation
- [ ] Task filtering works
- [ ] Empty state shows when no tasks
- [ ] Form validation works
- [ ] API errors displayed to user
- [ ] Success messages show and auto-hide
- [ ] Loading states prevent duplicate requests
- [ ] XSS attacks prevented
- [ ] Modal has multiple close methods
- [ ] Responsive on all screen sizes
- [ ] Escape key closes modal

## Definition of Done

Task 6 is complete when:
1. Complete frontend interface is functional
2. All authentication features work
3. All task management features work
4. Responsive design implemented
5. Error handling provides good UX
6. Security measures in place (XSS prevention)
7. No frameworks used (vanilla JS only)
8. All acceptance criteria met

## Notes

- localStorage used for token storage (consider security implications in production)
- Client-side validation supplements server validation
- Loading states prevent race conditions
- Auto-hide timers improve UX
- Escape key support improves accessibility