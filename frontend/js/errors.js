let monacoEditor = null;

export async function initializeEditor() {
    return new Promise((resolve, reject) => {
        require.config({ paths: { 'vs': 'https://unpkg.com/monaco-editor@latest/min/vs' }});
        require(['vs/editor/editor.main'], function () {
            try {
                // Define Pixardis language
                monaco.languages.register({ id: 'pixardis' });
                
                // Define syntax highlighting
                monaco.languages.setMonarchTokensProvider('pixardis', {
                    keywords: [
                        'fun', 'let', 'if', 'else', 'while', 'for', 'return', 'as',
                        'true', 'false'
                    ],
                    
                    typeKeywords: [
                        'bool', 'int', 'float', 'colour', 'function'
                    ],
                    
                    builtinFunctions: [
                        '__clear', '__write_box', '__random_int', '__width', '__height', 
                        '__print', '__pixel', '__wait', '__read_pixel'
                    ],
                    
                    operators: [
                        '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=', '!=',
                        '&&', '||', '++', '--', '+', '-', '*', '/', '&', '|', '^', '%',
                        '<<', '>>', '>>>', '+=', '-=', '*=', '/=', '&=', '|=', '^=',
                        '%=', '<<=', '>>=', '>>>='
                    ],
                    
                    symbols: /[=><!~?:&|+\-*\/\^%]+/,
                    
                    tokenizer: {
                        root: [
                            [/\/\/.*$/, 'comment'],
                            [/\bfun\b/, 'keyword'],
                            [/\blet\b/, 'keyword'],
                            [/\b(if|else|while|for|return|as|true|false)\b/, 'keyword'],
                            [/\b(bool|int|float|colour|function)\b/, 'keyword.type'],
                            [/\b(__clear|__write_box|__random_int|__width|__height|__print|__pixel|__wait|__read_pixel)\b/, 'keyword.control'],
                            [/\b[a-zA-Z_][a-zA-Z0-9_]*(?=\s*\()/, 'entity.name.function'],
                            [/\[\s*\d*\s*\]/, 'keyword.type.array'],
                            [/:\s*(bool|int|float|colour|function)/, 'keyword.type'],
                            [/->\s*(bool|int|float|colour|function)/, 'keyword.type'],
                            [/#[0-9a-fA-F]{6}\b/, 'number.hex'],
                            [/\d*\.\d+([eE][\-+]?\d+)?/, 'number.float'],
                            [/\d+/, 'number'],
                            [/"([^"\\]|\\.)*"/, 'string'],
                            [/\bas\b/, 'keyword.cast'],
                            [/[=!<>]=?/, 'operator'],
                            [/[+\-*/%]/, 'operator'],
                            [/->/, 'operator'],
                            [/&&|\|\|/, 'operator'],
                            [/\[/, 'delimiter.bracket'],
                            [/\]/, 'delimiter.bracket'],
                            [/[a-zA-Z_$][\w$]*/, 'identifier'],
                            [/[{}()]/, 'delimiter.bracket'],
                            [/[;,.]/, 'delimiter'],
                            [/:/, 'delimiter.type'],
                        ]
                    }                    
                });

                // Enhanced color theme
                monaco.editor.defineTheme('pixardis-dark', {
                    base: 'vs-dark',
                    inherit: true,
                    rules: [
                        { token: 'keyword', foreground: '569cd6', fontStyle: 'bold' },
                        { token: 'keyword.type', foreground: '4ec9b0', fontStyle: 'bold' },
                        { token: 'keyword.type.array', foreground: '4ec9b0' },
                        { token: 'keyword.control', foreground: 'c586c0', fontStyle: 'bold' },
                        { token: 'keyword.cast', foreground: '569cd6', fontStyle: 'italic' },
                        { token: 'entity.name.function', foreground: 'dcdcaa', fontStyle: 'bold' },
                        { token: 'number.hex', foreground: 'b5cea8', fontStyle: 'bold' },
                        { token: 'number.float', foreground: 'b5cea8' },
                        { token: 'number', foreground: 'b5cea8' },
                        { token: 'string', foreground: 'ce9178' },
                        { token: 'string.invalid', foreground: 'f44747' },
                        { token: 'comment', foreground: '6a9955', fontStyle: 'italic' },
                        { token: 'operator', foreground: 'd4d4d4' },
                        { token: 'operator.arrow', foreground: '569cd6' },
                        { token: 'delimiter.type', foreground: '569cd6' },
                        { token: 'identifier', foreground: '9cdcfe' },
                    ],
                    colors: {
                        'editor.background': '#1e1e1e',
                        'editor.foreground': '#d4d4d4',
                        'editorLineNumber.foreground': '#858585',
                        'editor.selectionBackground': '#264f78',
                        'editor.inactiveSelectionBackground': '#3a3d41'
                    }
                });
                
                // Create the editor
                monacoEditor = monaco.editor.create(document.getElementById('monaco-editor'), {
                    value: getDefaultCode(),
                    language: 'pixardis',
                    theme: 'pixardis-dark',
                    automaticLayout: true,
                    minimap: { enabled: false },
                    scrollBeyondLastLine: false,
                    fontSize: 14,
                    fontFamily: "'Courier New', monospace"
                });
                
                console.log("✅ Monaco Editor initialized!");
                resolve();
            } catch (error) {
                reject(error);
            }
        });
    });
}

export function getEditor() {
    return monacoEditor;
}

export function setEditorValue(code) {
    if (monacoEditor) {
        monacoEditor.setValue(code);
    }
}

export function getEditorValue() {
    return monacoEditor ? monacoEditor.getValue() : '';
}

function getDefaultCode() {
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

export function setupAutoSave() {
    setTimeout(() => {
        if (monacoEditor) {
            monacoEditor.onDidChangeModelContent(() => {
                localStorage.setItem('pixardis_code', monacoEditor.getValue());
            });
            
            // Load saved code
            const savedCode = localStorage.getItem('pixardis_code');
            if (savedCode) {
                monacoEditor.setValue(savedCode);
            }
        }
    }, 1000);
}