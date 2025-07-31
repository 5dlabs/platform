# Task 8: File and Image Sharing Implementation

## Overview

This task implements a comprehensive file and image sharing system for the chat application, enabling users to upload, share, preview, and download various file types within chat messages. The implementation includes secure file storage, upload APIs, database schema extensions, and rich frontend components for file handling.

## Technical Implementation Guide

### 1. File Storage Architecture

#### Storage Options

**Option A: AWS S3 (Recommended)**
```typescript
// config/aws.ts
import { S3Client } from '@aws-sdk/client-s3';

export const s3Client = new S3Client({
  region: process.env.AWS_REGION || 'us-east-1',
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!
  }
});

export const S3_BUCKET_NAME = process.env.S3_BUCKET_NAME || 'chat-app-files';
```

**Option B: Local Storage with CDN**
```typescript
// config/storage.ts
import multer from 'multer';
import path from 'path';

export const UPLOAD_PATH = process.env.UPLOAD_PATH || './uploads';
export const CDN_URL = process.env.CDN_URL || 'https://cdn.example.com';

const storage = multer.diskStorage({
  destination: (req, file, cb) => {
    cb(null, UPLOAD_PATH);
  },
  filename: (req, file, cb) => {
    const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
    cb(null, file.fieldname + '-' + uniqueSuffix + path.extname(file.originalname));
  }
});
```

### 2. Upload API Implementation

#### File Upload Endpoint
```typescript
// api/upload.ts
import { Request, Response } from 'express';
import { Upload } from '@aws-sdk/lib-storage';
import { nanoid } from 'nanoid';
import sharp from 'sharp';
import { scanFile } from '../services/virusScanner';

interface UploadedFile {
  id: string;
  url: string;
  thumbnailUrl?: string;
  name: string;
  type: string;
  size: number;
  uploadedAt: Date;
}

export async function uploadFiles(req: Request, res: Response) {
  try {
    const files = req.files as Express.Multer.File[];
    const uploadedFiles: UploadedFile[] = [];
    
    for (const file of files) {
      // Validate file type
      if (!isAllowedFileType(file.mimetype)) {
        return res.status(400).json({ 
          error: `File type ${file.mimetype} not allowed` 
        });
      }
      
      // Validate file size (10MB limit)
      if (file.size > 10 * 1024 * 1024) {
        return res.status(400).json({ 
          error: 'File size exceeds 10MB limit' 
        });
      }
      
      // Optional: Virus scanning
      if (process.env.ENABLE_VIRUS_SCAN === 'true') {
        const scanResult = await scanFile(file.buffer);
        if (!scanResult.clean) {
          return res.status(400).json({ 
            error: 'File failed security scan' 
          });
        }
      }
      
      // Generate unique file key
      const fileKey = `${nanoid()}/${file.originalname}`;
      
      // Upload to S3
      const upload = new Upload({
        client: s3Client,
        params: {
          Bucket: S3_BUCKET_NAME,
          Key: fileKey,
          Body: file.buffer,
          ContentType: file.mimetype,
          Metadata: {
            originalName: file.originalname,
            uploadedBy: req.user.id
          }
        }
      });
      
      const result = await upload.done();
      
      // Generate thumbnail for images
      let thumbnailUrl;
      if (file.mimetype.startsWith('image/')) {
        thumbnailUrl = await generateThumbnail(file.buffer, fileKey);
      }
      
      uploadedFiles.push({
        id: nanoid(),
        url: result.Location!,
        thumbnailUrl,
        name: file.originalname,
        type: file.mimetype,
        size: file.size,
        uploadedAt: new Date()
      });
    }
    
    res.json({ files: uploadedFiles });
  } catch (error) {
    console.error('Upload error:', error);
    res.status(500).json({ error: 'Upload failed' });
  }
}

function isAllowedFileType(mimetype: string): boolean {
  const allowedTypes = [
    'image/jpeg',
    'image/png',
    'image/gif',
    'image/webp',
    'application/pdf',
    'application/msword',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'text/plain',
    'application/zip'
  ];
  
  return allowedTypes.includes(mimetype);
}

async function generateThumbnail(buffer: Buffer, fileKey: string): Promise<string> {
  const thumbnailKey = `thumbnails/${fileKey}`;
  
  const thumbnail = await sharp(buffer)
    .resize(200, 200, {
      fit: 'inside',
      withoutEnlargement: true
    })
    .jpeg({ quality: 80 })
    .toBuffer();
  
  const upload = new Upload({
    client: s3Client,
    params: {
      Bucket: S3_BUCKET_NAME,
      Key: thumbnailKey,
      Body: thumbnail,
      ContentType: 'image/jpeg'
    }
  });
  
  const result = await upload.done();
  return result.Location!;
}
```

### 3. Database Schema Extensions

#### PostgreSQL Schema
```sql
-- Add attachments column to messages table
ALTER TABLE messages 
ADD COLUMN attachments JSONB DEFAULT '[]'::jsonb;

-- Create attachments table for detailed tracking
CREATE TABLE attachments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
  file_id VARCHAR(255) NOT NULL,
  file_url TEXT NOT NULL,
  thumbnail_url TEXT,
  file_name VARCHAR(255) NOT NULL,
  file_type VARCHAR(100) NOT NULL,
  file_size INTEGER NOT NULL,
  uploaded_by UUID REFERENCES users(id),
  uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  metadata JSONB DEFAULT '{}'::jsonb
);

-- Index for faster queries
CREATE INDEX idx_attachments_message_id ON attachments(message_id);
CREATE INDEX idx_attachments_uploaded_by ON attachments(uploaded_by);
```

#### Prisma Schema Update
```prisma
model Message {
  id          String       @id @default(uuid())
  content     String
  attachments Json[]       @default([])
  // ... other fields
  
  Attachment  Attachment[]
}

model Attachment {
  id           String   @id @default(uuid())
  messageId    String   @map("message_id")
  fileId       String   @map("file_id")
  fileUrl      String   @map("file_url")
  thumbnailUrl String?  @map("thumbnail_url")
  fileName     String   @map("file_name")
  fileType     String   @map("file_type")
  fileSize     Int      @map("file_size")
  uploadedBy   String   @map("uploaded_by")
  uploadedAt   DateTime @default(now()) @map("uploaded_at")
  metadata     Json     @default("{}")
  
  message      Message  @relation(fields: [messageId], references: [id], onDelete: Cascade)
  user         User     @relation(fields: [uploadedBy], references: [id])
  
  @@index([messageId])
  @@index([uploadedBy])
  @@map("attachments")
}
```

### 4. Frontend Upload Components

#### Enhanced File Upload Component
```typescript
// components/FileUpload.tsx
import React, { useState, useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { Upload, X, File, Image } from 'lucide-react';
import { uploadFiles } from '../api/fileApi';

interface FileUploadProps {
  onFilesUploaded: (files: UploadedFile[]) => void;
  maxFiles?: number;
  maxSize?: number;
}

export const FileUpload: React.FC<FileUploadProps> = ({ 
  onFilesUploaded, 
  maxFiles = 5,
  maxSize = 10 * 1024 * 1024 
}) => {
  const [files, setFiles] = useState<File[]>([]);
  const [uploading, setUploading] = useState(false);
  const [progress, setProgress] = useState<{ [key: string]: number }>({});
  const [errors, setErrors] = useState<string[]>([]);
  
  const onDrop = useCallback((acceptedFiles: File[], rejectedFiles: any[]) => {
    setErrors([]);
    
    // Handle rejected files
    if (rejectedFiles.length > 0) {
      const errorMessages = rejectedFiles.map(rejection => {
        const error = rejection.errors[0];
        return `${rejection.file.name}: ${error.message}`;
      });
      setErrors(errorMessages);
    }
    
    // Add accepted files
    setFiles(prev => {
      const newFiles = [...prev, ...acceptedFiles];
      return newFiles.slice(0, maxFiles);
    });
  }, [maxFiles]);
  
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'image/*': ['.png', '.jpg', '.jpeg', '.gif', '.webp'],
      'application/pdf': ['.pdf'],
      'application/msword': ['.doc'],
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document': ['.docx'],
      'text/plain': ['.txt'],
      'application/zip': ['.zip']
    },
    maxSize,
    maxFiles: maxFiles - files.length,
    disabled: uploading
  });
  
  const removeFile = (index: number) => {
    setFiles(files.filter((_, i) => i !== index));
  };
  
  const uploadAllFiles = async () => {
    if (files.length === 0) return;
    
    setUploading(true);
    setProgress({});
    
    try {
      const uploadedFiles = await uploadFiles(files, {
        onProgress: (fileName: string, percent: number) => {
          setProgress(prev => ({ ...prev, [fileName]: percent }));
        }
      });
      
      onFilesUploaded(uploadedFiles);
      setFiles([]);
      setErrors([]);
    } catch (error) {
      setErrors(['Upload failed. Please try again.']);
    } finally {
      setUploading(false);
      setProgress({});
    }
  };
  
  const getFileIcon = (file: File) => {
    if (file.type.startsWith('image/')) {
      return <Image className="w-4 h-4" />;
    }
    return <File className="w-4 h-4" />;
  };
  
  return (
    <div className="file-upload-container">
      <div
        {...getRootProps()}
        className={`
          border-2 border-dashed rounded-lg p-6 text-center cursor-pointer
          transition-colors duration-200
          ${isDragActive ? 'border-blue-500 bg-blue-50' : 'border-gray-300 hover:border-gray-400'}
          ${uploading ? 'opacity-50 cursor-not-allowed' : ''}
        `}
      >
        <input {...getInputProps()} />
        <Upload className="w-8 h-8 mx-auto mb-2 text-gray-400" />
        {isDragActive ? (
          <p className="text-blue-600">Drop the files here...</p>
        ) : (
          <div>
            <p className="text-gray-600">Drag & drop files here, or click to select</p>
            <p className="text-sm text-gray-400 mt-1">
              Max {maxFiles} files, up to {maxSize / (1024 * 1024)}MB each
            </p>
          </div>
        )}
      </div>
      
      {errors.length > 0 && (
        <div className="mt-2 space-y-1">
          {errors.map((error, index) => (
            <p key={index} className="text-sm text-red-600">{error}</p>
          ))}
        </div>
      )}
      
      {files.length > 0 && (
        <div className="mt-4 space-y-2">
          {files.map((file, index) => (
            <div key={index} className="flex items-center justify-between p-2 bg-gray-50 rounded">
              <div className="flex items-center space-x-2">
                {getFileIcon(file)}
                <span className="text-sm truncate max-w-xs">{file.name}</span>
                <span className="text-xs text-gray-500">
                  ({(file.size / 1024).toFixed(1)} KB)
                </span>
              </div>
              <div className="flex items-center space-x-2">
                {progress[file.name] !== undefined && (
                  <div className="w-24 bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                      style={{ width: `${progress[file.name]}%` }}
                    />
                  </div>
                )}
                {!uploading && (
                  <button
                    onClick={() => removeFile(index)}
                    className="text-red-600 hover:text-red-800"
                  >
                    <X className="w-4 h-4" />
                  </button>
                )}
              </div>
            </div>
          ))}
          
          <button
            onClick={uploadAllFiles}
            disabled={uploading}
            className="mt-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
          >
            {uploading ? 'Uploading...' : `Upload ${files.length} file(s)`}
          </button>
        </div>
      )}
    </div>
  );
};
```

### 5. File Preview Components

#### Image Preview with Lightbox
```typescript
// components/ImagePreview.tsx
import React, { useState } from 'react';
import { X, Download, ZoomIn, ZoomOut } from 'lucide-react';

interface ImagePreviewProps {
  src: string;
  thumbnailSrc?: string;
  alt: string;
  fileName: string;
}

export const ImagePreview: React.FC<ImagePreviewProps> = ({
  src,
  thumbnailSrc,
  alt,
  fileName
}) => {
  const [isLightboxOpen, setIsLightboxOpen] = useState(false);
  const [zoom, setZoom] = useState(1);
  
  const handleDownload = async () => {
    try {
      const response = await fetch(src);
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (error) {
      console.error('Download failed:', error);
    }
  };
  
  return (
    <>
      <div className="inline-block cursor-pointer" onClick={() => setIsLightboxOpen(true)}>
        <img
          src={thumbnailSrc || src}
          alt={alt}
          className="max-w-xs max-h-48 rounded shadow hover:shadow-lg transition-shadow"
        />
      </div>
      
      {isLightboxOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-90">
          <div className="relative max-w-4xl max-h-screen p-4">
            <div className="absolute top-4 right-4 flex space-x-2">
              <button
                onClick={() => setZoom(z => Math.max(0.5, z - 0.25))}
                className="p-2 bg-white rounded-full shadow hover:bg-gray-100"
              >
                <ZoomOut className="w-5 h-5" />
              </button>
              <button
                onClick={() => setZoom(z => Math.min(3, z + 0.25))}
                className="p-2 bg-white rounded-full shadow hover:bg-gray-100"
              >
                <ZoomIn className="w-5 h-5" />
              </button>
              <button
                onClick={handleDownload}
                className="p-2 bg-white rounded-full shadow hover:bg-gray-100"
              >
                <Download className="w-5 h-5" />
              </button>
              <button
                onClick={() => setIsLightboxOpen(false)}
                className="p-2 bg-white rounded-full shadow hover:bg-gray-100"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            
            <img
              src={src}
              alt={alt}
              style={{ transform: `scale(${zoom})` }}
              className="max-w-full max-h-full object-contain transition-transform duration-200"
            />
          </div>
        </div>
      )}
    </>
  );
};
```

#### Document Preview Component
```typescript
// components/DocumentPreview.tsx
import React from 'react';
import { FileText, Download } from 'lucide-react';

interface DocumentPreviewProps {
  url: string;
  fileName: string;
  fileType: string;
  fileSize: number;
}

export const DocumentPreview: React.FC<DocumentPreviewProps> = ({
  url,
  fileName,
  fileType,
  fileSize
}) => {
  const getFileIcon = () => {
    if (fileType.includes('pdf')) return '📄';
    if (fileType.includes('word')) return '📝';
    if (fileType.includes('zip')) return '🗜️';
    return '📎';
  };
  
  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };
  
  const handleDownload = () => {
    window.open(url, '_blank');
  };
  
  return (
    <div className="flex items-center p-3 bg-gray-100 rounded-lg space-x-3">
      <span className="text-2xl">{getFileIcon()}</span>
      <div className="flex-1">
        <p className="font-medium text-sm truncate">{fileName}</p>
        <p className="text-xs text-gray-500">{formatFileSize(fileSize)}</p>
      </div>
      <button
        onClick={handleDownload}
        className="p-2 text-blue-600 hover:bg-blue-50 rounded"
      >
        <Download className="w-4 h-4" />
      </button>
    </div>
  );
};
```

### 6. Security Considerations

#### Content Security Policy
```typescript
// middleware/security.ts
export const securityHeaders = {
  'Content-Security-Policy': [
    "default-src 'self'",
    "img-src 'self' https://your-s3-bucket.s3.amazonaws.com data:",
    "script-src 'self' 'unsafe-inline' 'unsafe-eval'",
    "style-src 'self' 'unsafe-inline'",
    "connect-src 'self' https://your-api.com",
    "frame-ancestors 'none'",
    "form-action 'self'"
  ].join('; '),
  'X-Content-Type-Options': 'nosniff',
  'X-Frame-Options': 'DENY',
  'X-XSS-Protection': '1; mode=block'
};
```

#### File Validation Service
```typescript
// services/fileValidator.ts
import FileType from 'file-type';
import isSvg from 'is-svg';

export class FileValidator {
  private static readonly ALLOWED_MIME_TYPES = new Set([
    'image/jpeg',
    'image/png',
    'image/gif',
    'image/webp',
    'application/pdf',
    'application/msword',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'text/plain',
    'application/zip'
  ]);
  
  private static readonly BLOCKED_EXTENSIONS = new Set([
    '.exe', '.bat', '.cmd', '.com', '.pif', '.scr',
    '.vbs', '.js', '.jar', '.app', '.dmg'
  ]);
  
  static async validateFile(buffer: Buffer, fileName: string): Promise<{
    valid: boolean;
    error?: string;
  }> {
    // Check file extension
    const ext = fileName.substring(fileName.lastIndexOf('.')).toLowerCase();
    if (this.BLOCKED_EXTENSIONS.has(ext)) {
      return { valid: false, error: 'File type not allowed' };
    }
    
    // Detect actual file type from buffer
    const fileTypeResult = await FileType.fromBuffer(buffer);
    
    if (!fileTypeResult) {
      // Check if it's SVG
      if (isSvg(buffer.toString())) {
        return { valid: true };
      }
      
      // Check if it's plain text
      if (this.isPlainText(buffer)) {
        return { valid: true };
      }
      
      return { valid: false, error: 'Unable to determine file type' };
    }
    
    // Validate MIME type
    if (!this.ALLOWED_MIME_TYPES.has(fileTypeResult.mime)) {
      return { valid: false, error: `MIME type ${fileTypeResult.mime} not allowed` };
    }
    
    return { valid: true };
  }
  
  private static isPlainText(buffer: Buffer): boolean {
    const sample = buffer.toString('utf8', 0, Math.min(1000, buffer.length));
    return /^[\x20-\x7E\s]*$/.test(sample);
  }
}
```

### 7. Performance Optimizations

#### Progressive Image Loading
```typescript
// hooks/useProgressiveImage.ts
import { useState, useEffect } from 'react';

export const useProgressiveImage = (lowQualitySrc: string, highQualitySrc: string) => {
  const [src, setSrc] = useState(lowQualitySrc);
  
  useEffect(() => {
    const img = new Image();
    img.src = highQualitySrc;
    img.onload = () => {
      setSrc(highQualitySrc);
    };
  }, [highQualitySrc]);
  
  return src;
};
```

#### Chunked Upload for Large Files
```typescript
// services/chunkedUpload.ts
export class ChunkedUploadService {
  private chunkSize = 5 * 1024 * 1024; // 5MB chunks
  
  async uploadFile(
    file: File,
    onProgress?: (percent: number) => void
  ): Promise<string> {
    const chunks = Math.ceil(file.size / this.chunkSize);
    const uploadId = await this.initiateUpload(file.name, file.type);
    const parts: any[] = [];
    
    for (let i = 0; i < chunks; i++) {
      const start = i * this.chunkSize;
      const end = Math.min(start + this.chunkSize, file.size);
      const chunk = file.slice(start, end);
      
      const part = await this.uploadChunk(uploadId, i + 1, chunk);
      parts.push(part);
      
      if (onProgress) {
        onProgress(((i + 1) / chunks) * 100);
      }
    }
    
    return await this.completeUpload(uploadId, parts);
  }
  
  private async initiateUpload(fileName: string, fileType: string): Promise<string> {
    const response = await fetch('/api/upload/initiate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ fileName, fileType })
    });
    
    const { uploadId } = await response.json();
    return uploadId;
  }
  
  private async uploadChunk(
    uploadId: string,
    partNumber: number,
    chunk: Blob
  ): Promise<any> {
    const formData = new FormData();
    formData.append('chunk', chunk);
    formData.append('uploadId', uploadId);
    formData.append('partNumber', partNumber.toString());
    
    const response = await fetch('/api/upload/chunk', {
      method: 'POST',
      body: formData
    });
    
    return response.json();
  }
  
  private async completeUpload(uploadId: string, parts: any[]): Promise<string> {
    const response = await fetch('/api/upload/complete', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ uploadId, parts })
    });
    
    const { url } = await response.json();
    return url;
  }
}
```

## Integration with Message System

```typescript
// components/MessageWithAttachments.tsx
import React from 'react';
import { ImagePreview } from './ImagePreview';
import { DocumentPreview } from './DocumentPreview';

interface MessageWithAttachmentsProps {
  content: string;
  attachments: Attachment[];
}

export const MessageWithAttachments: React.FC<MessageWithAttachmentsProps> = ({
  content,
  attachments
}) => {
  const renderAttachment = (attachment: Attachment) => {
    if (attachment.fileType.startsWith('image/')) {
      return (
        <ImagePreview
          key={attachment.id}
          src={attachment.fileUrl}
          thumbnailSrc={attachment.thumbnailUrl}
          alt={attachment.fileName}
          fileName={attachment.fileName}
        />
      );
    }
    
    return (
      <DocumentPreview
        key={attachment.id}
        url={attachment.fileUrl}
        fileName={attachment.fileName}
        fileType={attachment.fileType}
        fileSize={attachment.fileSize}
      />
    );
  };
  
  return (
    <div className="message">
      {content && <p className="mb-2">{content}</p>}
      {attachments.length > 0 && (
        <div className="attachments space-y-2">
          {attachments.map(renderAttachment)}
        </div>
      )}
    </div>
  );
};
```

## Testing Considerations

- Test file upload with various file types and sizes
- Verify file type validation and security checks
- Test error handling for network failures
- Verify thumbnail generation for images
- Test file preview components across different browsers
- Measure upload performance for large files
- Test concurrent uploads
- Verify proper cleanup of failed uploads

## Next Steps

After implementing the file sharing system:
1. Add support for file compression
2. Implement file expiration policies
3. Add support for video files with streaming
4. Implement file sharing permissions
5. Add analytics for file usage
6. Consider implementing a file virus scanning service