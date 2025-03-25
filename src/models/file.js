/**
 * File model structure based on MongoDB schema requirements
 * This will be used for client-side validation and structure
 */

class File {
  constructor(data = {}) {
    this.id = data.id || '';
    this.filename = data.filename || '';
    this.originalName = data.originalName || '';
    this.path = data.path || '';
    this.fileType = data.fileType || '';
    this.mimeType = data.mimeType || '';
    this.extension = data.extension || '';
    this.size = data.size || 0;
    this.storageUrl = data.storageUrl || '';
    this.metadata = data.metadata || {};
    this.tags = data.tags || [];
    this.isPublic = data.isPublic !== undefined ? data.isPublic : true;
    this.uploadedBy = data.uploadedBy || '';
    this.createdAt = data.createdAt || new Date();
    this.updatedAt = data.updatedAt || new Date();
    this.lastAccessed = data.lastAccessed || new Date();
  }

  /**
   * Validate file data
   * @returns {Object} Validation result with isValid flag and errors array
   */
  validate() {
    const errors = [];

    // Required fields
    if (!this.filename) errors.push('Filename is required');
    if (!this.mimeType) errors.push('MIME type is required');
    if (!this.size || this.size <= 0) errors.push('Valid file size is required');
    if (!this.storageUrl) errors.push('Storage URL is required');

    return {
      isValid: errors.length === 0,
      errors
    };
  }

  /**
   * Format file size in a human-readable format
   * @returns {string} Formatted file size
   */
  get sizeFormatted() {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = this.size;
    let unitIndex = 0;
    
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    
    return `${size.toFixed(2)} ${units[unitIndex]}`;
  }

  /**
   * Convert to plain object for storage
   * @returns {Object} Plain object representation
   */
  toObject() {
    return {
      id: this.id,
      filename: this.filename,
      originalName: this.originalName,
      path: this.path,
      fileType: this.fileType,
      mimeType: this.mimeType,
      extension: this.extension,
      size: this.size,
      storageUrl: this.storageUrl,
      metadata: this.metadata,
      tags: this.tags,
      isPublic: this.isPublic,
      uploadedBy: this.uploadedBy,
      createdAt: this.createdAt,
      updatedAt: new Date(), // Always update when saving
      lastAccessed: this.lastAccessed
    };
  }

  /**
   * Create a File instance from a plain object
   * @param {Object} data Plain object data
   * @returns {File} New File instance
   */
  static fromObject(data) {
    return new File(data);
  }
}

module.exports = File;
