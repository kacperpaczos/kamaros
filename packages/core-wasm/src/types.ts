/**
 * TypeScript type definitions for Kamaros JCF Format
 */

/**
 * Storage adapter interface for platform-agnostic I/O
 */
export interface StorageAdapter {
    read(path: string): Promise<Uint8Array>;
    write(path: string, data: Uint8Array): Promise<void>;
    delete(path: string): Promise<void>;
    exists(path: string): Promise<boolean>;
    list(dir: string): Promise<string[]>;
    size(path: string): Promise<number>;
    listBlobs(): Promise<string[]>;
}

/**
 * JCF Manifest - main project metadata
 */
export interface Manifest {
    formatVersion: string;
    metadata: ProjectMetadata;
    fileMap: Record<string, FileEntry>;
    versionHistory: Version[];
    refs: Record<string, string>;
    renameLog: RenameEntry[];
}

export interface ProjectMetadata {
    name: string;
    description?: string;
    created: string;
    lastModified: string;
    author?: string;
}

export interface FileEntry {
    inodeId: string;
    type: 'text' | 'binary';
    currentHash?: string;
    created: string;
    modified: string;
}

export interface Version {
    id: string;
    parentId?: string;
    timestamp: string;
    message: string;
    author: string;
    fileStates: Record<string, FileState>;
}

export interface FileState {
    inodeId: string;
    hash?: string;
    contentRef?: string;
    deleted?: boolean;
    encrypted?: boolean;
}

export interface RenameEntry {
    from: string;
    to: string;
    timestamp: string;
    versionId: string;
}

/**
 * Results
 */
export interface SaveCheckpointResult {
    manifest: Record<string, unknown>;
    manifestJson?: string;
    versionId: string;
    filesAdded: number;
    filesModified: number;
    filesDeleted: number;
}

export interface GcResult {
    blobsChecked: number;
    blobsDeleted: number;
    bytesFreed: number;
}

export interface ImportZipResult {
    projectName: string;
    filesImported: number;
    totalSize: number;
}

/**
 * Options for JCFManager operations
 */
export interface SaveOptions {
    message?: string;
    author?: string;
    encryptionKey?: Uint8Array;
}

export interface LoadOptions {
    validateIntegrity?: boolean;
    encryptionKey?: Uint8Array;
}
