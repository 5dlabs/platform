# Task 8: File and Image Sharing Implementation - Autonomous AI Agent Prompt

You are tasked with implementing a comprehensive file and image sharing system for a chat application. This feature will enable users to upload, share, preview, and download various file types within chat messages.

## Primary Objectives

1. **Set up a secure file storage solution** using AWS S3 or an alternative cloud storage service
2. **Create a robust file upload API** with proper validation, security scanning, and error handling
3. **Extend the database schema** to support file attachments in messages
4. **Implement frontend components** for file upload with drag-and-drop support
5. **Build file preview components** with support for images, documents, and other file types

## Technical Requirements

### Storage Solution
- Configure AWS S3 bucket with appropriate permissions and CORS settings
- Implement secure pre-signed URLs for file access
- Set up CDN distribution for optimized file delivery
- Configure lifecycle policies for file retention
- Implement backup and disaster recovery strategies

### File Upload API
- Create POST /api/upload endpoint accepting multipart form data
- Implement file type validation (images, PDFs, documents, archives)
- Enforce file size limits (10MB default, configurable)
- Add virus scanning integration for uploaded files
- Generate unique file identifiers using nanoid or UUID
- Create image thumbnails for preview optimization
- Support chunked uploads for large files
- Implement upload progress tracking

### Database Schema
- Extend messages table with attachments JSONB column
- Create dedicated attachments table with relationships
- Store file metadata: URL, type, size, original name
- Track upload timestamps and user information
- Implement proper indexing for performance

### Frontend Components
- Build FileUpload component with react-dropzone
- Support multiple file selection and batch uploads
- Display upload progress with visual indicators
- Show file type icons and size information
- Implement drag-and-drop with visual feedback
- Add file removal before upload
- Handle upload errors gracefully

### File Preview System
- Create ImagePreview component with lightbox functionality
- Implement zoom controls for detailed image viewing
- Build DocumentPreview for PDFs and office documents
- Add download functionality for all file types
- Support inline preview for common formats
- Implement lazy loading for performance

## Security Requirements

1. **Input Validation**
   - Validate file extensions against whitelist
   - Check actual file type using magic bytes
   - Prevent path traversal attacks
   - Sanitize file names

2. **Access Control**
   - Implement proper authentication for uploads
   - Use pre-signed URLs with expiration
   - Restrict file access based on chat permissions
   - Log all file access for auditing

3. **Content Security**
   - Scan files for malware/viruses
   - Implement Content Security Policy headers
   - Prevent XSS through file uploads
   - Block executable file types

4. **Data Protection**
   - Encrypt files at rest in storage
   - Use HTTPS for all file transfers
   - Implement secure deletion procedures
   - Follow GDPR compliance for file retention

## Performance Optimizations

1. **Image Optimization**
   - Generate multiple thumbnail sizes
   - Implement progressive image loading
   - Use WebP format where supported
   - Compress images without quality loss

2. **Upload Performance**
   - Support parallel chunk uploads
   - Implement resume capability for failed uploads
   - Use multipart uploads for large files
   - Add client-side compression option

3. **Caching Strategy**
   - Configure CDN caching headers
   - Implement browser caching policies
   - Use ETags for cache validation
   - Cache thumbnail versions

## Implementation Steps

1. **Storage Setup**
   ```bash
   # Install AWS SDK
   npm install @aws-sdk/client-s3 @aws-sdk/lib-storage
   
   # Configure environment variables
   AWS_REGION=us-east-1
   AWS_ACCESS_KEY_ID=your-access-key
   AWS_SECRET_ACCESS_KEY=your-secret-key
   S3_BUCKET_NAME=chat-app-files
   ```

2. **Database Migration**
   ```bash
   # Update Prisma schema
   npx prisma migrate dev --name add-attachments
   ```

3. **API Implementation**
   - Create upload middleware with multer
   - Implement file validation service
   - Add S3 upload service
   - Create thumbnail generation service

4. **Frontend Integration**
   - Install react-dropzone and dependencies
   - Create reusable upload components
   - Integrate with existing message system
   - Add file preview capabilities

## Testing Requirements

1. **Unit Tests**
   - File validation logic
   - Upload service methods
   - Thumbnail generation
   - Security checks

2. **Integration Tests**
   - Complete upload flow
   - File preview rendering
   - Error handling scenarios
   - Permission checks

3. **Performance Tests**
   - Large file uploads
   - Concurrent uploads
   - Thumbnail generation speed
   - CDN delivery times

4. **Security Tests**
   - Malicious file upload attempts
   - Path traversal attacks
   - File type spoofing
   - Access control violations

## Success Criteria

- Users can successfully upload files up to 10MB
- All common file types are supported (images, PDFs, documents)
- Files are securely stored and accessible only to authorized users
- Image previews load quickly with thumbnail optimization
- Upload progress is clearly visible to users
- Error messages are helpful and actionable
- The system handles network interruptions gracefully
- All security vulnerabilities are addressed

## Code Quality Standards

- Follow TypeScript best practices with strict typing
- Implement comprehensive error handling
- Add JSDoc comments for public APIs
- Create reusable, modular components
- Follow the existing project structure
- Maintain test coverage above 80%

## Dependencies

This task depends on:
- Task 4: Real-time messaging system (for message integration)
- Task 7: User authentication (for access control)

Begin by analyzing the current codebase structure, then implement the file sharing system following the security and performance guidelines provided.