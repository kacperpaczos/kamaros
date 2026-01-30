# API Reference

This document provides a detailed reference for the **Kamaros TypeScript/JavaScript API** available in `@kamaros/core-wasm`, `@kamaros/web`, and `@kamaros/node`.

## Initialization

### `JCFManager.create(adapter: StorageAdapter): Promise<JCFManager>`

Creates a new instance of the JCFManager. This method initializes the WebAssembly core if it hasn't been loaded yet.

**Parameters:**
- `adapter`: An instance of a class implementing `StorageAdapter` (e.g., `NodeAdapter`, `OPFSAdapter`).

**Returns:**
- A `Promise` resolving to the initialized `JCFManager`.

**Example:**
```typescript
const manager = await JCFManager.create(new NodeAdapter('./my-project'));
```

## Project Management

### `createProject(name: string, options?: { description?: string, author?: string }): Promise<void>`

Initializes a new, empty project in memory.

**Parameters:**
- `name`: The name of the project.
- `options`: Optional metadata.

### `getProjectInfo(): { name: string; versionCount: number; fileCount: number } | null`

Returns metadata about the currently loaded project, or `null` if no project is loaded.

## File Operations

### `addFile(path: string, content: Uint8Array): void`

Adds or updates a file in the working directory. This change is **not** committed to history until `saveCheckpoint` is called.

**Parameters:**
- `path`: Relative path of the file (e.g., "src/main.rs").
- `content`: The raw binary content of the file.

### `getFile(path: string): Uint8Array | undefined`

Retrieves the content of a file from the current working directory.

### `deleteFile(path: string): boolean`

Removes a file from the working directory.

### `listFiles(): string[]`

Returns a list of all files currently in the working directory.

## Version Control

### `saveCheckpoint(message: string, options?: SaveOptions): Promise<string>`

Creates a new immutable version (checkpoint) of the current project state.

**Parameters:**
- `message`: Commit message describing the changes.
- `options`:
    - `author`: (string) Name of the author.
    - `encryptionKey`: (Uint8Array, optional) Key to encrypt the checkpoint blobs.

**Returns:**
- The `versionId` (UUID) of the new version.

### `restoreVersion(versionId: string, options?: LoadOptions): Promise<void>`

Restores the working directory to the state of a specific version.

**Parameters:**
- `versionId`: Valid UUID of a previously saved version.
- `options`:
    - `encryptionKey`: (Uint8Array, optional) Key to decrypt content if the version was encrypted.

## Security (Encryption)

### `deriveKey(passphrase: string, salt: Uint8Array): Promise<Uint8Array>`

Derives a strong encryption key from a user password using PBKDF2-HMAC-SHA256.

**Parameters:**
- `passphrase`: User password.
- `salt`: A 16-byte random salt. **Note:** You must store this salt (e.g., in a separate config file or unencrypted file) to regenerate the key later.

**Returns:**
- A 32-byte (256-bit) AES key.

## Portability (ZIP)

### `exportZip(): Promise<Uint8Array>`

Packages the entire project (including `.store/` history and `content/` working directory) into a standard ZIP archive.

**Returns:**
- The raw bytes of the ZIP file, ready to be saved or downloaded.

### `importZip(zipData: Uint8Array): Promise<ImportZipResult>`

Imports a project from a ZIP archive. This overwrites the current project state.

**Parameters:**
- `zipData`: The raw bytes of the ZIP file.

**Returns:**
- Object containing import statistics (`projectName`, `filesImported`).
