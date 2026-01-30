/**
 * JCFManager - High-level API for JCF file operations
 * 
 * @example
 * ```typescript
 * const manager = await JCFManager.create(new NodeAdapter('./data'));
 * await manager.createProject('MyProject');
 * await manager.addFile('src/main.ts', new TextEncoder().encode('console.log("Hello")'));
 * await manager.saveCheckpoint('Initial commit');
 * await manager.save('project.jcf');
 * ```
 */

import { initWasm, getWasm } from './wasm';
import type {
    StorageAdapter,
    Manifest,
    SaveOptions,
    LoadOptions,
    SaveCheckpointResult,
    GcResult,
    ImportZipResult
} from './types';
import { zipSync, unzipSync, strToU8, strFromU8 } from 'fflate';

export class JCFManager {
    private adapter: StorageAdapter;
    private manifest: Manifest | null = null;
    private workingDir: Map<string, Uint8Array> = new Map();
    private encryptionKey: Uint8Array | null = null;

    private constructor(adapter: StorageAdapter) {
        this.adapter = adapter;
    }

    /**
     * Create a new JCFManager instance
     * Initializes WASM lazily on first use
     */
    static async create(adapter: StorageAdapter): Promise<JCFManager> {
        await initWasm();
        return new JCFManager(adapter);
    }

    /**
     * Create a new empty project
     */
    async createProject(name: string, options?: { description?: string; author?: string }): Promise<void> {
        const wasm = getWasm();
        const rawManifest = wasm.create_empty_manifest(name) as Record<string, unknown>;

        // Normalize from WASM format to TS format
        this.manifest = {
            formatVersion: (rawManifest['format_version'] as string) || '1.0.0',
            metadata: {
                name,
                description: options?.description,
                created: new Date().toISOString(),
                lastModified: new Date().toISOString(),
                author: options?.author,
            },
            fileMap: {},
            versionHistory: [],
            refs: { head: '' },
            renameLog: [],
        };

        this.workingDir.clear();
    }

    /**
     * Load a JCF file from storage
     */
    async load(path: string, _options?: LoadOptions): Promise<void> {
        const data = await this.adapter.read(path);
        const unzipped = unzipSync(data);

        // Read manifest
        const manifestData = unzipped['manifest.json'];
        if (!manifestData) {
            throw new Error('Invalid JCF file: missing manifest.json');
        }

        const manifestJson = strFromU8(manifestData);
        const rawManifest = JSON.parse(manifestJson);

        // Parse through WASM for validation
        const wasm = getWasm();
        wasm.parse_manifest(rawManifest);

        this.manifest = this.normalizeManifest(rawManifest);

        // Load working directory and store
        this.workingDir.clear();
        for (const [filePath, fileData] of Object.entries(unzipped)) {
            if (filePath.startsWith('content/')) {
                const relativePath = filePath.slice('content/'.length);
                this.workingDir.set(relativePath, fileData as Uint8Array);
            } else if (filePath.startsWith('.store/')) {
                // Keep .store files in workingDir but they will be filtered in listFiles
                this.workingDir.set(filePath, fileData as Uint8Array);
            }
        }
    }

    /**
     * Save JCF file to storage
     */
    async save(path: string): Promise<void> {
        if (!this.manifest) {
            throw new Error('No project loaded. Call createProject() or load() first.');
        }

        // Update lastModified
        this.manifest.metadata.lastModified = new Date().toISOString();

        // Build ZIP structure
        const files: Record<string, Uint8Array> = {
            'mimetype': strToU8('application/x-jcf'),
            'manifest.json': strToU8(JSON.stringify(this.manifest, null, 2)),
        };

        // Add files
        for (const [filePath, data] of this.workingDir) {
            if (filePath.startsWith('.store/')) {
                files[filePath] = data;
            } else {
                files[`content/${filePath}`] = data;
            }
        }

        // Create ZIP
        const zipped = zipSync(files);
        await this.adapter.write(path, zipped);
    }

    /**
     * Add or update a file in working directory
     */
    addFile(path: string, content: Uint8Array): void {
        if (!this.manifest) {
            throw new Error('No project loaded.');
        }

        this.workingDir.set(path, content);

        // Update file map
        const now = new Date().toISOString();
        if (!this.manifest.fileMap[path]) {
            this.manifest.fileMap[path] = {
                inodeId: crypto.randomUUID(),
                type: this.isTextFile(path) ? 'text' : 'binary',
                created: now,
                modified: now,
            };
        } else {
            this.manifest.fileMap[path].modified = now;
        }
    }

    /**
     * Get a file from working directory
     */
    getFile(path: string): Uint8Array | undefined {
        return this.workingDir.get(path);
    }

    /**
     * Delete a file from working directory
     */
    deleteFile(path: string): boolean {
        if (!this.manifest) return false;

        const existed = this.workingDir.delete(path);
        delete this.manifest.fileMap[path];
        return existed;
    }

    /**
     * List all files in working directory (excludes .store/)
     */
    listFiles(): string[] {
        return Array.from(this.workingDir.keys()).filter(p => !p.startsWith('.store/'));
    }

    /**
     * Get current manifest
     */
    getManifest(): Manifest | null {
        return this.manifest;
    }

    /**
     * Get project info
     */
    getProjectInfo(): { name: string; versionCount: number; fileCount: number } | null {
        if (!this.manifest) return null;

        return {
            name: this.manifest.metadata.name,
            versionCount: this.manifest.versionHistory.length,
            fileCount: Object.keys(this.manifest.fileMap).length,
        };
    }

    /**
     * Derive encryption key from passphrase
     */
    async deriveKey(passphrase: string, salt: Uint8Array): Promise<Uint8Array> {
        const wasm = getWasm();
        this.encryptionKey = wasm.derive_key(passphrase, salt);
        return this.encryptionKey;
    }

    /**
     * Save checkpoint (create new version)
     * Uses WASM save_checkpoint for change detection and version creation
     */
    async saveCheckpoint(message: string, options?: SaveOptions): Promise<string> {
        if (!this.manifest) {
            throw new Error('No project loaded. Call createProject() or load() first.');
        }

        const wasm = getWasm();
        const author = options?.author ?? 'unknown';
        const key = options?.encryptionKey ?? this.encryptionKey ?? undefined;

        // Create WASM-compatible storage adapter from workingDir
        const wasmStorageAdapter = this.createWasmStorageAdapter();

        // Convert manifest to WASM format (snake_case)
        const wasmManifest = this.toWasmManifest(this.manifest);

        try {
            // Call WASM save_checkpoint
            const result = await wasm.save_checkpoint(
                wasmManifest,
                wasmStorageAdapter,
                message,
                author,
                key
            );
            if (result.manifestJson) {
                const raw = JSON.parse(result.manifestJson);
                this.manifest = this.normalizeManifest(raw);
            } else {
                this.manifest = this.normalizeManifest(result.manifest);
            }

            return result.versionId;
        } catch (e: any) {
            console.error('saveCheckpoint raw error:', e);
            let msg = e.toString();
            if (typeof e === 'object') {
                try { msg = JSON.stringify(e); } catch { }
            }
            throw new Error(`saveCheckpoint failed: ${msg}`);
        }
    }

    /**
     * Restore version (checkout)
     */
    async restoreVersion(versionId: string, options?: LoadOptions): Promise<void> {
        if (!this.manifest) {
            throw new Error('No project loaded.');
        }

        const wasm = getWasm();
        const wasmStorageAdapter = this.createWasmStorageAdapter();
        const wasmManifest = this.toWasmManifest(this.manifest);
        const key = options?.encryptionKey ?? this.encryptionKey ?? undefined;

        try {
            const result = await wasm.restore_version(
                wasmManifest,
                wasmStorageAdapter,
                versionId,
                key
            );

            // Update local manifest
            this.manifest = this.normalizeManifest(result.manifest);
        } catch (e) {
            throw new Error(`restoreVersion failed: ${e}`);
        }
    }

    /**
     * Garbage Collection
     */
    async gc(): Promise<GcResult> {
        if (!this.manifest) {
            throw new Error('No project loaded.');
        }

        const wasm = getWasm();
        const wasmStorageAdapter = this.createWasmStorageAdapter();
        const wasmManifest = this.toWasmManifest(this.manifest);

        const result = await wasm.gc(wasmManifest, wasmStorageAdapter);
        return result;
    }

    /**
     * Export project as ZIP archive
     */
    async exportZip(): Promise<Uint8Array> {
        if (!this.manifest) {
            throw new Error('No project loaded.');
        }

        // Ensure manifest is saved to storage before exporting
        // WASM ExportArchiveUseCase reads from storage directly.
        const manifestJson = JSON.stringify(this.manifest, null, 2);
        await this.adapter.write('.store/manifest.json', strToU8(manifestJson));

        const wasm = getWasm();
        const wasmStorageAdapter = this.createWasmStorageAdapter();
        return wasm.export_zip(wasmStorageAdapter);
    }

    /**
     * Import project from ZIP archive
     */
    async importZip(zipData: Uint8Array): Promise<ImportZipResult> {
        const wasm = getWasm();
        const wasmStorageAdapter = this.createWasmStorageAdapter();

        try {
            const result = await wasm.import_zip(wasmStorageAdapter, zipData);

            // Reload manifest from storage (Import overwrites it in adapter)
            // Note: .store/manifest.json goes to adapter, content/ goes to workingDir via wrapper logic
            const manifestData = await this.adapter.read('.store/manifest.json');
            const manifestJson = strFromU8(manifestData);
            const rawManifest = JSON.parse(manifestJson);

            this.manifest = this.normalizeManifest(rawManifest);

            return result as ImportZipResult;
        } catch (e) {
            throw new Error(`importZip failed: ${e}`);
        }
    }

    /**
     * Create a WASM-compatible storage adapter from workingDir
     * This adapter provides read/write/delete/exists/list methods
     */
    private createWasmStorageAdapter() {
        const adapter = this.adapter;
        const workingDir = this.workingDir;

        return {
            async read(path: string): Promise<Uint8Array> {
                if (path.startsWith('content/')) {
                    const cleanPath = path.slice(8);
                    const data = workingDir.get(cleanPath);
                    if (data) return data;
                }
                return adapter.read(path);
            },

            async write(path: string, data: Uint8Array): Promise<void> {
                if (path.startsWith('content/')) {
                    const cleanPath = path.slice(8);
                    workingDir.set(cleanPath, data);
                    return;
                }
                return adapter.write(path, data);
            },

            async delete(path: string): Promise<void> {
                if (path.startsWith('content/')) {
                    const cleanPath = path.slice(8);
                    workingDir.delete(cleanPath);
                    return;
                }
                return adapter.delete(path);
            },

            async exists(path: string): Promise<boolean> {
                if (path.startsWith('content/')) {
                    const cleanPath = path.slice(8);
                    if (workingDir.has(cleanPath)) return true;
                }
                return adapter.exists(path);
            },

            async list(dir: string): Promise<string[]> {
                if (dir === 'content' || dir === 'content/' || dir.startsWith('content/')) {
                    // Logic to emulate directory listing from flat workingDir keys
                    const prefix = dir.endsWith('/') ? dir : `${dir}/`;
                    const relativePrefix = prefix.slice(8); // 'content/'.length

                    return Array.from(workingDir.keys())
                        .filter(k => k.startsWith(relativePrefix) && k !== relativePrefix)
                        .map(k => k.slice(relativePrefix.length).split('/')[0])
                        .filter((v, i, a) => v && a.indexOf(v) === i);
                }
                return adapter.list(dir);
            },

            async size(path: string): Promise<number> {
                if (path.startsWith('content/')) {
                    const cleanPath = path.slice(8);
                    const data = workingDir.get(cleanPath);
                    if (data) return data.length;
                }
                return adapter.size(path);
            },

            async list_blobs(): Promise<string[]> {
                return adapter.listBlobs();
            }
        };
    }

    /**
     * Convert TS manifest to WASM format (snake_case keys)
     */
    private toWasmManifest(manifest: Manifest): Record<string, unknown> {
        return {
            formatVersion: manifest.formatVersion,
            metadata: {
                name: manifest.metadata.name,
                description: manifest.metadata.description,
                created: manifest.metadata.created,
                lastModified: manifest.metadata.lastModified,
                author: manifest.metadata.author,
            },
            fileMap: Object.fromEntries(
                Object.entries(manifest.fileMap).map(([path, entry]) => [
                    path,
                    {
                        inodeId: entry.inodeId,
                        type: entry.type === 'text' ? 'text' : 'binary',
                        created: entry.created,
                        modified: entry.modified,
                        currentHash: entry.currentHash,
                    },
                ])
            ),
            versionHistory: manifest.versionHistory.map((v) => ({
                id: v.id,
                parentId: v.parentId,
                timestamp: v.timestamp,
                message: v.message,
                author: v.author,
                fileStates: v.fileStates,
            })),
            refs: manifest.refs,
            renameLog: manifest.renameLog,
            // Map rename_log to renameLog
        };
    }

    // Private helpers

    private isTextFile(path: string): boolean {
        const textExtensions = ['.txt', '.md', '.json', '.js', '.ts', '.css', '.html', '.xml', '.yaml', '.yml'];
        return textExtensions.some(ext => path.toLowerCase().endsWith(ext));
    }

    private normalizeManifest(raw: Record<string, unknown>): Manifest {
        const fileMapRaw = raw['file_map'] || raw['fileMap'];
        const fileMap = fileMapRaw instanceof Map
            ? Object.fromEntries(fileMapRaw)
            : (fileMapRaw || {});

        const versionHistoryRaw = (raw['version_history'] || raw['versionHistory'] || []) as any[];
        const versionHistory = versionHistoryRaw.map(v => {
            const fileStatesRaw = v.fileStates || v.file_states;
            const fileStates = fileStatesRaw instanceof Map
                ? Object.fromEntries(fileStatesRaw)
                : (fileStatesRaw || {});

            return {
                id: v.id,
                parentId: v.parentId || v.parent_id,
                timestamp: v.timestamp,
                message: v.message,
                author: v.author,
                fileStates: fileStates,
            } as any; // Cast to avoid strict type checking on normalized obj
        });

        return {
            formatVersion: (raw['format_version'] || raw['formatVersion'] || '1.0.0') as string,
            metadata: (raw['metadata'] || {}) as Manifest['metadata'],
            fileMap: fileMap as Manifest['fileMap'],
            versionHistory: versionHistory as Manifest['versionHistory'],
            refs: (raw['refs'] instanceof Map
                ? Object.fromEntries(raw['refs'])
                : (raw['refs'] || { head: '' })) as Manifest['refs'],
            renameLog: (raw['rename_log'] || raw['renameLog'] || []) as Manifest['renameLog'],
        };
    }
}
