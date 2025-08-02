# Task 8: File and Image Sharing Implementation - AI Agent Prompt

You are a senior full-stack developer tasked with implementing comprehensive file and image sharing functionality for a chat application. Your implementation must handle secure file uploads, efficient storage, preview capabilities, and seamless integration with the messaging system.

## Primary Objectives

1. **Storage Infrastructure**: Set up cloud storage solution (AWS S3 or compatible) with proper security and CDN integration.

2. **Upload API**: Create secure file upload endpoints with validation, virus scanning, and progress tracking.

3. **Database Schema**: Extend the message schema to support file attachments with proper metadata storage.

4. **Frontend Upload**: Build drag-and-drop file upload UI with progress indicators and file management.

5. **Preview Components**: Implement file preview functionality with download capabilities and image lightbox.

## Required Actions

### Phase 1: Storage Setup (15 minutes)
1. Configure storage service:
   ```bash
   npm install aws-sdk multer multer-s3
   npm install sharp uuid
   npm install -D @types/multer @types/sharp
   ```

2. Set up environment variables:
   ```
   AWS_ACCESS_KEY_ID=
   AWS_SECRET_ACCESS_KEY=
   AWS_REGION=
   S3_BUCKET_NAME=
   CDN_URL=
   ```

3. Create S3 bucket with:
   - Private access (use signed URLs)
   - CORS configuration
   - Lifecycle policies
   - CDN distribution

### Phase 2: Backend Implementation (25 minutes)
1. **Storage Service**:
   - S3 client configuration
   - File upload methods
   - Signed URL generation
   - File deletion
   - Image processing

2. **Upload Controller**:
   - Multer middleware setup
   - File validation
   - Size limits
   - MIME type checking
   - Virus scanning (optional)

3. **Database Updates**:
   ```sql
   -- Files table
   CREATE TABLE files (
     id UUID PRIMARY KEY,
     user_id UUID REFERENCES users(id),
     key VARCHAR(500),
     original_name VARCHAR(255),
     mime_type VARCHAR(100),
     size BIGINT,
     created_at TIMESTAMP
   );
   
   -- Message attachments
   ALTER TABLE messages 
   ADD COLUMN attachments JSONB DEFAULT '[]';
   ```

### Phase 3: File Processing (20 minutes)
1. **Image Optimization**:
   - Resize large images
   - Generate thumbnails
   - Convert to efficient formats
   - Progressive loading

2. **File Validation**:
   - Content-based type detection
   - File size limits
   - Allowed types whitelist
   - Malware scanning

3. **Metadata Extraction**:
   - Image dimensions
   - File properties
   - EXIF data handling
   - Duration for videos

### Phase 4: Frontend Upload UI (20 minutes)
1. **Upload Component**:
   ```typescript
   - Drag and drop zone
   - File selection dialog
   - Multiple file support
   - Progress tracking
   - Cancel capability
   ```

2. **File Management**:
   - Preview before upload
   - Remove files
   - Retry failed uploads
   - Queue management

3. **Integration**:
   - Attach to messages
   - Show in message input
   - Clear after send

### Phase 5: Preview Components (15 minutes)
1. **Image Preview**:
   - Thumbnail display
   - Lightbox modal
   - Zoom controls
   - Download button
   - Swipe navigation

2. **File Preview**:
   - Icon by file type
   - File info display
   - Download link
   - Open in new tab

3. **Message Integration**:
   - Show attachments
   - Multiple files
   - Mixed content

### Phase 6: Testing & Security (5 minutes)
1. Test scenarios:
   - Large file uploads
   - Multiple files
   - Network interruptions
   - Invalid files
   - Access control

2. Security measures:
   - Signed URLs
   - Access validation
   - Rate limiting
   - Input sanitization

## Implementation Details

### S3 Configuration
```javascript
const s3Config = {
  accessKeyId: process.env.AWS_ACCESS_KEY_ID,
  secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  region: process.env.AWS_REGION,
  signatureVersion: 'v4',
};

// Bucket policy for private access
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Deny",
    "Principal": "*",
    "Action": "s3:GetObject",
    "Resource": "arn:aws:s3:::bucket-name/*",
    "Condition": {
      "StringNotEquals": {
        "s3:x-amz-server-side-encryption": "AES256"
      }
    }
  }]
}
```

### File Upload Flow
```typescript
// 1. Client selects files
// 2. Validate on frontend
// 3. Upload to server
// 4. Server validates again
// 5. Process images
// 6. Upload to S3
// 7. Save metadata
// 8. Return URLs
// 9. Attach to message
```

### Image Processing Pipeline
```typescript
const processImage = async (file: Express.Multer.File) => {
  // 1. Read image
  const image = sharp(file.buffer);
  
  // 2. Get metadata
  const metadata = await image.metadata();
  
  // 3. Resize if needed
  if (metadata.width > 2000 || metadata.height > 2000) {
    image.resize(2000, 2000, { fit: 'inside' });
  }
  
  // 4. Optimize
  image.jpeg({ quality: 85, progressive: true });
  
  // 5. Generate thumbnail
  const thumbnail = sharp(file.buffer)
    .resize(200, 200, { fit: 'cover' })
    .jpeg({ quality: 70 });
    
  return { image, thumbnail };
};
```

## Security Requirements

### File Validation
- [ ] Whitelist allowed MIME types
- [ ] Verify file content matches type
- [ ] Enforce size limits
- [ ] Scan for malware
- [ ] Strip EXIF data from images

### Access Control
- [ ] Validate user permissions
- [ ] Check room membership
- [ ] Use signed URLs
- [ ] Set URL expiration
- [ ] Log access attempts

### Upload Security
- [ ] Rate limit uploads
- [ ] Validate file names
- [ ] Sanitize metadata
- [ ] Prevent path traversal
- [ ] Check disk space

## Error Handling

### Upload Errors
```typescript
const uploadErrorHandler = (error: any) => {
  if (error.code === 'LIMIT_FILE_SIZE') {
    return { message: 'File too large', code: 'FILE_TOO_LARGE' };
  }
  if (error.code === 'LIMIT_FILE_COUNT') {
    return { message: 'Too many files', code: 'TOO_MANY_FILES' };
  }
  if (error.code === 'LIMIT_UNEXPECTED_FILE') {
    return { message: 'Unexpected field', code: 'INVALID_FIELD' };
  }
  // ... more error cases
};
```

### Network Errors
- Retry with exponential backoff
- Resume partial uploads
- Queue failed uploads
- Show clear error messages

## UI/UX Requirements

### Upload Experience
- Drag anywhere to upload
- Clear progress indication
- Cancel/pause capability
- Batch operations
- Keyboard shortcuts

### File Display
- Appropriate icons
- File size formatting
- Upload timestamps
- Download progress
- Preview quality

### Mobile Considerations
- Touch-friendly targets
- Camera integration
- Gallery access
- Reduced quality option
- Offline queue

## Performance Optimization

### Upload Optimization
```typescript
// Chunked upload for large files
const uploadChunked = async (file: File) => {
  const chunkSize = 5 * 1024 * 1024; // 5MB chunks
  const chunks = Math.ceil(file.size / chunkSize);
  
  for (let i = 0; i < chunks; i++) {
    const start = i * chunkSize;
    const end = Math.min(start + chunkSize, file.size);
    const chunk = file.slice(start, end);
    
    await uploadChunk(chunk, i, chunks);
  }
};
```

### Image Loading
```typescript
// Progressive image loading
<img
  src={thumbnailUrl}
  loading="lazy"
  onLoad={(e) => {
    // Load full image
    e.target.src = fullImageUrl;
  }}
/>
```

## Testing Requirements

### Upload Tests
```typescript
describe('File Upload', () => {
  test('accepts valid file types');
  test('rejects invalid types');
  test('enforces size limits');
  test('handles multiple files');
  test('tracks progress accurately');
  test('resumes interrupted uploads');
});
```

### Integration Tests
- Upload and attach to message
- Download uploaded files
- Preview different file types
- Delete files and cleanup
- Handle expired URLs

## Monitoring

### Metrics to Track
- Upload success rate
- Average upload time
- Storage usage
- Bandwidth consumption
- Error frequency
- File type distribution

### Alerts
- Storage quota warnings
- Upload failures spike
- Malware detection
- Bandwidth exceeded
- S3 errors

## Documentation

### API Documentation
```typescript
/**
 * POST /api/upload
 * Upload files to storage
 * 
 * @body {files[]: File} - Array of files (max 5)
 * @returns {files[]: {id, url, thumbnailUrl, name, size}}
 * 
 * @throws {413} - File too large
 * @throws {415} - Unsupported file type
 * @throws {429} - Rate limit exceeded
 */
```

### User Guide
- Supported file types
- Size limits
- How to upload
- Troubleshooting
- Mobile usage

## Final Deliverables

Before marking complete:
- [ ] S3 storage configured
- [ ] Upload API working
- [ ] File validation comprehensive
- [ ] Image processing functional
- [ ] Frontend upload smooth
- [ ] Preview components polished
- [ ] Download works reliably
- [ ] Security measures in place
- [ ] Performance optimized
- [ ] Documentation complete

Execute this task systematically, ensuring secure file handling, efficient storage, and seamless user experience for sharing files in chat conversations.