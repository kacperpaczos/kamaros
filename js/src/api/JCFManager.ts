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

import { initWasm, getWasm } from '../wasm';
import type { StorageAdapter, Manifest, SaveOptions, LoadOptions } from '../types';
import { zipSync, unzipSync, strToU8, strFromU8 } from 'fflate';

export class JCFManager {
    private adapter: StorageAdapter;
    private manifest: Manifest | null = null;
    private workingDir: Map<string, Uint8Array> = new Map();

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

        // Load working directory (/content/)
        this.workingDir.clear();
        for (const [filePath, fileData] of Object.entries(unzipped)) {
            if (filePath.startsWith('content/')) {
                const relativePath = filePath.slice('content/'.length);
                this.workingDir.set(relativePath, fileData as Uint8Array);
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

        // Add working directory files
        for (const [filePath, data] of this.workingDir) {
            files[`content/${filePath}`] = data;
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
     * List all files in working directory
     */
    listFiles(): string[] {
        return Array.from(this.workingDir.keys());
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
     * Save checkpoint (create new version)
     */
    async saveCheckpoint(_message: string, _options?: SaveOptions): Promise<string> {
        // TODO: Implement full versioning with WASM Use Cases
        // For now, just return a placeholder
        const versionId = crypto.randomUUID();
        return versionId;
    }

    // Private helpers

    private isTextFile(path: string): boolean {
        const textExtensions = ['.txt', '.md', '.json', '.js', '.ts', '.css', '.html', '.xml', '.yaml', '.yml'];
        return textExtensions.some(ext => path.toLowerCase().endsWith(ext));
    }

    private normalizeManifest(raw: Record<string, unknown>): Manifest {
        return {
            formatVersion: (raw['format_version'] || raw['formatVersion'] || '1.0.0') as string,
            metadata: (raw['metadata'] || {}) as Manifest['metadata'],
            fileMap: (raw['file_map'] || raw['fileMap'] || {}) as Manifest['fileMap'],
            versionHistory: (raw['version_history'] || raw['versionHistory'] || []) as Manifest['versionHistory'],
            refs: (raw['refs'] || { head: '' }) as Manifest['refs'],
            renameLog: (raw['rename_log'] || raw['renameLog'] || []) as Manifest['renameLog'],
        };
    }
}
