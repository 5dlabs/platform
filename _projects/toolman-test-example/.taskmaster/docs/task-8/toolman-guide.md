# Toolman Guide for Task 8: File and Image Sharing Implementation

## Overview

This guide provides comprehensive instructions for using the selected tools to implement Task 8, which focuses on building file and image sharing capabilities within the chat application, including cloud storage integration, upload APIs, preview components, and download functionality.

## Core Tools

### 1. **create_directory** (Local - filesystem)
**Purpose**: Create file handling directory structure for uploads and storage

**When to Use**: 
- At the beginning to organize file handling components
- When creating upload and preview components
- For organizing file utilities and services

**How to Use**:
```
# Create file handling structure
create_directory /chat-application/backend/src/upload
create_directory /chat-application/backend/src/upload/controllers
create_directory /chat-application/backend/src/upload/middleware
create_directory /chat-application/backend/src/upload/services
create_directory /chat-application/backend/src/upload/validators
create_directory /chat-application/frontend/src/components/files
create_directory /chat-application/frontend/src/components/files/upload
create_directory /chat-application/frontend/src/components/files/preview
```

**Parameters**:
- `path`: Directory path to create

### 2. **write_file** (Local - filesystem)
**Purpose**: Create file upload handlers, storage services, and preview components

**When to Use**: 
- To create upload API endpoints
- To implement storage service integration
- To create file preview components
- To implement validation utilities

**How to Use**:
```
# Create upload controller
write_file /chat-application/backend/src/upload/controllers/uploadController.ts <controller-content>

# Create storage service
write_file /chat-application/backend/src/upload/services/storageService.ts <storage-service>

# Create file upload component
write_file /chat-application/frontend/src/components/files/upload/FileUpload.tsx <upload-component>

# Create image preview component
write_file /chat-application/frontend/src/components/files/preview/ImagePreview.tsx <preview-component>

# Create file validator
write_file /chat-application/backend/src/upload/validators/fileValidator.ts <validator-content>
```

**Parameters**:
- `path`: File path to write
- `content`: Complete file content

### 3. **read_file** (Local - filesystem)
**Purpose**: Review existing message schema and API structure

**When to Use**: 
- To check message model for extension
- To review existing API patterns
- To understand current file handling

**How to Use**:
```
# Read message model
read_file /chat-application/backend/src/database/models/Message.ts

# Check API routes
read_file /chat-application/backend/src/api/routes/messageRoutes.ts

# Review environment config
read_file /chat-application/backend/.env.example
```

**Parameters**:
- `path`: File to read
- `head`/`tail`: Optional line limits

### 4. **edit_file** (Local - filesystem)
**Purpose**: Update existing files to integrate file sharing

**When to Use**: 
- To extend message schema with attachments
- To add file upload dependencies
- To update API routes
- To modify environment variables

**How to Use**:
```
# Add file upload dependencies
edit_file /chat-application/backend/package.json
# Add: multer, @types/multer, aws-sdk (or cloud storage SDK)

# Update message schema
edit_file /chat-application/backend/src/database/schemas/chat_schema.sql
# Add attachments column to messages table

# Add frontend dependencies
edit_file /chat-application/frontend/package.json
# Add: react-dropzone, file-type validation libraries

# Update environment variables
edit_file /chat-application/backend/.env.example
# Add cloud storage credentials
```

**Parameters**:
- `old_string`: Exact text to replace
- `new_string`: New text
- `path`: File to edit

### 5. **list_directory** (Local - filesystem)
**Purpose**: Verify file handling structure creation

**When to Use**: 
- After creating upload directories
- To confirm component organization
- Before testing implementation

**How to Use**:
```
# Verify upload structure
list_directory /chat-application/backend/src/upload

# Check file components
list_directory /chat-application/frontend/src/components/files
```

**Parameters**:
- `path`: Directory to list

## Implementation Flow

1. **Directory Structure Phase**
   - Use `create_directory` to build file handling structure
   - Organize backend upload logic
   - Create frontend file components

2. **Database Schema Update Phase**
   - Use `edit_file` to modify message schema
   - Add attachments field with JSON structure:
     ```json
     {
       "files": [
         {
           "id": "uuid",
           "name": "filename.pdf",
           "url": "https://storage.url/file",
           "type": "application/pdf",
           "size": 1024000
         }
       ]
     }
     ```

3. **Storage Service Implementation**
   - Use `write_file` to create storage service
   - Implement cloud storage integration (S3/GCS/Azure)
   - Add local development fallback
   - Configure secure URLs with expiration

4. **Upload API Implementation**
   - Create upload endpoint with multer
   - Implement file validation:
     - Type restrictions (images, PDFs, docs)
     - Size limits (10MB default)
     - Optional virus scanning
   - Return file metadata for message attachment

5. **Frontend Upload Components**
   - Implement FileUpload.tsx with react-dropzone
   - Add drag-and-drop support
   - Show upload progress
   - Handle multiple files
   - Display file list with removal

6. **Preview Components Implementation**
   - Create ImagePreview.tsx with lightbox
   - Implement DocumentPreview.tsx
   - Add download functionality
   - Handle different file types appropriately

## Best Practices

1. **Security**: Validate file types on both client and server
2. **Storage**: Use signed URLs for secure access
3. **Performance**: Implement thumbnail generation for images
4. **UX**: Show clear upload progress and errors
5. **Limits**: Enforce reasonable file size limits
6. **Cleanup**: Implement file deletion when messages are deleted

## Task-Specific Implementation Details

### Upload Controller Pattern
```typescript
// uploadController.ts
import multer from 'multer';
import { storageService } from '../services/storageService';

const upload = multer({
  limits: { fileSize: 10 * 1024 * 1024 }, // 10MB
  fileFilter: (req, file, cb) => {
    const allowedTypes = ['image/jpeg', 'image/png', 'application/pdf'];
    if (allowedTypes.includes(file.mimetype)) {
      cb(null, true);
    } else {
      cb(new Error('Invalid file type'));
    }
  }
});

export const uploadFile = [
  upload.single('file'),
  async (req, res) => {
    try {
      const file = req.file;
      const userId = req.userId;

      // Upload to cloud storage
      const uploadResult = await storageService.upload(file, userId);

      res.json({
        id: uploadResult.id,
        name: file.originalname,
        url: uploadResult.url,
        type: file.mimetype,
        size: file.size
      });
    } catch (error) {
      res.status(500).json({ error: error.message });
    }
  }
];
```

### React Dropzone Pattern
```typescript
// FileUpload.tsx
import { useDropzone } from 'react-dropzone';

export const FileUpload: React.FC<{ onUpload: (files: File[]) => void }> = ({ onUpload }) => {
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    accept: {
      'image/*': ['.png', '.jpg', '.jpeg'],
      'application/pdf': ['.pdf']
    },
    maxSize: 10485760, // 10MB
    onDrop: onUpload
  });

  return (
    <div
      {...getRootProps()}
      className={`border-2 border-dashed p-4 rounded ${
        isDragActive ? 'border-blue-500 bg-blue-50' : 'border-gray-300'
      }`}
    >
      <input {...getInputProps()} />
      <p className="text-center">
        {isDragActive ? 'Drop files here' : 'Drag files or click to upload'}
      </p>
    </div>
  );
};
```

### Storage Service Pattern
```typescript
// storageService.ts
import AWS from 'aws-sdk';

const s3 = new AWS.S3({
  accessKeyId: process.env.AWS_ACCESS_KEY,
  secretAccessKey: process.env.AWS_SECRET_KEY,
  region: process.env.AWS_REGION
});

export const storageService = {
  upload: async (file: Express.Multer.File, userId: string) => {
    const key = `uploads/${userId}/${Date.now()}-${file.originalname}`;
    
    const params = {
      Bucket: process.env.S3_BUCKET!,
      Key: key,
      Body: file.buffer,
      ContentType: file.mimetype
    };

    const result = await s3.upload(params).promise();
    
    return {
      id: key,
      url: result.Location
    };
  },

  getSignedUrl: async (key: string) => {
    return s3.getSignedUrl('getObject', {
      Bucket: process.env.S3_BUCKET!,
      Key: key,
      Expires: 3600 // 1 hour
    });
  }
};
```

### Image Preview Pattern
```typescript
// ImagePreview.tsx
export const ImagePreview: React.FC<{ attachment: Attachment }> = ({ attachment }) => {
  const [showLightbox, setShowLightbox] = useState(false);

  return (
    <>
      <div 
        className="cursor-pointer max-w-xs"
        onClick={() => setShowLightbox(true)}
      >
        <img 
          src={attachment.url} 
          alt={attachment.name}
          className="rounded shadow-md hover:shadow-lg transition-shadow"
        />
      </div>

      {showLightbox && (
        <Lightbox
          src={attachment.url}
          alt={attachment.name}
          onClose={() => setShowLightbox(false)}
        />
      )}
    </>
  );
};
```

## Troubleshooting

- **Upload Failures**: Check file size and type restrictions
- **CORS Issues**: Configure storage bucket CORS policy
- **Preview Errors**: Handle missing or expired URLs
- **Performance**: Implement chunked uploads for large files
- **Storage Costs**: Implement file cleanup policies

## Testing Approach

1. **Unit Tests**:
   - Test file validation logic
   - Test storage service methods
   - Test component rendering

2. **Integration Tests**:
   - Test complete upload flow
   - Test file retrieval
   - Test error scenarios

3. **Performance Tests**:
   - Test large file uploads
   - Test concurrent uploads
   - Measure upload speeds