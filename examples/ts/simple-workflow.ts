
import { JCFManager, NodeAdapter, initWasm } from '../../js/src/index.ts';
import * as fs from 'fs';
import * as path from 'path';

async function main() {
    // Navigate to project root to find wasm pkg if needed, or rely on correct relative paths
    // In this dev setup, we are inside examples/ts.

    const projectPath = path.resolve('./demo-project-store-ts');

    // Cleanup
    if (fs.existsSync(projectPath)) {
        fs.rmSync(projectPath, { recursive: true, force: true });
    }
    fs.mkdirSync(projectPath);

    console.log("--- Kamaros TypeScript Demo ---");
    console.log(`Storage path: ${projectPath}`);

    // 0. Init WASM (required for TS/Browser)
    console.log("\n[0] Initializing WASM...");
    await initWasm();

    // 1. Initialize Manager
    const adapter = new NodeAdapter(projectPath);
    const manager = await JCFManager.create(adapter);

    // 2. Create Project
    console.log("\n[1] Creating project 'DemoAppTS'...");
    await manager.createProject("DemoAppTS");

    // 3. Add initial file
    console.log("[2] Adding 'index.ts'...");
    const initialContent = new TextEncoder().encode("console.log('Hello v1');");
    await manager.addFile("index.ts", initialContent);

    // 4. Save Checkpoint v1
    const v1Id = await manager.saveCheckpoint("Initial commit TS");
    console.log(` -> Checkpoint saved: ${v1Id}`);

    // 5. Modify file
    console.log("\n[3] Modifying 'index.ts'...");
    const updatedContent = new TextEncoder().encode("console.log('Hello v2 - updated');");
    await manager.addFile("index.ts", updatedContent);

    // 6. Save Checkpoint v2
    const v2Id = await manager.saveCheckpoint("Update index.ts");
    console.log(` -> Checkpoint saved: ${v2Id}`);

    // Verify current
    const currentFile = await manager.getFile("index.ts");
    const currentStr = new TextDecoder().decode(currentFile);
    console.log(`Current content: ${currentStr}`);
    if (!currentStr.includes("v2")) throw new Error("Verification failed: expected v2 content");

    // 7. Restore v1
    console.log(`\n[4] Restoring version ${v1Id}...`);
    await manager.restoreVersion(v1Id);
    console.log(` -> Restored to: ${v1Id}`);

    // 8. Verify restoration
    const restoredFile = await manager.getFile("index.ts");
    const restoredStr = new TextDecoder().decode(restoredFile);
    console.log(`Restored content: ${restoredStr}`);

    if (restoredStr.includes("v1")) {
        console.log("\nSUCCESS: Content restored correctly!");
    } else {
        console.error("\nFAILURE: Content mismatch!");
        process.exit(1);
    }
}

main().catch(err => {
    console.error("Error:", err);
    process.exit(1);
});
