/**
 * Kamaros TypeScript Bindings
 * 
 * Re-exports WASM module and provides high-level API.
 */

// Re-export WASM bindings
// Note: Consumer must call init() before using other functions
export {
    default as init,
    version,
    greet,
    create_empty_manifest,
    parse_manifest,
    get_manifest_info,
} from '../../wasm/pkg/kamaros_wasm.js';

// Type definitions
export interface StorageAdapter {
    read(path: string): Promise<Uint8Array>;
    write(path: string, data: Uint8Array): Promise<void>;
    delete(path: string): Promise<void>;
    exists(path: string): Promise<boolean>;
    list(dir: string): Promise<string[]>;
}

export interface Manifest {
    formatVersion: string;
    metadata: {
        name: string;
        description?: string;
        created: string;
        lastModified: string;
        author?: string;
    };
    fileMap: Record<string, FileEntry>;
    versionHistory: Version[];
    refs: Record<string, string>;
    renameLog: RenameEntry[];
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
}

export interface RenameEntry {
    from: string;
    to: string;
    timestamp: string;
    versionId: string;
}
