# Toolman Guide: File and Image Sharing Implementation

This guide provides step-by-step instructions for using Toolman AI agents to implement the file and image sharing system efficiently and securely.

## Prerequisites

Before starting, ensure you have:
- Access to AWS account or alternative storage provider
- Node.js environment set up
- Database with user authentication (Task 7) and messaging (Task 4) implemented
- Toolman CLI installed and configured

## Phase 1: Storage Infrastructure Setup

### Step 1: Configure Cloud Storage

```bash
# Using Toolman to set up AWS S3
toolman execute "Create S3 bucket configuration for file uploads"

# Toolman will:
# 1. Create S3 bucket with proper naming
# 2. Configure CORS policy for browser uploads
# 3. Set up IAM roles and policies
# 4. Generate access keys securely
```

Expected Toolman actions:
- Create `config/aws.ts` with S3 client configuration
- Set up `.env` variables for AWS credentials
- Configure bucket policies for secure access

### Step 2: Install Dependencies

```bash
# Let Toolman handle package installation
toolman execute "Install file upload dependencies including AWS SDK, multer, and image processing libraries"
```

Toolman will install:
- `@aws-sdk/client-s3` - AWS S3 client
- `@aws-sdk/lib-storage` - Upload utilities
- `multer` - Multipart form parsing
- `sharp` - Image processing
- `file-type` - File type detection
- `react-dropzone` - Drag-and-drop UI

## Phase 2: Database Schema Extension

### Step 3: Update Database Schema

```bash
# Toolman creates and runs migrations
toolman execute "Extend database schema to support file attachments in messages"
```

Toolman will:
1. Update Prisma schema with Attachment model
2. Add attachments JSONB column to messages
3. Create proper indexes for performance
4. Run migrations safely

### Step 4: Generate Database Client

```bash
toolman execute "Generate updated Prisma client with attachment support"
```

## Phase 3: Backend Implementation

### Step 5: Create Upload Service

```bash
# Toolman implements core upload logic
toolman execute "Create file upload service with S3 integration and thumbnail generation"
```

Toolman creates:
- `services/uploadService.ts` - Main upload logic
- `services/storageService.ts` - S3 operations
- `services/thumbnailService.ts` - Image processing
- `services/fileValidator.ts` - Security validation

### Step 6: Implement Upload API

```bash
toolman execute "Create secure file upload API endpoint with validation middleware"
```

Toolman will:
1. Create `/api/upload` endpoint
2. Add multipart form handling
3. Implement file type validation
4. Add size limit enforcement
5. Include progress tracking

### Step 7: Security Middleware

```bash
# Critical security implementation
toolman execute "Implement comprehensive file upload security including type validation, virus scanning, and path sanitization"
```

Security measures Toolman implements:
- Magic byte validation
- File extension whitelist
- Path traversal prevention
- File name sanitization
- Optional virus scanning integration
- Rate limiting for uploads

## Phase 4: Frontend Components

### Step 8: Create Upload Component

```bash
toolman execute "Build React file upload component with drag-and-drop support and progress tracking"
```

Toolman creates:
- `components/FileUpload.tsx` - Main upload UI
- `hooks/useFileUpload.ts` - Upload logic hook
- `utils/fileHelpers.ts` - Utility functions
- Styling with Tailwind classes

### Step 9: Preview Components

```bash
toolman execute "Create file preview components for images and documents with lightbox functionality"
```

Components created:
- `components/ImagePreview.tsx` - Image viewer with zoom
- `components/DocumentPreview.tsx` - Document display
- `components/FileLightbox.tsx` - Full-screen preview
- Download functionality integration

### Step 10: Message Integration

```bash
toolman execute "Integrate file attachments with existing message system"
```

Toolman will:
1. Update message sending to include attachments
2. Modify message display components
3. Handle mixed content (text + files)
4. Ensure real-time updates work

## Phase 5: Performance Optimization

### Step 11: Implement Lazy Loading

```bash
toolman execute "Add progressive image loading and lazy loading for better performance"
```

Optimizations include:
- Thumbnail preloading
- Progressive image enhancement
- Intersection Observer for lazy loading
- Image format optimization (WebP support)

### Step 12: Chunked Upload Support

```bash
# For large files
toolman execute "Implement chunked upload for files larger than 5MB"
```

Features implemented:
- File chunking logic
- Parallel chunk uploads
- Resume capability
- Progress aggregation

## Phase 6: Testing Implementation

### Step 13: Unit Tests

```bash
toolman execute "Create comprehensive unit tests for file validation and upload services"
```

Test coverage includes:
- File type validation
- Size limit enforcement
- Security checks
- Service method testing

### Step 14: Integration Tests

```bash
toolman execute "Write integration tests for complete upload flow"
```

Tests created:
- Upload API endpoint tests
- Database integration tests
- S3 mock testing
- Error scenario handling

### Step 15: Security Testing

```bash
toolman execute "Implement security tests for file upload vulnerabilities"
```

Security tests:
- Path traversal attempts
- File type spoofing
- XSS injection tests
- Access control verification

## Phase 7: Documentation

### Step 16: API Documentation

```bash
toolman execute "Generate OpenAPI documentation for file upload endpoints"
```

### Step 17: Component Documentation

```bash
toolman execute "Create Storybook stories for file upload components"
```

## Common Issues and Solutions

### Issue 1: CORS Errors
```bash
# Fix CORS configuration
toolman debug "Fix CORS errors for S3 uploads"
```

Toolman will:
- Update S3 bucket CORS policy
- Configure proper headers
- Set allowed origins

### Issue 2: Large File Timeouts
```bash
toolman fix "Implement timeout handling for large file uploads"
```

Solutions applied:
- Increase timeout limits
- Implement chunked uploads
- Add retry logic

### Issue 3: Memory Issues with Images
```bash
toolman optimize "Reduce memory usage for image processing"
```

Optimizations:
- Stream processing for large images
- Garbage collection improvements
- Worker thread usage

## Performance Monitoring

### Setup Monitoring
```bash
toolman configure "Set up performance monitoring for file uploads"
```

Metrics tracked:
- Upload duration
- File sizes
- Success/failure rates
- Storage usage
- Bandwidth consumption

## Security Best Practices

### Regular Security Audits
```bash
# Run security audit
toolman audit "Perform security audit on file upload system"
```

### Update Dependencies
```bash
# Check for vulnerabilities
toolman secure "Update dependencies and fix vulnerabilities"
```

## Deployment Checklist

Before deploying to production:

1. **Environment Variables**
   ```bash
   toolman validate "Check all required environment variables are set"
   ```

2. **Storage Configuration**
   ```bash
   toolman verify "Verify S3 bucket configuration and permissions"
   ```

3. **Security Headers**
   ```bash
   toolman test "Test Content Security Policy headers"
   ```

4. **Performance Testing**
   ```bash
   toolman benchmark "Run upload performance benchmarks"
   ```

5. **Backup Strategy**
   ```bash
   toolman configure "Set up file backup and recovery procedures"
   ```

## Maintenance Tasks

### Monthly Tasks
- Review storage usage and costs
- Audit file access logs
- Update virus definitions
- Clean up orphaned files

### Quarterly Tasks
- Security vulnerability scan
- Performance optimization review
- Dependency updates
- Disaster recovery test

## Extending the System

### Add Video Support
```bash
toolman extend "Add video file support with streaming playback"
```

### Implement Compression
```bash
toolman enhance "Add automatic file compression for storage optimization"
```

### Add OCR for Documents
```bash
toolman feature "Implement OCR for searchable document content"
```

## Troubleshooting Commands

```bash
# Debug upload failures
toolman debug "Investigate file upload failures in last 24 hours"

# Check storage usage
toolman analyze "Report on current storage usage and file statistics"

# Test file access
toolman test "Verify file access permissions and pre-signed URLs"

# Monitor performance
toolman monitor "Show real-time upload performance metrics"
```

## Conclusion

This guide provides a comprehensive approach to implementing file and image sharing using Toolman. The AI agent handles the complex implementation details while ensuring security and performance best practices are followed.

Remember to:
- Always validate file inputs
- Monitor storage costs
- Keep dependencies updated
- Test thoroughly before production deployment
- Document any custom modifications

For additional support, consult the Task Master documentation or run:
```bash
toolman help "file upload implementation"
```