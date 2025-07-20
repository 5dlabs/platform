# Task 6: Create Basic Frontend Interface - Autonomous AI Agent Prompt

You are tasked with creating a simple but functional frontend interface using vanilla HTML, CSS, and JavaScript. This single-page application will allow users to interact with the API endpoints through a web browser, demonstrating authentication and task management features.

## Your Mission

Build a complete frontend interface with user authentication (login/register) and task management (CRUD operations) using only vanilla JavaScript. The interface should be responsive, handle errors gracefully, and protect against XSS attacks.

## Prerequisites

Ensure Tasks 1-5 are complete:
- Express server with static file serving
- Authentication endpoints working
- Task management API functional
- Validation and error handling in place

## Step-by-Step Instructions

### 1. Create Public Directory

```bash
mkdir -p public
```

### 2. Create HTML Structure

Create `public/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Task Manager</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div id="app">
        <header>
            <h1>Task Manager</h1>
            <nav id="nav" class="hidden">
                <span id="user-email"></span>
                <button id="logout-btn">Logout</button>
            </nav>
        </header>

        <main>
            <!-- Loading Spinner -->
            <div id="loading" class="loading hidden">
                <div class="spinner"></div>
            </div>

            <!-- Error Messages -->
            <div id="error-container" class="error-container hidden">
                <p id="error-message"></p>
                <button id="error-close">&times;</button>
            </div>

            <!-- Success Messages -->
            <div id="success-container" class="success-container hidden">
                <p id="success-message"></p>
            </div>

            <!-- Auth Section -->
            <section id="auth-section" class="auth-section">
                <div class="auth-container">
                    <div class="auth-tabs">
                        <button class="tab-btn active" data-tab="login">Login</button>
                        <button class="tab-btn" data-tab="register">Register</button>
                    </div>

                    <!-- Login Form -->
                    <form id="login-form" class="auth-form">
                        <h2>Login</h2>
                        <div class="form-group">
                            <label for="login-email">Email</label>
                            <input 
                                type="email" 
                                id="login-email" 
                                name="email" 
                                required 
                                autocomplete="email"
                            >
                        </div>
                        <div class="form-group">
                            <label for="login-password">Password</label>
                            <input 
                                type="password" 
                                id="login-password" 
                                name="password" 
                                required 
                                autocomplete="current-password"
                            >
                        </div>
                        <button type="submit" class="btn btn-primary">Login</button>
                    </form>

                    <!-- Register Form -->
                    <form id="register-form" class="auth-form hidden">
                        <h2>Register</h2>
                        <div class="form-group">
                            <label for="register-email">Email</label>
                            <input 
                                type="email" 
                                id="register-email" 
                                name="email" 
                                required 
                                autocomplete="email"
                            >
                        </div>
                        <div class="form-group">
                            <label for="register-password">Password</label>
                            <input 
                                type="password" 
                                id="register-password" 
                                name="password" 
                                required 
                                minlength="8"
                                autocomplete="new-password"
                            >
                            <small>Minimum 8 characters</small>
                        </div>
                        <button type="submit" class="btn btn-primary">Register</button>
                    </form>
                </div>
            </section>

            <!-- Tasks Section -->
            <section id="tasks-section" class="tasks-section hidden">
                <div class="tasks-container">
                    <!-- Add Task Form -->
                    <div class="add-task-form">
                        <h2>Add New Task</h2>
                        <form id="add-task-form">
                            <div class="form-group">
                                <input 
                                    type="text" 
                                    id="task-title" 
                                    placeholder="Task title" 
                                    required 
                                    maxlength="255"
                                >
                            </div>
                            <div class="form-group">
                                <textarea 
                                    id="task-description" 
                                    placeholder="Task description (optional)" 
                                    maxlength="1000"
                                    rows="3"
                                ></textarea>
                            </div>
                            <button type="submit" class="btn btn-primary">Add Task</button>
                        </form>
                    </div>

                    <!-- Task Filters -->
                    <div class="task-filters">
                        <h2>My Tasks</h2>
                        <div class="filter-buttons">
                            <button class="filter-btn active" data-filter="all">All</button>
                            <button class="filter-btn" data-filter="active">Active</button>
                            <button class="filter-btn" data-filter="completed">Completed</button>
                        </div>
                    </div>

                    <!-- Tasks List -->
                    <div id="tasks-list" class="tasks-list">
                        <!-- Tasks will be dynamically inserted here -->
                    </div>

                    <!-- Empty State -->
                    <div id="empty-state" class="empty-state hidden">
                        <p>No tasks yet. Create your first task above!</p>
                    </div>
                </div>
            </section>
        </main>
    </div>

    <!-- Edit Task Modal -->
    <div id="edit-modal" class="modal hidden">
        <div class="modal-content">
            <div class="modal-header">
                <h2>Edit Task</h2>
                <button class="modal-close">&times;</button>
            </div>
            <form id="edit-task-form">
                <input type="hidden" id="edit-task-id">
                <div class="form-group">
                    <label for="edit-task-title">Title</label>
                    <input 
                        type="text" 
                        id="edit-task-title" 
                        required 
                        maxlength="255"
                    >
                </div>
                <div class="form-group">
                    <label for="edit-task-description">Description</label>
                    <textarea 
                        id="edit-task-description" 
                        maxlength="1000"
                        rows="3"
                    ></textarea>
                </div>
                <div class="form-group">
                    <label>
                        <input type="checkbox" id="edit-task-completed">
                        Mark as completed
                    </label>
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn btn-secondary modal-cancel">Cancel</button>
                    <button type="submit" class="btn btn-primary">Save Changes</button>
                </div>
            </form>
        </div>
    </div>

    <script src="app.js"></script>
</body>
</html>
```

### 3. Create CSS Styles

Create `public/styles.css`:

```css
/* CSS Reset and Variables */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

:root {
    --primary-color: #3498db;
    --primary-hover: #2980b9;
    --success-color: #27ae60;
    --error-color: #e74c3c;
    --warning-color: #f39c12;
    --background-color: #f5f5f5;
    --card-background: #ffffff;
    --text-color: #333333;
    --text-light: #666666;
    --border-color: #dddddd;
    --shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    --shadow-hover: 0 4px 8px rgba(0, 0, 0, 0.15);
}

/* Base Styles */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--background-color);
}

.hidden {
    display: none !important;
}

/* Header */
header {
    background-color: var(--card-background);
    box-shadow: var(--shadow);
    padding: 1rem 0;
    position: sticky;
    top: 0;
    z-index: 100;
}

header h1 {
    text-align: center;
    color: var(--primary-color);
    margin-bottom: 0.5rem;
}

nav {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
}

#user-email {
    color: var(--text-light);
}

/* Buttons */
.btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    transition: all 0.3s ease;
}

.btn:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow);
}

.btn:active {
    transform: translateY(0);
}

.btn-primary {
    background-color: var(--primary-color);
    color: white;
}

.btn-primary:hover {
    background-color: var(--primary-hover);
}

.btn-secondary {
    background-color: var(--text-light);
    color: white;
}

.btn-secondary:hover {
    background-color: #555;
}

.btn-danger {
    background-color: var(--error-color);
    color: white;
}

.btn-danger:hover {
    background-color: #c0392b;
}

.btn-sm {
    padding: 0.25rem 0.5rem;
    font-size: 0.875rem;
}

#logout-btn {
    background: none;
    border: 1px solid var(--error-color);
    color: var(--error-color);
    padding: 0.25rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.3s ease;
}

#logout-btn:hover {
    background-color: var(--error-color);
    color: white;
}

/* Forms */
.form-group {
    margin-bottom: 1rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: 500;
}

.form-group input,
.form-group textarea {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 1rem;
    font-family: inherit;
}

.form-group input:focus,
.form-group textarea:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.2);
}

.form-group small {
    color: var(--text-light);
    font-size: 0.875rem;
    margin-top: 0.25rem;
    display: block;
}

/* Auth Section */
.auth-section {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: calc(100vh - 100px);
    padding: 2rem;
}

.auth-container {
    background-color: var(--card-background);
    border-radius: 8px;
    box-shadow: var(--shadow);
    width: 100%;
    max-width: 400px;
    overflow: hidden;
}

.auth-tabs {
    display: flex;
    border-bottom: 1px solid var(--border-color);
}

.tab-btn {
    flex: 1;
    padding: 1rem;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    color: var(--text-light);
    transition: all 0.3s ease;
    position: relative;
}

.tab-btn:hover {
    background-color: rgba(52, 152, 219, 0.1);
}

.tab-btn.active {
    color: var(--primary-color);
}

.tab-btn.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 2px;
    background-color: var(--primary-color);
}

.auth-form {
    padding: 2rem;
}

.auth-form h2 {
    margin-bottom: 1.5rem;
    text-align: center;
}

/* Tasks Section */
.tasks-section {
    padding: 2rem;
    max-width: 800px;
    margin: 0 auto;
}

.tasks-container {
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.add-task-form,
.task-filters {
    background-color: var(--card-background);
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: var(--shadow);
}

.add-task-form h2,
.task-filters h2 {
    margin-bottom: 1rem;
}

.filter-buttons {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
}

.filter-btn {
    padding: 0.5rem 1rem;
    background: none;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.3s ease;
}

.filter-btn:hover {
    border-color: var(--primary-color);
}

.filter-btn.active {
    background-color: var(--primary-color);
    color: white;
    border-color: var(--primary-color);
}

/* Task Items */
.tasks-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.task-item {
    background-color: var(--card-background);
    border-radius: 8px;
    padding: 1rem;
    box-shadow: var(--shadow);
    transition: all 0.3s ease;
}

.task-item:hover {
    box-shadow: var(--shadow-hover);
    transform: translateY(-2px);
}

.task-item.completed {
    opacity: 0.7;
}

.task-item.completed .task-title {
    text-decoration: line-through;
    color: var(--text-light);
}

.task-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    gap: 1rem;
    margin-bottom: 0.5rem;
}

.task-title {
    font-size: 1.1rem;
    font-weight: 500;
    margin: 0;
    word-break: break-word;
}

.task-description {
    color: var(--text-light);
    margin: 0.5rem 0;
    word-break: break-word;
}

.task-meta {
    display: flex;
    gap: 1rem;
    font-size: 0.875rem;
    color: var(--text-light);
    flex-wrap: wrap;
}

.task-status {
    color: var(--success-color);
    font-weight: 500;
}

.task-actions {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
}

/* Messages */
.error-container,
.success-container {
    position: fixed;
    top: 80px;
    left: 50%;
    transform: translateX(-50%);
    padding: 1rem 2rem;
    border-radius: 4px;
    box-shadow: var(--shadow);
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 1rem;
    max-width: 90%;
    animation: slideDown 0.3s ease;
}

@keyframes slideDown {
    from {
        transform: translate(-50%, -100%);
        opacity: 0;
    }
    to {
        transform: translate(-50%, 0);
        opacity: 1;
    }
}

.error-container {
    background-color: #fee;
    border: 1px solid #fcc;
    color: var(--error-color);
}

.success-container {
    background-color: #efe;
    border: 1px solid #cfc;
    color: var(--success-color);
}

#error-close {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--error-color);
    line-height: 1;
}

/* Loading */
.loading {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
}

.spinner {
    width: 50px;
    height: 50px;
    border: 4px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* Modal */
.modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    padding: 1rem;
}

.modal-content {
    background-color: var(--card-background);
    border-radius: 8px;
    padding: 2rem;
    width: 100%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
    animation: modalSlideIn 0.3s ease;
}

@keyframes modalSlideIn {
    from {
        transform: translateY(-50px);
        opacity: 0;
    }
    to {
        transform: translateY(0);
        opacity: 1;
    }
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
}

.modal-header h2 {
    margin: 0;
}

.modal-close {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--text-light);
    cursor: pointer;
    line-height: 1;
}

.modal-close:hover {
    color: var(--text-color);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    margin-top: 1.5rem;
}

/* Empty State */
.empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--text-light);
    background-color: var(--card-background);
    border-radius: 8px;
    box-shadow: var(--shadow);
}

/* Responsive Design */
@media (max-width: 768px) {
    .tasks-section {
        padding: 1rem;
    }
    
    .add-task-form,
    .task-filters {
        padding: 1rem;
    }
    
    .task-header {
        flex-direction: column;
        align-items: stretch;
    }
    
    .task-actions {
        margin-top: 0.5rem;
        justify-content: flex-end;
    }
    
    .modal-content {
        padding: 1.5rem;
    }
}

@media (max-width: 480px) {
    header h1 {
        font-size: 1.5rem;
    }
    
    .auth-container {
        margin: 0 1rem;
    }
    
    .filter-buttons {
        width: 100%;
    }
    
    .filter-btn {
        flex: 1;
        text-align: center;
    }
}
```

### 4. Create JavaScript Application

Create `public/app.js`:

```javascript
// API Configuration
const API_BASE_URL = window.location.origin;
const API_ENDPOINTS = {
    register: '/auth/register',
    login: '/auth/login',
    refresh: '/auth/refresh',
    me: '/auth/me',
    tasks: '/api/tasks'
};

// Application State
const state = {
    user: null,
    tasks: [],
    filter: 'all',
    loading: false
};

// DOM Elements Cache
const elements = {
    // Main sections
    nav: document.getElementById('nav'),
    userEmail: document.getElementById('user-email'),
    logoutBtn: document.getElementById('logout-btn'),
    authSection: document.getElementById('auth-section'),
    tasksSection: document.getElementById('tasks-section'),
    
    // Forms
    loginForm: document.getElementById('login-form'),
    registerForm: document.getElementById('register-form'),
    addTaskForm: document.getElementById('add-task-form'),
    editTaskForm: document.getElementById('edit-task-form'),
    
    // Lists and containers
    tasksList: document.getElementById('tasks-list'),
    emptyState: document.getElementById('empty-state'),
    
    // UI elements
    loading: document.getElementById('loading'),
    errorContainer: document.getElementById('error-container'),
    errorMessage: document.getElementById('error-message'),
    errorClose: document.getElementById('error-close'),
    successContainer: document.getElementById('success-container'),
    successMessage: document.getElementById('success-message'),
    editModal: document.getElementById('edit-modal')
};

// Utility Functions
const escapeHtml = (unsafe) => {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
};

const showLoading = () => {
    state.loading = true;
    elements.loading.classList.remove('hidden');
};

const hideLoading = () => {
    state.loading = false;
    elements.loading.classList.add('hidden');
};

const showError = (message) => {
    elements.errorMessage.textContent = message;
    elements.errorContainer.classList.remove('hidden');
    
    // Auto-hide after 5 seconds
    setTimeout(() => {
        elements.errorContainer.classList.add('hidden');
    }, 5000);
};

const showSuccess = (message) => {
    elements.successMessage.textContent = message;
    elements.successContainer.classList.remove('hidden');
    
    // Auto-hide after 3 seconds
    setTimeout(() => {
        elements.successContainer.classList.add('hidden');
    }, 3000);
};

const formatDate = (dateString) => {
    const date = new Date(dateString);
    const options = {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    };
    return date.toLocaleDateString('en-US', options);
};

// API Functions
const api = {
    async request(url, options = {}) {
        const token = localStorage.getItem('accessToken');
        
        const config = {
            ...options,
            headers: {
                'Content-Type': 'application/json',
                ...(token && { 'Authorization': `Bearer ${token}` }),
                ...options.headers
            }
        };
        
        try {
            const response = await fetch(API_BASE_URL + url, config);
            
            // Handle 204 No Content
            if (response.status === 204) {
                return null;
            }
            
            const data = await response.json();
            
            if (!response.ok) {
                throw new Error(data.error?.message || 'Request failed');
            }
            
            return data;
        } catch (error) {
            if (error.message === 'Failed to fetch') {
                throw new Error('Network error. Please check your connection.');
            }
            throw error;
        }
    },
    
    async register(email, password) {
        const data = await this.request(API_ENDPOINTS.register, {
            method: 'POST',
            body: JSON.stringify({ email, password })
        });
        
        // Store tokens
        localStorage.setItem('accessToken', data.tokens.accessToken);
        localStorage.setItem('refreshToken', data.tokens.refreshToken);
        
        return data;
    },
    
    async login(email, password) {
        const data = await this.request(API_ENDPOINTS.login, {
            method: 'POST',
            body: JSON.stringify({ email, password })
        });
        
        // Store tokens
        localStorage.setItem('accessToken', data.tokens.accessToken);
        localStorage.setItem('refreshToken', data.tokens.refreshToken);
        
        return data;
    },
    
    async getMe() {
        return this.request(API_ENDPOINTS.me);
    },
    
    async getTasks(filter = {}) {
        let url = API_ENDPOINTS.tasks;
        const params = new URLSearchParams();
        
        if (filter.completed !== undefined) {
            params.append('completed', filter.completed);
        }
        
        if (params.toString()) {
            url += '?' + params.toString();
        }
        
        return this.request(url);
    },
    
    async createTask(title, description) {
        return this.request(API_ENDPOINTS.tasks, {
            method: 'POST',
            body: JSON.stringify({ 
                title: title.trim(), 
                description: description.trim() || null 
            })
        });
    },
    
    async updateTask(id, updates) {
        return this.request(`${API_ENDPOINTS.tasks}/${id}`, {
            method: 'PUT',
            body: JSON.stringify(updates)
        });
    },
    
    async deleteTask(id) {
        return this.request(`${API_ENDPOINTS.tasks}/${id}`, {
            method: 'DELETE'
        });
    }
};

// Auth Functions
const checkAuth = async () => {
    const token = localStorage.getItem('accessToken');
    
    if (!token) {
        showAuthSection();
        return;
    }
    
    try {
        showLoading();
        const data = await api.getMe();
        state.user = data.user;
        showTasksSection();
    } catch (error) {
        // Token invalid or expired
        localStorage.removeItem('accessToken');
        localStorage.removeItem('refreshToken');
        showAuthSection();
    } finally {
        hideLoading();
    }
};

const logout = () => {
    // Clear tokens
    localStorage.removeItem('accessToken');
    localStorage.removeItem('refreshToken');
    
    // Clear state
    state.user = null;
    state.tasks = [];
    
    // Show auth section
    showAuthSection();
    showSuccess('Logged out successfully');
};

// UI Functions
const showAuthSection = () => {
    elements.authSection.classList.remove('hidden');
    elements.tasksSection.classList.add('hidden');
    elements.nav.classList.add('hidden');
};

const showTasksSection = () => {
    elements.authSection.classList.add('hidden');
    elements.tasksSection.classList.remove('hidden');
    elements.nav.classList.remove('hidden');
    elements.userEmail.textContent = state.user.email;
    loadTasks();
};

const renderTasks = () => {
    // Filter tasks based on current filter
    const filteredTasks = state.tasks.filter(task => {
        if (state.filter === 'all') return true;
        if (state.filter === 'active') return !task.completed;
        if (state.filter === 'completed') return task.completed;
        return true;
    });
    
    // Show empty state if no tasks
    if (filteredTasks.length === 0) {
        elements.tasksList.classList.add('hidden');
        elements.emptyState.classList.remove('hidden');
        return;
    }
    
    // Hide empty state and show tasks
    elements.tasksList.classList.remove('hidden');
    elements.emptyState.classList.add('hidden');
    
    // Render task items
    elements.tasksList.innerHTML = filteredTasks.map(task => `
        <div class="task-item ${task.completed ? 'completed' : ''}" data-id="${task.id}">
            <div class="task-header">
                <div>
                    <h3 class="task-title">${escapeHtml(task.title)}</h3>
                    ${task.description ? `<p class="task-description">${escapeHtml(task.description)}</p>` : ''}
                </div>
                <div class="task-actions">
                    <button class="btn btn-secondary btn-sm edit-btn" data-id="${task.id}">Edit</button>
                    <button class="btn btn-danger btn-sm delete-btn" data-id="${task.id}">Delete</button>
                </div>
            </div>
            <div class="task-meta">
                <span>Created: ${formatDate(task.createdAt)}</span>
                ${task.completed ? '<span class="task-status">✓ Completed</span>' : ''}
            </div>
        </div>
    `).join('');
    
    // Add event listeners to task actions
    document.querySelectorAll('.edit-btn').forEach(btn => {
        btn.addEventListener('click', handleEditClick);
    });
    
    document.querySelectorAll('.delete-btn').forEach(btn => {
        btn.addEventListener('click', handleDeleteClick);
    });
};

// Task Functions
const loadTasks = async () => {
    try {
        showLoading();
        
        const filter = {};
        if (state.filter === 'active') filter.completed = false;
        if (state.filter === 'completed') filter.completed = true;
        
        const data = await api.getTasks(filter);
        state.tasks = data.tasks;
        renderTasks();
    } catch (error) {
        showError(error.message);
    } finally {
        hideLoading();
    }
};

const createTask = async (title, description) => {
    try {
        showLoading();
        await api.createTask(title, description);
        showSuccess('Task created successfully');
        
        // Reset form
        document.getElementById('task-title').value = '';
        document.getElementById('task-description').value = '';
        
        // Reload tasks
        await loadTasks();
    } catch (error) {
        showError(error.message);
    } finally {
        hideLoading();
    }
};

const updateTask = async (id, updates) => {
    try {
        showLoading();
        await api.updateTask(id, updates);
        showSuccess('Task updated successfully');
        
        // Close modal
        elements.editModal.classList.add('hidden');
        
        // Reload tasks
        await loadTasks();
    } catch (error) {
        showError(error.message);
    } finally {
        hideLoading();
    }
};

const deleteTask = async (id) => {
    if (!confirm('Are you sure you want to delete this task?')) return;
    
    try {
        showLoading();
        await api.deleteTask(id);
        showSuccess('Task deleted successfully');
        await loadTasks();
    } catch (error) {
        showError(error.message);
    } finally {
        hideLoading();
    }
};

// Event Handlers
const handleEditClick = (e) => {
    const taskId = parseInt(e.target.dataset.id);
    const task = state.tasks.find(t => t.id === taskId);
    
    if (!task) return;
    
    // Populate edit form
    document.getElementById('edit-task-id').value = task.id;
    document.getElementById('edit-task-title').value = task.title;
    document.getElementById('edit-task-description').value = task.description || '';
    document.getElementById('edit-task-completed').checked = task.completed;
    
    // Show modal
    elements.editModal.classList.remove('hidden');
};

const handleDeleteClick = (e) => {
    const taskId = parseInt(e.target.dataset.id);
    deleteTask(taskId);
};

// Initialize Application
document.addEventListener('DOMContentLoaded', () => {
    // Tab switching
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const tab = e.target.dataset.tab;
            
            // Update active tab
            document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
            e.target.classList.add('active');
            
            // Show corresponding form
            if (tab === 'login') {
                elements.loginForm.classList.remove('hidden');
                elements.registerForm.classList.add('hidden');
            } else {
                elements.loginForm.classList.add('hidden');
                elements.registerForm.classList.remove('hidden');
            }
        });
    });
    
    // Login form submission
    elements.loginForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        if (state.loading) return;
        
        const formData = new FormData(e.target);
        const email = formData.get('email');
        const password = formData.get('password');
        
        try {
            showLoading();
            const data = await api.login(email, password);
            state.user = data.user;
            showSuccess('Login successful');
            showTasksSection();
            e.target.reset();
        } catch (error) {
            showError(error.message);
        } finally {
            hideLoading();
        }
    });
    
    // Register form submission
    elements.registerForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        if (state.loading) return;
        
        const formData = new FormData(e.target);
        const email = formData.get('email');
        const password = formData.get('password');
        
        try {
            showLoading();
            const data = await api.register(email, password);
            state.user = data.user;
            showSuccess('Registration successful');
            showTasksSection();
            e.target.reset();
        } catch (error) {
            showError(error.message);
        } finally {
            hideLoading();
        }
    });
    
    // Logout button
    elements.logoutBtn.addEventListener('click', logout);
    
    // Add task form submission
    elements.addTaskForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        if (state.loading) return;
        
        const title = document.getElementById('task-title').value;
        const description = document.getElementById('task-description').value;
        
        await createTask(title, description);
    });
    
    // Edit task form submission
    elements.editTaskForm.addEventListener('submit', async (e) => {
        e.preventDefault();
        if (state.loading) return;
        
        const id = parseInt(document.getElementById('edit-task-id').value);
        const updates = {
            title: document.getElementById('edit-task-title').value.trim(),
            description: document.getElementById('edit-task-description').value.trim() || null,
            completed: document.getElementById('edit-task-completed').checked
        };
        
        await updateTask(id, updates);
    });
    
    // Filter buttons
    document.querySelectorAll('.filter-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            // Update active filter
            document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
            e.target.classList.add('active');
            
            // Update filter and reload
            state.filter = e.target.dataset.filter;
            if (state.user) {
                loadTasks();
            }
        });
    });
    
    // Error close button
    elements.errorClose.addEventListener('click', () => {
        elements.errorContainer.classList.add('hidden');
    });
    
    // Modal close buttons
    document.querySelectorAll('.modal-close, .modal-cancel').forEach(btn => {
        btn.addEventListener('click', () => {
            elements.editModal.classList.add('hidden');
        });
    });
    
    // Close modal on background click
    elements.editModal.addEventListener('click', (e) => {
        if (e.target === elements.editModal) {
            elements.editModal.classList.add('hidden');
        }
    });
    
    // Escape key to close modal
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !elements.editModal.classList.contains('hidden')) {
            elements.editModal.classList.add('hidden');
        }
    });
    
    // Check authentication on load
    checkAuth();
});
```

### 5. Update Express to Serve Static Files

Update `src/app.js` to serve the public directory:

```javascript
// Add after body parsing middleware, before routes
app.use(express.static('public'));

// Optionally add a catch-all route to serve index.html for SPA
app.get('*', (req, res) => {
    // Only serve index.html for non-API routes
    if (!req.path.startsWith('/api') && !req.path.startsWith('/auth')) {
        res.sendFile(path.join(__dirname, '../public/index.html'));
    } else {
        // Let the 404 handler catch it
        next();
    }
});
```

## Verification Steps

### 1. Start the Server
```bash
npm run dev
```

### 2. Open Browser
Navigate to http://localhost:3000

### 3. Test Authentication Flow
1. **Register a new user**:
   - Click "Register" tab
   - Enter email and password (8+ characters)
   - Submit form
   - Verify success message and redirect to tasks

2. **Login with existing user**:
   - Refresh page (should stay logged in)
   - Click Logout
   - Login with same credentials
   - Verify success

3. **Test validation**:
   - Try short password (<8 chars)
   - Try invalid email format
   - Verify error messages

### 4. Test Task Management
1. **Create tasks**:
   - Add title only
   - Add title and description
   - Verify tasks appear in list

2. **Filter tasks**:
   - Click "All", "Active", "Completed"
   - Verify filtering works

3. **Edit tasks**:
   - Click Edit on a task
   - Change title/description
   - Mark as completed
   - Save changes

4. **Delete tasks**:
   - Click Delete
   - Confirm deletion
   - Verify task removed

### 5. Test Error Handling
1. **Network errors**:
   - Stop server
   - Try to create task
   - Verify error message

2. **Token expiration**:
   - Clear localStorage
   - Try to access tasks
   - Verify redirect to login

### 6. Test Responsive Design
1. **Mobile view**:
   - Resize browser to mobile width
   - Test all functionality
   - Verify layout adapts

2. **Tablet view**:
   - Test medium screen sizes
   - Verify layout remains usable

### 7. Test XSS Protection
1. **Try injecting HTML**:
   - Create task with title: `<script>alert('XSS')</script>`
   - Verify HTML is escaped

## Success Criteria

- Authentication forms work (login/register)
- JWT tokens stored and used correctly
- Tasks can be created, viewed, edited, deleted
- Filtering works correctly
- Error messages display properly
- Success messages show for actions
- Loading states show during requests
- Responsive design works on all devices
- XSS attacks are prevented
- Logout clears session
- Page refresh maintains session

## Important Notes

- No frameworks used - pure vanilla JavaScript
- All user input is escaped to prevent XSS
- Tokens stored in localStorage (consider security implications)
- Error handling provides user-friendly messages
- Loading states prevent duplicate submissions
- Modal can be closed with Escape key
- Responsive design uses flexbox

You have now successfully created a functional frontend interface. Users can register, login, and manage their tasks through a web browser, demonstrating all API functionality.