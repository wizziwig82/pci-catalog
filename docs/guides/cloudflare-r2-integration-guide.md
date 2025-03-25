# Cloudflare R2 Integration Guide

*Last Updated: March 25, 2024*

This guide provides detailed instructions on working with Cloudflare R2 storage in the PCI File Manager application.

## Overview

PCI File Manager uses Cloudflare R2 as its primary storage solution for:
- Storing uploaded files securely
- Managing file versions
- Enabling multipart uploads for large files
- Controlling file access through permission systems

## Prerequisites

- Cloudflare account with R2 enabled
- R2 bucket created for the application
- Access credentials (Account ID, Access Key ID, Secret Access Key)

## Setup Instructions

### 1. Creating R2 Bucket and Credentials

1. **Sign up for Cloudflare R2**:
   - Go to [Cloudflare Dashboard](https://dash.cloudflare.com)
   - Navigate to R2 from the sidebar
   - Create a new bucket for your application (e.g., `pci-file-manager`)

2. **Generate API Tokens**:
   - Go to Account Home → R2 → "Manage R2 API Tokens"
   - Create a new API token with read/write permissions to your bucket
   - Save the `Access Key ID` and `Secret Access Key` securely

### 2. Configuring the Application

Add the following to your `.env` file:

```
CF_ACCOUNT_ID=your-account-id
CF_ACCESS_KEY_ID=your-access-key
CF_SECRET_ACCESS_KEY=your-secret-key
CF_BUCKET_NAME=your-bucket-name
CF_PUBLIC_URL=https://your-bucket-name.yoursubdomain.r2.dev (optional)
```

## Using R2 in the Application

### File Upload Process

When a file is uploaded through the application:

1. The application generates a unique key based on file metadata
2. The file is streamed directly to R2 storage
3. Metadata about the file is stored in MongoDB
4. A reference to the R2 object is maintained in the database

### Key Example Code Snippets

#### Initializing the R2 Client

```javascript
const { S3Client } = require('@aws-sdk/client-s3');

const r2Client = new S3Client({
  region: 'auto',
  endpoint: `https://${process.env.CF_ACCOUNT_ID}.r2.cloudflarestorage.com`,
  credentials: {
    accessKeyId: process.env.CF_ACCESS_KEY_ID,
    secretAccessKey: process.env.CF_SECRET_ACCESS_KEY,
  },
});
```

#### Uploading a File

```javascript
const { PutObjectCommand } = require('@aws-sdk/client-s3');
const fs = require('fs');

async function uploadFile(filePath, key) {
  const fileStream = fs.createReadStream(filePath);
  
  const params = {
    Bucket: process.env.CF_BUCKET_NAME,
    Key: key,
    Body: fileStream,
    ContentType: determineContentType(filePath),
  };
  
  try {
    const command = new PutObjectCommand(params);
    const response = await r2Client.send(command);
    return response;
  } catch (error) {
    console.error('Error uploading to R2:', error);
    throw error;
  }
}
```

#### Retrieving a File

```javascript
const { GetObjectCommand } = require('@aws-sdk/client-s3');

async function getFile(key) {
  const params = {
    Bucket: process.env.CF_BUCKET_NAME,
    Key: key,
  };
  
  try {
    const command = new GetObjectCommand(params);
    const response = await r2Client.send(command);
    return response.Body;
  } catch (error) {
    console.error('Error retrieving from R2:', error);
    throw error;
  }
}
```

#### Deleting a File

```javascript
const { DeleteObjectCommand } = require('@aws-sdk/client-s3');

async function deleteFile(key) {
  const params = {
    Bucket: process.env.CF_BUCKET_NAME,
    Key: key,
  };
  
  try {
    const command = new DeleteObjectCommand(params);
    const response = await r2Client.send(command);
    return response;
  } catch (error) {
    console.error('Error deleting from R2:', error);
    throw error;
  }
}
```

### Managing File Access Control

R2 does not natively include fine-grained access controls, so these are implemented at the application level:

1. The application stores permission metadata in MongoDB
2. When serving files, the application checks permissions before retrieving from R2
3. For temporary access, signed URLs can be generated

#### Generating a Presigned URL

```javascript
const { getSignedUrl } = require("@aws-sdk/s3-request-presigner");
const { GetObjectCommand } = require('@aws-sdk/client-s3');

async function generateSignedUrl(key, expirationSeconds = 3600) {
  const command = new GetObjectCommand({
    Bucket: process.env.CF_BUCKET_NAME,
    Key: key,
  });
  
  try {
    const signedUrl = await getSignedUrl(r2Client, command, {
      expiresIn: expirationSeconds,
    });
    return signedUrl;
  } catch (error) {
    console.error('Error generating signed URL:', error);
    throw error;
  }
}
```

## Best Practices

### Performance Optimization

- Use multipart uploads for files larger than 100MB
- Implement resumable uploads for large files
- Cache frequently accessed files at the application level

### Security Considerations

- Never expose R2 credentials in client-side code
- Implement strict access controls in the application
- Set appropriate CORS policies for the R2 bucket
- Regularly rotate R2 access credentials

### Monitoring and Maintenance

- Implement logging for all R2 operations
- Set up monitoring for storage usage and costs
- Regularly clean up unused or temporary files
- Implement lifecycle policies for archiving older files

## Troubleshooting

### Common Issues

1. **Upload Failures**:
   - Check network connectivity
   - Verify credentials and permissions
   - Ensure bucket exists
   - Check for file size limitations

2. **Access Denied Errors**:
   - Verify API key permissions
   - Check bucket policy configurations
   - Ensure credentials are correct and not expired

3. **Performance Issues**:
   - Optimize file chunking for large uploads
   - Check network bandwidth limitations
   - Consider using Workers for edge processing

## Related Documentation

- [R2 Storage Design](../architecture/r2-storage-design.md)
- [API Endpoints Reference](../api/endpoints-reference.md)
- [Security Best Practices Guide](security-best-practices-guide.md)
- [Performance Optimization Guide](performance-optimization-guide.md)

## External Resources

- [Cloudflare R2 Documentation](https://developers.cloudflare.com/r2/)
- [AWS SDK for JavaScript Documentation](https://docs.aws.amazon.com/AWSJavaScriptSDK/v3/latest/) 