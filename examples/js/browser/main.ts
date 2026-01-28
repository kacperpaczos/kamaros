import { JCFManager, OPFSAdapter } from '@kamaros/web';

const output = document.getElementById('output')!;
const btnInit = document.getElementById('btn-init') as HTMLButtonElement;
const btnSave = document.getElementById('btn-save') as HTMLButtonElement;
const dropZone = document.getElementById('drop-zone') as HTMLDivElement;
const fileList = document.getElementById('file-list') as HTMLUListElement;

let manager: JCFManager;

function log(msg: string, type: 'info' | 'success' | 'error' = 'info') {
    const time = new Date().toLocaleTimeString();
    const el = document.createElement('div');
    el.textContent = `[${time}] ${msg}`;
    if (type !== 'info') el.classList.add(type);
    output.appendChild(el);
    output.scrollTop = output.scrollHeight;
}

// === UI Helpers ===

async function refreshFileList() {
    if (!manager) return;
    fileList.innerHTML = 'Loading...';

    try {
        const files = await manager.listFiles();
        fileList.innerHTML = '';

        if (files.length === 0) {
            fileList.innerHTML = '<li>No files in project</li>';
            return;
        }

        for (const path of files) {
            const li = document.createElement('li');
            li.style.marginBottom = '5px';

            // Get file content
            const content = manager.getFile(path);
            let preview = '';

            if (content) {
                const size = (content.byteLength / 1024).toFixed(2) + ' KB';
                // Detect image
                if (path.match(/\.(jpg|jpeg|png|gif|webp)$/i)) {
                    const blob = new Blob([content as any]);
                    const url = URL.createObjectURL(blob);
                    preview = `<br><img src="${url}" style="max-height: 100px; border: 1px solid #ddd; margin-top: 5px;">`;
                }
                li.innerHTML = `<strong>${path}</strong> (${size}) ${preview}`;
            } else {
                li.textContent = path + ' (content missing)';
            }

            fileList.appendChild(li);
        }
    } catch (e) {
        log(`List error: ${e}`, 'error');
    }
}

// === Event Listeners ===

btnInit.addEventListener('click', async () => {
    try {
        log('Initializing OPFS Adapter...', 'info');
        const adapter = new OPFSAdapter();

        manager = await JCFManager.create(adapter);
        await manager.createProject("BrowserDemo", { description: "Demo running in browser via OPFS" });

        log('Project created successfully!', 'success');
        btnInit.disabled = true;
        btnSave.disabled = false;

        await refreshFileList();

        // Enable Drop Zone
        dropZone.style.borderColor = '#666';
        dropZone.textContent = 'Drop files here!';
    } catch (e) {
        log(`Error: ${e}`, 'error');
    }
});

btnSave.addEventListener('click', async () => {
    try {
        const versionId = await manager.saveCheckpoint("Update via Web UI");
        log(`Snapshot saved! Version: ${versionId}`, 'success');
    } catch (e) {
        log(`Error: ${e}`, 'error');
    }
});

// === Drag & Drop ===

dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.style.backgroundColor = '#eef';
});

dropZone.addEventListener('dragleave', (e) => {
    e.preventDefault();
    dropZone.style.backgroundColor = '';
});

dropZone.addEventListener('drop', async (e) => {
    e.preventDefault();
    dropZone.style.backgroundColor = '';

    if (!manager) {
        log('Please Init Project first!', 'error');
        return;
    }

    if (e.dataTransfer?.files) {
        for (const file of e.dataTransfer.files) {
            try {
                const buffer = await file.arrayBuffer();
                const uint8 = new Uint8Array(buffer);

                // Use file.name directly (flat structure for demo)
                await manager.addFile(file.name, uint8);
                log(`Added file: ${file.name} (${uint8.byteLength} bytes)`, 'success');
            } catch (err) {
                log(`Failed to add ${file.name}: ${err}`, 'error');
            }
        }
        await refreshFileList();
    }
});
