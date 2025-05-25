import { getEditor, setEditorValue, getEditorValue, setupAutoSave } from './editor.js';
import { hideCompilerErrors } from './errors.js'; 
import { 
    compileAndRun, 
    resizeVM, 
    resetVM,
    pauseVM,
    resumeVM,
    stepVM,
    setCyclesPerFrame,
    getVMRunningState
} from './vm.js';

export function setupEventHandlers() {
    const compileBtn = document.getElementById('compile-btn');
    
    // Compilation
    compileBtn.addEventListener('click', () => {
        const sourceCode = getEditorValue();
        if (sourceCode) {
            compileAndRun(sourceCode);
        }
    });
    
    // F8 shortcut
    document.addEventListener('keydown', (e) => {
        if (e.key === 'F8') {
            e.preventDefault();
            const sourceCode = getEditorValue();
            if (sourceCode) {
                compileAndRun(sourceCode);
            }
        }
    });
    
    // Error panel close button
    document.getElementById('close-errors').addEventListener('click', () => {
        hideCompilerErrors();
    });

    // Sample programs
    document.getElementById('load-bounce').addEventListener('click', () => {
        setEditorValue(getBounceCode());
    });

    document.getElementById('clear-editor').addEventListener('click', () => {
        setEditorValue(getNewFileCode());
    });

    // VM controls
    document.getElementById('reset-vm').addEventListener('click', () => {
        resetVM();
    });

    document.getElementById('pause-btn')?.addEventListener('click', () => {
        if (getVMRunningState()) {
            pauseVM();
        } else {
            resumeVM();
        }
    });

    document.getElementById('step-btn')?.addEventListener('click', stepVM);

    document.getElementById('cycles-slider')?.addEventListener('input', (e) => {
        setCyclesPerFrame(parseInt(e.target.value));
    });

    document.getElementById('vm-size-select').addEventListener('change', (e) => {
        const [width, height] = e.target.value.split(',').map(Number);
        resizeVM(width, height);
    });
    
    // Setup auto-save
    setupAutoSave();

    // About modal
    document.getElementById('about-btn')?.addEventListener('click', () => {
        document.getElementById('about-modal').style.display = 'block';
    });

    document.getElementById('close-about')?.addEventListener('click', () => {
        document.getElementById('about-modal').style.display = 'none';
    });

    // Close modal when clicking outside
    window.addEventListener('click', (event) => {
        const modal = document.getElementById('about-modal');
        if (event.target === modal) {
            modal.style.display = 'none';
        }
    });

    // Close modal with Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            document.getElementById('about-modal').style.display = 'none';
        }
    });

    console.log("✅ Event handlers setup!");
}

function getBounceCode() {
    return `// Bouncing ball example
fun to_int(a:float) -> int {
    let b:int = 0;
    if (a > 0.0) {
        while (a > 0.5) {
            a = a - 1.0;
            b = b + 1;
        }
    } else {
        while (a < -0.5) {
            a = a + 1.0;
            b = b - 1;
        }
    }

    return b;
}

fun draw_ball(x:float, y:float, r:float, c:colour) -> bool
{
    let half_r:float = r / 2.0;
    let x1:int = to_int((x - half_r));
    let y1:int = to_int((y - half_r));
    let r1:int = r as int;

    __clear #778899;
    __write_box x1, y1, r1, r1, c;

    return true;
}

let result:bool = true;
let velocity_x:float = ((__random_int 10) / 10) as float;
let velocity_y:float = 0.0 - ((__random_int 10) / 10) as float;

let x:float = (__width / 2) as float;
let y:float = (__height / 2) as float;
let r:float = 3.0;

while (true) {
    y = y + velocity_y;
    velocity_y = velocity_y - 0.1;
    if ((y - r) < 0.0) {
        if (velocity_y < 0.0) {
            velocity_y = 0.0 - velocity_y;
        }
    }

    result = draw_ball(x, y, r, #FFFFFF);
}`;
}

function getNewFileCode() {
    return `// New Pixardis program
    __clear #FFFFFF;
    // Write your code here...`;
}