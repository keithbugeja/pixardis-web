import init from '../pkg/web.js';
import { initializeEditor } from './editor.js';
import { initializeVM } from './vm.js';
import { setupEventHandlers } from './ui.js';

async function initializeIDE() {
    try {
        await init();
        console.log("✅ WASM module loaded!");

        await initializeEditor();
        await initializeVM();
        setupEventHandlers();
        
        console.log("🎮 Pixardis IDE ready!");
        
    } catch (error) {
        console.error("❌ IDE initialization failed:", error);
        document.getElementById('status-bar').textContent = `❌ Failed to load WASM: ${error}`;
        document.getElementById('status-bar').className = "status-bar status-error";
    }
}

initializeIDE();