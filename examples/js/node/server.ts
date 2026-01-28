import { JCFManager, NodeAdapter } from '@kamaros/node';
import path from 'path';
import http from 'http';
import fs from 'fs/promises';

const PORT = 3000;
const PROJECT_DIR = path.resolve('server-data');

async function main() {
    // Ensure clean state
    try { await fs.rm(PROJECT_DIR, { recursive: true, force: true }); } catch { }

    // Init Manager
    const adapter = new NodeAdapter(PROJECT_DIR);
    const manager = await JCFManager.create(adapter);
    await manager.createProject("ServerDemo", { description: "HTTP Server Data" });

    // Add some initial data
    await manager.addFile("index.html", new TextEncoder().encode("<h1>Hello form JCF Server!</h1>"));
    await manager.addFile("data.json", new TextEncoder().encode(JSON.stringify({ status: "ok", time: Date.now() })));
    await manager.saveCheckpoint("Initial server data");

    // Create Server
    const server = http.createServer(async (req, res) => {
        try {
            const url = req.url || '/';
            const filePath = url.slice(1) || 'index.html'; // Remove leading slash

            console.log(`GET /${filePath}`);

            if (filePath === 'favicon.ico') {
                res.writeHead(404);
                res.end();
                return;
            }

            const content = manager.getFile(filePath);

            if (content) {
                // Determine mime type (simple check)
                let contentType = 'application/octet-stream';
                if (filePath.endsWith('.html')) contentType = 'text/html';
                if (filePath.endsWith('.json')) contentType = 'application/json';
                if (filePath.endsWith('.txt')) contentType = 'text/plain';

                res.writeHead(200, { 'Content-Type': contentType });
                // Note: content is Uint8Array, acceptable by res.end() in Node
                res.end(content);
            } else {
                res.writeHead(404, { 'Content-Type': 'text/plain' });
                res.end(`File not found: ${filePath}`);
            }
        } catch (e) {
            console.error('Server Error:', e);
            res.writeHead(500);
            res.end('Internal Server Error');
        }
    });

    server.listen(PORT, () => {
        console.log(`\n🚀 Store Server running at http://localhost:${PORT}`);
        console.log(`   Try: http://localhost:${PORT}/index.html`);
        console.log(`   Try: http://localhost:${PORT}/data.json`);
    });
}

main().catch(console.error);
