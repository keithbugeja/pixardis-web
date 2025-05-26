import { clear_vm_print_output } from '../pkg/web.js';

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
    document.getElementById('load-pong').addEventListener('click', () => {
        setEditorValue(getPongCode());
    });

    document.getElementById('load-fibonacci').addEventListener('click', () => {
        setEditorValue(getFibonacciCode());
    });

    document.getElementById('clear-editor').addEventListener('click', () => {
        setEditorValue(getNewFileCode());
    });

    // Add clear console functionality
    document.getElementById('clear-console')?.addEventListener('click', () => {
        const consoleContent = document.getElementById('console-content');
        const consoleDiv = document.getElementById('console-output');
        consoleContent.innerHTML = '';
        consoleDiv.style.display = 'none';
        
        // Also clear the VM's print buffer if available
        if (window.vm && typeof clear_vm_print_output !== 'undefined') {
            clear_vm_print_output(window.vm);
        }
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

    // Setup auto-save
    setupAutoSave();

    console.log("✅ Event handlers setup!");
}

function getPongCode() {
    return `// Pong for Pixardis
// A simple auto-play Pong game

let paddle_size : int[2] = [1, 8];
let paddle_lt : int[2] = [0, (__height - paddle_size[1]) / 2];
let paddle_rt : int[2] = [__width - paddle_size[0], (__height - paddle_size[1]) / 2];
let ball_pos : int[2] = [__width / 2, __height / 2];
let ball_vel : int[2] = [1, -1];

fun sign(value : int) -> int {
    if (value < 0) { 
        return -1;
    } else {
        if (value > 0) {
            return 1;
        } else {
            return 0;
        }
    }
}

// Simulate a game loop
while (true) {
    // Clear previous positions 
    __clear #000000;

    // Ball movement logic
    ball_pos[0] = ball_pos[0] + ball_vel[0];
    ball_pos[1] = ball_pos[1] + ball_vel[1];

    // Ball collision with screen edges
    if ((ball_pos[0] <= 1) or (ball_pos[0] >= __width - 1)) {
        ball_vel[0] = -ball_vel[0];
    }

    if ((ball_pos[1] <= 0) or (ball_pos[1] >= __height)) {
        ball_vel[1] = -ball_vel[1];
    }

    // AI Paddle movement towards the ball's y-position
    if (ball_pos[0] < __width / 2) {
        paddle_lt[1] = paddle_lt[1] + sign(ball_pos[1] - paddle_lt[1]);
    } else {
        paddle_rt[1] = paddle_rt[1] + sign(ball_pos[1] - paddle_rt[1]);
    }

    // Redraw the paddles and ball
    __write_box paddle_lt[0], paddle_lt[1], paddle_size[0], paddle_size[1], #FFFFFF; 
    __write_box paddle_rt[0], paddle_rt[1], paddle_size[0], paddle_size[1], #FFFFFF; 
    __write_box ball_pos[0], ball_pos[1], 1, 1, #FF0000;

    // Delay to control the speed of the game
    __delay 15;
}`;
}

function getFibonacciCode() {
    return `// Fibonacci example
fun fibonacci(n:int) -> int {
    if (n == 0) {
        return 0;
    } else { 
        if (n == 1) {
            return 1;
        }
    }

    return fibonacci(n - 1) + fibonacci(n - 2);
}

let result:int = fibonacci(5);
__print result;
`;
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