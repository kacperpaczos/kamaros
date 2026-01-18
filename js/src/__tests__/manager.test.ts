/**
 * Tests for JCFManager (without WASM dependency)
 * 
 * These tests focus on the pure TypeScript/JavaScript functionality
 * without requiring WASM initialization.
 * 
 * Note: Tests requiring actual WASM (like create_empty_manifest) 
 * would need browser environment or Node.js WASM polyfill.
 */

import { describe, it, expect } from 'vitest';
import { MemoryAdapter } from '../adapters/MemoryAdapter';
import { zipSync, strToU8, strFromU8 } from 'fflate';

describe('JCFManager - ZIP Operations (No WASM)', () => {

    // =========================================================================
    // ZIP FORMAT TESTS (doesn't require WASM)
    // =========================================================================

    /** Test: Can create valid JCF ZIP structure */
    it('should create valid JCF ZIP structure', () => {
        const manifest = {
            formatVersion: "1.0.0",
            metadata: { name: "TestProject", created: "2024-01-01", lastModified: "2024-01-01" },
            fileMap: {},
            versionHistory: [],
            refs: { head: "" },
            renameLog: []
        };

        const files = {
            'mimetype': strToU8('application/x-jcf'),
            'manifest.json': strToU8(JSON.stringify(manifest, null, 2)),
            'content/test.txt': strToU8('Hello World'),
        };

        const zipped = zipSync(files);

        expect(zipped).toBeInstanceOf(Uint8Array);
        expect(zipped.length).toBeGreaterThan(0);
    });

    /** Test: ZIP contains correct mimetype */
    it('should have correct mimetype in ZIP', () => {
        const files = {
            'mimetype': strToU8('application/x-jcf'),
            'manifest.json': strToU8('{}'),
        };

        const zipped = zipSync(files);

        // Verify by unzipping
        const { unzipSync } = require('fflate');
        const unzipped = unzipSync(zipped);

        expect(strFromU8(unzipped['mimetype'])).toBe('application/x-jcf');
    });

    /** Test: Can store and retrieve file content in ZIP */
    it('should store and retrieve file content', () => {
        const content = 'console.log("Hello World")';
        const files = {
            'mimetype': strToU8('application/x-jcf'),
            'manifest.json': strToU8('{}'),
            'content/main.js': strToU8(content),
        };

        const zipped = zipSync(files);
        const { unzipSync } = require('fflate');
        const unzipped = unzipSync(zipped);

        expect(strFromU8(unzipped['content/main.js'])).toBe(content);
    });

    /** Test: Can handle binary content */
    it('should handle binary content', () => {
        const binaryData = new Uint8Array([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]); // PNG header
        const files = {
            'mimetype': strToU8('application/x-jcf'),
            'manifest.json': strToU8('{}'),
            'content/image.png': binaryData,
        };

        const zipped = zipSync(files);
        const { unzipSync } = require('fflate');
        const unzipped = unzipSync(zipped);

        expect(unzipped['content/image.png']).toEqual(binaryData);
    });

    /** Test: Manifest JSON is valid */
    it('should create valid JSON manifest', () => {
        const manifest = {
            formatVersion: "1.0.0",
            metadata: {
                name: "TestProject",
                description: "Test description",
                created: "2024-01-01T00:00:00Z",
                lastModified: "2024-01-01T00:00:00Z",
                author: "Tester"
            },
            fileMap: {
                "test.txt": {
                    inodeId: "uuid-123",
                    type: "text",
                    currentHash: "sha256hash",
                    created: "2024-01-01T00:00:00Z",
                    modified: "2024-01-01T00:00:00Z"
                }
            },
            versionHistory: [],
            refs: { head: "" },
            renameLog: []
        };

        const json = JSON.stringify(manifest, null, 2);
        const parsed = JSON.parse(json);

        expect(parsed.formatVersion).toBe("1.0.0");
        expect(parsed.metadata.name).toBe("TestProject");
        expect(parsed.fileMap["test.txt"].type).toBe("text");
    });

    // =========================================================================
    // EDGE CASES
    // =========================================================================

    /** Edge case: Empty manifest */
    it('should handle minimal manifest', () => {
        const manifest = {
            formatVersion: "1.0.0",
            metadata: { name: "", created: "", lastModified: "" },
            fileMap: {},
            versionHistory: [],
            refs: {},
            renameLog: []
        };

        const json = JSON.stringify(manifest);
        expect(() => JSON.parse(json)).not.toThrow();
    });

    /** Edge case: Unicode in manifest */
    it('should handle Unicode in manifest', () => {
        const manifest = {
            formatVersion: "1.0.0",
            metadata: { name: "Проект 日本語 🚀", created: "", lastModified: "" },
            fileMap: {},
            versionHistory: [],
            refs: {},
            renameLog: []
        };

        const files = {
            'manifest.json': strToU8(JSON.stringify(manifest)),
        };

        const zipped = zipSync(files);
        const { unzipSync } = require('fflate');
        const unzipped = unzipSync(zipped);
        const parsed = JSON.parse(strFromU8(unzipped['manifest.json']));

        expect(parsed.metadata.name).toBe("Проект 日本語 🚀");
    });

    /** Edge case: Many files in ZIP */
    it('should handle many files', () => {
        const files: Record<string, Uint8Array> = {
            'mimetype': strToU8('application/x-jcf'),
            'manifest.json': strToU8('{}'),
        };

        for (let i = 0; i < 100; i++) {
            files[`content/file_${i}.txt`] = strToU8(`Content of file ${i}`);
        }

        const zipped = zipSync(files);
        const { unzipSync } = require('fflate');
        const unzipped = unzipSync(zipped);

        expect(Object.keys(unzipped).length).toBe(102); // 100 files + mimetype + manifest
    });
});
