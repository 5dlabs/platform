# Task 8: File and Image Sharing Implementation - Toolman Usage Guide

## Overview
This guide explains how to use the selected Toolman tools to implement file and image sharing functionality. The tools focus on creating upload services, storage integration, and frontend components for handling file uploads and previews.

## Core Tools

### 1. brave_web_search
**Purpose**: Research file upload patterns and cloud storage best practices
**When to use**: 
- Before implementing storage architecture
- When choosing between S3, Cloudinary, or other solutions
- For security best practices
- To find optimal image processing techniques

**How to use**:
```json
{
  "tool": "brave_web_search",
  "query": "AWS S3 file upload Node.js best practices 2024",
  "freshness": "year"
}
```

**Key research topics**:
- "S3 presigned URLs vs direct upload patterns"
- "Image optimization Sharp.js best practices"
- "React drag and drop file upload libraries comparison"
- "Multer S3 configuration security"
- "File upload progress tracking WebSocket vs polling"

### 2. create_directory
**Purpose**: Organize file handling code structure
**When to use**:
- Setting up service directories
- Creating upload component folders
- Organizing storage utilities

**How to use**:
```json
{
  "tool": "create_directory",
  "path": "/chat-application/backend/src/services/storage"
}
```

**Directory structure**:
```
/backend/src/
├── services/
│   ├── storage/
│   │   ├── storageService.ts
│   │   ├── imageProcessor.ts
│   │   └── fileValidator.ts
│   └── upload/
│       └── uploadService.ts
├── controllers/
│   └── fileController.ts
└── middleware/
    └── upload.ts

/frontend/src/components/
├── FileUpload/
│   ├── index.tsx
│   ├── DropZone.tsx
│   └── UploadProgress.tsx
└── FilePreview/
    ├── ImagePreview.tsx
    ├── FilePreview.tsx
    └── PreviewModal.tsx
```

### 3. write_file
**Purpose**: Create file handling implementation files
**When to use**:
- Writing storage service classes
- Creating upload controllers
- Implementing preview components
- Setting up configuration files

**How to use**:
```json
{
  "tool": "write_file",
  "path": "/chat-application/backend/src/services/storage/storageService.ts",
  "content": "// S3 storage service implementation"
}
```

### 4. edit_file
**Purpose**: Update existing files with file sharing integration
**When to use**:
- Adding upload routes to API
- Updating message schema
- Modifying frontend to include uploads
- Adding new dependencies

**How to use**:
```json
{
  "tool": "edit_file",
  "path": "/chat-application/backend/src/routes/api.ts",
  "old_string": "// API routes",
  "new_string": "// API routes\nrouter.use('/upload', uploadRoutes);"
}
```

### 5. read_file
**Purpose**: Review existing code before modifications
**When to use**:
- Before updating message components
- To check current API structure
- To understand storage configuration
- Before modifying database schema

## Implementation Flow

### Phase 1: Research Storage Solutions (15 minutes)
1. **S3 patterns**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "S3 vs Cloudinary vs self-hosted file storage comparison"
   }
   ```

2. **Security practices**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Secure file upload validation Node.js OWASP"
   }
   ```

3. **Performance optimization**:
   ```json
   {
     "tool": "brave_web_search",
     "query": "Image optimization Sharp.js vs ImageMagick performance"
   }
   ```

### Phase 2: Backend Storage Setup (25 minutes)
1. **Create storage structure**:
   ```json
   {
     "tool": "create_directory",
     "path": "/chat-application/backend/src/services/storage"
   }
   ```

2. **Write storage service**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/services/storage/storageService.ts",
     "content": "// AWS S3 integration with signed URLs"
   }
   ```

3. **Create upload controller**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/controllers/fileController.ts",
     "content": "// File upload endpoint with validation"
   }
   ```

### Phase 3: Image Processing (15 minutes)
1. **Image processor service**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/backend/src/services/storage/imageProcessor.ts",
     "content": "// Sharp.js image processing pipeline"
   }
   ```

2. **Update package.json**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/package.json",
     "old_string": "\"dependencies\": {",
     "new_string": "\"dependencies\": {\n    \"aws-sdk\": \"^2.1000.0\",\n    \"multer\": \"^1.4.5\",\n    \"multer-s3\": \"^3.0.0\",\n    \"sharp\": \"^0.31.0\","
   }
   ```

### Phase 4: Frontend Upload Components (20 minutes)
1. **Create upload component**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/components/FileUpload/index.tsx",
     "content": "// Drag and drop file upload component"
   }
   ```

2. **Preview components**:
   ```json
   {
     "tool": "write_file",
     "path": "/chat-application/frontend/src/components/FilePreview/ImagePreview.tsx",
     "content": "// Image preview with lightbox"
   }
   ```

### Phase 5: Integration (15 minutes)
1. **Update message schema**:
   ```json
   {
     "tool": "read_file",
     "path": "/chat-application/backend/src/models/Message.js"
   }
   ```

2. **Add attachments field**:
   ```json
   {
     "tool": "edit_file",
     "path": "/chat-application/backend/src/models/Message.js",
     "old_string": "content: String,",
     "new_string": "content: String,\n  attachments: [{\n    id: String,\n    url: String,\n    thumbnailUrl: String,\n    name: String,\n    type: String,\n    size: Number\n  }],"
   }
   ```

## Best Practices

### Storage Configuration
```typescript
// Use environment variables
const s3Config = {
  accessKeyId: process.env.AWS_ACCESS_KEY_ID,
  secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
  region: process.env.AWS_REGION,
};

// Private bucket with signed URLs
const generateSignedUrl = (key: string) => {
  return s3.getSignedUrlPromise('getObject', {
    Bucket: bucketName,
    Key: key,
    Expires: 3600, // 1 hour
  });
};
```

### File Validation
```typescript
// Comprehensive validation
const validateFile = (file: Express.Multer.File) => {
  // Check MIME type
  const allowedMimes = ['image/jpeg', 'image/png', 'application/pdf'];
  if (!allowedMimes.includes(file.mimetype)) {
    throw new Error('Invalid file type');
  }
  
  // Check file size
  if (file.size > 10 * 1024 * 1024) { // 10MB
    throw new Error('File too large');
  }
  
  // Check file extension
  const ext = path.extname(file.originalname).toLowerCase();
  const allowedExts = ['.jpg', '.jpeg', '.png', '.pdf'];
  if (!allowedExts.includes(ext)) {
    throw new Error('Invalid file extension');
  }
};
```

### Image Processing
```typescript
// Optimize images before storage
const processImage = async (input: Buffer) => {
  const image = sharp(input);
  const metadata = await image.metadata();
  
  // Resize if too large
  if (metadata.width > 2000 || metadata.height > 2000) {
    image.resize(2000, 2000, {
      fit: 'inside',
      withoutEnlargement: true,
    });
  }
  
  // Convert to JPEG for consistency
  return image
    .jpeg({ quality: 85, progressive: true })
    .toBuffer();
};
```

## Common Patterns

### Research → Design → Implement
```javascript
// 1. Research storage options
const options = await brave_web_search("S3 vs Azure Blob vs Google Cloud Storage");

// 2. Design based on findings
const storageDesign = planStorageArchitecture(options);

// 3. Implement
await write_file("services/storageService.ts", storageImplementation);
```

### Progressive Enhancement
```javascript
// 1. Basic upload
await write_file("FileUpload.tsx", basicUploadComponent);

// 2. Add drag and drop
await edit_file("FileUpload.tsx",
  "return (",
  "const { getRootProps, getInputProps } = useDropzone();\n\n  return ("
);

// 3. Add progress tracking
await edit_file("FileUpload.tsx",
  "const [files, setFiles]",
  "const [files, setFiles] = useState([]);\n  const [progress, setProgress] = useState({});"
);
```

## Security Patterns

### Input Validation
```typescript
// Never trust client-side validation
const secureUpload = async (req: Request) => {
  // Re-validate on server
  const files = req.files as Express.Multer.File[];
  
  for (const file of files) {
    // Check magic numbers
    const buffer = file.buffer;
    const magic = buffer.toString('hex', 0, 4);
    
    if (file.mimetype === 'image/jpeg' && magic !== 'ffd8ffe0') {
      throw new Error('File content does not match MIME type');
    }
  }
};
```

### Access Control
```typescript
// Validate file access
const canAccessFile = async (fileId: string, userId: string) => {
  const file = await fileRepository.findById(fileId);
  
  // Check if user uploaded the file
  if (file.userId === userId) return true;
  
  // Check if file is in a room user has access to
  const messageWithFile = await messageRepository.findByFileId(fileId);
  if (messageWithFile) {
    return roomUserRepository.isUserInRoom(messageWithFile.roomId, userId);
  }
  
  return false;
};
```

## Troubleshooting

### Issue: Upload fails with CORS error
**Solution**: Configure S3 CORS policy, check API CORS middleware

### Issue: Large files timeout
**Solution**: Implement chunked upload, increase timeout limits

### Issue: Images not displaying
**Solution**: Check signed URL expiration, verify content-type headers

### Issue: Memory issues with Sharp
**Solution**: Process images in streams, limit concurrent processing

## Performance Optimization

### Upload Optimization
```typescript
// Stream large files
const uploadStream = () => {
  const pass = new PassThrough();
  
  const params = {
    Bucket: bucketName,
    Key: key,
    Body: pass,
  };
  
  s3.upload(params, (err, data) => {
    if (err) console.error(err);
  });
  
  return pass;
};
```

### Caching Strategy
```typescript
// Cache signed URLs
const urlCache = new Map();

const getCachedSignedUrl = async (key: string) => {
  const cached = urlCache.get(key);
  if (cached && cached.expires > Date.now()) {
    return cached.url;
  }
  
  const url = await generateSignedUrl(key);
  urlCache.set(key, {
    url,
    expires: Date.now() + 3000000, // 50 minutes
  });
  
  return url;
};
```

## Task Completion Checklist
- [ ] Storage service configured (S3/alternative)
- [ ] Upload API endpoint working
- [ ] File validation comprehensive
- [ ] Image processing functional
- [ ] Frontend upload component smooth
- [ ] Preview components polished
- [ ] Progress tracking accurate
- [ ] Security measures in place
- [ ] Performance optimized
- [ ] Error handling complete

This systematic approach ensures secure, efficient file sharing functionality that enhances the chat experience with rich media support.