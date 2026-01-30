import { JCFManager, NodeAdapter } from '@kamaros/node';
import path from 'path';
import fs from 'fs/promises';

async function main() {
    const projectDir = path.resolve('test-project');

    // Clean up previous run
    try {
        await fs.rm(projectDir, { recursive: true, force: true });
    } catch { }

    console.log('🚀 Initializing Kamaros Node Example...');

    // Initialize Manager with NodeAdapter
    // This will store files in ./test-project/.store/
    const adapter = new NodeAdapter(projectDir);
    const manager = await JCFManager.create(adapter);

    // 1. Create Project
    console.log('📦 Creating project...');
    await manager.createProject("NodeDemo", { description: "A demo project running on Node.js" });

    // 2. Add some files
    console.log('📝 Adding files...');
    const content = new TextEncoder().encode("Hello from Node.js!");
    await manager.addFile("hello.txt", content);

    // 3. Save snapshot (commit)
    console.log('💾 Saving snapshot...');
    const versionId = await manager.saveCheckpoint("Initial commit");
    console.log(`✅ Snapshot saved! Version: ${versionId}`);

    // 4. Verify file exists on disk (simulated check)
    const files = await manager.listFiles();
    console.log('📂 Files in project:', files);

    // 5. Encryption Demo
    console.log('\n🔐 Testing Encryption...');
    const salt = new Uint8Array(16); // In real app, use crypto.getRandomValues and store salt
    const key = await manager.deriveKey("secret123", salt);

    await manager.addFile("secret.txt", new TextEncoder().encode("Classified Information"));
    const v2 = await manager.saveCheckpoint("Added encrypted file", { encryptionKey: key });
    console.log(`✅ Encrypted Snapshot saved! Version: ${v2}`);

    // 6. ZIP Export
    console.log('\n📦 Exporting to ZIP...');
    const zipData = await manager.exportZip();
    await fs.writeFile('demo-export.zip', zipData);
    console.log(`✅ Exported ${zipData.length} bytes to demo-export.zip`);

    console.log('\n🎉 Done!');
}

main().catch(console.error);
