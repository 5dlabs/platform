# Autonomous Agent Prompt: File and Image Sharing Implementation

You are tasked with implementing comprehensive file and image sharing functionality in the chat application, including upload, storage, preview, and download capabilities.

## Objective
Build a secure file sharing system that supports multiple file types, provides image previews with lightbox functionality, handles large files efficiently, and integrates seamlessly with the existing chat interface.

## Detailed Requirements

### 1. Backend File Storage Setup
Configure file storage with:
- AWS S3 or similar cloud storage
- Environment variables for credentials
- Bucket configuration with proper permissions
- File size limits (10MB)
- Allowed file types validation
- Secure URL generation

### 2. File Upload API
Create upload endpoint:
- `POST /api/upload` with multipart form data
- Support multiple files (max 5)
- File type validation (images, PDFs, documents)
- Virus scanning (optional but recommended)
- Generate unique file IDs
- Return file metadata and URLs

### 3. Image Processing
Implement image optimization:
- Generate thumbnails (200x200)
- Optimize large images (max 1920x1080)
- Maintain aspect ratios
- Support formats: JPEG, PNG, GIF, WebP
- Progressive loading for large images

### 4. Database Schema Updates
Extend message system:
```sql
-- Attachments table
CREATE TABLE attachments (
    id UUID PRIMARY KEY,
    message_id UUID REFERENCES messages(id),
    file_id UUID NOT NULL,
    url VARCHAR(500) NOT NULL,
    thumbnail_url VARCHAR(500),
    name VARCHAR(255) NOT NULL,
    size INTEGER NOT NULL,
    type VARCHAR(100) NOT NULL
);
```

### 5. Frontend Upload Component
Build drag-and-drop uploader:
- Dropzone area with visual feedback
- File type filtering
- Size validation
- Progress indicators
- Multiple file support
- Preview before upload
- Remove files before upload

### 6. File Preview Components
Create preview system:
- Image grid for multiple images
- Lightbox for full-size viewing
- Navigation between images
- Download buttons
- File type icons
- Size display

### 7. Lightbox Implementation
Build image viewer:
- Full-screen overlay
- Keyboard navigation (arrows, ESC)
- Touch gestures for mobile
- Zoom functionality (optional)
- Image info display
- Smooth transitions

### 8. Message Integration
Update message components:
- Display attachments below text
- Thumbnail grid for images
- File list for documents
- Click handlers for preview/download
- Loading states

### 9. Security Measures
Implement protections:
- File type validation (client & server)
- Size limits enforcement
- Malware scanning (if possible)
- Access control (room membership)
- Presigned URLs for downloads
- CORS configuration

### 10. Performance Optimization
Optimize for speed:
- Lazy load images
- Progressive image loading
- Cache thumbnails
- Compress uploads
- CDN integration
- Parallel uploads

## Expected Deliverables

1. Upload controller with S3 integration
2. File validation middleware
3. Image processing service
4. Database migrations
5. FileUpload component
6. ImageLightbox component
7. MessageAttachments component
8. File preview utilities
9. Updated message input
10. Security middleware

## Technical Stack

### Backend
- Multer for file handling
- Sharp for image processing
- AWS SDK for S3
- File-type for validation

### Frontend
- React Dropzone
- Portal for lightbox
- Canvas for image preview

## API Endpoints

```typescript
// Upload files
POST /api/upload
Body: FormData with files[]
Response: { files: UploadedFile[] }

// Get download URL
GET /api/download/:fileId
Response: { url: string } // Presigned URL

// Delete file
DELETE /api/files/:fileId
Response: { success: boolean }
```

## File Type Support

Images:
- JPEG/JPG
- PNG
- GIF
- WebP

Documents:
- PDF
- DOC/DOCX
- TXT

## Error Handling

Handle these cases:
- File too large
- Invalid file type
- Upload failure
- Network interruption
- Storage quota exceeded
- Access denied

## Upload Flow

1. User selects/drops files
2. Client validates type/size
3. Show upload UI with progress
4. Upload to server
5. Server validates again
6. Process images (thumbnails)
7. Store in S3
8. Save metadata to DB
9. Return URLs to client
10. Display in message

## Testing Requirements

Test scenarios:
1. Upload single image
2. Upload multiple files
3. Drag and drop
4. Large file rejection
5. Invalid type rejection
6. Progress indication
7. Cancel upload
8. View in lightbox
9. Download file
10. Mobile upload

## Performance Targets

- Upload speed: Limited by connection
- Thumbnail generation: < 2 seconds
- Lightbox open: < 100ms
- Image optimization: 50-70% size reduction
- Concurrent uploads: Up to 5 files

## Security Checklist

- [ ] File type validation (MIME and extension)
- [ ] Size limits enforced
- [ ] Malware scanning configured
- [ ] Access control implemented
- [ ] Presigned URLs expire
- [ ] No directory traversal
- [ ] Input sanitization
- [ ] CORS properly configured

Begin by setting up the backend storage and upload endpoint, then build the frontend upload components, followed by preview and integration features.