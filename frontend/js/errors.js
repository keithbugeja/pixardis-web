// export function showCompilerErrors(errorText) {
//     const errorPanel = document.getElementById('error-panel');
//     const errorContent = document.getElementById('error-content');
    
//     // Parse the error text and create formatted error messages
//     const errors = parseCompilerErrors(errorText);
    
//     // Clear previous errors
//     errorContent.innerHTML = '';
    
//     // Add each error
//     errors.forEach(error => {
//         const errorDiv = document.createElement('div');
//         errorDiv.className = 'error-message';
        
//         errorDiv.innerHTML = `
//             <div class="error-location">${error.location}</div>
//             <div class="error-description">${error.description}</div>
//             ${error.code ? `<div class="error-code">${error.code}</div>` : ''}
//         `;
        
//         errorContent.appendChild(errorDiv);
//     });
    
//     // Show the error panel
//     errorPanel.style.display = 'flex';
// }

import { getEditor } from './editor.js';  // Import to access Monaco editor

export function showCompilerErrors(errorText) {
    const errorPanel = document.getElementById('error-panel');
    const errorContent = document.getElementById('error-content');
    
    // Parse the error text and create formatted error messages
    const errors = parseCompilerErrors(errorText);
    
    // Clear previous errors
    errorContent.innerHTML = '';
    
    // Add each error
    errors.forEach(error => {
        const errorDiv = document.createElement('div');
        errorDiv.className = 'error-message clickable';  // Add clickable class for CSS
        
        errorDiv.innerHTML = `
            <div class="error-location">${error.location}</div>
            <div class="error-description">${error.description}</div>
            ${error.code ? `<div class="error-code">${error.code}</div>` : ''}
        `;
        
        // Add click handler to jump to line
        const lineMatch = error.location.match(/Line (\d+)/);
        if (lineMatch) {
            const lineNumber = parseInt(lineMatch[1]);
            errorDiv.addEventListener('click', () => {
                jumpToLine(lineNumber);
            });
            errorDiv.style.cursor = 'pointer';
        }
        
        errorContent.appendChild(errorDiv);
    });
    
    // Show the error panel
    errorPanel.style.display = 'flex';
}

function jumpToLine(lineNumber) {
    const editor = getEditor();
    if (editor) {
        // Jump to the line and column 1
        editor.setPosition({ lineNumber: lineNumber, column: 1 });
        
        // Reveal the line (scroll into view)
        editor.revealLineInCenter(lineNumber);
        
        // Highlight the line temporarily
        const decorations = editor.deltaDecorations([], [{
            range: new monaco.Range(lineNumber, 1, lineNumber, 1000),
            options: {
                isWholeLine: true,
                className: 'error-line-highlight',
                marginClassName: 'error-line-margin'
            }
        }]);
        
        // Remove highlight after 2 seconds
        setTimeout(() => {
            editor.deltaDecorations(decorations, []);
        }, 2000);
        
        // Focus the editor
        editor.focus();
    }
}

export function hideCompilerErrors() {
    const errorPanel = document.getElementById('error-panel');
    errorPanel.style.display = 'none';
}

function parseCompilerErrors(errorText) {
    if (!errorText) {
        return [{
            location: 'Unknown location',
            description: 'No error details available',
            code: null
        }];
    }
    
    const errorString = typeof errorText === 'string' ? errorText : String(errorText);
    const lines = errorString.trim().split('\n').filter(line => line.trim());
    const errors = [];
    
    if (lines.length === 0) {
        return [{
            location: 'Unknown location',
            description: 'Empty error message',
            code: null
        }];
    }
    
    let currentError = null;
    
    for (const line of lines) {
        if (line.startsWith('In Line ')) {
            // This starts a new error - save the previous one if it exists
            if (currentError && currentError.description) {
                errors.push(currentError);
            }
            
            // Extract line number and start collecting source code
            const locationMatch = line.match(/In Line (\d+):\s*(.*)$/);
            currentError = {
                location: locationMatch ? `Line ${locationMatch[1]}` : 'Unknown line',
                description: '',
                code: locationMatch ? locationMatch[2] : line.replace('In Line ', '').split(':')[1]?.trim() || ''
            };
        } else if (line.match(/^(Lexical Error|Syntax Error|Semantic Error|Type Error|Name Resolution Error):/)) {
            // This is an error description
            if (currentError) {
                currentError.description = line;
            } else {
                // Error without location context
                errors.push({
                    location: 'General error',
                    description: line,
                    code: null
                });
            }
        } else {
            // This is continuation of source code
            if (currentError && currentError.code !== null) {
                currentError.code += '\n' + line;
            }
        }
    }
    
    // Don't forget the last error
    if (currentError && currentError.description) {
        errors.push(currentError);
    }
    
    return errors.length > 0 ? errors : [{
        location: 'Unknown location',
        description: errorString,
        code: null
    }];
}