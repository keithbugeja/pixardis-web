// Try importing everything first to see what's available
import * as WebModule from '../pkg/web.js';

console.log("Available exports:", Object.keys(WebModule));

import { 
    compile_pixardis_source, 
    compile_pixardis_source_with_errors,
    create_vm, 
    step_vm, 
    get_vm_framebuffer, 
    get_vm_print_output,
    clear_vm_print_output,
    load_vm_program 
} from '../pkg/web.js';

import { showCompilerErrors, hideCompilerErrors } from './errors.js'; 

let vm = null;
let animationId = null;
let vmWidth = 64;
let vmHeight = 48;
let canvas = null;
let ctx = null;

let isRunning = false;
let cyclesPerFrame = 1000;
let performanceStats = {
    lastTime: 0,
    frameCount: 0,
    fps: 0,
    ips: 0, // Instructions per second
    totalCycles: 0
};

// Add this helper function
export function getVMRunningState() {
    return isRunning;
}

export async function initializeVM() {
    // Initialize VM with default size
    vm = create_vm(vmWidth, vmHeight);
    
    // Get canvas elements
    canvas = document.getElementById('vm-canvas');
    ctx = canvas.getContext('2d');
    
    // Set up canvas
    canvas.width = vmWidth * 10;
    canvas.height = vmHeight * 10;
    ctx.imageSmoothingEnabled = false;
    
    console.log("✅ VM initialized!");
}

export function compileAndRun(sourceCode) {
    console.log("Testing WASM functions:");
    console.log("compile_pixardis_source_with_errors:", typeof compile_pixardis_source_with_errors);
    
    const statusBar = document.getElementById('status-bar');
    
    try {
        statusBar.textContent = "Compiling...";
        statusBar.className = "status-bar";
        
        hideCompilerErrors();
        
        // Debug the result
        console.log("Calling compile_pixardis_source_with_errors...");
        const result = compile_pixardis_source_with_errors(sourceCode);
        console.log("Full result:", result);
        console.log("Result type:", typeof result);
        
        // Handle Map object correctly
        const success = result.get('success');
        const assembly = result.get('assembly');
        const errors = result.get('errors');
        
        console.log("Success:", success);
        console.log("Assembly:", assembly);
        console.log("Errors:", errors);
        console.log("Errors type:", typeof errors);
        console.log("Errors length:", errors?.length);
        
        if (success) {
            // Assembly might also be a Map - let's check
            console.log("Assembly type:", typeof assembly);
            console.log("Assembly content:", assembly);
            
            // If assembly is a Map, we might need to extract the actual string
            const assemblyString = typeof assembly === 'string' ? assembly : 
                                 (assembly && assembly.get ? assembly.get('assembly') || String(assembly) : String(assembly));
            
            load_vm_program(vm, assemblyString);
            
            statusBar.textContent = `✅ Compiled successfully! Running...`;
            statusBar.className = "status-bar status-success";
            
            startVMLoop();
        } else {
            statusBar.textContent = `❌ Compilation failed`;
            statusBar.className = "status-bar status-error";
            
            // Handle errors safely
            let errorText = "";
            if (errors) {
                if (Array.isArray(errors)) {
                    errorText = errors.join('\n');
                } else if (typeof errors === 'string') {
                    errorText = errors;
                } else {
                    errorText = String(errors);
                }
            } else {
                errorText = "Unknown compilation error";
            }
            
            console.log("Final error text:", errorText);
            showCompilerErrors(errorText);
        }
        
    } catch (error) {
        statusBar.textContent = `❌ Error: ${error}`;
        statusBar.className = "status-bar status-error";
        console.error("Compilation error:", error);
    }
}

function startVMLoop() {
    if (animationId) {
        cancelAnimationFrame(animationId);
    }

    isRunning = true;
    performanceStats.lastTime = performance.now();
    performanceStats.frameCount = 0;
    performanceStats.totalCycles = 0;

    updateVMControls();

    function render() {
        if (!isRunning) return;
        
        try {
            const currentTime = performance.now();
            
             // Execute VM cycles and check for errors
            const vmResult = step_vm(vm, cyclesPerFrame);
            performanceStats.totalCycles += cyclesPerFrame;
            
            updateConsoleOutput();

            const success = vmResult.get('success');
            const error = vmResult.get('error');
            
            // Check if VM encountered an error
            if (!success) {
                console.error("VM Runtime Error:", error);
                const statusBar = document.getElementById('status-bar');
                statusBar.textContent = `❌ VM Runtime Error: ${error || 'Unknown error'}`;
                statusBar.className = "status-bar status-error";
                pauseVM();
                return;
            }

            // Get framebuffer data
            const framebuffer = get_vm_framebuffer(vm);
            
            // Dynamic imageData creation
            const imageData = ctx.createImageData(vmWidth, vmHeight);
            
            for (let i = 0; i < framebuffer.length; i += 3) {
                const pixelIndex = i / 3;
                const x = pixelIndex % vmWidth;
                const y = Math.floor(pixelIndex / vmWidth);
                
                // Flip Y coordinate dynamically
                const flippedY = (vmHeight - 1) - y;
                const flippedIndex = (flippedY * vmWidth + x) * 4;
                
                imageData.data[flippedIndex] = framebuffer[i];     // R
                imageData.data[flippedIndex + 1] = framebuffer[i + 1]; // G
                imageData.data[flippedIndex + 2] = framebuffer[i + 2]; // B
                imageData.data[flippedIndex + 3] = 255; // A
            }
            
            // Dynamic scaling
            const tempCanvas = document.createElement('canvas');
            tempCanvas.width = vmWidth;
            tempCanvas.height = vmHeight;
            const tempCtx = tempCanvas.getContext('2d');
            tempCtx.putImageData(imageData, 0, 0);
            
            ctx.drawImage(tempCanvas, 0, 0, vmWidth * 10, vmHeight * 10);        

            // Update performance stats
            performanceStats.frameCount++;
            if (currentTime - performanceStats.lastTime >= 1000) {
                performanceStats.fps = performanceStats.frameCount;
                performanceStats.ips = performanceStats.totalCycles;
                performanceStats.frameCount = 0;
                performanceStats.totalCycles = 0;
                performanceStats.lastTime = currentTime;
                updatePerformanceDisplay();
            }
            
        } catch (error) {
            console.error("VM execution error:", error);
            const statusBar = document.getElementById('status-bar');
            statusBar.textContent = `❌ VM Error: ${error}`;
            statusBar.className = "status-bar status-error";
            pauseVM();
            return;
        }
        
        animationId = requestAnimationFrame(render);
    }
    
    render();
}

export function resizeVM(newWidth, newHeight) {
    vmWidth = newWidth;
    vmHeight = newHeight;
    
    // Create new VM
    vm = create_vm(vmWidth, vmHeight);
    
    // Resize canvas
    canvas.width = vmWidth * 10;
    canvas.height = vmHeight * 10;
    
    // Update UI display
    document.querySelector('.vm-header span:nth-child(2)').textContent = 
        `${vmWidth}×${vmHeight} Display`;
    
    // Clear and reset
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.imageSmoothingEnabled = false;
    const statusBar = document.getElementById('status-bar');
    statusBar.textContent = `VM resized to ${vmWidth}×${vmHeight}. Ready to compile...`;
    statusBar.className = "status-bar";
    
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }
}

export function resetVM() {
    vm = create_vm(vmWidth, vmHeight);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.imageSmoothingEnabled = false;    
    const statusBar = document.getElementById('status-bar');
    statusBar.textContent = "VM reset. Ready to compile...";
    statusBar.className = "status-bar";
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }
}

export function pauseVM() {
    isRunning = false;
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }
    updateVMControls();
}

export function resumeVM() {
    if (!isRunning) {
        startVMLoop();
    }
}

export function stepVM() {
    if (!isRunning) {
        try {
            const vmResult = step_vm(vm, 1);
            const success = vmResult.get('success');
            const error = vmResult.get('error');
            
            updateConsoleOutput();

            if (!success) {
                console.error("VM Step Error:", error);
                return;
            }

            // Render single frame
            const framebuffer = get_vm_framebuffer(vm);
            const imageData = ctx.createImageData(vmWidth, vmHeight);
            
            for (let i = 0; i < framebuffer.length; i += 3) {
                const pixelIndex = i / 3;
                const x = pixelIndex % vmWidth;
                const y = Math.floor(pixelIndex / vmWidth);
                const flippedY = (vmHeight - 1) - y;
                const flippedIndex = (flippedY * vmWidth + x) * 4;
                
                imageData.data[flippedIndex] = framebuffer[i];
                imageData.data[flippedIndex + 1] = framebuffer[i + 1];
                imageData.data[flippedIndex + 2] = framebuffer[i + 2];
                imageData.data[flippedIndex + 3] = 255;
            }
            
            const tempCanvas = document.createElement('canvas');
            tempCanvas.width = vmWidth;
            tempCanvas.height = vmHeight;
            const tempCtx = tempCanvas.getContext('2d');
            tempCtx.putImageData(imageData, 0, 0);
            
            ctx.drawImage(tempCanvas, 0, 0, vmWidth * 10, vmHeight * 10);
        } catch (error) {
            console.error("VM step error:", error);
        }
    }
}

export function setCyclesPerFrame(cycles) {
    cyclesPerFrame = Math.max(1, Math.min(5000, cycles));
    updateVMControls();
}

export function updateVMControls() {
    const pauseBtn = document.getElementById('pause-btn');
    const stepBtn = document.getElementById('step-btn');
    const cyclesSlider = document.getElementById('cycles-slider');
    const cyclesValue = document.getElementById('cycles-value');
    
    if (pauseBtn) pauseBtn.textContent = isRunning ? '⏸️ Pause' : '▶️ Resume';
    if (stepBtn) stepBtn.disabled = isRunning;
    if (cyclesSlider) cyclesSlider.value = cyclesPerFrame;
    if (cyclesValue) cyclesValue.textContent = cyclesPerFrame;
}

export function updatePerformanceDisplay() {
    const fpsDisplay = document.getElementById('fps-display');
    const ipsDisplay = document.getElementById('ips-display');
    
    if (fpsDisplay) fpsDisplay.textContent = `FPS: ${performanceStats.fps}`;
    if (ipsDisplay) ipsDisplay.textContent = `IPS: ${performanceStats.ips.toLocaleString()}`;
}

function updateConsoleOutput() {
    if (!vm) return;
    
    try {
        const output = get_vm_print_output(vm);
        const consoleDiv = document.getElementById('console-output');
        const consoleContent = document.getElementById('console-content');
        
        if (output && output.length > 0) {
            // Show console if there's output
            consoleDiv.style.display = 'block';
            
            // Add new output lines
            output.forEach(line => {
                const lineDiv = document.createElement('div');
                lineDiv.textContent = line;
                lineDiv.className = 'console-line';
                consoleContent.appendChild(lineDiv);
            });
            
            // Clear the VM's print buffer after displaying
            clear_vm_print_output(vm);
            
            // Auto-scroll to bottom
            consoleContent.scrollTop = consoleContent.scrollHeight;
        }
    } catch (error) {
        console.error('Error updating console output:', error);
    }
}