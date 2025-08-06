# Task 8: File and Image Sharing Implementation

## Overview
Implement comprehensive file and image sharing capabilities for the chat application, including secure file uploads, storage management, preview functionality, and seamless integration with the messaging system. This feature enhances communication by allowing users to share documents, images, and other files within chat conversations.

## Technical Architecture

### Storage Solutions
- **Primary Storage**: AWS S3 or compatible object storage
- **CDN**: CloudFront or similar for fast delivery
- **Local Cache**: Temporary storage for uploads
- **Database**: File metadata and references
- **Security**: Signed URLs with expiration

### File Processing Pipeline
```
User Upload → Validation → Virus Scan → Storage → CDN → Message Attachment
     ↓            ↓           ↓          ↓        ↓           ↓
  Progress    Size/Type   Optional    S3/Minio  Cache    Database
```

## Implementation Details

### 1. Storage Service Configuration

```typescript
// backend/src/services/storageService.ts
import AWS from 'aws-sdk';
import multer from 'multer';
import multerS3 from 'multer-s3';
import { v4 as uuidv4 } from 'uuid';
import sharp from 'sharp';
import { promisify } from 'util';
import fs from 'fs';

const unlink = promisify(fs.unlink);

export class StorageService {
  private s3: AWS.S3;
  private bucketName: string;
  private cdnUrl: string;

  constructor() {
    this.s3 = new AWS.S3({
      accessKeyId: process.env.AWS_ACCESS_KEY_ID,
      secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
      region: process.env.AWS_REGION,
    });
    
    this.bucketName = process.env.S3_BUCKET_NAME!;
    this.cdnUrl = process.env.CDN_URL || `https://${this.bucketName}.s3.amazonaws.com`;
  }

  // Configure multer for direct S3 upload
  getMulterUpload() {
    return multer({
      storage: multerS3({
        s3: this.s3,
        bucket: this.bucketName,
        acl: 'private', // Use signed URLs for access
        contentType: multerS3.AUTO_CONTENT_TYPE,
        key: (req, file, cb) => {
          const userId = (req as any).userId;
          const fileExtension = file.originalname.split('.').pop();
          const fileName = `${userId}/${uuidv4()}.${fileExtension}`;
          cb(null, fileName);
        },
      }),
      limits: {
        fileSize: 10 * 1024 * 1024, // 10MB default
      },
      fileFilter: this.fileFilter,
    });
  }

  // File type validation
  private fileFilter = (req: any, file: Express.Multer.File, cb: any) => {
    const allowedMimes = [
      'image/jpeg',
      'image/png',
      'image/gif',
      'image/webp',
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'application/vnd.ms-excel',
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      'text/plain',
      'application/zip',
    ];

    if (allowedMimes.includes(file.mimetype)) {
      cb(null, true);
    } else {
      cb(new Error(`File type ${file.mimetype} not allowed`), false);
    }
  };

  // Generate signed URL for secure access
  async getSignedUrl(key: string, expiresIn: number = 3600): Promise<string> {
    const params = {
      Bucket: this.bucketName,
      Key: key,
      Expires: expiresIn,
    };

    return this.s3.getSignedUrlPromise('getObject', params);
  }

  // Upload file with processing
  async uploadFile(file: Express.Multer.File, userId: string): Promise<FileUploadResult> {
    const fileId = uuidv4();
    const fileExtension = file.originalname.split('.').pop()?.toLowerCase();
    const key = `${userId}/${fileId}.${fileExtension}`;

    let processedFile = file;

    // Process images
    if (file.mimetype.startsWith('image/')) {
      processedFile = await this.processImage(file);
    }

    // Upload to S3
    const uploadParams = {
      Bucket: this.bucketName,
      Key: key,
      Body: processedFile.buffer || fs.createReadStream(processedFile.path),
      ContentType: file.mimetype,
      Metadata: {
        originalName: file.originalname,
        uploadedBy: userId,
      },
    };

    const result = await this.s3.upload(uploadParams).promise();

    // Generate thumbnail for images
    let thumbnailUrl;
    if (file.mimetype.startsWith('image/')) {
      thumbnailUrl = await this.generateThumbnail(key, file);
    }

    // Clean up temp file
    if (processedFile.path && processedFile.path !== file.path) {
      await unlink(processedFile.path);
    }

    return {
      fileId,
      key,
      url: result.Location,
      signedUrl: await this.getSignedUrl(key),
      thumbnailUrl,
      originalName: file.originalname,
      mimeType: file.mimetype,
      size: file.size,
    };
  }

  // Process images for optimization
  private async processImage(file: Express.Multer.File): Promise<Express.Multer.File> {
    const processedPath = `/tmp/processed-${file.filename}`;
    
    await sharp(file.path)
      .resize(2000, 2000, {
        fit: 'inside',
        withoutEnlargement: true,
      })
      .jpeg({ quality: 85, progressive: true })
      .toFile(processedPath);

    return {
      ...file,
      path: processedPath,
      size: (await fs.promises.stat(processedPath)).size,
    };
  }

  // Generate thumbnail for images
  private async generateThumbnail(key: string, file: Express.Multer.File): Promise<string> {
    const thumbnailKey = key.replace(/\.(\w+)$/, '-thumb.$1');
    
    const thumbnailBuffer = await sharp(file.path)
      .resize(200, 200, {
        fit: 'cover',
        position: 'center',
      })
      .jpeg({ quality: 70 })
      .toBuffer();

    await this.s3.upload({
      Bucket: this.bucketName,
      Key: thumbnailKey,
      Body: thumbnailBuffer,
      ContentType: 'image/jpeg',
    }).promise();

    return this.getSignedUrl(thumbnailKey);
  }

  // Delete file
  async deleteFile(key: string): Promise<void> {
    await this.s3.deleteObject({
      Bucket: this.bucketName,
      Key: key,
    }).promise();

    // Also delete thumbnail if exists
    const thumbnailKey = key.replace(/\.(\w+)$/, '-thumb.$1');
    try {
      await this.s3.deleteObject({
        Bucket: this.bucketName,
        Key: thumbnailKey,
      }).promise();
    } catch (error) {
      // Thumbnail might not exist
    }
  }
}

interface FileUploadResult {
  fileId: string;
  key: string;
  url: string;
  signedUrl: string;
  thumbnailUrl?: string;
  originalName: string;
  mimeType: string;
  size: number;
}
```

### 2. File Upload API Endpoint

```typescript
// backend/src/controllers/fileController.ts
import { Request, Response } from 'express';
import { StorageService } from '../services/storageService';
import { FileRepository } from '../repositories/fileRepository';
import { AuthRequest } from '../middleware/auth';
import { ValidationError } from '../utils/errors';

const MAX_FILES_PER_UPLOAD = 5;
const MAX_TOTAL_SIZE = 50 * 1024 * 1024; // 50MB total

export class FileController {
  private storageService = new StorageService();
  private fileRepository = new FileRepository();
  private upload = this.storageService.getMulterUpload().array('files', MAX_FILES_PER_UPLOAD);

  uploadFiles = async (req: AuthRequest, res: Response): Promise<void> => {
    const userId = req.userId!;

    // Use multer middleware
    this.upload(req, res, async (err) => {
      if (err) {
        if (err.code === 'LIMIT_FILE_SIZE') {
          throw new ValidationError('File size exceeds limit');
        }
        if (err.code === 'LIMIT_FILE_COUNT') {
          throw new ValidationError(`Maximum ${MAX_FILES_PER_UPLOAD} files allowed`);
        }
        throw new ValidationError(err.message);
      }

      const files = req.files as Express.Multer.File[];
      if (!files || files.length === 0) {
        throw new ValidationError('No files provided');
      }

      // Check total size
      const totalSize = files.reduce((sum, file) => sum + file.size, 0);
      if (totalSize > MAX_TOTAL_SIZE) {
        throw new ValidationError('Total file size exceeds 50MB limit');
      }

      try {
        // Process each file
        const uploadResults = await Promise.all(
          files.map(file => this.processFileUpload(file, userId))
        );

        res.json({
          success: true,
          files: uploadResults,
        });
      } catch (error) {
        // Clean up uploaded files on error
        await this.cleanupFailedUploads(files);
        throw error;
      }
    });
  };

  private async processFileUpload(file: Express.Multer.File, userId: string) {
    // Virus scan (optional)
    if (process.env.ENABLE_VIRUS_SCAN === 'true') {
      await this.scanFile(file);
    }

    // Upload to storage
    const uploadResult = await this.storageService.uploadFile(file, userId);

    // Save metadata to database
    const fileRecord = await this.fileRepository.create({
      fileId: uploadResult.fileId,
      userId,
      key: uploadResult.key,
      originalName: uploadResult.originalName,
      mimeType: uploadResult.mimeType,
      size: uploadResult.size,
      url: uploadResult.url,
      thumbnailUrl: uploadResult.thumbnailUrl,
    });

    return {
      id: fileRecord.id,
      url: uploadResult.signedUrl,
      thumbnailUrl: uploadResult.thumbnailUrl,
      name: uploadResult.originalName,
      type: uploadResult.mimeType,
      size: uploadResult.size,
    };
  }

  private async scanFile(file: Express.Multer.File): Promise<void> {
    // Implement virus scanning using ClamAV or similar
    // Throw error if virus detected
  }

  private async cleanupFailedUploads(files: Express.Multer.File[]): Promise<void> {
    // Clean up S3 uploads if any
    for (const file of files) {
      if ((file as any).key) {
        try {
          await this.storageService.deleteFile((file as any).key);
        } catch (error) {
          console.error('Failed to cleanup file:', error);
        }
      }
    }
  }

  getFile = async (req: AuthRequest, res: Response): Promise<void> => {
    const { fileId } = req.params;
    const userId = req.userId!;

    const file = await this.fileRepository.findById(fileId);
    if (!file) {
      throw new NotFoundError('File not found');
    }

    // Check access permissions
    const hasAccess = await this.fileRepository.checkUserAccess(fileId, userId);
    if (!hasAccess) {
      throw new ForbiddenError('Access denied');
    }

    // Generate fresh signed URL
    const signedUrl = await this.storageService.getSignedUrl(file.key, 3600);

    res.json({
      id: file.id,
      url: signedUrl,
      thumbnailUrl: file.thumbnailUrl,
      name: file.originalName,
      type: file.mimeType,
      size: file.size,
      uploadedAt: file.createdAt,
    });
  };
}
```

### 3. Extended Message Schema

```sql
-- Add attachments support to messages
ALTER TABLE messages 
ADD COLUMN attachments JSONB DEFAULT '[]';

-- Create files table
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id VARCHAR(255) UNIQUE NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    key VARCHAR(500) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    size BIGINT NOT NULL,
    url TEXT,
    thumbnail_url TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create message_files junction table
CREATE TABLE message_files (
    message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    file_id UUID REFERENCES files(id) ON DELETE CASCADE,
    PRIMARY KEY (message_id, file_id)
);

-- Indexes
CREATE INDEX idx_files_user_id ON files(user_id);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
```

### 4. Frontend File Upload Component

```typescript
// frontend/src/components/chat/FileUpload/index.tsx
import React, { useState, useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import {
  Box,
  Button,
  IconButton,
  LinearProgress,
  Typography,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  Paper,
  Snackbar,
  Alert,
} from '@mui/material';
import {
  CloudUpload as UploadIcon,
  InsertDriveFile as FileIcon,
  Image as ImageIcon,
  Delete as DeleteIcon,
  AttachFile as AttachIcon,
} from '@mui/icons-material';
import { formatBytes } from '../../../utils/format';
import { uploadFiles } from '../../../services/fileService';

interface FileUploadProps {
  onFilesUploaded: (files: UploadedFile[]) => void;
  maxFiles?: number;
  maxSize?: number;
  accept?: Record<string, string[]>;
}

interface UploadedFile {
  id: string;
  url: string;
  thumbnailUrl?: string;
  name: string;
  type: string;
  size: number;
}

export const FileUpload: React.FC<FileUploadProps> = ({
  onFilesUploaded,
  maxFiles = 5,
  maxSize = 10 * 1024 * 1024, // 10MB
  accept = {
    'image/*': ['.jpeg', '.jpg', '.png', '.gif', '.webp'],
    'application/pdf': ['.pdf'],
    'application/msword': ['.doc'],
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document': ['.docx'],
  },
}) => {
  const [files, setFiles] = useState<File[]>([]);
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const onDrop = useCallback((acceptedFiles: File[], rejectedFiles: any[]) => {
    if (rejectedFiles.length > 0) {
      const errors = rejectedFiles.map(f => f.errors.map(e => e.message).join(', '));
      setError(`Some files were rejected: ${errors.join('; ')}`);
    }

    setFiles(prev => {
      const newFiles = [...prev, ...acceptedFiles];
      if (newFiles.length > maxFiles) {
        setError(`Maximum ${maxFiles} files allowed`);
        return prev;
      }
      return newFiles;
    });
  }, [maxFiles]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept,
    maxSize,
    maxFiles: maxFiles - files.length,
  });

  const removeFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index));
  };

  const handleUpload = async () => {
    if (files.length === 0) return;

    setUploading(true);
    setUploadProgress(0);
    setError(null);

    try {
      const uploadedFiles = await uploadFiles(files, (progress) => {
        setUploadProgress(Math.round(progress));
      });

      onFilesUploaded(uploadedFiles);
      setFiles([]);
      setUploadProgress(0);
    } catch (error: any) {
      setError(error.message || 'Upload failed');
    } finally {
      setUploading(false);
    }
  };

  const getFileIcon = (file: File) => {
    if (file.type.startsWith('image/')) {
      return <ImageIcon />;
    }
    return <FileIcon />;
  };

  return (
    <Box>
      <Paper
        {...getRootProps()}
        sx={{
          p: 3,
          textAlign: 'center',
          cursor: 'pointer',
          bgcolor: isDragActive ? 'action.hover' : 'background.paper',
          border: '2px dashed',
          borderColor: isDragActive ? 'primary.main' : 'divider',
          transition: 'all 0.2s ease',
          '&:hover': {
            borderColor: 'primary.main',
            bgcolor: 'action.hover',
          },
        }}
      >
        <input {...getInputProps()} />
        <UploadIcon sx={{ fontSize: 48, color: 'text.secondary', mb: 2 }} />
        <Typography variant="h6" gutterBottom>
          {isDragActive ? 'Drop files here' : 'Drag & drop files here'}
        </Typography>
        <Typography variant="body2" color="text.secondary">
          or click to browse files
        </Typography>
        <Typography variant="caption" color="text.secondary" display="block" mt={1}>
          Max {maxFiles} files, up to {formatBytes(maxSize)} each
        </Typography>
      </Paper>

      {files.length > 0 && (
        <Box mt={2}>
          <Typography variant="subtitle2" gutterBottom>
            Selected Files ({files.length}/{maxFiles})
          </Typography>
          <List dense>
            {files.map((file, index) => (
              <ListItem key={index}>
                <ListItemIcon>{getFileIcon(file)}</ListItemIcon>
                <ListItemText
                  primary={file.name}
                  secondary={formatBytes(file.size)}
                />
                <ListItemSecondaryAction>
                  <IconButton
                    edge="end"
                    onClick={() => removeFile(index)}
                    disabled={uploading}
                  >
                    <DeleteIcon />
                  </IconButton>
                </ListItemSecondaryAction>
              </ListItem>
            ))}
          </List>

          {uploading && (
            <Box mt={2}>
              <LinearProgress variant="determinate" value={uploadProgress} />
              <Typography variant="caption" color="text.secondary" align="center" display="block" mt={1}>
                Uploading... {uploadProgress}%
              </Typography>
            </Box>
          )}

          <Box mt={2} display="flex" gap={1}>
            <Button
              variant="contained"
              startIcon={<UploadIcon />}
              onClick={handleUpload}
              disabled={uploading || files.length === 0}
              fullWidth
            >
              Upload {files.length} {files.length === 1 ? 'File' : 'Files'}
            </Button>
            <Button
              variant="outlined"
              onClick={() => setFiles([])}
              disabled={uploading}
            >
              Clear
            </Button>
          </Box>
        </Box>
      )}

      <Snackbar
        open={!!error}
        autoHideDuration={6000}
        onClose={() => setError(null)}
      >
        <Alert onClose={() => setError(null)} severity="error">
          {error}
        </Alert>
      </Snackbar>
    </Box>
  );
};
```

### 5. File Preview Components

```typescript
// frontend/src/components/chat/FilePreview/ImagePreview.tsx
import React, { useState } from 'react';
import {
  Box,
  IconButton,
  Modal,
  Fade,
  Backdrop,
  CircularProgress,
} from '@mui/material';
import {
  Close as CloseIcon,
  Download as DownloadIcon,
  ZoomIn as ZoomInIcon,
  ZoomOut as ZoomOutIcon,
} from '@mui/icons-material';

interface ImagePreviewProps {
  src: string;
  alt: string;
  thumbnailUrl?: string;
  onClose?: () => void;
}

export const ImagePreview: React.FC<ImagePreviewProps> = ({
  src,
  alt,
  thumbnailUrl,
  onClose,
}) => {
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [zoom, setZoom] = useState(1);

  const handleOpen = () => setOpen(true);
  const handleClose = () => {
    setOpen(false);
    setZoom(1);
    onClose?.();
  };

  const handleDownload = () => {
    const link = document.createElement('a');
    link.href = src;
    link.download = alt || 'image';
    link.click();
  };

  const handleZoomIn = () => setZoom(prev => Math.min(prev + 0.2, 3));
  const handleZoomOut = () => setZoom(prev => Math.max(prev - 0.2, 0.5));

  return (
    <>
      <Box
        component="img"
        src={thumbnailUrl || src}
        alt={alt}
        onClick={handleOpen}
        sx={{
          maxWidth: '100%',
          maxHeight: 300,
          borderRadius: 1,
          cursor: 'pointer',
          transition: 'transform 0.2s',
          '&:hover': {
            transform: 'scale(1.02)',
          },
        }}
      />

      <Modal
        open={open}
        onClose={handleClose}
        closeAfterTransition
        BackdropComponent={Backdrop}
        BackdropProps={{
          timeout: 500,
          sx: { bgcolor: 'rgba(0, 0, 0, 0.9)' },
        }}
      >
        <Fade in={open}>
          <Box
            sx={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              outline: 'none',
              maxWidth: '90vw',
              maxHeight: '90vh',
            }}
          >
            {/* Controls */}
            <Box
              sx={{
                position: 'absolute',
                top: 0,
                right: 0,
                zIndex: 1,
                display: 'flex',
                gap: 1,
                p: 1,
                bgcolor: 'rgba(0, 0, 0, 0.5)',
                borderRadius: 1,
              }}
            >
              <IconButton color="inherit" onClick={handleZoomOut}>
                <ZoomOutIcon />
              </IconButton>
              <IconButton color="inherit" onClick={handleZoomIn}>
                <ZoomInIcon />
              </IconButton>
              <IconButton color="inherit" onClick={handleDownload}>
                <DownloadIcon />
              </IconButton>
              <IconButton color="inherit" onClick={handleClose}>
                <CloseIcon />
              </IconButton>
            </Box>

            {/* Image */}
            <Box
              sx={{
                position: 'relative',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              {loading && (
                <CircularProgress
                  sx={{
                    position: 'absolute',
                    color: 'white',
                  }}
                />
              )}
              <img
                src={src}
                alt={alt}
                onLoad={() => setLoading(false)}
                style={{
                  maxWidth: '90vw',
                  maxHeight: '90vh',
                  transform: `scale(${zoom})`,
                  transition: 'transform 0.2s',
                  display: loading ? 'none' : 'block',
                }}
              />
            </Box>
          </Box>
        </Fade>
      </Modal>
    </>
  );
};
```

## Security Considerations

### File Validation
- Strict MIME type checking
- File extension validation
- Size limits enforcement
- Content-based file type detection

### Access Control
- Signed URLs with expiration
- User permission checks
- Room-based file access
- Audit logging

### Storage Security
- Encrypted storage at rest
- Private S3 buckets
- CloudFront for CDN delivery
- Regular security scans

## Performance Optimizations

### Upload Optimization
- Chunked uploads for large files
- Parallel upload streams
- Resume capability
- Progress tracking

### Image Processing
- Automatic resizing
- Format optimization
- Thumbnail generation
- Lazy loading

### Caching Strategy
- CDN caching for static files
- Browser caching headers
- Thumbnail caching
- Signed URL caching

## Error Handling

### Upload Errors
- Network failure recovery
- Quota exceeded handling
- Invalid file type messages
- Virus detection alerts

### Display Errors
- Broken image placeholders
- Download failure retry
- Access denied messages
- Expired URL refresh