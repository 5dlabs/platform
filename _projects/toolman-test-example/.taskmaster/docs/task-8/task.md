# Task 8: File and Image Sharing Implementation

## Overview
Implement comprehensive file and image sharing capabilities in the chat application, including upload functionality, storage management, preview components, and download features with proper security and performance optimization.

## Technical Implementation Guide

### Phase 1: Backend File Storage Setup

#### File Upload Controller
```typescript
// backend/src/controllers/uploadController.ts
import multer from 'multer';
import { Request, Response } from 'express';
import { AuthRequest } from '../middleware/auth';
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';
import { v4 as uuidv4 } from 'uuid';
import sharp from 'sharp';
import { fileTypeFromBuffer } from 'file-type';

// S3 Client configuration
const s3Client = new S3Client({
  region: process.env.AWS_REGION || 'us-east-1',
  credentials: {
    accessKeyId: process.env.AWS_ACCESS_KEY_ID!,
    secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY!,
  },
});

// Multer configuration
const upload = multer({
  limits: {
    fileSize: 10 * 1024 * 1024, // 10MB max
    files: 5, // Max 5 files at once
  },
  fileFilter: (req, file, cb) => {
    const allowedMimeTypes = [
      'image/jpeg',
      'image/png',
      'image/gif',
      'image/webp',
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'text/plain',
    ];

    if (allowedMimeTypes.includes(file.mimetype)) {
      cb(null, true);
    } else {
      cb(new Error('Invalid file type'));
    }
  },
});

export const uploadFiles = [
  upload.array('files', 5),
  async (req: AuthRequest, res: Response) => {
    try {
      if (!req.files || !Array.isArray(req.files)) {
        return res.status(400).json({ error: 'No files provided' });
      }

      const uploadedFiles = await Promise.all(
        req.files.map(async (file) => {
          // Validate file type from buffer
          const fileType = await fileTypeFromBuffer(file.buffer);
          if (!fileType) {
            throw new Error('Invalid file content');
          }

          // Generate unique filename
          const fileId = uuidv4();
          const fileExtension = fileType.ext;
          const fileName = `${fileId}.${fileExtension}`;
          
          let processedBuffer = file.buffer;
          let thumbnailUrl = null;

          // Process images
          if (fileType.mime.startsWith('image/')) {
            // Create thumbnail for images
            const thumbnail = await sharp(file.buffer)
              .resize(200, 200, { fit: 'cover', withoutEnlargement: true })
              .toBuffer();

            const thumbnailName = `${fileId}_thumb.${fileExtension}`;
            
            // Upload thumbnail
            await s3Client.send(new PutObjectCommand({
              Bucket: process.env.S3_BUCKET!,
              Key: `thumbnails/${thumbnailName}`,
              Body: thumbnail,
              ContentType: fileType.mime,
              ACL: 'public-read',
            }));

            thumbnailUrl = `https://${process.env.S3_BUCKET}.s3.amazonaws.com/thumbnails/${thumbnailName}`;

            // Optimize original image
            if (file.size > 1024 * 1024) { // If larger than 1MB
              processedBuffer = await sharp(file.buffer)
                .resize(1920, 1080, { fit: 'inside', withoutEnlargement: true })
                .jpeg({ quality: 85 })
                .toBuffer();
            }
          }

          // Upload original/processed file
          await s3Client.send(new PutObjectCommand({
            Bucket: process.env.S3_BUCKET!,
            Key: `uploads/${fileName}`,
            Body: processedBuffer,
            ContentType: fileType.mime,
            ACL: 'public-read',
            Metadata: {
              userId: req.userId!,
              originalName: file.originalname,
            },
          }));

          const fileUrl = `https://${process.env.S3_BUCKET}.s3.amazonaws.com/uploads/${fileName}`;

          return {
            id: fileId,
            url: fileUrl,
            thumbnailUrl,
            name: file.originalname,
            size: processedBuffer.length,
            type: fileType.mime,
            uploadedBy: req.userId!,
            uploadedAt: new Date(),
          };
        })
      );

      res.json({ files: uploadedFiles });
    } catch (error: any) {
      console.error('Upload error:', error);
      res.status(500).json({ error: error.message || 'Upload failed' });
    }
  },
];

// Get presigned URL for secure downloads
export const getDownloadUrl = async (req: AuthRequest, res: Response) => {
  try {
    const { fileId } = req.params;
    
    // Verify user has access to the file
    const file = await fileRepository.findById(fileId);
    if (!file) {
      return res.status(404).json({ error: 'File not found' });
    }

    // Check if user is in the same room as the file
    const hasAccess = await messageRepository.userHasAccessToFile(req.userId!, fileId);
    if (!hasAccess) {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Generate presigned URL
    const command = new GetObjectCommand({
      Bucket: process.env.S3_BUCKET!,
      Key: `uploads/${file.fileName}`,
    });

    const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });
    
    res.json({ url });
  } catch (error) {
    console.error('Download URL error:', error);
    res.status(500).json({ error: 'Failed to generate download URL' });
  }
};
```

### Phase 2: Database Schema Updates

#### Extended Message Schema
```sql
-- Add attachments table
CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    file_id UUID NOT NULL,
    url VARCHAR(500) NOT NULL,
    thumbnail_url VARCHAR(500),
    name VARCHAR(255) NOT NULL,
    size INTEGER NOT NULL,
    type VARCHAR(100) NOT NULL,
    uploaded_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for quick lookups
CREATE INDEX idx_attachments_message_id ON attachments(message_id);
CREATE INDEX idx_attachments_file_id ON attachments(file_id);

-- Extend messages table
ALTER TABLE messages ADD COLUMN has_attachments BOOLEAN DEFAULT false;
```

### Phase 3: Frontend File Upload Component

#### Advanced File Upload with Drag & Drop
```typescript
// frontend/src/components/chat/FileUpload.tsx
import React, { useState, useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { api } from '../../services/api';
import { FilePreview } from './FilePreview';

interface FileUploadProps {
  onFilesUploaded: (files: UploadedFile[]) => void;
  onCancel: () => void;
}

interface UploadedFile {
  id: string;
  url: string;
  thumbnailUrl?: string;
  name: string;
  size: number;
  type: string;
}

export const FileUpload: React.FC<FileUploadProps> = ({
  onFilesUploaded,
  onCancel,
}) => {
  const [files, setFiles] = useState<File[]>([]);
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<Record<string, number>>({});
  const [errors, setErrors] = useState<string[]>([]);

  const onDrop = useCallback((acceptedFiles: File[], rejectedFiles: any[]) => {
    setFiles(prev => [...prev, ...acceptedFiles]);
    
    if (rejectedFiles.length > 0) {
      const errors = rejectedFiles.map(rejection => {
        const error = rejection.errors[0];
        return `${rejection.file.name}: ${error.message}`;
      });
      setErrors(prev => [...prev, ...errors]);
    }
  }, []);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'image/*': ['.png', '.jpg', '.jpeg', '.gif', '.webp'],
      'application/pdf': ['.pdf'],
      'application/msword': ['.doc'],
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document': ['.docx'],
      'text/plain': ['.txt'],
    },
    maxSize: 10 * 1024 * 1024, // 10MB
    maxFiles: 5,
  });

  const removeFile = (index: number) => {
    setFiles(files.filter((_, i) => i !== index));
  };

  const uploadFiles = async () => {
    if (files.length === 0) return;

    setUploading(true);
    setErrors([]);

    const formData = new FormData();
    files.forEach(file => formData.append('files', file));

    try {
      const response = await api.post('/api/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
        onUploadProgress: (progressEvent) => {
          if (progressEvent.total) {
            const progress = Math.round(
              (progressEvent.loaded * 100) / progressEvent.total
            );
            setUploadProgress({ overall: progress });
          }
        },
      });

      onFilesUploaded(response.data.files);
      setFiles([]);
    } catch (error: any) {
      console.error('Upload failed:', error);
      setErrors([error.response?.data?.error || 'Upload failed']);
    } finally {
      setUploading(false);
      setUploadProgress({});
    }
  };

  const totalSize = files.reduce((sum, file) => sum + file.size, 0);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg max-w-2xl w-full max-h-[80vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="p-4 border-b dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Upload Files
          </h3>
        </div>

        {/* Dropzone */}
        <div className="p-4">
          <div
            {...getRootProps()}
            className={`border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors ${
              isDragActive
                ? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20'
                : 'border-gray-300 dark:border-gray-600 hover:border-gray-400'
            }`}
          >
            <input {...getInputProps()} />
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
              />
            </svg>
            <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
              {isDragActive
                ? 'Drop files here'
                : 'Drag & drop files here, or click to select'}
            </p>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-500">
              Max 5 files, up to 10MB each
            </p>
          </div>
        </div>

        {/* File List */}
        {files.length > 0 && (
          <div className="flex-1 overflow-y-auto px-4">
            <div className="space-y-2">
              {files.map((file, index) => (
                <div
                  key={index}
                  className="flex items-center p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
                >
                  <FilePreview file={file} />
                  <div className="flex-1 ml-3">
                    <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                      {file.name}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {(file.size / 1024 / 1024).toFixed(2)} MB
                    </p>
                  </div>
                  {!uploading && (
                    <button
                      onClick={() => removeFile(index)}
                      className="ml-2 text-red-500 hover:text-red-700"
                    >
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  )}
                </div>
              ))}
            </div>
            <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
              Total: {(totalSize / 1024 / 1024).toFixed(2)} MB
            </p>
          </div>
        )}

        {/* Errors */}
        {errors.length > 0 && (
          <div className="px-4">
            {errors.map((error, index) => (
              <p key={index} className="text-sm text-red-600 dark:text-red-400">
                {error}
              </p>
            ))}
          </div>
        )}

        {/* Upload Progress */}
        {uploading && (
          <div className="px-4 pb-2">
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div
                className="bg-indigo-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${uploadProgress.overall || 0}%` }}
              />
            </div>
            <p className="text-xs text-gray-600 dark:text-gray-400 mt-1">
              Uploading... {uploadProgress.overall || 0}%
            </p>
          </div>
        )}

        {/* Actions */}
        <div className="p-4 border-t dark:border-gray-700 flex justify-end space-x-3">
          <button
            onClick={onCancel}
            disabled={uploading}
            className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={uploadFiles}
            disabled={files.length === 0 || uploading}
            className="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {uploading ? 'Uploading...' : `Upload ${files.length} file${files.length !== 1 ? 's' : ''}`}
          </button>
        </div>
      </div>
    </div>
  );
};
```

### Phase 4: File Preview Components

#### Image Lightbox and File Preview
```typescript
// frontend/src/components/chat/ImageLightbox.tsx
import React, { useState } from 'react';
import { createPortal } from 'react-dom';

interface ImageLightboxProps {
  images: Array<{
    url: string;
    name: string;
  }>;
  initialIndex: number;
  onClose: () => void;
}

export const ImageLightbox: React.FC<ImageLightboxProps> = ({
  images,
  initialIndex,
  onClose,
}) => {
  const [currentIndex, setCurrentIndex] = useState(initialIndex);

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') onClose();
    if (e.key === 'ArrowLeft') navigate(-1);
    if (e.key === 'ArrowRight') navigate(1);
  };

  React.useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    document.body.style.overflow = 'hidden';
    
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = '';
    };
  }, [currentIndex]);

  const navigate = (direction: number) => {
    const newIndex = currentIndex + direction;
    if (newIndex >= 0 && newIndex < images.length) {
      setCurrentIndex(newIndex);
    }
  };

  return createPortal(
    <div className="fixed inset-0 bg-black bg-opacity-90 z-50 flex items-center justify-center">
      {/* Close button */}
      <button
        onClick={onClose}
        className="absolute top-4 right-4 text-white hover:text-gray-300 z-10"
      >
        <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      {/* Navigation */}
      {currentIndex > 0 && (
        <button
          onClick={() => navigate(-1)}
          className="absolute left-4 top-1/2 -translate-y-1/2 text-white hover:text-gray-300"
        >
          <svg className="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
      )}

      {currentIndex < images.length - 1 && (
        <button
          onClick={() => navigate(1)}
          className="absolute right-4 top-1/2 -translate-y-1/2 text-white hover:text-gray-300"
        >
          <svg className="w-12 h-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        </button>
      )}

      {/* Image */}
      <div className="max-w-7xl max-h-screen p-4">
        <img
          src={images[currentIndex].url}
          alt={images[currentIndex].name}
          className="max-w-full max-h-full object-contain"
        />
        
        {/* Info bar */}
        <div className="absolute bottom-0 left-0 right-0 bg-black bg-opacity-50 text-white p-4">
          <p className="text-center">
            {images[currentIndex].name} ({currentIndex + 1} / {images.length})
          </p>
        </div>
      </div>
    </div>,
    document.body
  );
};
```

#### File Attachment Display
```typescript
// frontend/src/components/chat/MessageAttachments.tsx
import React, { useState } from 'react';
import { ImageLightbox } from './ImageLightbox';

interface Attachment {
  id: string;
  url: string;
  thumbnailUrl?: string;
  name: string;
  size: number;
  type: string;
}

interface MessageAttachmentsProps {
  attachments: Attachment[];
}

export const MessageAttachments: React.FC<MessageAttachmentsProps> = ({
  attachments,
}) => {
  const [lightboxOpen, setLightboxOpen] = useState(false);
  const [lightboxIndex, setLightboxIndex] = useState(0);

  const imageAttachments = attachments.filter(a => a.type.startsWith('image/'));
  const fileAttachments = attachments.filter(a => !a.type.startsWith('image/'));

  const openLightbox = (index: number) => {
    setLightboxIndex(index);
    setLightboxOpen(true);
  };

  const downloadFile = async (attachment: Attachment) => {
    // Get presigned URL for secure download
    try {
      const response = await api.get(`/api/download/${attachment.id}`);
      window.open(response.data.url, '_blank');
    } catch (error) {
      console.error('Download failed:', error);
    }
  };

  const getFileIcon = (type: string) => {
    if (type.includes('pdf')) return '📄';
    if (type.includes('word')) return '📝';
    if (type.includes('text')) return '📃';
    return '📎';
  };

  return (
    <div className="mt-2 space-y-2">
      {/* Image Grid */}
      {imageAttachments.length > 0 && (
        <div className={`grid gap-2 ${
          imageAttachments.length === 1 ? 'grid-cols-1' : 
          imageAttachments.length === 2 ? 'grid-cols-2' : 
          'grid-cols-3'
        }`}>
          {imageAttachments.map((attachment, index) => (
            <div
              key={attachment.id}
              className="relative group cursor-pointer overflow-hidden rounded-lg"
              onClick={() => openLightbox(index)}
            >
              <img
                src={attachment.thumbnailUrl || attachment.url}
                alt={attachment.name}
                className="w-full h-48 object-cover group-hover:scale-105 transition-transform"
              />
              <div className="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-30 transition-opacity flex items-center justify-center">
                <svg
                  className="w-8 h-8 text-white opacity-0 group-hover:opacity-100 transition-opacity"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7"
                  />
                </svg>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* File List */}
      {fileAttachments.length > 0 && (
        <div className="space-y-1">
          {fileAttachments.map((attachment) => (
            <div
              key={attachment.id}
              className="flex items-center p-2 bg-gray-100 dark:bg-gray-700 rounded-lg group hover:bg-gray-200 dark:hover:bg-gray-600 cursor-pointer"
              onClick={() => downloadFile(attachment)}
            >
              <span className="text-2xl mr-3">{getFileIcon(attachment.type)}</span>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 dark:text-white truncate">
                  {attachment.name}
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {(attachment.size / 1024).toFixed(1)} KB
                </p>
              </div>
              <svg
                className="w-5 h-5 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
                />
              </svg>
            </div>
          ))}
        </div>
      )}

      {/* Lightbox */}
      {lightboxOpen && (
        <ImageLightbox
          images={imageAttachments}
          initialIndex={lightboxIndex}
          onClose={() => setLightboxOpen(false)}
        />
      )}
    </div>
  );
};
```

### Phase 5: Integration with Message Input

#### Enhanced Message Input with File Support
```typescript
// Update MessageInput component
const MessageInput: React.FC<MessageInputProps> = ({ onSendMessage }) => {
  const [message, setMessage] = useState('');
  const [attachments, setAttachments] = useState<UploadedFile[]>([]);
  const [showFileUpload, setShowFileUpload] = useState(false);

  const handleSend = () => {
    if (message.trim() || attachments.length > 0) {
      onSendMessage({
        content: message,
        attachments: attachments.map(a => a.id),
      });
      setMessage('');
      setAttachments([]);
    }
  };

  const handleFilesUploaded = (files: UploadedFile[]) => {
    setAttachments(prev => [...prev, ...files]);
    setShowFileUpload(false);
  };

  return (
    <div className="border-t dark:border-gray-700 p-4">
      {/* Attachment Preview */}
      {attachments.length > 0 && (
        <div className="mb-2 flex flex-wrap gap-2">
          {attachments.map((file) => (
            <div
              key={file.id}
              className="flex items-center bg-gray-100 dark:bg-gray-700 rounded px-2 py-1"
            >
              <span className="text-sm truncate max-w-xs">{file.name}</span>
              <button
                onClick={() => setAttachments(attachments.filter(a => a.id !== file.id))}
                className="ml-2 text-red-500 hover:text-red-700"
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}

      <div className="flex items-end space-x-2">
        {/* File Upload Button */}
        <button
          onClick={() => setShowFileUpload(true)}
          className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
        >
          <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
          </svg>
        </button>

        {/* Message Input */}
        <textarea
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder="Type a message..."
          className="flex-1 resize-none rounded-lg border dark:border-gray-600 px-3 py-2"
        />

        {/* Send Button */}
        <button
          onClick={handleSend}
          disabled={!message.trim() && attachments.length === 0}
          className="p-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
          </svg>
        </button>
      </div>

      {/* File Upload Modal */}
      {showFileUpload && (
        <FileUpload
          onFilesUploaded={handleFilesUploaded}
          onCancel={() => setShowFileUpload(false)}
        />
      )}
    </div>
  );
};
```

## Success Metrics

- Files upload successfully with progress indication
- Image thumbnails generated and displayed
- Lightbox navigation smooth and responsive
- File downloads work with proper authentication
- Upload size limits enforced
- Invalid file types rejected with clear messages
- Attachments display correctly in messages
- Mobile experience optimized